//! Nivel e promocao das classes jogaveis.
//!
//! `max_class_level` escreve XP diretamente e reverte tudo se qualquer etapa
//! falhar. `promote_all_classes` usa a rotina nativa `RetireCharacter`, que e a
//! unica forma de promover sem corromper os contadores derivados.

use crate::build_profiles::Capability;
use crate::domain::dto::{
    ClassLevelChange, MaxClassLevelResult, PromoteAllResult, PromotionChange,
};
use crate::domain::progression::persist_to_disk;
use crate::domain::save_game::{CharacterProgress, SaveGame};
use crate::domain::session::TrainerSession;
use crate::infrastructure::process::{MemoryReader, MemoryWriter, NativeCall};
use crate::shared::error::{Result, codes};

/// Tempo para o jogo propagar o efeito de uma promocao antes da releitura.
const PROMOTION_SETTLE_MS: u64 = 80;

pub fn max_class_level() -> Result<MaxClassLevelResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::ClassLevel)?;

    let runtime = session.runtime();
    let profile = session.profile();
    let memory = runtime.memory();
    let target_xp = profile.catalog.max_class_xp;
    runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let (save, entries) = save_game.class_progress()?;
    runtime.verify_native(profile.natives.save_to_disk)?;

    let transaction = session.begin_save_transaction("max-class-level")?;

    // Rollback explicito: se qualquer classe falhar, as anteriores voltam ao
    // valor original antes de reportarmos o erro.
    let mut written: Vec<&CharacterProgress> = Vec::with_capacity(entries.len());
    let restore = |written: &[&CharacterProgress]| {
        for entry in written.iter().rev() {
            let _ = memory.write_i32(entry.xp_address, entry.xp);
        }
    };

    for entry in &entries {
        if let Err(error) = memory.write_i32(entry.xp_address, target_xp) {
            restore(&written);
            return Err(transaction.wrap(error.context(format!(
                "Falha ao atualizar {}; os valores anteriores foram restaurados",
                entry.class_name
            ))));
        }
        written.push(entry);
    }

    for entry in &entries {
        let verified = match memory.read_i32(entry.xp_address) {
            Ok(value) => value,
            Err(error) => {
                restore(&written);
                return Err(transaction.wrap(error));
            }
        };
        if verified != target_xp {
            restore(&written);
            return Err(transaction
                .verify(
                    false,
                    format!(
                        "A verificacao de {} falhou; os valores anteriores foram restaurados",
                        entry.class_name
                    ),
                )
                .unwrap_err());
        }
    }

    if let Err(error) = persist_to_disk(runtime, save, &transaction) {
        restore(&written);
        return Err(error
            .context("O save nao confirmou a gravacao; os valores anteriores foram restaurados"));
    }

    let changes = entries
        .iter()
        .map(|entry| ClassLevelChange {
            class_name: entry.class_name.into(),
            previous_xp: entry.xp,
            current_xp: target_xp,
        })
        .collect::<Vec<_>>();

    Ok(MaxClassLevelResult {
        pid: session.pid(),
        classes_updated: changes.len(),
        changes,
        backup: transaction.commit(),
        message: "Driller, Engineer, Gunner e Scout foram definidos no nivel 25.".into(),
    })
}

pub fn promote_all_classes() -> Result<PromoteAllResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::Promotion)?;

    let runtime = session.runtime();
    let profile = session.profile();
    let memory = runtime.memory();
    let target_xp = profile.catalog.max_class_xp;
    runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let (save, entries) = save_game.class_progress()?;

    // Os `PlayerCharacterID` sao resolvidos na mesma ordem das entradas do
    // save, para que nenhuma promocao caia na classe errada.
    let id_names = entries
        .iter()
        .map(|entry| entry.character_id_object)
        .collect::<Vec<_>>();
    let character_ids = runtime.find_named_objects(&id_names, "PlayerCharacterID")?;

    let retire = profile.natives.retire_character;
    runtime.verify_native(retire)?;
    runtime.verify_native(profile.natives.save_to_disk)?;

    let transaction = session.begin_save_transaction("promote-all-classes")?;

    let mut changes = Vec::with_capacity(entries.len());
    for (entry, character_id) in entries.iter().zip(character_ids) {
        // Promover exige nivel maximo; a rotina nativa zera o XP em seguida.
        transaction.guard(memory.write_i32(entry.xp_address, target_xp))?;

        let expected = entry.promotions.saturating_add(1);
        let returned = transaction.guard(runtime.call_verified(
            retire,
            NativeCall::TwoArgs {
                first: save,
                second: character_id,
            },
        ))? as i32;
        std::thread::sleep(std::time::Duration::from_millis(PROMOTION_SETTLE_MS));

        let current_promotions = transaction.guard(memory.read_i32(entry.promotions_address))?;
        let current_xp = transaction.guard(memory.read_i32(entry.xp_address))?;
        if returned != expected || current_promotions != expected || current_xp != 0 {
            return Err(codes::invalid_state(format!(
                "A promocao de {} nao foi confirmada. Restaure o backup em {} antes de tentar novamente.",
                entry.class_name,
                transaction.backup_path()
            )));
        }

        changes.push(PromotionChange {
            class_name: entry.class_name.into(),
            previous_promotions: entry.promotions,
            current_promotions,
        });
    }

    persist_to_disk(runtime, save, &transaction)?;

    Ok(PromoteAllResult {
        pid: session.pid(),
        classes_promoted: changes.len(),
        changes,
        backup: transaction.commit(),
        message: "Driller, Engineer, Gunner e Scout receberam uma promocao nativa.".into(),
    })
}

#[cfg(test)]
mod tests {
    use crate::domain::save_game::SaveGame;
    use crate::domain::testing::{SaveWorld, profile};

    #[test]
    fn class_entries_are_ordered_so_character_ids_line_up() {
        let world = SaveWorld::new();
        let runtime = world.world.runtime();
        let save_game = SaveGame::resolve(&runtime).unwrap();
        let (_, entries) = save_game.class_progress().unwrap();

        let names = entries
            .iter()
            .map(|entry| entry.class_name)
            .collect::<Vec<_>>();
        let ids = entries
            .iter()
            .map(|entry| entry.character_id_object)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Driller", "Engineer", "Gunner", "Scout"]);
        assert_eq!(ids, vec!["DrillerID", "EngineerID", "GunnerID", "ScoutID"]);
    }

    #[test]
    fn every_class_declares_the_object_that_identifies_it() {
        for class in profile().catalog.playable_classes {
            assert!(
                class.character_id_object.ends_with("ID"),
                "{} deve apontar para um PlayerCharacterID",
                class.name
            );
        }
    }

    #[test]
    fn addresses_resolved_from_the_save_point_at_the_right_entries() {
        let world = SaveWorld::new();
        world.set_class_progress("Gunner", 1_234, 3);

        let runtime = world.world.runtime();
        let save_game = SaveGame::resolve(&runtime).unwrap();
        let (_, entries) = save_game.class_progress().unwrap();
        let gunner = entries
            .iter()
            .find(|entry| entry.class_name == "Gunner")
            .unwrap();

        assert_eq!(gunner.xp_address, world.class_xp_address("Gunner"));
        assert_eq!(
            gunner.promotions_address,
            world.class_promotions_address("Gunner")
        );
        assert_eq!(gunner.xp, 1_234);
        assert_eq!(gunner.promotions, 3);
    }
}
