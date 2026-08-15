//! Overclocks e esquemas cosmeticos.
//!
//! Esta e a operacao mais delicada do trainer: em vez de chamar
//! `Cheat_Schematic_UnlockAll` — que dispara analytics sem checar ponteiro nulo
//! e derruba builds shipping — ela concede cada recompensa individualmente e
//! depois marca os esquemas como forjados, alterando apenas o `SchematicSave`.

use std::collections::HashSet;

use crate::build_profiles::{BuildProfile, Capability, NativeFunction};
use crate::domain::dto::SchematicUnlockResult;
use crate::domain::save_game::SaveGame;
use crate::domain::session::TrainerSession;
use crate::infrastructure::process::{MemoryReader, NativeCall};
use crate::infrastructure::unreal::UnrealRuntime;
use crate::shared::error::{Result, codes};

use super::{SETTLE_DELAY, persist_to_disk};

/// Limites plausiveis do `TSet` de esquemas. Fora disso o dado e lixo.
const MAX_SCHEMATIC_COUNT: i32 = 2_000;
const MAX_SCHEMATIC_INDEX: i32 = 2_500;

/// Tipo de recompensa concedida por um esquema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardKind {
    Overclock,
    Skin,
    Vanity,
    VictoryPose,
}

impl RewardKind {
    /// Rotina nativa correspondente no perfil.
    fn native(self, profile: &'static BuildProfile) -> NativeFunction {
        match self {
            Self::Overclock => profile.natives.overclock_reward,
            Self::Skin => profile.natives.skin_reward,
            Self::Vanity => profile.natives.vanity_reward,
            Self::VictoryPose => profile.natives.victory_pose_reward,
        }
    }

    /// Classe do item que corresponde a este tipo de recompensa.
    fn from_class_name(class_name: &str) -> Option<Self> {
        match class_name {
            "OverclockShematicItem" => Some(Self::Overclock),
            "SkinSchematicItem" => Some(Self::Skin),
            "VanitySchematicItem" => Some(Self::Vanity),
            "VictoryPoseSchematicItem" => Some(Self::VictoryPose),
            _ => None,
        }
    }

    fn is_cosmetic(self) -> bool {
        !matches!(self, Self::Overclock)
    }
}

/// Um esquema pronto para receber recompensa.
struct RewardTarget {
    schematic: usize,
    item: usize,
    kind: RewardKind,
    /// Presente apenas em overclocks: usado para confirmar a concessao.
    overclock_id: Option<[u8; 16]>,
}

/// Le `GD_SchematicSettings.AllSchematics` e devolve (endereco, SavegameID).
fn all_schematics<M: MemoryReader>(runtime: &UnrealRuntime<M>) -> Result<Vec<(usize, [u8; 16])>> {
    let offsets = &runtime.profile().offsets.save;
    let memory = runtime.memory();

    let settings = runtime.find_named_objects(&["GD_SchematicSettings"], "SchematicSettings")?[0];
    let set = settings + offsets.schematic_settings_all_schematics;

    let elements = memory.read_pointer(set)?;
    let count = memory.read_i32(set + 8)?;
    let max_index = memory.read_i32(set + offsets.tset_max_index)?;
    let flags = memory.read_pointer(set + offsets.tset_allocation_flags)?;
    if elements == 0
        || flags == 0
        || !(1..=MAX_SCHEMATIC_COUNT).contains(&count)
        || !(count..=MAX_SCHEMATIC_INDEX).contains(&max_index)
    {
        return Err(codes::invalid_state(format!(
            "AllSchematics invalido: elements=0x{elements:X}, flags=0x{flags:X}, count={count}, max_index={max_index}."
        )));
    }

    let allocation = memory.read_bytes(flags, (max_index as usize).div_ceil(32) * 4)?;
    let raw_elements =
        memory.read_bytes(elements, max_index as usize * offsets.tset_element_size)?;

    let mut schematics = Vec::with_capacity(count as usize);
    let mut ids = HashSet::with_capacity(count as usize);
    for index in 0..max_index as usize {
        let word_offset = index / 32 * 4;
        let word = u32::from_le_bytes(allocation[word_offset..word_offset + 4].try_into().unwrap());
        if word & (1 << (index % 32)) == 0 {
            continue;
        }
        let offset = index * offsets.tset_element_size;
        let schematic =
            u64::from_le_bytes(raw_elements[offset..offset + 8].try_into().unwrap()) as usize;
        if schematic == 0 || !runtime.is_instance_of(schematic, "Schematic")? {
            return Err(codes::invalid_state(format!(
                "Entrada invalida em AllSchematics[{index}]: 0x{schematic:X}."
            )));
        }
        let id = memory.read_guid(schematic + offsets.schematic_savegame_id)?;
        if id == [0; 16] || !ids.insert(id) {
            return Err(codes::invalid_state(format!(
                "SavegameID invalido ou duplicado em AllSchematics[{index}]."
            )));
        }
        schematics.push((schematic, id));
    }

    if schematics.len() != count as usize {
        return Err(codes::invalid_state(format!(
            "AllSchematics incompleto: esperado {count}, lido {}.",
            schematics.len()
        )));
    }
    Ok(schematics)
}

/// Classifica cada esquema e confere que a vtable aponta para a rotina do perfil.
///
/// A checagem da vtable e o que impede chamar `GrantReward` de um tipo em um
/// item de outro tipo caso o catalogo mude.
fn reward_targets<M: MemoryReader>(
    runtime: &UnrealRuntime<M>,
    schematics: &[(usize, [u8; 16])],
) -> Result<Vec<RewardTarget>> {
    let profile = runtime.profile();
    let offsets = &profile.offsets.save;
    let unreal = &profile.offsets.unreal;
    let memory = runtime.memory();

    let mut targets = Vec::with_capacity(profile.catalog.total_schematic_count());
    let mut overclocks = 0_usize;
    let mut cosmetics = 0_usize;

    for (schematic, _) in schematics {
        let item = memory.read_pointer(*schematic + offsets.schematic_item)?;
        if item == 0 {
            return Err(codes::invalid_state(format!(
                "Schematic.Item nulo em 0x{schematic:X}."
            )));
        }
        let class = memory.read_pointer(item + unreal.uobject_class)?;
        let class_name = runtime.read_object_name(class)?;

        // Esquemas em branco e de recurso nao concedem recompensa.
        if matches!(
            class_name.as_str(),
            "BlankSchematicItem" | "ResourceSchematicItem"
        ) {
            continue;
        }
        let kind = RewardKind::from_class_name(&class_name).ok_or_else(|| {
            codes::invalid_state(format!(
                "Tipo de recompensa de esquema nao reconhecido: {class_name}."
            ))
        })?;
        if kind.is_cosmetic() {
            cosmetics += 1;
        } else {
            overclocks += 1;
        }

        let native = kind.native(profile);
        let vtable = memory.read_pointer(item)?;
        let virtual_reward = memory.read_pointer(vtable + unreal.schematic_reward_vtable_slot)?;
        if virtual_reward != native.address(runtime.module_base()) {
            return Err(codes::invalid_state(format!(
                "Rotina virtual invalida para {}: 0x{virtual_reward:X}.",
                native.label
            )));
        }

        let overclock_id = if kind == RewardKind::Overclock {
            let overclock = memory.read_pointer(item + offsets.overclock_item_overclock)?;
            if overclock == 0 {
                return Err(codes::invalid_state(
                    "OverclockShematicItem.Overclock esta nulo.",
                ));
            }
            Some(memory.read_guid(overclock + offsets.schematic_savegame_id)?)
        } else {
            None
        };

        targets.push(RewardTarget {
            schematic: *schematic,
            item,
            kind,
            overclock_id,
        });
    }

    let catalog = &profile.catalog;
    if overclocks != catalog.overclock_schematic_count
        || cosmetics != catalog.cosmetic_schematic_count
    {
        return Err(codes::invalid_state(format!(
            "Catalogo de recompensas inesperado: overclocks={overclocks}, cosmeticos={cosmetics}."
        )));
    }
    Ok(targets)
}

pub fn unlock_all_overclocks_and_cosmetics() -> Result<SchematicUnlockResult> {
    let session = TrainerSession::writable()?;
    session.require(Capability::SchematicUnlock)?;

    let runtime = session.runtime();
    let profile = session.profile();
    let offsets = &profile.offsets.save;
    runtime.world_context()?;

    let save_game = SaveGame::resolve(runtime)?;
    let (forged_before, owned_before, save) = save_game.schematic_counts()?;
    let (forged_ids, verified_save) = save_game.forged_schematic_ids()?;
    if verified_save != save || forged_ids.len() != forged_before as usize {
        return Err(codes::save_state_changed(
            "O save ativo mudou durante a leitura dos esquemas.",
        ));
    }

    let schematics = all_schematics(runtime)?;
    let targets = reward_targets(runtime, &schematics)?;

    let purchased_label = "upgrades e overclocks adquiridos";
    let (purchased_before, purchased_save) =
        save_game.progression_ids(offsets.purchased_item_upgrades, purchased_label)?;
    if purchased_save != save {
        return Err(codes::save_state_changed(
            "O save ativo mudou durante a leitura das recompensas.",
        ));
    }

    let pending = schematics
        .iter()
        .filter(|(_, id)| !forged_ids.contains(id))
        .collect::<Vec<_>>();

    // Valida toda rotina que sera usada antes de criar o backup.
    let forge = profile.natives.forge_schematic_save;
    runtime.verify_native(forge)?;
    runtime.verify_native(profile.natives.save_to_disk)?;
    for kind in [
        RewardKind::Overclock,
        RewardKind::Skin,
        RewardKind::Vanity,
        RewardKind::VictoryPose,
    ] {
        runtime.verify_native(kind.native(profile))?;
    }

    let transaction = session.begin_save_transaction("unlock-all-schematics")?;

    let mut overclocks_processed = 0_usize;
    let mut cosmetics_processed = 0_usize;
    for target in &targets {
        // Overclock ja adquirido nao precisa de nova concessao.
        if target
            .overclock_id
            .is_some_and(|id| purchased_before.contains(&id))
        {
            continue;
        }
        transaction.guard(
            runtime
                .call_verified(
                    target.kind.native(profile),
                    NativeCall::ThreeArgs {
                        first: target.item,
                        second: target.schematic,
                        third: save,
                    },
                )
                .map(|_| ()),
        )?;
        if target.kind.is_cosmetic() {
            cosmetics_processed += 1;
        } else {
            overclocks_processed += 1;
        }
    }

    let schematic_save = save + offsets.schematic_save;
    for (schematic, _) in &pending {
        transaction.guard(
            runtime
                .call_verified(
                    forge,
                    NativeCall::ThreeArgs {
                        first: schematic_save,
                        second: *schematic,
                        third: 0,
                    },
                )
                .map(|_| ()),
        )?;
    }
    std::thread::sleep(SETTLE_DELAY / 2);

    let (forged_after, owned_after, verified_save) =
        transaction.guard(save_game.schematic_counts())?;
    transaction.ensure_same_save(save, verified_save)?;
    transaction.verify(
        forged_after == forged_before + pending.len() as i32 && owned_after <= owned_before,
        format!(
            "A verificacao dos esquemas falhou: forjados {forged_before} -> {forged_after} (esperado +{}), pendentes {owned_before} -> {owned_after}",
            pending.len()
        ),
    )?;

    let (purchased_after, purchased_save) = transaction
        .guard(save_game.progression_ids(offsets.purchased_item_upgrades, purchased_label))?;
    transaction.ensure_same_save(save, purchased_save)?;
    let missing_overclocks = targets
        .iter()
        .filter_map(|target| target.overclock_id)
        .filter(|id| !purchased_after.contains(id))
        .count();
    transaction.verify(
        missing_overclocks == 0,
        format!("A concessao dos overclocks nao foi confirmada ({missing_overclocks} ausentes)"),
    )?;

    persist_to_disk(runtime, save, &transaction)?;

    Ok(SchematicUnlockResult {
        pid: session.pid(),
        overclocks_processed,
        cosmetics_processed,
        forged_before,
        forged_after,
        owned_before,
        owned_after,
        backup: transaction.commit(),
        message: format!(
            "Recompensas processadas: {overclocks_processed} overclocks e {cosmetics_processed} cosmeticos. Esquemas concluidos: {forged_before} -> {forged_after}; pendentes: {owned_before} -> {owned_after}."
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::testing::profile;

    #[test]
    fn reward_kinds_map_to_the_documented_item_classes() {
        assert_eq!(
            RewardKind::from_class_name("OverclockShematicItem"),
            Some(RewardKind::Overclock)
        );
        assert_eq!(
            RewardKind::from_class_name("SkinSchematicItem"),
            Some(RewardKind::Skin)
        );
        assert_eq!(
            RewardKind::from_class_name("VanitySchematicItem"),
            Some(RewardKind::Vanity)
        );
        assert_eq!(
            RewardKind::from_class_name("VictoryPoseSchematicItem"),
            Some(RewardKind::VictoryPose)
        );
        assert_eq!(
            RewardKind::from_class_name("SchematicItemDesconhecido"),
            None
        );
    }

    #[test]
    fn only_overclocks_are_not_cosmetic() {
        assert!(!RewardKind::Overclock.is_cosmetic());
        for kind in [
            RewardKind::Skin,
            RewardKind::Vanity,
            RewardKind::VictoryPose,
        ] {
            assert!(kind.is_cosmetic());
        }
    }

    #[test]
    fn each_reward_kind_resolves_to_a_distinct_profile_routine() {
        let profile = profile();
        let overclock = RewardKind::Overclock.native(profile);
        let skin = RewardKind::Skin.native(profile);
        assert_ne!(overclock.rva, skin.rva);
        assert_eq!(overclock.label, "OverclockShematicItem::GrantReward");
        assert_eq!(skin.label, "SkinSchematicItem::GrantReward");
    }

    #[test]
    fn blank_and_resource_schematics_are_skipped_rather_than_failing() {
        // A regra de negocio vive em `from_class_name` + o filtro explicito.
        assert_eq!(RewardKind::from_class_name("BlankSchematicItem"), None);
        assert_eq!(RewardKind::from_class_name("ResourceSchematicItem"), None);
    }
}
