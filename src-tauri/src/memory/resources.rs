use super::*;

pub fn read_resources() -> Result<Vec<ResourceSnapshot>> {
    let (_, reader) = context(false)?;
    reader.resource_amounts().map(|(resources, _)| resources)
}

fn add_resources(
    definitions: &[&ResourceDefinition],
    amount: i32,
    backup_label: &str,
) -> Result<ResourceWriteResult> {
    if !(1..=MAX_RESOURCE_DELTA).contains(&amount) {
        return Err(MemoryError(format!(
            "A quantidade deve estar entre 1 e {MAX_RESOURCE_DELTA}."
        )));
    }

    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;
    let runtime = ClipReader::connect(pid, module_base, true)?;
    runtime.world_context()?;
    let save_reader = CreditsReader::connect(pid, module_base, true)?;
    let (before, save) = save_reader.resource_amounts()?;
    for definition in definitions {
        let current = before
            .iter()
            .find(|resource| resource.id == definition.id)
            .map(|resource| resource.amount)
            .unwrap_or(0.0);
        if current + amount as f32 > MAX_RESOURCE_TOTAL {
            return Err(MemoryError(format!(
                "{} excederia o limite seguro de {:.0}.",
                definition.name, MAX_RESOURCE_TOTAL
            )));
        }
    }

    let object_names = definitions
        .iter()
        .map(|definition| definition.object_name)
        .collect::<Vec<_>>();
    let objects = runtime.find_named_instances(&object_names, "ResourceData")?;
    for (definition, object) in definitions.iter().zip(&objects) {
        let actual: [u8; 16] = runtime
            .memory
            .read_bytes(*object + RESOURCE_SAVEGAME_ID_OFFSET, 16)?
            .try_into()
            .unwrap();
        if actual != definition.savegame_id {
            return Err(MemoryError(format!(
                "O SavegameID de {} nao corresponde ao catalogo validado.",
                definition.name
            )));
        }
    }

    let add_address = module_base + ADD_RESOURCE_RVA;
    runtime.memory.validate_code_prefix(
        add_address,
        ADD_RESOURCE_PREFIX,
        "FSDSaveGame::AddResource",
    )?;
    let save_address = module_base + SAVE_TO_DISK_RVA;
    runtime.memory.validate_code_prefix(
        save_address,
        SAVE_TO_DISK_PREFIX,
        "FSDSaveGame::SaveToDisk",
    )?;
    let backup_path = backup_save_games(&executable, backup_label)?;

    let mut applied = Vec::with_capacity(objects.len());
    for (definition, object) in definitions.iter().zip(&objects) {
        if let Err(error) = runtime.memory.call_remote_resource_delta(
            add_address,
            save,
            *object,
            amount as f32,
            &format!("Adicionar {}", definition.name),
        ) {
            for rollback_object in applied.into_iter().rev() {
                let _ = runtime.memory.call_remote_resource_delta(
                    add_address,
                    save,
                    rollback_object,
                    -(amount as f32),
                    "Restaurar recurso",
                );
            }
            return Err(MemoryError(format!(
                "Falha ao adicionar {}. As alteracoes em memoria foram revertidas; o backup esta em {}: {error}",
                definition.name,
                backup_path.display()
            )));
        }
        applied.push(*object);
    }

    let (after, verified_save) = save_reader.resource_amounts()?;
    if verified_save != save {
        return Err(MemoryError(format!(
            "O save ativo mudou durante a operacao. O backup permanece em {}.",
            backup_path.display()
        )));
    }
    for definition in definitions {
        let previous = before
            .iter()
            .find(|resource| resource.id == definition.id)
            .map(|resource| resource.amount)
            .unwrap_or(0.0);
        let current = after
            .iter()
            .find(|resource| resource.id == definition.id)
            .map(|resource| resource.amount)
            .unwrap_or(0.0);
        if (current - (previous + amount as f32)).abs() > 0.01 {
            return Err(MemoryError(format!(
                "A verificacao de {} falhou: {:.0} -> {:.0}. O backup permanece em {}.",
                definition.name,
                previous,
                current,
                backup_path.display()
            )));
        }
    }

    runtime
        .memory
        .call_remote(save_address, save, "SaveToDisk")?;
    Ok(ResourceWriteResult {
        pid,
        amount_added: amount,
        resources: after,
        backup_path: backup_path.display().to_string(),
        message: if definitions.len() == 1 {
            format!("{} adicionado: +{amount}.", definitions[0].name)
        } else {
            format!(
                "{amount} unidades adicionadas a {} recursos.",
                definitions.len()
            )
        },
    })
}

pub fn add_resource(resource_id: &str, amount: i32) -> Result<ResourceWriteResult> {
    let definition = RESOURCE_DEFINITIONS
        .iter()
        .find(|definition| definition.id == resource_id)
        .ok_or_else(|| MemoryError(format!("Recurso desconhecido: {resource_id}.")))?;
    add_resources(
        &[definition],
        amount,
        &format!("add-resource-{}", definition.id),
    )
}

pub fn add_all_resources(amount: i32) -> Result<ResourceWriteResult> {
    let definitions = RESOURCE_DEFINITIONS.iter().collect::<Vec<_>>();
    add_resources(&definitions, amount, "add-all-resources")
}
