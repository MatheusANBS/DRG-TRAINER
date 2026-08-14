use super::*;

#[test]
fn reads_live_credits_from_validated_build() {
    let snapshot = read_credits().expect("o jogo deve estar aberto para este teste");
    println!(
        "credits={} pid={} address={}",
        snapshot.credits, snapshot.pid, snapshot.address
    );
    assert!(snapshot.credits >= 0);
    assert!(snapshot.address.starts_with("0x"));
}

#[test]
fn reads_all_inventory_resources_without_writing() {
    let resources = read_resources().expect("os recursos do save devem ser legiveis");
    assert_eq!(resources.len(), RESOURCE_DEFINITIONS.len());
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
fn resolves_all_resource_write_targets_without_mutating() {
    let pid = find_process_id().expect("o jogo deve estar aberto para este teste");
    let (module_base, executable) =
        find_main_module(pid).expect("o modulo principal deve estar carregado");
    validate_build(&executable).expect("a build deve corresponder ao perfil");
    let runtime = ClipReader::connect(pid, module_base, false)
        .expect("a reflexao do runtime deve estar disponivel");
    runtime
        .world_context()
        .expect("o controller da Space Rig deve estar carregado");
    runtime
        .memory
        .validate_code_prefix(
            module_base + ADD_RESOURCE_RVA,
            ADD_RESOURCE_PREFIX,
            "FSDSaveGame::AddResource",
        )
        .expect("a rotina de recursos deve corresponder a build");
    let names = RESOURCE_DEFINITIONS
        .iter()
        .map(|definition| definition.object_name)
        .collect::<Vec<_>>();
    let objects = runtime
        .find_named_instances(&names, "ResourceData")
        .expect("todos os ResourceData persistentes devem estar carregados");
    assert_eq!(objects.len(), RESOURCE_DEFINITIONS.len());
    for (definition, object) in RESOURCE_DEFINITIONS.iter().zip(objects) {
        let id: [u8; 16] = runtime
            .memory
            .read_bytes(object + RESOURCE_SAVEGAME_ID_OFFSET, 16)
            .expect("SavegameID deve ser legivel")
            .try_into()
            .unwrap();
        assert_eq!(id, definition.savegame_id, "{}", definition.name);
    }
}

#[test]
fn confirms_write_path_without_changing_balance() {
    let snapshot = read_credits().expect("o jogo deve estar aberto para este teste");
    let result = set_credits(snapshot.credits).expect("a escrita do mesmo valor deve funcionar");
    println!(
        "previous={} current={} address={}",
        result.previous, result.current, result.address
    );
    assert_eq!(result.previous, result.current);
    assert_eq!(result.current, snapshot.credits);
}

#[test]
fn resolves_unlock_all_path_without_executing_it() {
    let pid = find_process_id().expect("o jogo deve estar aberto para este teste");
    let (module_base, executable) =
        find_main_module(pid).expect("o modulo principal deve estar carregado");
    validate_build(&executable).expect("a build deve corresponder ao perfil");

    let runtime = ClipReader::connect(pid, module_base, false)
        .expect("a reflexao do runtime deve estar disponivel");
    runtime
        .memory
        .validate_code_prefix(
            module_base + UNLOCK_ALL_WEAPONS_RVA,
            UNLOCK_ALL_WEAPONS_PREFIX,
            "Cheat_UnlockAllWeapons",
        )
        .expect("a rotina de unlock deve corresponder a build");
    runtime
        .memory
        .validate_code_prefix(
            module_base + SAVE_TO_DISK_RVA,
            SAVE_TO_DISK_PREFIX,
            "FSDSaveGame::SaveToDisk",
        )
        .expect("a rotina de save deve corresponder a build");

    let save =
        CreditsReader::connect(pid, module_base, false).expect("o save ativo deve ser resolvido");
    let (unlocked, owned, address) = save.item_counts().expect("os arrays devem ser validos");
    let (character_save, classes) = save
        .playable_class_xp()
        .expect("as quatro classes devem ser identificadas pelos GUIDs");
    assert_eq!(character_save, address);
    assert_eq!(classes.len(), 4);
    let context = match runtime.world_context() {
        Ok(context) => context,
        Err(error) => {
            println!(
                "save=0x{address:X} unlocked={unlocked} owned={owned} classes={classes:?}; Space Rig indisponivel: {error}"
            );
            return;
        }
    };
    assert_ne!(context, 0);
    println!("context=0x{context:X} save=0x{address:X} unlocked={unlocked} owned={owned}");
}

#[test]
fn resolves_progression_actions_without_executing_them() {
    let pid = find_process_id().expect("o jogo deve estar aberto para este teste");
    let (module_base, executable) =
        find_main_module(pid).expect("o modulo principal deve estar carregado");
    validate_build(&executable).expect("a build deve corresponder ao perfil");

    let runtime = ClipReader::connect(pid, module_base, false)
        .expect("a reflexao do runtime deve estar disponivel");
    let context = runtime
        .world_context()
        .expect("o controller da Space Rig deve estar carregado");
    for (rva, prefix, label) in [
        (
            UNLOCK_ALL_PERKS_RVA,
            UNLOCK_ALL_PERKS_PREFIX,
            "C_UnlockAll_Perks",
        ),
        (
            UNLOCK_ALL_UPGRADES_RVA,
            UNLOCK_ALL_UPGRADES_PREFIX,
            "Cheat_UnlockAllUpgrades",
        ),
        (
            FORGE_SCHEMATIC_SAVE_RVA,
            FORGE_SCHEMATIC_SAVE_PREFIX,
            "SchematicSave::ForgeSchematic",
        ),
        (
            RETIRE_CHARACTER_RVA,
            RETIRE_CHARACTER_PREFIX,
            "FSDSaveGame::RetireCharacter",
        ),
    ] {
        runtime
            .memory
            .validate_code_prefix(module_base + rva, prefix, label)
            .unwrap_or_else(|_| panic!("assinatura invalida para {label}"));
    }

    let ids = runtime
        .find_named_objects(
            &["DrillerID", "EngineerID", "GunnerID", "ScoutID"],
            "PlayerCharacterID",
        )
        .expect("os quatro PlayerCharacterID devem estar carregados");
    let save =
        CreditsReader::connect(pid, module_base, false).expect("o save ativo deve ser resolvido");
    let (perks, save_address) = save
        .progression_count(OWNED_PERKS_OFFSET, "perks")
        .expect("OwnedPerks deve ser valido");
    let (upgrades, verified_save) = save
        .progression_count(PURCHASED_ITEM_UPGRADES_OFFSET, "modificacoes")
        .expect("PurchasedItemUpgrades deve ser valido");
    let (forged, owned_schematics, schematic_save) = save
        .schematic_counts()
        .expect("SchematicSave deve ser valido");
    let (forged_ids, forged_save) = save
        .forged_schematic_ids()
        .expect("os SavegameIDs forjados devem ser validos");
    let all_schematics = runtime
        .all_schematics()
        .expect("GD_SchematicSettings.AllSchematics deve ser valido");
    let (character_save, classes) = save
        .playable_class_progress()
        .expect("as quatro classes devem estar no save");

    assert_ne!(context, 0);
    assert_eq!(ids.len(), 4);
    assert_eq!(save_address, verified_save);
    assert_eq!(save_address, character_save);
    assert_eq!(save_address, schematic_save);
    assert_eq!(save_address, forged_save);
    assert_eq!(forged_ids.len(), forged as usize);
    assert!(all_schematics.len() >= forged_ids.len());
    assert_eq!(classes.len(), 4);
    println!(
        "context=0x{context:X} save=0x{save_address:X} perks={perks} upgrades={upgrades} forged={forged} owned_schematics={owned_schematics} all_schematics={} ids={ids:X?} promotions={:?}",
        all_schematics.len(),
        classes
            .iter()
            .map(|entry| (entry.class_name, entry.promotions))
            .collect::<Vec<_>>()
    );
}

#[test]
fn summarizes_schematic_rewards_without_executing_them() {
    use std::collections::BTreeMap;

    let pid = find_process_id().expect("o jogo deve estar aberto para este teste");
    let (module_base, executable) =
        find_main_module(pid).expect("o modulo principal deve estar carregado");
    validate_build(&executable).expect("a build deve corresponder ao perfil");
    let runtime = ClipReader::connect(pid, module_base, false)
        .expect("a reflexao do runtime deve estar disponivel");
    let mut summary = BTreeMap::<(String, usize), usize>::new();
    for (schematic, _) in runtime
        .all_schematics()
        .expect("AllSchematics deve ser valido")
    {
        let item = runtime
            .memory
            .read_u64(schematic + 0xA8)
            .expect("Schematic.Item deve ser legivel") as usize;
        assert_ne!(item, 0);
        let class = runtime
            .memory
            .read_u64(item + UOBJECT_CLASS_OFFSET)
            .expect("a classe do item deve ser legivel") as usize;
        let class_name = runtime
            .read_object_name(class)
            .expect("a classe do item deve possuir nome");
        let vtable = runtime
            .memory
            .read_u64(item)
            .expect("a vtable do item deve ser legivel") as usize;
        let reward = runtime
            .memory
            .read_u64(vtable + 0x270)
            .expect("a rotina de recompensa deve ser legivel") as usize;
        *summary
            .entry((class_name, reward - module_base))
            .or_default() += 1;
    }
    println!("schematic reward summary: {summary:#X?}");
    assert_eq!(summary.values().sum::<usize>(), 515);
}

#[test]
fn follows_the_current_equipped_weapon() {
    let status = read_clip_status().expect("uma arma deve estar equipada para este teste");
    println!(
        "weapon={:?} clip={:?}/{:?} address={:?}",
        status.weapon_name, status.clip_count, status.clip_size, status.address
    );
    assert!(status.available);
    assert!(status.clip_count.is_some());
    assert!(status.clip_size.is_some());
}

#[test]
fn starts_and_stops_infinite_magazine_worker() {
    let starting = set_infinite_magazine(true).expect("o freeze deve ser ativado");
    assert!(starting.enabled);
    let mut active = starting;
    for _ in 0..200 {
        thread::sleep(Duration::from_millis(50));
        active = read_clip_status().expect("o worker deve publicar seu status");
        if active.available {
            break;
        }
    }
    println!(
        "enabled={} available={} weapon={:?} clip={:?}/{:?} message={}",
        active.enabled,
        active.available,
        active.weapon_name,
        active.clip_count,
        active.clip_size,
        active.message
    );
    assert!(active.enabled);
    assert!(active.available);
    assert_eq!(active.clip_count, active.clip_size);

    let stopped = set_infinite_magazine(false).expect("o freeze deve ser desativado");
    assert!(!stopped.enabled);
    thread::sleep(Duration::from_millis(75));
    assert!(!INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire));
}

#[test]
fn finds_damage_fields_for_equipped_weapon_without_writing() {
    let status = read_damage_status().expect("uma arma com dano deve estar equipada");
    println!(
        "weapon={:?} value={:?} targets={} addresses={:?}",
        status.weapon_name, status.value, status.target_count, status.addresses
    );
    assert!(status.available);
    assert!(status.value.is_some());
    assert!(status.target_count > 0);
}

#[test]
fn starts_and_stops_weapon_damage_worker() {
    let starting = set_weapon_damage(true).expect("o freeze de dano deve ser ativado");
    assert!(starting.enabled);
    let mut active = starting;
    for _ in 0..200 {
        thread::sleep(Duration::from_millis(50));
        active = read_damage_status().expect("o worker deve publicar seu status");
        if active.available && active.value == Some(DAMAGE_VALUE) {
            break;
        }
    }
    println!(
        "enabled={} available={} weapon={:?} value={:?} targets={} message={}",
        active.enabled,
        active.available,
        active.weapon_name,
        active.value,
        active.target_count,
        active.message
    );
    assert!(active.enabled);
    assert!(active.available);
    assert_eq!(active.value, Some(DAMAGE_VALUE));
    assert!(active.target_count > 0);

    let stopped = set_weapon_damage(false).expect("o freeze de dano deve ser desativado");
    assert!(!stopped.enabled);
    thread::sleep(Duration::from_millis(75));
    assert!(!WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire));
}
