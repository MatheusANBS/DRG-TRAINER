//! Desbloqueios permanentes via rotinas nativas do jogo.
//!
//! As tres operacoes seguem o mesmo contrato:
//! validar build e capacidade -> exigir mundo carregado -> ler contadores ->
//! validar assinaturas -> **backup** -> chamar a rotina -> reverificar ->
//! gravar em disco.

use crate::build_profiles::Capability;
use crate::domain::dto::{PermanentUnlockResult, UnlockAllResult};
use crate::domain::save_game::SaveGame;
use crate::domain::session::TrainerSession;
use crate::shared::error::Result;

use super::{invoke_and_settle, persist_to_disk};

pub fn unlock_all_weapons() -> Result<UnlockAllResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::WeaponUnlock)?;

    let runtime = session.runtime();
    let profile = session.profile();
    let world_context = runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let (unlocked_before, owned_before, save) = save_game.item_counts()?;

    // Ambas as rotinas sao validadas antes do backup: se a build divergir, a
    // operacao para sem ter tocado no save.
    let unlock = profile.natives.unlock_all_weapons;
    runtime.verify_native(unlock)?;
    runtime.verify_native(profile.natives.save_to_disk)?;

    let transaction = session.begin_save_transaction("unlock-all")?;
    invoke_and_settle(runtime, unlock, world_context, &transaction)?;

    let (unlocked_after, owned_after, verified_save) =
        transaction.guard(save_game.item_counts())?;
    transaction.ensure_same_save(save, verified_save)?;
    transaction.verify(
        unlocked_after >= unlocked_before && owned_after >= owned_before,
        format!(
            "Os contadores regrediram: unlocked {unlocked_before} -> {unlocked_after}, owned {owned_before} -> {owned_after}"
        ),
    )?;

    persist_to_disk(runtime, save, &transaction)?;

    let message = if unlocked_after == unlocked_before && owned_after == owned_before {
        "A rotina nativa foi concluida; nenhuma arma nova precisava ser liberada.".to_string()
    } else {
        format!(
            "Unlock concluido: UnlockedItems {unlocked_before} -> {unlocked_after}, OwnedItems {owned_before} -> {owned_after}."
        )
    };

    Ok(UnlockAllResult {
        pid: session.pid(),
        weapon_count: profile.catalog.weapon_count,
        unlocked_before,
        unlocked_after,
        owned_before,
        owned_after,
        backup: transaction.commit(),
        message,
    })
}

pub fn unlock_all_perks() -> Result<PermanentUnlockResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::PerkUnlock)?;

    let runtime = session.runtime();
    let profile = session.profile();
    runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let label = "perks adquiridos";
    let (items_before, save) =
        save_game.progression_count(profile.offsets.save.owned_perks, label)?;

    let unlock = profile.natives.unlock_all_perks;
    runtime.verify_native(unlock)?;
    runtime.verify_native(profile.natives.save_to_disk)?;

    let transaction = session.begin_save_transaction("unlock-all-perks")?;
    // Esta rotina recebe o save, nao o contexto de mundo.
    invoke_and_settle(runtime, unlock, save, &transaction)?;

    let (items_after, verified_save) =
        transaction.guard(save_game.progression_count(profile.offsets.save.owned_perks, label))?;
    transaction.ensure_same_save(save, verified_save)?;
    transaction.verify(
        items_after >= items_before,
        format!("A verificacao de perks falhou: {items_before} -> {items_after}"),
    )?;

    persist_to_disk(runtime, save, &transaction)?;

    Ok(PermanentUnlockResult {
        pid: session.pid(),
        items_before,
        items_after,
        backup: transaction.commit(),
        message: if items_after == items_before {
            "Todos os perks ja estavam liberados.".into()
        } else {
            format!("Perks liberados: {items_before} -> {items_after}.")
        },
    })
}

pub fn unlock_all_gear_modifications() -> Result<PermanentUnlockResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::GearUnlock)?;

    let runtime = session.runtime();
    let profile = session.profile();
    let world_context = runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let label = "modificacoes de equipamento";
    let offset = profile.offsets.save.purchased_item_upgrades;
    let (items_before, save) = save_game.progression_count(offset, label)?;

    let unlock = profile.natives.unlock_all_upgrades;
    runtime.verify_native(unlock)?;
    runtime.verify_native(profile.natives.save_to_disk)?;

    let transaction = session.begin_save_transaction("unlock-all-gear-modifications")?;
    invoke_and_settle(runtime, unlock, world_context, &transaction)?;

    let (items_after, verified_save) =
        transaction.guard(save_game.progression_count(offset, label))?;
    transaction.ensure_same_save(save, verified_save)?;
    transaction.verify(
        items_after >= items_before,
        format!("A verificacao das modificacoes falhou: {items_before} -> {items_after}"),
    )?;

    persist_to_disk(runtime, save, &transaction)?;

    Ok(PermanentUnlockResult {
        pid: session.pid(),
        items_before,
        items_after,
        backup: transaction.commit(),
        message: if items_after == items_before {
            "Todas as modificacoes de equipamento ja estavam adquiridas.".into()
        } else {
            format!("Modificacoes adquiridas: {items_before} -> {items_after}.")
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::domain::testing::{SaveWorld, profile};
    use crate::infrastructure::process::NativeCall;
    use crate::shared::ErrorCode;

    /// Cenario offline: a rotina nunca e chamada quando a assinatura diverge.
    #[test]
    fn a_corrupted_signature_blocks_the_native_call() {
        let world = SaveWorld::new();
        world.map_native_routines();
        world.corrupt_native(profile().natives.unlock_all_weapons);

        let runtime = world.world.runtime();
        let error = runtime
            .call_verified(
                profile().natives.unlock_all_weapons,
                NativeCall::Direct { argument: 0x5000 },
            )
            .expect_err("assinatura corrompida deve bloquear");
        assert_eq!(error.code(), ErrorCode::SignatureMismatch);
        assert!(world.world.memory().recorded_calls().is_empty());
    }

    /// Todas as rotinas do perfil precisam ser validaveis com os bytes
    /// declarados: um prefixo errado no perfil quebra este teste.
    #[test]
    fn every_native_in_the_profile_validates_against_its_own_prefix() {
        let world = SaveWorld::new();
        world.map_native_routines();
        let runtime = world.world.runtime();

        for native in crate::domain::testing::native_list(profile()) {
            runtime
                .verify_native(native)
                .unwrap_or_else(|error| panic!("{} falhou: {error}", native.label));
        }
    }

    #[test]
    fn item_counters_are_read_from_the_save_before_any_mutation() {
        let world = SaveWorld::new();
        world.set_item_counts(12, 9);

        let runtime = world.world.runtime();
        let save_game = crate::domain::save_game::SaveGame::resolve(&runtime).unwrap();
        let (unlocked, owned, address) = save_game.item_counts().unwrap();
        assert_eq!((unlocked, owned), (12, 9));
        assert_eq!(address, world.save);
        assert!(world.world.memory().recorded_calls().is_empty());
    }
}
