//! Testes live-game (SPEC-006, nivel 3).
//!
//! Exigem o DRG em execucao, na build suportada, com o personagem carregado na
//! Space Rig. Ficam atras da feature `live-tests` para que `cargo test` rode em
//! maquina sem o jogo instalado:
//!
//! ```text
//! cargo test --features live-tests -- --nocapture --test-threads=1
//! ```
//!
//! Nenhum teste aqui executa uma mutacao permanente: eles resolvem os alvos e
//! validam as assinaturas, mas nao chamam as rotinas de unlock.

#![cfg(feature = "live-tests")]

use crate::build_profiles::Capability;
use crate::domain::save_game::SaveGame;
use crate::domain::session::TrainerSession;
use crate::domain::weapons::targets;
use crate::domain::{inventory, weapons};
use crate::infrastructure::process::MemoryReader;

fn session() -> TrainerSession {
    TrainerSession::read_only().expect("o jogo deve estar aberto na build suportada")
}

#[test]
fn attaches_to_a_supported_build() {
    let session = session();
    println!(
        "pid={} profile={} sha256={}",
        session.pid(),
        session.profile().id,
        session.sha256()
    );
    assert!(session.profile().status.allows_memory_operations());
    assert!(session.require(Capability::Credits).is_ok());
}

#[test]
fn reads_live_credits_from_the_validated_build() {
    let snapshot = inventory::read_credits().expect("os creditos devem ser legiveis");
    println!(
        "credits={} pid={} address={}",
        snapshot.credits, snapshot.pid, snapshot.address
    );
    assert!(snapshot.credits >= 0);
    assert!(snapshot.address.starts_with("0x"));
}

#[test]
fn reads_all_inventory_resources_without_writing() {
    let session = session();
    let resources = inventory::read_resources().expect("os recursos devem ser legiveis");
    assert_eq!(resources.len(), session.profile().resources.len());
    assert!(
        resources
            .iter()
            .all(|resource| resource.amount.is_finite() && resource.amount >= 0.0)
    );
    println!(
        "resources={:?}",
        resources
            .iter()
            .map(|resource| (&resource.name, resource.amount))
            .collect::<Vec<_>>()
    );
}

#[test]
fn confirms_the_write_path_without_changing_the_balance() {
    let snapshot = inventory::read_credits().expect("os creditos devem ser legiveis");
    let result =
        inventory::set_credits(snapshot.credits).expect("escrever o mesmo valor deve funcionar");
    assert_eq!(result.previous, result.current);
    assert_eq!(result.current, snapshot.credits);
}

#[test]
fn every_native_routine_matches_the_profile_signature() {
    let session = session();
    let runtime = session.runtime();
    let natives = &session.profile().natives;

    for native in [
        natives.unlock_all_weapons,
        natives.unlock_all_perks,
        natives.unlock_all_upgrades,
        natives.forge_schematic_save,
        natives.overclock_reward,
        natives.skin_reward,
        natives.vanity_reward,
        natives.victory_pose_reward,
        natives.retire_character,
        natives.save_to_disk,
        natives.add_resource,
    ] {
        runtime
            .verify_native(native)
            .unwrap_or_else(|error| panic!("{} divergiu da build: {error}", native.label));
    }
}

#[test]
fn resolves_every_resource_write_target_without_mutating() {
    let session = session();
    let runtime = session.runtime();
    let profile = session.profile();
    runtime
        .world_context()
        .expect("o controller da Space Rig deve estar carregado");

    let names = profile
        .resources
        .iter()
        .map(|definition| definition.object_name)
        .collect::<Vec<_>>();
    let objects = runtime
        .find_named_instances(&names, "ResourceData")
        .expect("todos os ResourceData persistentes devem estar carregados");
    assert_eq!(objects.len(), profile.resources.len());

    for (definition, object) in profile.resources.iter().zip(objects) {
        let id = runtime
            .memory()
            .read_guid(object + profile.offsets.save.resource_savegame_id)
            .expect("SavegameID deve ser legivel");
        assert_eq!(id, definition.savegame_id, "{}", definition.name);
    }
}

#[test]
fn resolves_progression_state_without_executing_anything() {
    let session = session();
    let runtime = session.runtime();
    let profile = session.profile();
    let offsets = &profile.offsets.save;

    let save_game = SaveGame::resolve(runtime).expect("o save ativo deve ser resolvido");
    let (unlocked, owned, save) = save_game
        .item_counts()
        .expect("os arrays devem ser validos");
    let (perks, _) = save_game
        .progression_count(offsets.owned_perks, "perks")
        .expect("OwnedPerks deve ser valido");
    let (upgrades, _) = save_game
        .progression_count(offsets.purchased_item_upgrades, "modificacoes")
        .expect("PurchasedItemUpgrades deve ser valido");
    let (forged, owned_schematics, _) = save_game
        .schematic_counts()
        .expect("SchematicSave deve ser valido");
    let (forged_ids, _) = save_game
        .forged_schematic_ids()
        .expect("os SavegameIDs forjados devem ser validos");
    let (_, classes) = save_game
        .class_progress()
        .expect("as quatro classes devem estar no save");

    assert_eq!(forged_ids.len(), forged as usize);
    assert_eq!(classes.len(), profile.catalog.playable_classes.len());
    println!(
        "save=0x{save:X} unlocked={unlocked} owned={owned} perks={perks} upgrades={upgrades} forged={forged} pending={owned_schematics} promotions={:?}",
        classes
            .iter()
            .map(|entry| (entry.class_name, entry.promotions))
            .collect::<Vec<_>>()
    );
}

#[test]
fn resolves_the_four_player_character_ids() {
    let session = session();
    let runtime = session.runtime();
    runtime
        .world_context()
        .expect("o controller da Space Rig deve estar carregado");

    let names = session
        .profile()
        .catalog
        .playable_classes
        .iter()
        .map(|class| class.character_id_object)
        .collect::<Vec<_>>();
    let ids = runtime
        .find_named_objects(&names, "PlayerCharacterID")
        .expect("os quatro PlayerCharacterID devem estar carregados");
    assert_eq!(ids.len(), 4);
}

#[test]
fn follows_the_current_equipped_weapon() {
    let status = weapons::read_clip_status().expect("uma arma deve estar equipada");
    println!(
        "weapon={:?} clip={:?}/{:?} address={:?}",
        status.weapon_name, status.clip_count, status.clip_size, status.address
    );
    assert!(status.available);
    assert!(status.clip_count.is_some());
    assert!(status.clip_size.is_some());
}

#[test]
fn finds_damage_fields_for_the_equipped_weapon_without_writing() {
    let session = session();
    let runtime = session.runtime();
    let equipped = targets::equipped_actor(runtime).expect("uma arma deve estar equipada");
    let fields =
        targets::damage_targets(runtime, equipped).expect("os campos devem ser resolvidos");
    println!("weapon=0x{equipped:X} targets={fields:X?}");
    assert!(!fields.is_empty());
}

#[test]
fn starts_and_stops_the_infinite_magazine_worker() {
    use std::thread::sleep;
    use std::time::Duration;

    let starting = weapons::set_infinite_magazine(true).expect("o freeze deve ser ativado");
    assert!(starting.enabled);

    let mut active = starting;
    for _ in 0..200 {
        sleep(Duration::from_millis(50));
        active = weapons::read_clip_status().expect("o worker deve publicar seu status");
        if active.available {
            break;
        }
    }
    println!(
        "enabled={} available={} weapon={:?} clip={:?}/{:?}",
        active.enabled, active.available, active.weapon_name, active.clip_count, active.clip_size
    );
    assert!(active.available);
    assert_eq!(active.clip_count, active.clip_size);

    let stopped = weapons::set_infinite_magazine(false).expect("o freeze deve ser desativado");
    assert!(!stopped.enabled);
    sleep(Duration::from_millis(150));
    assert!(!weapons::infinite_magazine_enabled());
}

#[test]
fn starts_and_stops_the_weapon_damage_worker() {
    use std::thread::sleep;
    use std::time::Duration;

    use crate::shared::limits::DAMAGE_VALUE;

    let starting = weapons::set_weapon_damage(true).expect("o freeze de dano deve ser ativado");
    assert!(starting.enabled);

    let mut active = starting;
    for _ in 0..200 {
        sleep(Duration::from_millis(50));
        active = weapons::read_damage_status().expect("o worker deve publicar seu status");
        if active.available && active.value == Some(DAMAGE_VALUE) {
            break;
        }
    }
    println!(
        "enabled={} available={} weapon={:?} value={:?} targets={}",
        active.enabled, active.available, active.weapon_name, active.value, active.target_count
    );
    assert!(active.available);
    assert_eq!(active.value, Some(DAMAGE_VALUE));
    assert!(active.target_count > 0);

    let stopped = weapons::set_weapon_damage(false).expect("o freeze de dano deve ser desativado");
    assert!(!stopped.enabled);
    sleep(Duration::from_millis(150));
    assert!(!weapons::weapon_damage_enabled());
}
