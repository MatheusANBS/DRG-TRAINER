//! Resolucao e travessia da `GUObjectArray` (SPEC-005).
//!
//! Responsabilidade unica: encontrar objetos. A varredura de chunks existe em
//! um unico lugar (`scan_objects`); quem precisa de um criterio diferente passa
//! um visitor, em vez de reescrever o laco.

use crate::infrastructure::process::MemoryReader;
use crate::shared::error::{Result, codes};

use super::cache;
use super::types::UnrealRuntime;

/// Decisao do visitor de varredura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scan {
    Continue,
    Stop,
}

impl<M: MemoryReader> UnrealRuntime<M> {
    /// Quantidade de objetos vivos registrados.
    pub fn object_count(&self) -> Result<u32> {
        self.memory
            .read_u32(self.guobject_array() + self.unreal().num_elements)
    }

    /// Resolve um objeto pelo indice global, conferindo o indice interno.
    ///
    /// A checagem cruzada existe porque um indice defasado apontaria para outro
    /// objeto valido, e a leitura seguinte pareceria bem-sucedida.
    pub fn resolve_object(&self, index: u32) -> Result<usize> {
        let offsets = self.unreal();
        let count = self.object_count()?;
        if index >= count {
            return Err(codes::object_not_found(format!(
                "O objeto {index} ainda nao foi carregado ({count} objetos disponiveis)."
            )));
        }
        let chunks = self
            .memory
            .read_pointer(self.guobject_array() + offsets.objects_member)?;
        let chunk_index = index / offsets.objects_per_chunk;
        let within_chunk = index % offsets.objects_per_chunk;
        let chunk = self
            .memory
            .read_pointer(chunks + chunk_index as usize * 8)?;
        if chunk == 0 {
            return Err(codes::invalid_state(format!(
                "Chunk {chunk_index} da GUObjectArray e nulo."
            )));
        }
        let object = self
            .memory
            .read_pointer(chunk + within_chunk as usize * offsets.uobject_item_size)?;
        if object == 0 {
            return Err(codes::object_not_found(format!(
                "Objeto Unreal {index} e nulo."
            )));
        }
        let actual = self
            .memory
            .read_u32(object + offsets.uobject_internal_index)?;
        if actual != index {
            return Err(codes::invalid_state(format!(
                "Indice interno invalido: esperado {index}, recebido {actual}."
            )));
        }
        Ok(object)
    }

    /// Percorre todos os objetos vivos, do mais recente para o mais antigo.
    ///
    /// A ordem reversa e intencional: os objetos de sessao (controllers, saves
    /// ativos) sao criados por ultimo, entao a busca termina mais cedo.
    pub(crate) fn scan_objects<F>(&self, mut visit: F) -> Result<()>
    where
        F: FnMut(usize) -> Result<Scan>,
    {
        let offsets = self.unreal();
        let count = self.object_count()?;
        if count == 0 {
            return Ok(());
        }
        let chunks = self
            .memory
            .read_pointer(self.guobject_array() + offsets.objects_member)?;
        let chunk_count = count.div_ceil(offsets.objects_per_chunk);

        for chunk_index in (0..chunk_count).rev() {
            let chunk = self
                .memory
                .read_pointer(chunks + chunk_index as usize * 8)?;
            if chunk == 0 {
                continue;
            }
            let slots = if chunk_index + 1 == chunk_count {
                count - chunk_index * offsets.objects_per_chunk
            } else {
                offsets.objects_per_chunk
            };
            let bytes = self
                .memory
                .read_bytes(chunk, slots as usize * offsets.uobject_item_size)?;

            for slot in (0..slots as usize).rev() {
                let offset = slot * offsets.uobject_item_size;
                let object =
                    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
                if object == 0 {
                    continue;
                }
                if visit(object)? == Scan::Stop {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// Indice de FName de um objeto, tolerando leitura falha (slot reciclado).
    pub(super) fn object_name_index(&self, object: usize) -> Option<u32> {
        self.memory
            .read_u32(object + self.unreal().uobject_fname)
            .ok()
    }

    /// Objetos "de catalogo" tem FName Number zero; instancias numeradas sao
    /// copias de runtime e nunca sao o alvo procurado.
    pub(super) fn is_catalog_instance(&self, object: usize) -> bool {
        self.memory
            .read_u32(object + self.unreal().uobject_fname_number)
            .ok()
            == Some(0)
    }

    /// Encontra objetos por nome exigindo correspondencia exata de classe.
    pub fn find_named_objects(
        &self,
        object_names: &[&str],
        class_name: &str,
    ) -> Result<Vec<usize>> {
        let offsets = self.unreal();
        let class_name_index = self.find_fname_index(class_name)?;
        let name_indices = object_names
            .iter()
            .map(|name| self.find_fname_index(name))
            .collect::<Result<Vec<_>>>()?;
        let mut found = vec![None; object_names.len()];

        self.scan_objects(|object| {
            let Some(object_name) = self.object_name_index(object) else {
                return Ok(Scan::Continue);
            };
            let Some(position) = name_indices.iter().position(|index| *index == object_name) else {
                return Ok(Scan::Continue);
            };
            if found[position].is_some() || !self.is_catalog_instance(object) {
                return Ok(Scan::Continue);
            }
            let class = self.memory.read_pointer(object + offsets.uobject_class)?;
            if class != 0
                && self.memory.read_u32(class + offsets.uobject_fname)? == class_name_index
            {
                found[position] = Some(object);
                if found.iter().all(Option::is_some) {
                    return Ok(Scan::Stop);
                }
            }
            Ok(Scan::Continue)
        })?;

        collect_found(found, object_names)
    }

    /// Encontra objetos por nome aceitando qualquer descendente da classe.
    pub fn find_named_instances(
        &self,
        object_names: &[&str],
        ancestor_class: &str,
    ) -> Result<Vec<usize>> {
        let name_indices = object_names
            .iter()
            .map(|name| self.find_fname_index(name))
            .collect::<Result<Vec<_>>>()?;
        let mut found = vec![None; object_names.len()];

        self.scan_objects(|object| {
            let Some(object_name) = self.object_name_index(object) else {
                return Ok(Scan::Continue);
            };
            let Some(position) = name_indices.iter().position(|index| *index == object_name) else {
                return Ok(Scan::Continue);
            };
            if found[position].is_some() || !self.is_catalog_instance(object) {
                return Ok(Scan::Continue);
            }
            if self.is_instance_of(object, ancestor_class)? {
                found[position] = Some(object);
                if found.iter().all(Option::is_some) {
                    return Ok(Scan::Stop);
                }
            }
            Ok(Scan::Continue)
        })?;

        collect_found(found, object_names)
    }

    /// Localiza o PlayerController da sessao, com cache revalidado.
    ///
    /// `require_pawn` distingue "o jogador existe" de "o jogador esta jogando":
    /// acoes de arma precisam do Pawn, acoes de save precisam apenas do mundo.
    pub fn find_player_controller(&self, require_pawn: bool) -> Result<usize> {
        let key = self.cache_key();

        let cached_names = cache::with(key, |caches| caches.controller_name_indices.clone())?;
        let name_indices = match cached_names {
            Some(indices) => indices,
            None => {
                let indices = self
                    .profile
                    .catalog
                    .player_controller_names
                    .iter()
                    .filter_map(|name| self.find_fname_index(name).ok())
                    .collect::<Vec<_>>();
                if indices.is_empty() {
                    return Err(codes::object_not_found(
                        "Nenhuma classe de PlayerController compativel foi carregada.",
                    ));
                }
                cache::with(key, |caches| {
                    caches.controller_name_indices = Some(indices.clone())
                })?;
                indices
            }
        };

        if let Some(cached) = cache::with(key, |caches| caches.player_controller)?
            && self.controller_is_valid(cached, &name_indices, require_pawn)
        {
            return Ok(cached);
        }

        let mut fallback = None;
        let mut exact = None;
        self.scan_objects(|object| {
            if !self.controller_is_valid(object, &name_indices, require_pawn) {
                return Ok(Scan::Continue);
            }
            if self.is_catalog_instance(object) {
                exact = Some(object);
                return Ok(Scan::Stop);
            }
            fallback.get_or_insert(object);
            Ok(Scan::Continue)
        })?;

        let object = exact.or(fallback).ok_or_else(|| {
            codes::object_not_found(if require_pawn {
                "Player controller com Pawn ativo ainda nao foi carregado."
            } else {
                "Player controller da sessao ainda nao foi carregado."
            })
        })?;
        cache::with(key, |caches| caches.player_controller = Some(object))?;
        Ok(object)
    }

    /// Controller com Pawn ativo: exigido pelas acoes de arma.
    pub fn active_controller(&self) -> Result<usize> {
        self.find_player_controller(true)
    }

    /// Contexto de mundo: exigido pelas rotinas nativas que recebem um
    /// `WorldContextObject`.
    pub fn world_context(&self) -> Result<usize> {
        self.find_player_controller(false).map_err(|error| {
            error.context(
                "Entre na Space Rig e aguarde o personagem carregar antes de executar esta acao",
            )
        })
    }
}

fn collect_found(found: Vec<Option<usize>>, object_names: &[&str]) -> Result<Vec<usize>> {
    found
        .into_iter()
        .enumerate()
        .map(|(index, object)| {
            object.ok_or_else(|| {
                codes::object_not_found(format!(
                    "{} nao foi encontrado na Space Rig.",
                    object_names[index]
                ))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::unreal::cache;
    use crate::infrastructure::unreal::testing::TestWorld;
    use crate::shared::ErrorCode;

    #[test]
    fn resolves_an_object_by_global_index() {
        let world = TestWorld::new();
        let class = world.spawn_class("ResourceData", None);
        let first = world.spawn_object("RES_CARVED_Bismor", class);
        let second = world.spawn_object("RES_VEIN_Croppa", class);

        let runtime = world.runtime();
        // Indice 0 e a meta-classe; 1 e a UClass criada acima.
        assert_eq!(runtime.resolve_object(2).unwrap(), first);
        assert_eq!(runtime.resolve_object(3).unwrap(), second);
        assert_eq!(
            runtime.read_object_name(first).unwrap(),
            "RES_CARVED_Bismor"
        );
    }

    #[test]
    fn refuses_an_index_beyond_the_live_object_count() {
        let world = TestWorld::new();
        world.spawn_class("Qualquer", None);
        let error = world
            .runtime()
            .resolve_object(99)
            .expect_err("indice fora do array deve falhar");
        assert_eq!(error.code(), ErrorCode::ObjectNotFound);
    }

    #[test]
    fn detects_a_stale_index_pointing_at_another_object() {
        let world = TestWorld::new();
        let class = world.spawn_class("Classe", None);
        let object = world.spawn_object("Objeto", class);
        // Corrompe o indice interno como aconteceria com um slot reciclado.
        world.memory().poke_u32(
            object
                + super::super::testing::profile()
                    .offsets
                    .unreal
                    .uobject_internal_index,
            77,
        );

        let error = world
            .runtime()
            .resolve_object(2)
            .expect_err("indice interno divergente deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
    }

    #[test]
    fn finds_named_objects_requiring_an_exact_class() {
        let world = TestWorld::new();
        let settings_class = world.spawn_class("SchematicSettings", None);
        let other_class = world.spawn_class("OutraClasse", None);
        let settings = world.spawn_object("GD_SchematicSettings", settings_class);
        world.spawn_object("GD_Impostor", other_class);

        let found = world
            .runtime()
            .find_named_objects(&["GD_SchematicSettings"], "SchematicSettings")
            .expect("objeto de catalogo deve ser encontrado");
        assert_eq!(found, vec![settings]);
    }

    #[test]
    fn exact_class_search_rejects_a_subclass() {
        let world = TestWorld::new();
        let base = world.spawn_class("ResourceData", None);
        let derived = world.spawn_class("SpecialResourceData", Some(base));
        world.spawn_object("RES_DataCell", derived);

        let error = world
            .runtime()
            .find_named_objects(&["RES_DataCell"], "ResourceData")
            .expect_err("classe exata nao deve aceitar subclasse");
        assert_eq!(error.code(), ErrorCode::ObjectNotFound);
    }

    #[test]
    fn instance_search_accepts_a_subclass() {
        let world = TestWorld::new();
        let base = world.spawn_class("ResourceData", None);
        let derived = world.spawn_class("SpecialResourceData", Some(base));
        let object = world.spawn_object("RES_DataCell", derived);

        let found = world
            .runtime()
            .find_named_instances(&["RES_DataCell"], "ResourceData")
            .expect("subclasse deve ser aceita por find_named_instances");
        assert_eq!(found, vec![object]);
    }

    #[test]
    fn numbered_runtime_copies_are_ignored() {
        let world = TestWorld::new();
        let class = world.spawn_class("ResourceData", None);
        let copy = world.spawn_object("RES_CARVED_Bismor", class);
        world.set_fname_number(copy, 7);

        let error = world
            .runtime()
            .find_named_instances(&["RES_CARVED_Bismor"], "ResourceData")
            .expect_err("copia numerada nao deve ser aceita");
        assert_eq!(error.code(), ErrorCode::ObjectNotFound);
    }

    #[test]
    fn missing_object_names_the_object_it_could_not_find() {
        let world = TestWorld::new();
        world.spawn_class("PlayerCharacterID", None);
        let error = world
            .runtime()
            .find_named_objects(&["DrillerID"], "PlayerCharacterID")
            .expect_err("objeto ausente deve falhar");
        assert!(error.message().contains("DrillerID"));
    }

    #[test]
    fn world_context_requires_a_loaded_controller() {
        cache::invalidate_all();
        let world = TestWorld::new();
        world.spawn_class("BP_PlayerController_SpaceRig_C", None);

        let error = world
            .runtime()
            .world_context()
            .expect_err("sem controller o contexto deve falhar");
        assert_eq!(error.code(), ErrorCode::ObjectNotFound);
        assert!(error.message().contains("Space Rig"));
    }

    #[test]
    fn finds_the_session_controller_and_caches_it() {
        cache::invalidate_all();
        let world = TestWorld::new();
        let class = world.spawn_class("BP_PlayerController_SpaceRig_C", None);
        let controller = world.spawn_object("BP_PlayerController_SpaceRig_C", class);

        let runtime = world.runtime();
        assert_eq!(runtime.world_context().unwrap(), controller);
        // Segunda chamada usa o cache e continua valida.
        assert_eq!(runtime.world_context().unwrap(), controller);
        assert_eq!(
            cache::with(runtime.cache_key(), |caches| caches.player_controller).unwrap(),
            Some(controller)
        );
        cache::invalidate_all();
    }

    #[test]
    fn controller_without_pawn_is_refused_when_a_pawn_is_required() {
        cache::invalidate_all();
        let world = TestWorld::new();
        let class = world.spawn_class("BP_PlayerController_SpaceRig_C", None);
        world.spawn_object("BP_PlayerController_SpaceRig_C", class);

        let runtime = world.runtime();
        assert!(runtime.world_context().is_ok());
        let error = runtime
            .active_controller()
            .expect_err("sem Pawn a acao de arma deve ser bloqueada");
        assert_eq!(error.code(), ErrorCode::ObjectNotFound);
        cache::invalidate_all();
    }

    #[test]
    fn scanning_an_empty_array_is_not_an_error() {
        let world = TestWorld::new();
        // Antes de o jogo popular a GUObjectArray o contador e zero.
        world.memory().poke_u32(
            super::super::testing::GUOBJECT_ARRAY
                + super::super::testing::profile().offsets.unreal.num_elements,
            0,
        );
        let mut visited = 0_usize;
        world
            .runtime()
            .scan_objects(|_| {
                visited += 1;
                Ok(Scan::Continue)
            })
            .expect("array vazio deve ser tolerado");
        assert_eq!(visited, 0);
    }
}
