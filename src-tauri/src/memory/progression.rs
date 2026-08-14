use super::*;

pub fn unlock_all_weapons() -> Result<UnlockAllResult> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;

    let runtime = ClipReader::connect(pid, module_base, true)?;
    let world_context = runtime.world_context()?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (unlocked_before, owned_before, save) = save_reader.item_counts()?;
    let backup_path = backup_save_games(&executable, "unlock-all")?;

    let unlock_address = module_base + UNLOCK_ALL_WEAPONS_RVA;
    runtime.memory.validate_code_prefix(
        unlock_address,
        UNLOCK_ALL_WEAPONS_PREFIX,
        "Cheat_UnlockAllWeapons",
    )?;
    runtime
        .memory
        .call_remote(unlock_address, world_context, "Unlock All Weapons")?;
    thread::sleep(Duration::from_millis(250));

    let (unlocked_after, owned_after, verified_save) = save_reader.item_counts()?;
    if verified_save != save {
        return Err(MemoryError(
            "O save ativo mudou durante o unlock; a gravacao foi interrompida.".into(),
        ));
    }

    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    runtime
        .memory
        .call_remote(save_address, save, "SaveToDisk")?;

    Ok(UnlockAllResult {
        pid,
        weapon_count: CURRENT_WEAPON_COUNT,
        unlocked_before,
        unlocked_after,
        owned_before,
        owned_after,
        backup_path: backup_path.display().to_string(),
        message: if unlocked_after == unlocked_before && owned_after == owned_before {
            "A rotina nativa foi concluida; nenhuma arma nova precisava ser liberada.".into()
        } else {
            format!(
                "Unlock concluido: UnlockedItems {unlocked_before} -> {unlocked_after}, OwnedItems {owned_before} -> {owned_after}."
            )
        },
    })
}

pub fn unlock_all_perks() -> Result<PermanentUnlockResult> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;

    let runtime = ClipReader::connect(pid, module_base, true)?;
    runtime.world_context()?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (items_before, save) =
        save_reader.progression_count(OWNED_PERKS_OFFSET, "perks adquiridos")?;
    let unlock_address = module_base + UNLOCK_ALL_PERKS_RVA;
    runtime.memory.validate_code_prefix(
        unlock_address,
        UNLOCK_ALL_PERKS_PREFIX,
        "C_UnlockAll_Perks",
    )?;
    let backup_path = backup_save_games(&executable, "unlock-all-perks")?;

    runtime
        .memory
        .call_remote(unlock_address, save, "Unlock All Perks")?;
    thread::sleep(Duration::from_millis(250));
    let (items_after, verified_save) =
        save_reader.progression_count(OWNED_PERKS_OFFSET, "perks adquiridos")?;
    if verified_save != save || items_after < items_before {
        return Err(MemoryError(format!(
            "A verificacao de perks falhou. O backup permanece em {}.",
            backup_path.display()
        )));
    }

    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    runtime
        .memory
        .call_remote(save_address, save, "SaveToDisk")?;

    Ok(PermanentUnlockResult {
        pid,
        items_before,
        items_after,
        backup_path: backup_path.display().to_string(),
        message: if items_after == items_before {
            "Todos os perks ja estavam liberados.".into()
        } else {
            format!("Perks liberados: {items_before} -> {items_after}.")
        },
    })
}

pub fn unlock_all_gear_modifications() -> Result<PermanentUnlockResult> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;

    let runtime = ClipReader::connect(pid, module_base, true)?;
    let world_context = runtime.world_context()?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (items_before, save) = save_reader.progression_count(
        PURCHASED_ITEM_UPGRADES_OFFSET,
        "modificacoes de equipamento",
    )?;
    let unlock_address = module_base + UNLOCK_ALL_UPGRADES_RVA;
    runtime.memory.validate_code_prefix(
        unlock_address,
        UNLOCK_ALL_UPGRADES_PREFIX,
        "Cheat_UnlockAllUpgrades",
    )?;
    let backup_path = backup_save_games(&executable, "unlock-all-gear-modifications")?;

    runtime.memory.call_remote(
        unlock_address,
        world_context,
        "Unlock All Gear Modifications",
    )?;
    thread::sleep(Duration::from_millis(250));
    let (items_after, verified_save) = save_reader.progression_count(
        PURCHASED_ITEM_UPGRADES_OFFSET,
        "modificacoes de equipamento",
    )?;
    if verified_save != save || items_after < items_before {
        return Err(MemoryError(format!(
            "A verificacao das modificacoes falhou. O backup permanece em {}.",
            backup_path.display()
        )));
    }

    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    runtime
        .memory
        .call_remote(save_address, save, "SaveToDisk")?;

    Ok(PermanentUnlockResult {
        pid,
        items_before,
        items_after,
        backup_path: backup_path.display().to_string(),
        message: if items_after == items_before {
            "Todas as modificacoes de equipamento ja estavam adquiridas.".into()
        } else {
            format!("Modificacoes adquiridas: {items_before} -> {items_after}.")
        },
    })
}

pub fn unlock_all_overclocks_and_cosmetics() -> Result<SchematicUnlockResult> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;

    let runtime = ClipReader::connect(pid, module_base, true)?;
    runtime.world_context()?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (forged_before, owned_before, save) = save_reader.schematic_counts()?;
    let (forged_ids, verified_save) = save_reader.forged_schematic_ids()?;
    if verified_save != save || forged_ids.len() != forged_before as usize {
        return Err(MemoryError(
            "O save ativo mudou durante a leitura dos esquemas.".into(),
        ));
    }
    let all_schematics = runtime.all_schematics()?;
    let reward_targets = runtime.schematic_reward_targets(&all_schematics)?;
    let (purchased_before, purchased_save) = save_reader.progression_ids(
        PURCHASED_ITEM_UPGRADES_OFFSET,
        "upgrades e overclocks adquiridos",
    )?;
    if purchased_save != save {
        return Err(MemoryError(
            "O save ativo mudou durante a leitura das recompensas.".into(),
        ));
    }
    let pending = all_schematics
        .iter()
        .filter(|(_, id)| !forged_ids.contains(id))
        .collect::<Vec<_>>();

    // A rotina Cheat_Schematic_UnlockAll dispara analytics sem validar ponteiro nulo
    // e causa crash em builds shipping. Esta rotina interna altera somente SchematicSave.
    let forge_address = module_base + FORGE_SCHEMATIC_SAVE_RVA;
    runtime.memory.validate_code_prefix(
        forge_address,
        FORGE_SCHEMATIC_SAVE_PREFIX,
        "SchematicSave::ForgeSchematic",
    )?;
    for kind in [
        SchematicRewardKind::Overclock,
        SchematicRewardKind::Skin,
        SchematicRewardKind::Vanity,
        SchematicRewardKind::VictoryPose,
    ] {
        let (address, prefix, label) = kind.routine(module_base);
        runtime
            .memory
            .validate_code_prefix(address, prefix, label)?;
    }
    let backup_path = backup_save_games(&executable, "unlock-all-schematics")?;

    let mut overclocks_processed = 0_usize;
    let mut cosmetics_processed = 0_usize;
    for target in &reward_targets {
        if target
            .overclock_id
            .is_some_and(|id| purchased_before.contains(&id))
        {
            continue;
        }
        let (reward_address, _, label) = target.kind.routine(module_base);
        runtime.memory.call_remote_three_args(
            reward_address,
            target.item,
            target.schematic,
            save,
            label,
        )?;
        match target.kind {
            SchematicRewardKind::Overclock => overclocks_processed += 1,
            _ => cosmetics_processed += 1,
        }
    }

    let schematic_save = save + SCHEMATIC_SAVE_OFFSET;
    for (schematic, _) in &pending {
        runtime.memory.call_remote_three_args(
            forge_address,
            schematic_save,
            *schematic,
            0,
            "Forge Schematic Save",
        )?;
    }
    thread::sleep(Duration::from_millis(150));

    let (forged_after, owned_after, verified_save) = save_reader.schematic_counts()?;
    if verified_save != save
        || forged_after != forged_before + pending.len() as i32
        || owned_after > owned_before
    {
        return Err(MemoryError(format!(
            "A verificacao dos esquemas falhou. O backup permanece em {}.",
            backup_path.display()
        )));
    }
    let (purchased_after, purchased_save) = save_reader.progression_ids(
        PURCHASED_ITEM_UPGRADES_OFFSET,
        "upgrades e overclocks adquiridos",
    )?;
    let missing_overclocks = reward_targets
        .iter()
        .filter_map(|target| target.overclock_id)
        .filter(|id| !purchased_after.contains(id))
        .count();
    if purchased_save != save || missing_overclocks != 0 {
        return Err(MemoryError(format!(
            "A concessao dos overclocks nao foi confirmada ({missing_overclocks} ausentes). O backup permanece em {}.",
            backup_path.display()
        )));
    }

    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    runtime
        .memory
        .call_remote(save_address, save, "SaveToDisk")?;

    Ok(SchematicUnlockResult {
        pid,
        overclocks_processed,
        cosmetics_processed,
        forged_before,
        forged_after,
        owned_before,
        owned_after,
        backup_path: backup_path.display().to_string(),
        message: format!(
            "Recompensas processadas: {overclocks_processed} overclocks e {cosmetics_processed} cosmeticos. Esquemas concluidos: {forged_before} -> {forged_after}; pendentes: {owned_before} -> {owned_after}."
        ),
    })
}

pub fn promote_all_classes() -> Result<PromoteAllResult> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;

    let runtime = ClipReader::connect(pid, module_base, true)?;
    runtime.world_context()?;
    let character_ids = runtime.find_named_objects(
        &["DrillerID", "EngineerID", "GunnerID", "ScoutID"],
        "PlayerCharacterID",
    )?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (save, entries) = save_reader.playable_class_progress()?;
    let retire_address = module_base + RETIRE_CHARACTER_RVA;
    runtime.memory.validate_code_prefix(
        retire_address,
        RETIRE_CHARACTER_PREFIX,
        "FSDSaveGame::RetireCharacter",
    )?;
    let backup_path = backup_save_games(&executable, "promote-all-classes")?;

    let mut changes = Vec::with_capacity(entries.len());
    for (entry, character_id) in entries.iter().zip(character_ids) {
        save_reader
            .memory
            .write_i32(entry.xp_address, MAX_CLASS_XP)?;
        let expected = entry.promotions.saturating_add(1);
        let returned = runtime.memory.call_remote_two_args(
            retire_address,
            save,
            character_id,
            &format!("Promote {}", entry.class_name),
        )? as i32;
        thread::sleep(Duration::from_millis(80));
        let current_promotions = save_reader.memory.read_i32(entry.promotions_address)?;
        let current_xp = save_reader.memory.read_i32(entry.xp_address)?;
        if returned != expected || current_promotions != expected || current_xp != 0 {
            return Err(MemoryError(format!(
                "A promocao de {} nao foi confirmada. Restaure o backup em {} antes de tentar novamente.",
                entry.class_name,
                backup_path.display()
            )));
        }
        changes.push(PromotionChange {
            class_name: entry.class_name.into(),
            previous_promotions: entry.promotions,
            current_promotions,
        });
    }

    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    runtime
        .memory
        .call_remote(save_address, save, "SaveToDisk")?;

    Ok(PromoteAllResult {
        pid,
        classes_promoted: changes.len(),
        changes,
        backup_path: backup_path.display().to_string(),
        message: "Driller, Engineer, Gunner e Scout receberam uma promocao nativa.".into(),
    })
}

pub fn max_class_level() -> Result<MaxClassLevelResult> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;

    let runtime = ClipReader::connect(pid, module_base, true)?;
    runtime.world_context()?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (save, entries) = save_reader.playable_class_xp()?;
    let backup_path = backup_save_games(&executable, "max-class-level")?;

    let mut written = Vec::with_capacity(entries.len());
    for (class_name, address, previous_xp) in &entries {
        if let Err(error) = save_reader.memory.write_i32(*address, MAX_CLASS_XP) {
            for (_, rollback_address, rollback_value) in written.iter().rev() {
                let _ = save_reader
                    .memory
                    .write_i32(*rollback_address, *rollback_value);
            }
            return Err(MemoryError(format!(
                "Falha ao atualizar {class_name}; os valores anteriores foram restaurados: {error}"
            )));
        }
        written.push((*class_name, *address, *previous_xp));
    }

    for (class_name, address, _) in &entries {
        let verified = save_reader.memory.read_i32(*address)?;
        if verified != MAX_CLASS_XP {
            for (_, rollback_address, rollback_value) in written.iter().rev() {
                let _ = save_reader
                    .memory
                    .write_i32(*rollback_address, *rollback_value);
            }
            return Err(MemoryError(format!(
                "A verificacao de {class_name} falhou; os valores anteriores foram restaurados."
            )));
        }
    }

    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    if let Err(error) = runtime.memory.call_remote(save_address, save, "SaveToDisk") {
        for (_, rollback_address, rollback_value) in written.iter().rev() {
            let _ = save_reader
                .memory
                .write_i32(*rollback_address, *rollback_value);
        }
        return Err(MemoryError(format!(
            "O save nao confirmou a gravacao; os valores anteriores foram restaurados: {error}"
        )));
    }

    let changes = entries
        .into_iter()
        .map(|(class_name, _, previous_xp)| ClassLevelChange {
            class_name: class_name.into(),
            previous_xp,
            current_xp: MAX_CLASS_XP,
        })
        .collect::<Vec<_>>();
    Ok(MaxClassLevelResult {
        pid,
        classes_updated: changes.len(),
        changes,
        backup_path: backup_path.display().to_string(),
        message: "Driller, Engineer, Gunner e Scout foram definidos no nivel 25.".into(),
    })
}
