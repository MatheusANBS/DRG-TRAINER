//! Adaptador do `FSDSaveGame` — leitura estruturada do save ativo.
//!
//! Aqui mora o conhecimento especifico do DRG sobre o layout do save. O runtime
//! Unreal permanece generico; este modulo traduz offsets do perfil em valores
//! de dominio, sempre validando faixas plausiveis antes de devolver o dado
//! (SPEC-001: nenhuma leitura corrompida vira base para uma escrita).

use std::collections::{HashMap, HashSet};

use crate::build_profiles::SaveOffsets;
use crate::infrastructure::process::MemoryReader;
use crate::infrastructure::unreal::{UnrealRuntime, cache};
use crate::shared::error::{Result, codes};
use crate::shared::limits::{MAX_RESOURCE_TOTAL, MAX_SAVE_COLLECTION};

use super::dto::ResourceSnapshot;

/// Nome da classe que precisa ser confirmado antes de qualquer leitura.
const SAVE_CLASS_NAME: &str = "FSDSaveGame";

/// Progresso de uma classe jogavel, com os enderecos ja resolvidos.
#[derive(Debug, Clone)]
pub struct CharacterProgress {
    pub class_name: &'static str,
    pub character_id_object: &'static str,
    pub xp_address: usize,
    pub xp: i32,
    pub promotions_address: usize,
    pub promotions: i32,
}

/// Acesso ao save ativo do jogo.
pub struct SaveGame<'a, M> {
    runtime: &'a UnrealRuntime<M>,
    class_address: usize,
}

// `Debug` manual: o backend de memoria nao precisa ser `Debug` so porque o
// adaptador aparece em uma mensagem de teste.
impl<M> std::fmt::Debug for SaveGame<'_, M> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SaveGame")
            .field("class_address", &format_args!("0x{:X}", self.class_address))
            .finish()
    }
}

impl<'a, M: MemoryReader> SaveGame<'a, M> {
    /// Resolve a `UClass` do save e confirma a identidade pelo nome.
    ///
    /// O indice sozinho nao basta: um indice defasado apontaria para outra
    /// classe valida e todas as leituras seguintes leriam lixo plausivel.
    pub fn resolve(runtime: &'a UnrealRuntime<M>) -> Result<Self> {
        let class_address =
            runtime.resolve_object(runtime.profile().offsets.save.fsd_savegame_class_index)?;
        let class_name = runtime.read_object_name(class_address)?;
        if class_name != SAVE_CLASS_NAME {
            return Err(codes::invalid_state(format!(
                "Validacao da classe falhou: esperado {SAVE_CLASS_NAME}, recebido {class_name:?}"
            )));
        }
        Ok(Self {
            runtime,
            class_address,
        })
    }

    fn memory(&self) -> &M {
        self.runtime.memory()
    }

    fn offsets(&self) -> &'static SaveOffsets {
        &self.runtime.profile().offsets.save
    }

    /// Um objeto e uma instancia do save se a classe e o nome batem.
    fn is_save_instance(&self, object: usize, name_index: u32) -> bool {
        let unreal = &self.runtime.profile().offsets.unreal;
        object != 0
            && self.memory().read_u64(object + unreal.uobject_class).ok()
                == Some(self.class_address as u64)
            && self.memory().read_u32(object + unreal.uobject_fname).ok() == Some(name_index)
    }

    /// Varre a `GUObjectArray` procurando a instancia ativa do save.
    fn scan_for_active_save(&self, name_index: u32) -> Result<usize> {
        let mut found = None;
        self.runtime.scan_objects(|object| {
            if self.is_save_instance(object, name_index) {
                found = Some(object);
                return Ok(crate::infrastructure::unreal::Scan::Stop);
            }
            Ok(crate::infrastructure::unreal::Scan::Continue)
        })?;
        found.ok_or_else(|| {
            codes::object_not_found("Nenhuma instancia ativa de FSDSaveGame foi encontrada.")
        })
    }

    /// Endereco da instancia ativa do save, com cache revalidado.
    ///
    /// Estrategia em tres passos: cache -> indice conhecido -> varredura.
    pub fn active_save(&self) -> Result<usize> {
        let unreal = &self.runtime.profile().offsets.unreal;
        let name_index = self
            .memory()
            .read_u32(self.class_address + unreal.uobject_fname)?;
        let key = self.runtime.cache_key();

        if let Some(cached) = cache::with(key, |caches| caches.active_save)?
            && self.is_save_instance(cached, name_index)
        {
            return Ok(cached);
        }

        if let Ok(save) = self
            .runtime
            .resolve_object(self.offsets().active_savegame_index)
            && self.is_save_instance(save, name_index)
        {
            cache::with(key, |caches| caches.active_save = Some(save))?;
            return Ok(save);
        }

        let save = self.scan_for_active_save(name_index)?;
        cache::with(key, |caches| caches.active_save = Some(save))?;
        Ok(save)
    }

    /// Endereco do campo de creditos no save ativo.
    pub fn credits_address(&self) -> Result<usize> {
        Ok(self.active_save()? + self.offsets().credits)
    }

    pub fn read_credits(&self) -> Result<(i32, usize)> {
        let address = self.credits_address()?;
        Ok((self.memory().read_i32(address)?, address))
    }

    /// Le o `TMap` de recursos e projeta sobre o catalogo do perfil.
    ///
    /// Recursos ausentes no mapa aparecem como zero; entradas invalidas abortam
    /// a leitura em vez de virar um numero plausivel.
    pub fn resource_amounts(&self) -> Result<(Vec<ResourceSnapshot>, usize)> {
        let offsets = self.offsets();
        let save = self.active_save()?;
        let map = save + offsets.resources_map;

        let elements = self.memory().read_pointer(map)?;
        let max_index = self.memory().read_i32(map + 8)?;
        let capacity = self.memory().read_i32(map + 12)?;
        let allocation_bits = self.memory().read_bytes(map + 0x10, 16)?;
        let allocation_count = self.memory().read_i32(map + 0x28)?;
        let allocation_capacity = self.memory().read_i32(map + 0x2C)?;
        if !(0..=128).contains(&max_index)
            || max_index > capacity
            || allocation_count != max_index
            || !(max_index..=128).contains(&allocation_capacity)
            || (max_index > 0 && elements == 0)
        {
            return Err(codes::invalid_state(format!(
                "Mapa de recursos invalido: elements=0x{elements:X}, max={max_index}, capacity={capacity}, flags={allocation_count}/{allocation_capacity}."
            )));
        }

        let mut values = HashMap::<[u8; 16], f32>::new();
        if max_index > 0 {
            let raw = self.memory().read_bytes(
                elements,
                max_index as usize * offsets.resource_map_element_size,
            )?;
            for index in 0..max_index as usize {
                let word_offset = index / 32 * 4;
                let word = u32::from_le_bytes(
                    allocation_bits[word_offset..word_offset + 4]
                        .try_into()
                        .unwrap(),
                );
                if word & (1 << (index % 32)) == 0 {
                    continue;
                }
                let offset = index * offsets.resource_map_element_size;
                let id: [u8; 16] = raw[offset..offset + 16].try_into().unwrap();
                let amount = f32::from_le_bytes(raw[offset + 16..offset + 20].try_into().unwrap());
                if id == [0; 16]
                    || !amount.is_finite()
                    || !(0.0..=MAX_RESOURCE_TOTAL).contains(&amount)
                {
                    return Err(codes::invalid_state(format!(
                        "Entrada de recurso invalida no indice {index}: amount={amount}."
                    )));
                }
                if values.insert(id, amount).is_some() {
                    return Err(codes::invalid_state(format!(
                        "GUID de recurso duplicado no indice {index}."
                    )));
                }
            }
        }

        Ok((
            self.runtime
                .profile()
                .resources
                .iter()
                .map(|definition| ResourceSnapshot {
                    id: definition.id.into(),
                    name: definition.name.into(),
                    category: definition.category.into(),
                    amount: values.get(&definition.savegame_id).copied().unwrap_or(0.0),
                })
                .collect(),
            save,
        ))
    }

    /// Contadores de itens desbloqueados e adquiridos.
    pub fn item_counts(&self) -> Result<(i32, i32, usize)> {
        let offsets = self.offsets();
        let save = self.active_save()?;
        let unlocked = self.memory().read_i32(save + offsets.unlocked_items + 8)?;
        let owned = self.memory().read_i32(save + offsets.owned_items + 8)?;
        if !(0..=MAX_SAVE_COLLECTION).contains(&unlocked)
            || !(0..=MAX_SAVE_COLLECTION).contains(&owned)
        {
            return Err(codes::invalid_state(format!(
                "Contadores de itens invalidos: unlocked={unlocked}, owned={owned}."
            )));
        }
        Ok((unlocked, owned, save))
    }

    /// Tamanho de um `TArray` de progressao no save.
    pub fn progression_count(&self, offset: usize, label: &str) -> Result<(i32, usize)> {
        let save = self.active_save()?;
        let count = self.memory().read_i32(save + offset + 8)?;
        if !(0..=MAX_SAVE_COLLECTION).contains(&count) {
            return Err(codes::invalid_state(format!(
                "Contador de {label} invalido: {count}."
            )));
        }
        Ok((count, save))
    }

    /// Conjunto de GUIDs de um `TArray` de progressao.
    pub fn progression_ids(
        &self,
        offset: usize,
        label: &str,
    ) -> Result<(HashSet<[u8; 16]>, usize)> {
        let save = self.active_save()?;
        let data = self.memory().read_pointer(save + offset)?;
        let count = self.memory().read_i32(save + offset + 8)?;
        let capacity = self.memory().read_i32(save + offset + 12)?;
        if !(0..=MAX_SAVE_COLLECTION).contains(&count)
            || count > capacity
            || capacity > MAX_SAVE_COLLECTION
        {
            return Err(codes::invalid_state(format!(
                "Lista de {label} invalida: count={count}, capacity={capacity}."
            )));
        }
        if count == 0 {
            return Ok((HashSet::new(), save));
        }
        if data == 0 {
            return Err(codes::invalid_state(format!(
                "A lista de {label} possui dados nulos."
            )));
        }
        let bytes = self.memory().read_bytes(data, count as usize * 16)?;
        Ok((
            bytes
                .chunks_exact(16)
                .map(|chunk| chunk.try_into().unwrap())
                .collect(),
            save,
        ))
    }

    /// Contadores de esquemas forjados e pendentes.
    pub fn schematic_counts(&self) -> Result<(i32, i32, usize)> {
        let offsets = self.offsets();
        let save = self.active_save()?;
        let schematics = save + offsets.schematic_save;
        let forged = self
            .memory()
            .read_i32(schematics + offsets.forged_schematics + 8)?;
        let owned = self
            .memory()
            .read_i32(schematics + offsets.owned_schematics + 8)?;
        if !(0..=MAX_SAVE_COLLECTION).contains(&forged)
            || !(0..=MAX_SAVE_COLLECTION).contains(&owned)
        {
            return Err(codes::invalid_state(format!(
                "Contadores de esquemas invalidos: forged={forged}, owned={owned}."
            )));
        }
        Ok((forged, owned, save))
    }

    /// GUIDs dos esquemas ja forjados, com deteccao de duplicatas.
    pub fn forged_schematic_ids(&self) -> Result<(HashSet<[u8; 16]>, usize)> {
        let offsets = self.offsets();
        let save = self.active_save()?;
        let array = save + offsets.schematic_save + offsets.forged_schematics;
        let data = self.memory().read_pointer(array)?;
        let count = self.memory().read_i32(array + 8)?;
        let capacity = self.memory().read_i32(array + 12)?;
        if !(0..=MAX_SAVE_COLLECTION).contains(&count)
            || count > capacity
            || capacity > MAX_SAVE_COLLECTION
        {
            return Err(codes::invalid_state(format!(
                "Lista de esquemas forjados invalida: count={count}, capacity={capacity}."
            )));
        }
        if count == 0 {
            return Ok((HashSet::new(), save));
        }
        if data == 0 {
            return Err(codes::invalid_state(
                "A lista de esquemas forjados possui dados nulos.",
            ));
        }
        let bytes = self.memory().read_bytes(data, count as usize * 16)?;
        let ids: HashSet<[u8; 16]> = bytes
            .chunks_exact(16)
            .map(|chunk| chunk.try_into().unwrap())
            .collect();
        if ids.len() != count as usize {
            return Err(codes::invalid_state(
                "A lista de esquemas forjados possui SavegameIDs duplicados.",
            ));
        }
        Ok((ids, save))
    }

    /// Progresso das quatro classes jogaveis, identificadas por GUID.
    ///
    /// A identificacao por GUID (e nao por posicao) e o que impede escrever XP
    /// na entrada errada quando a ordem do array muda.
    pub fn class_progress(&self) -> Result<(usize, Vec<CharacterProgress>)> {
        let offsets = self.offsets();
        let catalog = &self.runtime.profile().catalog;
        let save = self.active_save()?;

        let data = self.memory().read_pointer(save + offsets.character_saves)?;
        let count = self.memory().read_i32(save + offsets.character_saves + 8)?;
        let capacity = self
            .memory()
            .read_i32(save + offsets.character_saves + 12)?;
        if data == 0 || !(4..=16).contains(&count) || count > capacity || capacity > 32 {
            return Err(codes::invalid_state(format!(
                "CharacterSaves invalido: data=0x{data:X}, count={count}, capacity={capacity}."
            )));
        }

        let mut entries = Vec::with_capacity(catalog.playable_classes.len());
        for index in 0..count as usize {
            let entry = data + index * offsets.character_save_size;
            let guid = self.memory().read_guid(entry)?;
            let Some(class) = catalog
                .playable_classes
                .iter()
                .find(|class| class.savegame_id == guid)
            else {
                continue;
            };

            let xp_address = entry + offsets.character_xp;
            let xp = self.memory().read_i32(xp_address)?;
            if !(0..=catalog.max_class_xp).contains(&xp) {
                return Err(codes::invalid_state(format!(
                    "XP invalido para {}: {xp}.",
                    class.name
                )));
            }
            let promotions_address = entry + offsets.character_promotions;
            let promotions = self.memory().read_i32(promotions_address)?;
            if !(0..=MAX_SAVE_COLLECTION).contains(&promotions) {
                return Err(codes::invalid_state(format!(
                    "Promocoes invalidas para {}: {promotions}.",
                    class.name
                )));
            }

            entries.push(CharacterProgress {
                class_name: class.name,
                character_id_object: class.character_id_object,
                xp_address,
                xp,
                promotions_address,
                promotions,
            });
        }

        entries.sort_unstable_by_key(|entry| entry.class_name);
        entries.dedup_by_key(|entry| entry.class_name);
        if entries.len() != catalog.playable_classes.len() {
            return Err(codes::invalid_state(format!(
                "Foram encontradas {} das {} classes jogaveis no save.",
                entries.len(),
                catalog.playable_classes.len()
            )));
        }
        Ok((save, entries))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::testing::SaveWorld;
    use crate::shared::ErrorCode;

    #[test]
    fn resolves_the_active_save_and_reads_credits() {
        let world = SaveWorld::new();
        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).expect("classe do save deve ser resolvida");

        assert_eq!(save.active_save().unwrap(), world.save);
        let (credits, address) = save.read_credits().unwrap();
        assert_eq!(credits, 49_995);
        assert_eq!(
            address,
            world.save + crate::domain::testing::profile().offsets.save.credits
        );
    }

    #[test]
    fn refuses_a_class_whose_name_does_not_match() {
        let world = SaveWorld::new();
        world.rename_save_class("OutraClasse");
        let runtime = world.world.runtime();

        let error = SaveGame::resolve(&runtime).expect_err("nome divergente deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
        assert!(error.message().contains("FSDSaveGame"));
    }

    #[test]
    fn projects_the_resource_map_over_the_profile_catalog() {
        let world = SaveWorld::new();
        world.set_resource("bismor", 250.0);
        world.set_resource("croppa", 40.0);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let (resources, address) = save.resource_amounts().unwrap();

        assert_eq!(address, world.save);
        assert_eq!(resources.len(), profile().resources.len());
        let bismor = resources.iter().find(|entry| entry.id == "bismor").unwrap();
        assert_eq!(bismor.amount, 250.0);
        assert_eq!(bismor.name, "Bismor");
        // Recurso ausente do mapa aparece como zero, nao como erro.
        let data_cell = resources
            .iter()
            .find(|entry| entry.id == "data_cell")
            .unwrap();
        assert_eq!(data_cell.amount, 0.0);
    }

    #[test]
    fn rejects_a_resource_amount_outside_the_safe_range() {
        let world = SaveWorld::new();
        world.set_resource("bismor", f32::INFINITY);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let error = save
            .resource_amounts()
            .expect_err("valor nao finito deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
    }

    #[test]
    fn rejects_an_inconsistent_resource_map_header() {
        let world = SaveWorld::new();
        let map = world.save + profile().offsets.save.resources_map;
        // max_index maior que a capacidade.
        world.world.memory().poke_i32(map + 8, 64);
        world.world.memory().poke_i32(map + 12, 4);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        assert_eq!(
            save.resource_amounts()
                .expect_err("cabecalho invalido deve falhar")
                .code(),
            ErrorCode::InvalidGameState
        );
    }

    #[test]
    fn reads_the_four_playable_classes_by_guid() {
        let world = SaveWorld::new();
        world.set_class_progress("Scout", 3_048, 0);
        world.set_class_progress("Gunner", 100_000, 2);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let (address, classes) = save.class_progress().unwrap();

        assert_eq!(address, world.save);
        assert_eq!(classes.len(), 4);
        let names = classes
            .iter()
            .map(|entry| entry.class_name)
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["Driller", "Engineer", "Gunner", "Scout"]);
        let scout = classes
            .iter()
            .find(|entry| entry.class_name == "Scout")
            .unwrap();
        assert_eq!(scout.xp, 3_048);
        let gunner = classes
            .iter()
            .find(|entry| entry.class_name == "Gunner")
            .unwrap();
        assert_eq!(gunner.promotions, 2);
    }

    #[test]
    fn rejects_class_xp_above_the_profile_maximum() {
        let world = SaveWorld::new();
        world.set_class_progress("Driller", profile().catalog.max_class_xp + 1, 0);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let error = save
            .class_progress()
            .expect_err("XP acima do maximo deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
        assert!(error.message().contains("Driller"));
    }

    #[test]
    fn a_missing_class_aborts_instead_of_returning_a_partial_list() {
        let world = SaveWorld::new();
        world.corrupt_class_guid("Engineer");

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let error = save
            .class_progress()
            .expect_err("classe ausente deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
        assert!(error.message().contains("3 das 4"));
    }

    #[test]
    fn item_counters_outside_the_plausible_range_are_refused() {
        let world = SaveWorld::new();
        let offsets = profile().offsets.save;
        world
            .world
            .memory()
            .poke_i32(world.save + offsets.unlocked_items + 8, -5);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        assert_eq!(
            save.item_counts()
                .expect_err("contador negativo deve falhar")
                .code(),
            ErrorCode::InvalidGameState
        );
    }

    #[test]
    fn progression_ids_reads_the_guid_list() {
        let world = SaveWorld::new();
        let offsets = profile().offsets.save;
        let ids = [[1_u8; 16], [2_u8; 16], [3_u8; 16]];
        world.write_guid_array(offsets.owned_perks, &ids);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let (found, _) = save.progression_ids(offsets.owned_perks, "perks").unwrap();
        assert_eq!(found.len(), 3);
        assert!(found.contains(&[2_u8; 16]));

        let (count, _) = save
            .progression_count(offsets.owned_perks, "perks")
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn a_null_data_pointer_with_a_positive_count_is_refused() {
        let world = SaveWorld::new();
        let offsets = profile().offsets.save;
        world
            .world
            .memory()
            .poke_u64(world.save + offsets.owned_perks, 0);
        world
            .world
            .memory()
            .poke_i32(world.save + offsets.owned_perks + 8, 5);
        world
            .world
            .memory()
            .poke_i32(world.save + offsets.owned_perks + 12, 8);

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        assert_eq!(
            save.progression_ids(offsets.owned_perks, "perks")
                .expect_err("ponteiro nulo deve falhar")
                .code(),
            ErrorCode::InvalidGameState
        );
    }

    #[test]
    fn forged_schematics_reject_duplicate_ids() {
        let world = SaveWorld::new();
        let offsets = profile().offsets.save;
        world.write_guid_array(
            offsets.schematic_save + offsets.forged_schematics,
            &[[9_u8; 16], [9_u8; 16]],
        );

        let runtime = world.world.runtime();
        let save = SaveGame::resolve(&runtime).unwrap();
        let error = save
            .forged_schematic_ids()
            .expect_err("ids duplicados devem falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
        assert!(error.message().contains("duplicados"));
    }

    fn profile() -> &'static crate::build_profiles::BuildProfile {
        crate::domain::testing::profile()
    }
}
