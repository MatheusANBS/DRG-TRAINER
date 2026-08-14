use super::*;

pub(super) struct CreditsReader {
    pid: u32,
    pub(super) memory: ProcessMemory,
    pub(super) module_base: usize,
    pub(super) guobject_array: usize,
    pub(super) fname_pool: usize,
    pub(super) class_address: usize,
}

impl CreditsReader {
    pub(super) fn connect(pid: u32, module_base: usize, writable: bool) -> Result<Self> {
        let memory = ProcessMemory::open(pid, writable)?;
        let mut reader = Self {
            pid,
            memory,
            module_base,
            guobject_array: module_base + GUOBJECT_ARRAY_RVA,
            fname_pool: module_base + FNAME_POOL_RVA,
            class_address: 0,
        };
        reader.class_address = reader.resolve_uobject(FSD_SAVEGAME_CLASS_INDEX)?;
        let class_name = reader.read_object_name(reader.class_address)?;
        if class_name != "FSDSaveGame" {
            return Err(MemoryError(format!(
                "Validacao da classe falhou: esperado FSDSaveGame, recebido {class_name:?}"
            )));
        }
        Ok(reader)
    }

    pub(super) fn resolve_uobject(&self, index: u32) -> Result<usize> {
        let count = self
            .memory
            .read_u32(self.guobject_array + NUM_ELEMENTS_OFFSET)?;
        if index >= count {
            return Err(MemoryError(format!(
                "O objeto {index} ainda nao foi carregado ({count} objetos disponiveis)."
            )));
        }
        let chunks = self
            .memory
            .read_u64(self.guobject_array + OBJECTS_MEMBER_OFFSET)? as usize;
        let chunk_index = index / OBJECTS_PER_CHUNK;
        let within_chunk = index % OBJECTS_PER_CHUNK;
        let chunk = self.memory.read_u64(chunks + chunk_index as usize * 8)? as usize;
        if chunk == 0 {
            return Err(MemoryError(format!(
                "Chunk {chunk_index} da GUObjectArray e nulo."
            )));
        }
        let item = chunk + within_chunk as usize * UOBJECT_ITEM_SIZE;
        let object = self.memory.read_u64(item)? as usize;
        if object == 0 {
            return Err(MemoryError(format!("Objeto Unreal {index} e nulo.")));
        }
        let actual = self
            .memory
            .read_u32(object + UOBJECT_INTERNAL_INDEX_OFFSET)?;
        if actual != index {
            return Err(MemoryError(format!(
                "Indice interno invalido: esperado {index}, recebido {actual}."
            )));
        }
        Ok(object)
    }

    pub(super) fn read_fname(&self, comparison_index: u32) -> Result<String> {
        let block_index = comparison_index >> 16;
        let offset_units = comparison_index & 0xFFFF;
        let block =
            self.memory
                .read_u64(self.fname_pool + 0x10 + block_index as usize * 8)? as usize;
        if block == 0 {
            return Err(MemoryError(format!(
                "Bloco {block_index} da FNamePool e nulo."
            )));
        }
        let entry = block + offset_units as usize * 2;
        let header = self.memory.read_u16(entry)?;
        let is_wide = header & 1 != 0;
        let length = (header >> 6) as usize;
        if length == 0 || length > 1024 {
            return Err(MemoryError(format!(
                "Comprimento FName invalido: {length}."
            )));
        }
        if is_wide {
            let bytes = self.memory.read_bytes(entry + 2, length * 2)?;
            let units = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<_>>();
            String::from_utf16(&units).map_err(|error| MemoryError(error.to_string()))
        } else {
            String::from_utf8(self.memory.read_bytes(entry + 2, length)?)
                .map_err(|error| MemoryError(error.to_string()))
        }
    }

    pub(super) fn read_object_name(&self, object: usize) -> Result<String> {
        let comparison_index = self.memory.read_u32(object + UOBJECT_FNAME_OFFSET)?;
        self.read_fname(comparison_index)
    }

    fn is_save_instance(&self, object: usize, name_index: u32) -> bool {
        object != 0
            && self.memory.read_u64(object + UOBJECT_CLASS_OFFSET).ok()
                == Some(self.class_address as u64)
            && self.memory.read_u32(object + UOBJECT_FNAME_OFFSET).ok() == Some(name_index)
    }

    fn find_active_save(&self, name_index: u32) -> Result<usize> {
        let count = self
            .memory
            .read_u32(self.guobject_array + NUM_ELEMENTS_OFFSET)?;
        let chunks = self
            .memory
            .read_u64(self.guobject_array + OBJECTS_MEMBER_OFFSET)? as usize;
        let chunk_count = count.div_ceil(OBJECTS_PER_CHUNK);

        for chunk_index in (0..chunk_count).rev() {
            let chunk = self.memory.read_u64(chunks + chunk_index as usize * 8)? as usize;
            if chunk == 0 {
                continue;
            }
            let slots = if chunk_index + 1 == chunk_count {
                count - chunk_index * OBJECTS_PER_CHUNK
            } else {
                OBJECTS_PER_CHUNK
            };
            let items = self
                .memory
                .read_bytes(chunk, slots as usize * UOBJECT_ITEM_SIZE)?;
            for slot in (0..slots as usize).rev() {
                let offset = slot * UOBJECT_ITEM_SIZE;
                let object =
                    u64::from_le_bytes(items[offset..offset + 8].try_into().unwrap()) as usize;
                if self.is_save_instance(object, name_index) {
                    return Ok(object);
                }
            }
        }

        Err(MemoryError(
            "Nenhuma instancia ativa de FSDSaveGame foi encontrada.".into(),
        ))
    }

    pub(super) fn active_save(&self) -> Result<usize> {
        let name_index = self
            .memory
            .read_u32(self.class_address + UOBJECT_FNAME_OFFSET)?;
        let cache = ACTIVE_SAVE_CACHE.get_or_init(|| Mutex::new(None));

        if let Some((cached_pid, save)) = cache.lock().ok().and_then(|cached| *cached)
            && cached_pid == self.pid
            && self.is_save_instance(save, name_index)
        {
            return Ok(save);
        }

        if let Ok(save) = self.resolve_uobject(ACTIVE_SAVEGAME_INDEX)
            && self.is_save_instance(save, name_index)
        {
            if let Ok(mut cached) = cache.lock() {
                *cached = Some((self.pid, save));
            }
            return Ok(save);
        }

        let save = self.find_active_save(name_index)?;
        if let Ok(mut cached) = cache.lock() {
            *cached = Some((self.pid, save));
        }
        Ok(save)
    }

    pub(super) fn read(&self) -> Result<(i32, usize)> {
        let address = self.active_save()? + CREDITS_OFFSET;
        Ok((self.memory.read_i32(address)?, address))
    }

    pub(super) fn write(&self, value: i32) -> Result<(i32, i32, usize)> {
        let address = self.active_save()? + CREDITS_OFFSET;
        let previous = self.memory.read_i32(address)?;
        self.memory.write_i32(address, value)?;
        let current = self.memory.read_i32(address)?;
        if current != value {
            return Err(MemoryError(format!(
                "A verificacao da escrita falhou: esperado {value}, lido {current}."
            )));
        }
        Ok((previous, current, address))
    }

    pub(super) fn resource_amounts(&self) -> Result<(Vec<ResourceSnapshot>, usize)> {
        let save = self.active_save()?;
        let map = save + RESOURCES_SAVE_OFFSET;
        let elements = self.memory.read_u64(map)? as usize;
        let max_index = self.memory.read_i32(map + 8)?;
        let capacity = self.memory.read_i32(map + 12)?;
        let allocation_bits = self.memory.read_bytes(map + 0x10, 16)?;
        let allocation_count = self.memory.read_i32(map + 0x28)?;
        let allocation_capacity = self.memory.read_i32(map + 0x2C)?;
        if !(0..=128).contains(&max_index)
            || max_index > capacity
            || allocation_count != max_index
            || !(max_index..=128).contains(&allocation_capacity)
            || (max_index > 0 && elements == 0)
        {
            return Err(MemoryError(format!(
                "Mapa de recursos invalido: elements=0x{elements:X}, max={max_index}, capacity={capacity}, flags={allocation_count}/{allocation_capacity}."
            )));
        }

        let mut values = std::collections::HashMap::<[u8; 16], f32>::new();
        if max_index > 0 {
            let raw = self
                .memory
                .read_bytes(elements, max_index as usize * RESOURCE_MAP_ELEMENT_SIZE)?;
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
                let offset = index * RESOURCE_MAP_ELEMENT_SIZE;
                let id: [u8; 16] = raw[offset..offset + 16].try_into().unwrap();
                let amount = f32::from_le_bytes(raw[offset + 16..offset + 20].try_into().unwrap());
                if id == [0; 16]
                    || !amount.is_finite()
                    || !(0.0..=MAX_RESOURCE_TOTAL).contains(&amount)
                {
                    return Err(MemoryError(format!(
                        "Entrada de recurso invalida no indice {index}: amount={amount}."
                    )));
                }
                if values.insert(id, amount).is_some() {
                    return Err(MemoryError(format!(
                        "GUID de recurso duplicado no indice {index}."
                    )));
                }
            }
        }

        Ok((
            RESOURCE_DEFINITIONS
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

    pub(super) fn item_counts(&self) -> Result<(i32, i32, usize)> {
        let save = self.active_save()?;
        let unlocked = self.memory.read_i32(save + UNLOCKED_ITEMS_OFFSET + 8)?;
        let owned = self.memory.read_i32(save + OWNED_ITEMS_OFFSET + 8)?;
        if !(0..=10_000).contains(&unlocked) || !(0..=10_000).contains(&owned) {
            return Err(MemoryError(format!(
                "Contadores de itens invalidos: unlocked={unlocked}, owned={owned}."
            )));
        }
        Ok((unlocked, owned, save))
    }

    pub(super) fn progression_count(&self, offset: usize, label: &str) -> Result<(i32, usize)> {
        let save = self.active_save()?;
        let count = self.memory.read_i32(save + offset + 8)?;
        if !(0..=10_000).contains(&count) {
            return Err(MemoryError(format!(
                "Contador de {label} invalido: {count}."
            )));
        }
        Ok((count, save))
    }

    pub(super) fn progression_ids(
        &self,
        offset: usize,
        label: &str,
    ) -> Result<(HashSet<[u8; 16]>, usize)> {
        let save = self.active_save()?;
        let data = self.memory.read_u64(save + offset)? as usize;
        let count = self.memory.read_i32(save + offset + 8)?;
        let capacity = self.memory.read_i32(save + offset + 12)?;
        if !(0..=10_000).contains(&count) || count > capacity || capacity > 10_000 {
            return Err(MemoryError(format!(
                "Lista de {label} invalida: count={count}, capacity={capacity}."
            )));
        }
        if count == 0 {
            return Ok((HashSet::new(), save));
        }
        if data == 0 {
            return Err(MemoryError(format!(
                "A lista de {label} possui dados nulos."
            )));
        }
        let bytes = self.memory.read_bytes(data, count as usize * 16)?;
        Ok((
            bytes
                .chunks_exact(16)
                .map(|chunk| chunk.try_into().unwrap())
                .collect(),
            save,
        ))
    }

    pub(super) fn schematic_counts(&self) -> Result<(i32, i32, usize)> {
        let save = self.active_save()?;
        let schematics = save + SCHEMATIC_SAVE_OFFSET;
        let forged = self
            .memory
            .read_i32(schematics + FORGED_SCHEMATICS_OFFSET + 8)?;
        let owned = self
            .memory
            .read_i32(schematics + OWNED_SCHEMATICS_OFFSET + 8)?;
        if !(0..=10_000).contains(&forged) || !(0..=10_000).contains(&owned) {
            return Err(MemoryError(format!(
                "Contadores de esquemas invalidos: forged={forged}, owned={owned}."
            )));
        }
        Ok((forged, owned, save))
    }

    pub(super) fn forged_schematic_ids(&self) -> Result<(HashSet<[u8; 16]>, usize)> {
        let save = self.active_save()?;
        let array = save + SCHEMATIC_SAVE_OFFSET + FORGED_SCHEMATICS_OFFSET;
        let data = self.memory.read_u64(array)? as usize;
        let count = self.memory.read_i32(array + 8)?;
        let capacity = self.memory.read_i32(array + 12)?;
        if !(0..=10_000).contains(&count) || count > capacity || capacity > 10_000 {
            return Err(MemoryError(format!(
                "Lista de esquemas forjados invalida: count={count}, capacity={capacity}."
            )));
        }
        if count == 0 {
            return Ok((HashSet::new(), save));
        }
        if data == 0 {
            return Err(MemoryError(
                "A lista de esquemas forjados possui dados nulos.".into(),
            ));
        }
        let bytes = self.memory.read_bytes(data, count as usize * 16)?;
        let ids: HashSet<[u8; 16]> = bytes
            .chunks_exact(16)
            .map(|chunk| chunk.try_into().unwrap())
            .collect();
        if ids.len() != count as usize {
            return Err(MemoryError(
                "A lista de esquemas forjados possui SavegameIDs duplicados.".into(),
            ));
        }
        Ok((ids, save))
    }

    pub(super) fn playable_class_xp(&self) -> Result<(usize, Vec<(&'static str, usize, i32)>)> {
        let (save, progress) = self.playable_class_progress()?;
        Ok((
            save,
            progress
                .into_iter()
                .map(|entry| (entry.class_name, entry.xp_address, entry.xp))
                .collect(),
        ))
    }

    pub(super) fn playable_class_progress(&self) -> Result<(usize, Vec<CharacterProgress>)> {
        let save = self.active_save()?;
        let data = self.memory.read_u64(save + CHARACTER_SAVES_OFFSET)? as usize;
        let count = self.memory.read_i32(save + CHARACTER_SAVES_OFFSET + 8)?;
        let capacity = self.memory.read_i32(save + CHARACTER_SAVES_OFFSET + 12)?;
        if data == 0 || !(4..=16).contains(&count) || count > capacity || capacity > 32 {
            return Err(MemoryError(format!(
                "CharacterSaves invalido: data=0x{data:X}, count={count}, capacity={capacity}."
            )));
        }

        let mut entries = Vec::with_capacity(4);
        for index in 0..count as usize {
            let entry = data + index * CHARACTER_SAVE_SIZE;
            let guid_bytes = self.memory.read_bytes(entry, 16)?;
            let Some((class_name, _)) = PLAYABLE_CLASS_IDS
                .iter()
                .find(|(_, guid)| guid_bytes.as_slice() == *guid)
            else {
                continue;
            };
            let xp_address = entry + CHARACTER_XP_OFFSET;
            let xp = self.memory.read_i32(xp_address)?;
            if !(0..=MAX_CLASS_XP).contains(&xp) {
                return Err(MemoryError(format!("XP invalido para {class_name}: {xp}.")));
            }
            let promotions_address = entry + CHARACTER_PROMOTIONS_OFFSET;
            let promotions = self.memory.read_i32(promotions_address)?;
            if !(0..=10_000).contains(&promotions) {
                return Err(MemoryError(format!(
                    "Promocoes invalidas para {class_name}: {promotions}."
                )));
            }
            entries.push(CharacterProgress {
                class_name: *class_name,
                xp_address,
                xp,
                promotions_address,
                promotions,
            });
        }
        entries.sort_unstable_by_key(|entry| entry.class_name);
        entries.dedup_by_key(|entry| entry.class_name);
        if entries.len() != PLAYABLE_CLASS_IDS.len() {
            return Err(MemoryError(format!(
                "Foram encontradas {} das 4 classes jogaveis no save.",
                entries.len()
            )));
        }
        Ok((save, entries))
    }
}
