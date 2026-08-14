use super::*;

pub(super) struct ClipReader {
    pub(super) pid: u32,
    pub(super) memory: ProcessMemory,
    pub(super) guobject_array: usize,
    pub(super) fname_pool: usize,
}

impl ClipReader {
    pub(super) fn connect(pid: u32, module_base: usize, writable: bool) -> Result<Self> {
        Ok(Self {
            pid,
            memory: ProcessMemory::open(pid, writable)?,
            guobject_array: module_base + GUOBJECT_ARRAY_RVA,
            fname_pool: module_base + FNAME_POOL_RVA,
        })
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
        let length = (header >> 6) as usize;
        if length == 0 || length > 1024 {
            return Err(MemoryError(format!(
                "Comprimento FName invalido: {length}."
            )));
        }
        if header & 1 != 0 {
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
        self.read_fname(self.memory.read_u32(object + UOBJECT_FNAME_OFFSET)?)
    }

    pub(super) fn find_fname_index(&self, target: &str) -> Result<u32> {
        let current_block = self.memory.read_u32(self.fname_pool + 0x08)?;
        let current_cursor = self.memory.read_u32(self.fname_pool + 0x0C)? as usize;
        let target_bytes = target.as_bytes();

        for block_index in 0..=current_block {
            let block = self
                .memory
                .read_u64(self.fname_pool + 0x10 + block_index as usize * 8)?
                as usize;
            if block == 0 {
                continue;
            }
            let size = if block_index == current_block {
                current_cursor.min(FNAME_BLOCK_SIZE)
            } else {
                FNAME_BLOCK_SIZE
            };
            let bytes = self.memory.read_bytes(block, size)?;
            let mut offset = 0_usize;
            while offset + 2 <= bytes.len() {
                let header = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                let length = (header >> 6) as usize;
                let wide = header & 1 != 0;
                let payload_size = length.saturating_mul(if wide { 2 } else { 1 });
                if length == 0 || length > 1024 || offset + 2 + payload_size > bytes.len() {
                    offset += 2;
                    continue;
                }
                if !wide
                    && length == target_bytes.len()
                    && bytes[offset + 2..offset + 2 + length] == *target_bytes
                {
                    return Ok((block_index << 16) | (offset as u32 >> 1));
                }
                offset += (2 + payload_size + 1) & !1;
            }
        }
        Err(MemoryError(format!(
            "FName {target:?} nao encontrado na build carregada."
        )))
    }

    pub(super) fn controller_is_valid(
        &self,
        object: usize,
        name_indices: &[u32],
        require_pawn: bool,
    ) -> bool {
        let valid = (|| -> Result<bool> {
            let object_name = self.memory.read_u32(object + UOBJECT_FNAME_OFFSET)?;
            if !name_indices.contains(&object_name) {
                return Ok(false);
            }
            let class = self.memory.read_u64(object + UOBJECT_CLASS_OFFSET)? as usize;
            if class == 0
                || self.memory.read_u32(class + UOBJECT_FNAME_OFFSET)? != object_name
                || (require_pawn && self.memory.read_u64(object + CONTROLLER_PAWN_OFFSET)? == 0)
            {
                return Ok(false);
            }
            Ok(true)
        })();
        valid.unwrap_or(false)
    }

    pub(super) fn find_player_controller(&self, require_pawn: bool) -> Result<usize> {
        let name_cache = CONTROLLER_FNAME_CACHE.get_or_init(|| Mutex::new(None));
        let name_indices = {
            let mut cached = name_cache
                .lock()
                .map_err(|_| MemoryError("Cache de FName indisponivel.".into()))?;
            if let Some((cached_pid, indices)) = cached.as_ref()
                && *cached_pid == self.pid
            {
                indices.clone()
            } else {
                let indices = PLAYER_CONTROLLER_NAMES
                    .iter()
                    .filter_map(|name| self.find_fname_index(name).ok())
                    .collect::<Vec<_>>();
                if indices.is_empty() {
                    return Err(MemoryError(
                        "Nenhuma classe de PlayerController compativel foi carregada.".into(),
                    ));
                }
                *cached = Some((self.pid, indices.clone()));
                indices
            }
        };
        let cache = CLIP_CONTROLLER_CACHE.get_or_init(|| Mutex::new(None));
        if let Some((cached_pid, cached_object)) = *cache
            .lock()
            .map_err(|_| MemoryError("Cache do controller indisponivel.".into()))?
            && cached_pid == self.pid
            && self.controller_is_valid(cached_object, &name_indices, require_pawn)
        {
            return Ok(cached_object);
        }

        let count = self
            .memory
            .read_u32(self.guobject_array + NUM_ELEMENTS_OFFSET)?;
        let chunks = self
            .memory
            .read_u64(self.guobject_array + OBJECTS_MEMBER_OFFSET)? as usize;
        let chunk_count = count.div_ceil(OBJECTS_PER_CHUNK);
        let mut fallback = None;

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
            let bytes = self
                .memory
                .read_bytes(chunk, slots as usize * UOBJECT_ITEM_SIZE)?;
            for slot in (0..slots as usize).rev() {
                let offset = slot * UOBJECT_ITEM_SIZE;
                let object =
                    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
                if object == 0 || !self.controller_is_valid(object, &name_indices, require_pawn) {
                    continue;
                }
                let number = self
                    .memory
                    .read_u32(object + UOBJECT_FNAME_NUMBER_OFFSET)
                    .unwrap_or(u32::MAX);
                if number == 0 {
                    *cache
                        .lock()
                        .map_err(|_| MemoryError("Cache do controller indisponivel.".into()))? =
                        Some((self.pid, object));
                    return Ok(object);
                }
                fallback.get_or_insert(object);
            }
        }

        let object = fallback.ok_or_else(|| {
            MemoryError(if require_pawn {
                "Player controller com Pawn ativo ainda nao foi carregado.".into()
            } else {
                "Player controller da sessao ainda nao foi carregado.".into()
            })
        })?;
        *cache
            .lock()
            .map_err(|_| MemoryError("Cache do controller indisponivel.".into()))? =
            Some((self.pid, object));
        Ok(object)
    }

    pub(super) fn find_controller(&self) -> Result<usize> {
        self.find_player_controller(true)
    }

    pub(super) fn world_context(&self) -> Result<usize> {
        self.find_player_controller(false).map_err(|error| {
            MemoryError(format!(
                "Entre na Space Rig e aguarde o personagem carregar antes de executar esta acao: {error}"
            ))
        })
    }

    pub(super) fn find_named_objects(
        &self,
        object_names: &[&str],
        class_name: &str,
    ) -> Result<Vec<usize>> {
        let class_name_index = self.find_fname_index(class_name)?;
        let name_indices = object_names
            .iter()
            .map(|name| self.find_fname_index(name))
            .collect::<Result<Vec<_>>>()?;
        let mut found = vec![None; object_names.len()];
        let count = self
            .memory
            .read_u32(self.guobject_array + NUM_ELEMENTS_OFFSET)?;
        let chunks = self
            .memory
            .read_u64(self.guobject_array + OBJECTS_MEMBER_OFFSET)? as usize;
        let chunk_count = count.div_ceil(OBJECTS_PER_CHUNK);

        'chunks: for chunk_index in (0..chunk_count).rev() {
            let chunk = self.memory.read_u64(chunks + chunk_index as usize * 8)? as usize;
            if chunk == 0 {
                continue;
            }
            let slots = if chunk_index + 1 == chunk_count {
                count - chunk_index * OBJECTS_PER_CHUNK
            } else {
                OBJECTS_PER_CHUNK
            };
            let bytes = self
                .memory
                .read_bytes(chunk, slots as usize * UOBJECT_ITEM_SIZE)?;
            for slot in (0..slots as usize).rev() {
                let offset = slot * UOBJECT_ITEM_SIZE;
                let object =
                    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
                if object == 0 {
                    continue;
                }
                let object_name = match self.memory.read_u32(object + UOBJECT_FNAME_OFFSET) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let Some(position) = name_indices.iter().position(|index| *index == object_name)
                else {
                    continue;
                };
                if found[position].is_some()
                    || self
                        .memory
                        .read_u32(object + UOBJECT_FNAME_NUMBER_OFFSET)
                        .ok()
                        != Some(0)
                {
                    continue;
                }
                let class = self.memory.read_u64(object + UOBJECT_CLASS_OFFSET)? as usize;
                if class != 0
                    && self.memory.read_u32(class + UOBJECT_FNAME_OFFSET)? == class_name_index
                {
                    found[position] = Some(object);
                    if found.iter().all(Option::is_some) {
                        break 'chunks;
                    }
                }
            }
        }

        found
            .into_iter()
            .enumerate()
            .map(|(index, object)| {
                object.ok_or_else(|| {
                    MemoryError(format!(
                        "{} nao foi encontrado na Space Rig.",
                        object_names[index]
                    ))
                })
            })
            .collect()
    }

    pub(super) fn find_named_instances(
        &self,
        object_names: &[&str],
        ancestor_class: &str,
    ) -> Result<Vec<usize>> {
        let name_indices = object_names
            .iter()
            .map(|name| self.find_fname_index(name))
            .collect::<Result<Vec<_>>>()?;
        let mut found = vec![None; object_names.len()];
        let count = self
            .memory
            .read_u32(self.guobject_array + NUM_ELEMENTS_OFFSET)?;
        let chunks = self
            .memory
            .read_u64(self.guobject_array + OBJECTS_MEMBER_OFFSET)? as usize;
        let chunk_count = count.div_ceil(OBJECTS_PER_CHUNK);

        'chunks: for chunk_index in (0..chunk_count).rev() {
            let chunk = self.memory.read_u64(chunks + chunk_index as usize * 8)? as usize;
            if chunk == 0 {
                continue;
            }
            let slots = if chunk_index + 1 == chunk_count {
                count - chunk_index * OBJECTS_PER_CHUNK
            } else {
                OBJECTS_PER_CHUNK
            };
            let bytes = self
                .memory
                .read_bytes(chunk, slots as usize * UOBJECT_ITEM_SIZE)?;
            for slot in (0..slots as usize).rev() {
                let offset = slot * UOBJECT_ITEM_SIZE;
                let object =
                    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap()) as usize;
                if object == 0 {
                    continue;
                }
                let object_name = match self.memory.read_u32(object + UOBJECT_FNAME_OFFSET) {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let Some(position) = name_indices.iter().position(|index| *index == object_name)
                else {
                    continue;
                };
                if found[position].is_some()
                    || self
                        .memory
                        .read_u32(object + UOBJECT_FNAME_NUMBER_OFFSET)
                        .ok()
                        != Some(0)
                {
                    continue;
                }
                if self.is_instance_of(object, ancestor_class)? {
                    found[position] = Some(object);
                    if found.iter().all(Option::is_some) {
                        break 'chunks;
                    }
                }
            }
        }

        found
            .into_iter()
            .enumerate()
            .map(|(index, object)| {
                object.ok_or_else(|| {
                    MemoryError(format!(
                        "{} nao foi encontrado na Space Rig.",
                        object_names[index]
                    ))
                })
            })
            .collect()
    }

    pub(super) fn all_schematics(&self) -> Result<Vec<(usize, [u8; 16])>> {
        let settings = self.find_named_objects(&["GD_SchematicSettings"], "SchematicSettings")?[0];
        let set = settings + SCHEMATIC_SETTINGS_ALL_SCHEMATICS_OFFSET;
        let elements = self.memory.read_u64(set)? as usize;
        let count = self.memory.read_i32(set + 8)?;
        let max_index = self.memory.read_i32(set + TSET_MAX_INDEX_OFFSET)?;
        let flags = self.memory.read_u64(set + TSET_ALLOCATION_FLAGS_OFFSET)? as usize;
        if elements == 0
            || flags == 0
            || !(1..=2_000).contains(&count)
            || !(count..=2_500).contains(&max_index)
        {
            return Err(MemoryError(format!(
                "AllSchematics invalido: elements=0x{elements:X}, flags=0x{flags:X}, count={count}, max_index={max_index}."
            )));
        }

        let allocation = self
            .memory
            .read_bytes(flags, ((max_index + 31) / 32) as usize * 4)?;
        let raw_elements = self
            .memory
            .read_bytes(elements, max_index as usize * TSET_ELEMENT_SIZE)?;
        let mut schematics = Vec::with_capacity(count as usize);
        let mut ids = HashSet::with_capacity(count as usize);
        for index in 0..max_index as usize {
            let word_offset = index / 32 * 4;
            let word =
                u32::from_le_bytes(allocation[word_offset..word_offset + 4].try_into().unwrap());
            if word & (1 << (index % 32)) == 0 {
                continue;
            }
            let offset = index * TSET_ELEMENT_SIZE;
            let schematic =
                u64::from_le_bytes(raw_elements[offset..offset + 8].try_into().unwrap()) as usize;
            if schematic == 0 || !self.is_instance_of(schematic, "Schematic")? {
                return Err(MemoryError(format!(
                    "Entrada invalida em AllSchematics[{index}]: 0x{schematic:X}."
                )));
            }
            let id: [u8; 16] = self
                .memory
                .read_bytes(schematic + SCHEMATIC_SAVEGAME_ID_OFFSET, 16)?
                .try_into()
                .unwrap();
            if id == [0; 16] || !ids.insert(id) {
                return Err(MemoryError(format!(
                    "SavegameID invalido ou duplicado em AllSchematics[{index}]."
                )));
            }
            schematics.push((schematic, id));
        }
        if schematics.len() != count as usize {
            return Err(MemoryError(format!(
                "AllSchematics incompleto: esperado {count}, lido {}.",
                schematics.len()
            )));
        }
        Ok(schematics)
    }

    pub(super) fn schematic_reward_targets(
        &self,
        schematics: &[(usize, [u8; 16])],
    ) -> Result<Vec<SchematicRewardTarget>> {
        let mut targets = Vec::with_capacity(
            CURRENT_OVERCLOCK_SCHEMATIC_COUNT + CURRENT_COSMETIC_SCHEMATIC_COUNT,
        );
        let mut overclocks = 0_usize;
        let mut cosmetics = 0_usize;
        for (schematic, _) in schematics {
            let item = self.memory.read_u64(*schematic + SCHEMATIC_ITEM_OFFSET)? as usize;
            if item == 0 {
                return Err(MemoryError(format!(
                    "Schematic.Item nulo em 0x{schematic:X}."
                )));
            }
            let class = self.memory.read_u64(item + UOBJECT_CLASS_OFFSET)? as usize;
            let class_name = self.read_object_name(class)?;
            let kind = match class_name.as_str() {
                "OverclockShematicItem" => {
                    overclocks += 1;
                    SchematicRewardKind::Overclock
                }
                "SkinSchematicItem" => {
                    cosmetics += 1;
                    SchematicRewardKind::Skin
                }
                "VanitySchematicItem" => {
                    cosmetics += 1;
                    SchematicRewardKind::Vanity
                }
                "VictoryPoseSchematicItem" => {
                    cosmetics += 1;
                    SchematicRewardKind::VictoryPose
                }
                "BlankSchematicItem" | "ResourceSchematicItem" => continue,
                _ => {
                    return Err(MemoryError(format!(
                        "Tipo de recompensa de esquema nao reconhecido: {class_name}."
                    )));
                }
            };
            let vtable = self.memory.read_u64(item)? as usize;
            let virtual_reward = self.memory.read_u64(vtable + 0x270)? as usize;
            let module_base = self.guobject_array - GUOBJECT_ARRAY_RVA;
            let (expected_reward, _, label) = kind.routine(module_base);
            if virtual_reward != expected_reward {
                return Err(MemoryError(format!(
                    "Rotina virtual invalida para {label}: 0x{virtual_reward:X}."
                )));
            }
            let overclock_id = if matches!(kind, SchematicRewardKind::Overclock) {
                let overclock = self.memory.read_u64(item + 0x30)? as usize;
                if overclock == 0 {
                    return Err(MemoryError(
                        "OverclockShematicItem.Overclock esta nulo.".into(),
                    ));
                }
                Some(
                    self.memory
                        .read_bytes(overclock + SCHEMATIC_SAVEGAME_ID_OFFSET, 16)?
                        .try_into()
                        .unwrap(),
                )
            } else {
                None
            };
            targets.push(SchematicRewardTarget {
                schematic: *schematic,
                item,
                kind,
                overclock_id,
            });
        }
        if overclocks != CURRENT_OVERCLOCK_SCHEMATIC_COUNT
            || cosmetics != CURRENT_COSMETIC_SCHEMATIC_COUNT
        {
            return Err(MemoryError(format!(
                "Catalogo de recompensas inesperado: overclocks={overclocks}, cosmeticos={cosmetics}."
            )));
        }
        Ok(targets)
    }

    pub(super) fn equipped_actor(&self) -> Result<usize> {
        let controller = self.find_controller()?;
        let pawn = self.memory.read_u64(controller + CONTROLLER_PAWN_OFFSET)? as usize;
        if pawn == 0 {
            return Err(MemoryError("O player ainda nao possui Pawn ativo.".into()));
        }
        let inventory = self.memory.read_u64(pawn + PLAYER_INVENTORY_OFFSET)? as usize;
        if inventory == 0 {
            return Err(MemoryError(
                "InventoryComponent do player ainda nao foi carregado.".into(),
            ));
        }
        let equipped = self
            .memory
            .read_u64(inventory + INVENTORY_EQUIPPED_ACTOR_OFFSET)? as usize;
        if equipped == 0 {
            return Err(MemoryError("Nenhum item esta equipado.".into()));
        }
        Ok(equipped)
    }

    pub(super) fn is_ammo_weapon(&self, object: usize) -> Result<bool> {
        self.is_instance_of(object, AMMO_WEAPON_CLASS_NAME)
    }

    pub(super) fn is_instance_of(&self, object: usize, expected_class: &str) -> Result<bool> {
        if object == 0 {
            return Ok(false);
        }
        let mut class = self.memory.read_u64(object + UOBJECT_CLASS_OFFSET)? as usize;
        for _ in 0..32 {
            if class == 0 {
                return Ok(false);
            }
            if self.read_object_name(class)? == expected_class {
                return Ok(true);
            }
            class = self.memory.read_u64(class + USTRUCT_SUPER_OFFSET)? as usize;
        }
        Ok(false)
    }

    pub(super) fn find_property_offset(
        &self,
        object: usize,
        target: &str,
    ) -> Result<Option<usize>> {
        let mut class = self.memory.read_u64(object + UOBJECT_CLASS_OFFSET)? as usize;
        for _ in 0..32 {
            if class == 0 {
                return Ok(None);
            }
            let mut field =
                self.memory
                    .read_u64(class + USTRUCT_CHILD_PROPERTIES_OFFSET)? as usize;
            for _ in 0..1_024 {
                if field == 0 {
                    break;
                }
                let comparison_index = self.memory.read_u32(field + FFIELD_FNAME_OFFSET)?;
                if self.read_fname(comparison_index)? == target {
                    let element_size = self
                        .memory
                        .read_u32(field + FPROPERTY_ELEMENT_SIZE_OFFSET)?
                        as usize;
                    let offset = self
                        .memory
                        .read_u32(field + FPROPERTY_OFFSET_INTERNAL_OFFSET)?
                        as usize;
                    if element_size == 0 || element_size > 0x10_000 || offset > 0x10_000 {
                        return Err(MemoryError(format!(
                            "Metadata invalida para {target}: size=0x{element_size:X}, offset=0x{offset:X}."
                        )));
                    }
                    return Ok(Some(offset));
                }
                field = self.memory.read_u64(field + FFIELD_NEXT_OFFSET)? as usize;
            }
            class = self.memory.read_u64(class + USTRUCT_SUPER_OFFSET)? as usize;
        }
        Ok(None)
    }

    pub(super) fn object_property(&self, object: usize, name: &str) -> Result<Option<usize>> {
        let Some(offset) = self.find_property_offset(object, name)? else {
            return Ok(None);
        };
        let value = self.memory.read_u64(object + offset)? as usize;
        Ok((value != 0).then_some(value))
    }

    pub(super) fn push_damage_component_targets(
        &self,
        component: usize,
        targets: &mut Vec<usize>,
    ) -> Result<()> {
        if !self.is_instance_of(component, "DamageComponent")? {
            return Ok(());
        }
        for field_name in ["Damage", "RadialDamage"] {
            let Some(offset) = self.find_property_offset(component, field_name)? else {
                continue;
            };
            let address = component + offset;
            let value = self.memory.read_f32(address)?;
            if value.is_finite()
                && (value > 0.0 || (value - DAMAGE_VALUE).abs() < f32::EPSILON)
                && value < 100_000_000.0
            {
                targets.push(address);
            }
        }
        Ok(())
    }

    pub(super) fn damage_targets_from_object(
        &self,
        object: usize,
        follow_projectiles: bool,
    ) -> Result<Vec<usize>> {
        let mut targets = Vec::new();
        let damage_properties = [
            "Damage",
            "DamageComponent",
            "DamageComp",
            "ShockWaveDamageComponent",
            "AoEDamageComponent",
            "ExplosionDamage",
            "CritcalOverheatDamage",
            "BurstFireBonusDamage",
            "FireExplosionDamage",
            "OverchargeDamageComponent",
            "WeaponBlastDamage",
            "ShotwaveBonusDamage",
            "MoleBonusDamage",
            "AoEHeatDamageComponent",
            "AoEColdDamageComponent",
            "ExplodingTargetsDamageComponent",
            "RadiantSuperheaterFrostShock",
            "BarrelProximityDamageComponent",
            "MainDamageComponent",
            "SimpleDamageComponent",
            "InitialDamageComponent",
        ];
        for property in damage_properties {
            if let Some(component) = self.object_property(object, property)? {
                self.push_damage_component_targets(component, &mut targets)?;
            }
        }

        let fire_properties = [
            "WeaponFire",
            "HitScan",
            "Hitscan",
            "HitscanComponent",
            "MultiHitscan",
            "AllPiercingHitscan",
            "CapsuleHitscanComp",
            "ReflectionHitscanComponent",
        ];
        for property in fire_properties {
            let Some(fire) = self.object_property(object, property)? else {
                continue;
            };
            if !self.is_instance_of(fire, "HitscanBaseComponent")? {
                continue;
            }
            let before = targets.len();
            if let Some(component) = self.object_property(fire, "DamageComponent")? {
                self.push_damage_component_targets(component, &mut targets)?;
            }
            if targets.len() == before
                && let Some(offset) = self.find_property_offset(fire, "Damage")?
            {
                let address = fire + offset;
                let value = self.memory.read_f32(address)?;
                if value.is_finite()
                    && (value > 0.0 || (value - DAMAGE_VALUE).abs() < f32::EPSILON)
                    && value < 100_000_000.0
                {
                    targets.push(address);
                }
            }
        }

        if follow_projectiles {
            for launcher_name in [
                "projectileLauncher",
                "ProjectileLancher",
                "ChargedProjectileLauncher",
            ] {
                let Some(launcher) = self.object_property(object, launcher_name)? else {
                    continue;
                };
                if !self.is_instance_of(launcher, "ProjectileLauncherBaseComponent")? {
                    continue;
                }
                for class_name in [
                    "ProjectileClass",
                    "NormalProjectileClass",
                    "ChargedProjectileClass",
                ] {
                    let Some(projectile_class) = self.object_property(launcher, class_name)? else {
                        continue;
                    };
                    let default_object = self
                        .memory
                        .read_u64(projectile_class + UCLASS_DEFAULT_OBJECT_OFFSET)?
                        as usize;
                    if default_object == 0
                        || self
                            .memory
                            .read_u64(default_object + UOBJECT_CLASS_OFFSET)?
                            as usize
                            != projectile_class
                    {
                        continue;
                    }
                    targets.extend(self.damage_targets_from_object(default_object, false)?);
                }
            }
        }

        targets.sort_unstable();
        targets.dedup();
        if targets.len() > 64 {
            return Err(MemoryError(format!(
                "Quantidade inesperada de campos de dano: {}.",
                targets.len()
            )));
        }
        Ok(targets)
    }

    pub(super) fn damage_targets(&self, weapon: usize) -> Result<Vec<usize>> {
        self.damage_targets_from_object(weapon, true)
    }

    pub(super) fn damage_snapshot(&self, freeze: bool) -> Result<DamageStatus> {
        let equipped = self.equipped_actor()?;
        let weapon_name = self
            .read_object_name(equipped)
            .unwrap_or_else(|_| "Unknown weapon".into());
        if !self.is_ammo_weapon(equipped)? {
            return Ok(DamageStatus {
                enabled: WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire),
                available: false,
                pid: Some(self.pid),
                weapon_name: Some(weapon_name),
                value: None,
                target_count: 0,
                addresses: Vec::new(),
                message: "O item equipado nao usa AmmoDrivenWeapon.".into(),
            });
        }

        let targets = self.damage_targets(equipped)?;
        if targets.is_empty() {
            return Ok(DamageStatus {
                enabled: WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire),
                available: false,
                pid: Some(self.pid),
                weapon_name: Some(weapon_name),
                value: None,
                target_count: 0,
                addresses: Vec::new(),
                message: "Nenhum DamageComponent ativo foi encontrado nesta arma.".into(),
            });
        }

        let mut value = self.memory.read_f32(targets[0])?;
        if freeze {
            for address in &targets {
                if (self.memory.read_f32(*address)? - DAMAGE_VALUE).abs() >= f32::EPSILON {
                    self.memory.write_f32(*address, DAMAGE_VALUE)?;
                }
                let verified = self.memory.read_f32(*address)?;
                if (verified - DAMAGE_VALUE).abs() >= f32::EPSILON {
                    return Err(MemoryError(format!(
                        "A verificacao do dano falhou em 0x{address:X}: {verified}."
                    )));
                }
            }
            value = DAMAGE_VALUE;
        }

        Ok(DamageStatus {
            enabled: WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire),
            available: true,
            pid: Some(self.pid),
            weapon_name: Some(weapon_name),
            value: Some(value),
            target_count: targets.len(),
            addresses: targets
                .iter()
                .map(|address| format!("0x{address:X}"))
                .collect(),
            message: if freeze {
                format!("Weapon Damage ativo em {} campo(s).", targets.len())
            } else {
                format!("{} campo(s) de dano encontrado(s).", targets.len())
            },
        })
    }

    pub(super) fn snapshot(&self, freeze: bool) -> Result<ClipStatus> {
        let equipped = self.equipped_actor()?;
        let weapon_name = self
            .read_object_name(equipped)
            .unwrap_or_else(|_| "Unknown weapon".into());
        if !self.is_ammo_weapon(equipped)? {
            return Ok(ClipStatus {
                enabled: INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire),
                available: false,
                pid: Some(self.pid),
                weapon_name: Some(weapon_name),
                clip_count: None,
                clip_size: None,
                address: None,
                message: "O item equipado nao usa AmmoDrivenWeapon.".into(),
            });
        }

        let clip_size = self.memory.read_i32(equipped + CLIP_SIZE_OFFSET)?;
        let address = equipped + CLIP_COUNT_OFFSET;
        let mut clip_count = self.memory.read_i32(address)?;
        if !(0..=1_000_000).contains(&clip_size) {
            return Err(MemoryError(format!(
                "ClipSize fora do intervalo seguro: {clip_size}."
            )));
        }
        if freeze && clip_count != clip_size {
            self.memory.write_i32(address, clip_size)?;
            clip_count = self.memory.read_i32(address)?;
            if clip_count != clip_size {
                return Err(MemoryError(
                    "A verificacao do freeze de ClipCount falhou.".into(),
                ));
            }
        }

        Ok(ClipStatus {
            enabled: INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire),
            available: true,
            pid: Some(self.pid),
            weapon_name: Some(weapon_name),
            clip_count: Some(clip_count),
            clip_size: Some(clip_size),
            address: Some(format!("0x{address:X}")),
            message: if freeze {
                "Infinite Magazine ativo.".into()
            } else {
                "Arma equipada encontrada.".into()
            },
        })
    }
}
