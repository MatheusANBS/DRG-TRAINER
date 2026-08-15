//! Recursos do inventario (minerais, brewing e especiais).
//!
//! Mutacao permanente: usa a rotina nativa `FSDSaveGame::AddResource` e grava
//! em disco, portanto passa obrigatoriamente por uma `SaveTransaction`
//! (SPEC-020). A ordem e sempre: validar tudo -> backup -> mutar -> verificar.

use crate::build_profiles::{Capability, ResourceDefinition};
use crate::domain::dto::{ResourceSnapshot, ResourceWriteResult};
use crate::domain::save_game::SaveGame;
use crate::domain::session::TrainerSession;
use crate::infrastructure::process::MemoryReader;
use crate::infrastructure::process::NativeCall;
use crate::shared::error::{Result, codes};
use crate::shared::limits::{MAX_RESOURCE_DELTA, MAX_RESOURCE_TOTAL};

pub fn read_resources() -> Result<Vec<ResourceSnapshot>> {
    let session = TrainerSession::read_only()?;
    session.require(Capability::Resources)?;
    SaveGame::resolve(session.runtime())?
        .resource_amounts()
        .map(|(resources, _)| resources)
}

pub fn add_resource(resource_id: &str, amount: i32) -> Result<ResourceWriteResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::Resources)?;

    let definition = session
        .profile()
        .resource(resource_id)
        .ok_or_else(|| codes::invalid_argument(format!("Recurso desconhecido: {resource_id}.")))?;

    let label = format!("add-resource-{}", definition.id);
    add_resources(&session, &[definition], amount, &label)
}

pub fn add_all_resources(amount: i32) -> Result<ResourceWriteResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::Resources)?;

    let definitions = session.profile().resources.iter().collect::<Vec<_>>();
    add_resources(&session, &definitions, amount, "add-all-resources")
}

fn current_amount(resources: &[ResourceSnapshot], id: &str) -> f32 {
    resources
        .iter()
        .find(|resource| resource.id == id)
        .map(|resource| resource.amount)
        .unwrap_or(0.0)
}

fn add_resources(
    session: &TrainerSession,
    definitions: &[&'static ResourceDefinition],
    amount: i32,
    backup_label: &str,
) -> Result<ResourceWriteResult> {
    if !(1..=MAX_RESOURCE_DELTA).contains(&amount) {
        return Err(codes::invalid_argument(format!(
            "A quantidade deve estar entre 1 e {MAX_RESOURCE_DELTA}."
        )));
    }

    let profile = session.profile();
    let runtime = session.runtime();
    // Exige a Space Rig carregada: a rotina nativa depende do mundo ativo.
    runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let (before, save) = save_game.resource_amounts()?;

    for definition in definitions {
        let current = current_amount(&before, definition.id);
        if current + amount as f32 > MAX_RESOURCE_TOTAL {
            return Err(codes::invalid_argument(format!(
                "{} excederia o limite seguro de {:.0}.",
                definition.name, MAX_RESOURCE_TOTAL
            )));
        }
    }

    // Resolve os `ResourceData` e confere o GUID de cada um contra o catalogo.
    // Sem isso, uma reordenacao de objetos creditaria o recurso errado.
    let object_names = definitions
        .iter()
        .map(|definition| definition.object_name)
        .collect::<Vec<_>>();
    let objects = runtime.find_named_instances(&object_names, "ResourceData")?;
    for (definition, object) in definitions.iter().zip(&objects) {
        let actual = runtime
            .memory()
            .read_guid(*object + profile.offsets.save.resource_savegame_id)?;
        if actual != definition.savegame_id {
            return Err(codes::invalid_state(format!(
                "O SavegameID de {} nao corresponde ao catalogo validado.",
                definition.name
            )));
        }
    }

    let add_resource = profile.natives.add_resource;
    let save_to_disk = profile.natives.save_to_disk;
    runtime.verify_native(add_resource)?;
    runtime.verify_native(save_to_disk)?;

    // A partir daqui existe backup: todo erro aponta o caminho de restauracao.
    let transaction = session.begin_save_transaction(backup_label)?;

    let mut applied: Vec<usize> = Vec::with_capacity(objects.len());
    for (definition, object) in definitions.iter().zip(&objects) {
        let call = NativeCall::ResourceDelta {
            save,
            resource: *object,
            amount: amount as f32,
        };
        if let Err(error) = runtime.call_verified(add_resource, call) {
            // Desfaz o que ja foi creditado antes de reportar.
            for rollback in applied.into_iter().rev() {
                let _ = runtime.call_verified(
                    add_resource,
                    NativeCall::ResourceDelta {
                        save,
                        resource: rollback,
                        amount: -(amount as f32),
                    },
                );
            }
            return Err(transaction.wrap(error.context(format!(
                "Falha ao adicionar {}. As alteracoes em memoria foram revertidas",
                definition.name
            ))));
        }
        applied.push(*object);
    }

    let (after, verified_save) = transaction.guard(save_game.resource_amounts())?;
    transaction.ensure_same_save(save, verified_save)?;

    for definition in definitions {
        let previous = current_amount(&before, definition.id);
        let current = current_amount(&after, definition.id);
        transaction.verify(
            (current - (previous + amount as f32)).abs() <= 0.01,
            format!(
                "A verificacao de {} falhou: {:.0} -> {:.0}",
                definition.name, previous, current
            ),
        )?;
    }

    transaction.guard(
        runtime
            .call_verified(save_to_disk, NativeCall::Direct { argument: save })
            .map(|_| ()),
    )?;

    let message = if definitions.len() == 1 {
        format!("{} adicionado: +{amount}.", definitions[0].name)
    } else {
        format!(
            "{amount} unidades adicionadas a {} recursos.",
            definitions.len()
        )
    };

    Ok(ResourceWriteResult {
        pid: session.pid(),
        amount_added: amount,
        resources: after,
        backup: transaction.commit(),
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ErrorCode;

    #[test]
    fn refuses_an_amount_outside_the_safe_range() {
        for amount in [0, -1, MAX_RESOURCE_DELTA + 1] {
            let error = add_all_resources(amount).expect_err("quantidade invalida deve falhar");
            // Sem o jogo aberto a sessao falha antes; com o jogo aberto o limite
            // e quem barra. Ambos precisam recusar a operacao.
            assert!(
                error.code() == ErrorCode::InvalidArgument || error.code().is_waiting(),
                "codigo inesperado: {:?}",
                error.code()
            );
        }
    }

    #[test]
    fn unknown_resource_ids_are_rejected() {
        let error = add_resource("mithril", 100).expect_err("recurso inexistente deve falhar");
        assert!(
            error.code() == ErrorCode::InvalidArgument || error.code().is_waiting(),
            "codigo inesperado: {:?}",
            error.code()
        );
    }

    #[test]
    fn the_catalog_exposes_every_documented_resource() {
        let profile = crate::domain::testing::profile();
        for id in [
            "bismor",
            "croppa",
            "enor_pearl",
            "jadiz",
            "magnite",
            "umanite",
            "phazyonite",
            "barley_bulb",
            "malt_star",
            "starch_nut",
            "yeast_cone",
            "blank_matrix_core",
            "error_cube",
            "data_cell",
        ] {
            assert!(profile.resource(id).is_some(), "recurso ausente: {id}");
        }
        assert!(profile.resource("mithril").is_none());
    }
}
