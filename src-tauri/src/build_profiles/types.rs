//! Tipos que descrevem um perfil de build do DRG (SPEC-002).
//!
//! Um `BuildProfile` e imutavel e `'static`: ele e o unico lugar onde vivem
//! hash esperada, offsets, assinaturas e enderecos de funcoes nativas. Nenhum
//! modulo generico de leitura/escrita pode conter esses valores.

use serde::Serialize;

/// Ciclo de vida de um perfil (SPEC-003).
///
/// Somente perfis `Supported` sao expostos em binarios publicos; `Draft` e
/// `Verified` existem para o fluxo de onboarding de uma build nova.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    /// Offsets preenchidos, nada validado ainda.
    Draft,
    /// Validacoes offline passaram; live tests ainda nao foram registrados.
    Verified,
    /// Live tests aprovados e evidencias registradas em `docs/build-support.md`.
    Supported,
    /// Build antiga mantida apenas para referencia historica.
    Deprecated,
}

impl ProfileStatus {
    /// Somente perfis promovidos podem executar operacoes dependentes de offset.
    pub fn allows_memory_operations(self) -> bool {
        matches!(self, Self::Supported)
    }
}

/// Capacidades verificadas por perfil. Uma capacidade so vira `true` depois de
/// validada na build correspondente; ate la a acao fica desabilitada na UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub credits: bool,
    pub resources: bool,
    pub weapon_unlock: bool,
    pub perk_unlock: bool,
    pub gear_unlock: bool,
    pub schematic_unlock: bool,
    pub class_level: bool,
    pub promotion: bool,
    pub infinite_magazine: bool,
    pub weapon_damage: bool,
}

impl Capabilities {
    /// Perfil sem nenhuma capacidade: usado como resposta para builds
    /// desconhecidas, garantindo que a UI desabilite tudo por padrao.
    pub const NONE: Self = Self {
        credits: false,
        resources: false,
        weapon_unlock: false,
        perk_unlock: false,
        gear_unlock: false,
        schematic_unlock: false,
        class_level: false,
        promotion: false,
        infinite_magazine: false,
        weapon_damage: false,
    };

    pub const ALL: Self = Self {
        credits: true,
        resources: true,
        weapon_unlock: true,
        perk_unlock: true,
        gear_unlock: true,
        schematic_unlock: true,
        class_level: true,
        promotion: true,
        infinite_magazine: true,
        weapon_damage: true,
    };
}

/// Identificador estavel de uma capacidade, usado para mensagens de erro e para
/// o gate declarativo em `BuildProfile::require`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Credits,
    Resources,
    WeaponUnlock,
    PerkUnlock,
    GearUnlock,
    SchematicUnlock,
    ClassLevel,
    Promotion,
    InfiniteMagazine,
    WeaponDamage,
}

impl Capability {
    pub fn label(self) -> &'static str {
        match self {
            Self::Credits => "Creditos",
            Self::Resources => "Recursos",
            Self::WeaponUnlock => "Unlock All Weapons",
            Self::PerkUnlock => "Unlock All Perks",
            Self::GearUnlock => "Unlock All Gear Modifications",
            Self::SchematicUnlock => "Unlock All Overclocks & Cosmetics",
            Self::ClassLevel => "Max Class Level",
            Self::Promotion => "Promote All Classes",
            Self::InfiniteMagazine => "Infinite Magazine",
            Self::WeaponDamage => "Weapon Damage",
        }
    }

    pub fn is_enabled_in(self, capabilities: &Capabilities) -> bool {
        match self {
            Self::Credits => capabilities.credits,
            Self::Resources => capabilities.resources,
            Self::WeaponUnlock => capabilities.weapon_unlock,
            Self::PerkUnlock => capabilities.perk_unlock,
            Self::GearUnlock => capabilities.gear_unlock,
            Self::SchematicUnlock => capabilities.schematic_unlock,
            Self::ClassLevel => capabilities.class_level,
            Self::Promotion => capabilities.promotion,
            Self::InfiniteMagazine => capabilities.infinite_magazine,
            Self::WeaponDamage => capabilities.weapon_damage,
        }
    }
}

/// Offsets do runtime Unreal (layout de UObject/FName/FProperty).
#[derive(Debug, Clone, Copy)]
pub struct UnrealOffsets {
    pub guobject_array_rva: usize,
    pub fname_pool_rva: usize,
    pub objects_member: usize,
    pub num_elements: usize,
    pub uobject_item_size: usize,
    pub objects_per_chunk: u32,
    pub uobject_internal_index: usize,
    pub uobject_class: usize,
    pub uobject_fname: usize,
    pub uobject_fname_number: usize,
    pub ustruct_super: usize,
    pub ustruct_child_properties: usize,
    pub uclass_default_object: usize,
    pub ffield_next: usize,
    pub ffield_fname: usize,
    pub fproperty_element_size: usize,
    pub fproperty_offset_internal: usize,
    pub fname_block_size: usize,
    /// Slot da vtable que aponta para `GrantReward` nos itens de esquema.
    pub schematic_reward_vtable_slot: usize,
}

/// Offsets dentro de `FSDSaveGame` e estruturas associadas.
#[derive(Debug, Clone, Copy)]
pub struct SaveOffsets {
    pub credits: usize,
    pub resources_map: usize,
    pub resource_savegame_id: usize,
    pub resource_map_element_size: usize,
    pub character_saves: usize,
    pub character_save_size: usize,
    pub character_xp: usize,
    pub character_promotions: usize,
    pub owned_perks: usize,
    pub purchased_item_upgrades: usize,
    pub unlocked_items: usize,
    pub owned_items: usize,
    pub schematic_save: usize,
    pub forged_schematics: usize,
    pub owned_schematics: usize,
    pub schematic_savegame_id: usize,
    pub schematic_settings_all_schematics: usize,
    pub schematic_item: usize,
    pub overclock_item_overclock: usize,
    pub tset_allocation_flags: usize,
    pub tset_max_index: usize,
    pub tset_element_size: usize,
    pub fsd_savegame_class_index: u32,
    pub active_savegame_index: u32,
}

/// Offsets do player e da arma equipada.
#[derive(Debug, Clone, Copy)]
pub struct PlayerOffsets {
    pub controller_pawn: usize,
    pub player_inventory: usize,
    pub inventory_equipped_actor: usize,
    pub clip_size: usize,
    pub clip_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Offsets {
    pub unreal: UnrealOffsets,
    pub save: SaveOffsets,
    pub player: PlayerOffsets,
}

/// Uma funcao nativa do jogo: onde ela esta e como confirmar que e ela mesma.
///
/// O prefixo e obrigatorio: nenhuma chamada remota acontece sem validar os
/// bytes iniciais contra o perfil (SPEC-001).
#[derive(Debug, Clone, Copy)]
pub struct NativeFunction {
    pub rva: usize,
    pub prefix: &'static [u8],
    pub label: &'static str,
}

impl NativeFunction {
    pub fn address(&self, module_base: usize) -> usize {
        module_base + self.rva
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NativeFunctions {
    pub unlock_all_weapons: NativeFunction,
    pub unlock_all_perks: NativeFunction,
    pub unlock_all_upgrades: NativeFunction,
    pub forge_schematic_save: NativeFunction,
    pub overclock_reward: NativeFunction,
    pub skin_reward: NativeFunction,
    pub vanity_reward: NativeFunction,
    pub victory_pose_reward: NativeFunction,
    pub retire_character: NativeFunction,
    pub save_to_disk: NativeFunction,
    pub add_resource: NativeFunction,
}

/// Uma classe jogavel identificada pelo GUID que aparece no save.
#[derive(Debug, Clone, Copy)]
pub struct PlayableClass {
    pub name: &'static str,
    pub savegame_id: [u8; 16],
    /// Nome do `PlayerCharacterID` correspondente carregado na Space Rig.
    pub character_id_object: &'static str,
}

/// Contagens esperadas do catalogo do jogo nesta build. Divergencias abortam a
/// operacao em vez de escrever em cima de dados desconhecidos.
#[derive(Debug, Clone, Copy)]
pub struct Catalog {
    pub weapon_count: usize,
    pub overclock_schematic_count: usize,
    pub cosmetic_schematic_count: usize,
    pub max_class_xp: i32,
    pub playable_classes: &'static [PlayableClass],
    pub player_controller_names: &'static [&'static str],
    pub ammo_weapon_class: &'static str,
}

impl Catalog {
    pub fn total_schematic_count(&self) -> usize {
        self.overclock_schematic_count + self.cosmetic_schematic_count
    }
}

/// Perfil completo de uma build suportada.
#[derive(Debug, Clone, Copy)]
pub struct BuildProfile {
    pub id: &'static str,
    pub display_name: &'static str,
    pub status: ProfileStatus,
    pub executable_name: &'static str,
    pub executable_sha256: &'static str,
    pub offsets: Offsets,
    pub natives: NativeFunctions,
    pub catalog: Catalog,
    pub capabilities: Capabilities,
    pub resources: &'static [ResourceDefinition],
}

/// Recurso do inventario: identidade estavel para a UI + GUID do save.
#[derive(Debug, Clone, Copy)]
pub struct ResourceDefinition {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub object_name: &'static str,
    pub savegame_id: [u8; 16],
}

impl BuildProfile {
    /// Gate declarativo: recusa a operacao quando a capacidade nao foi
    /// verificada nesta build, antes de qualquer acesso a memoria.
    pub fn require(&self, capability: Capability) -> crate::shared::Result<()> {
        if !capability.is_enabled_in(&self.capabilities) {
            return Err(crate::shared::TrainerError::new(
                crate::shared::ErrorCode::CapabilityUnavailable,
                format!(
                    "{} nao foi verificado no perfil {}.",
                    capability.label(),
                    self.id
                ),
            ));
        }
        Ok(())
    }

    pub fn resource(&self, id: &str) -> Option<&'static ResourceDefinition> {
        self.resources.iter().find(|resource| resource.id == id)
    }
}

/// Descricao serializavel de um perfil, enviada ao frontend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDescriptor {
    pub id: String,
    pub display_name: String,
    pub status: ProfileStatus,
    pub executable_sha256: String,
    pub capabilities: Capabilities,
}

impl From<&'static BuildProfile> for ProfileDescriptor {
    fn from(profile: &'static BuildProfile) -> Self {
        Self {
            id: profile.id.into(),
            display_name: profile.display_name.into(),
            status: profile.status,
            executable_sha256: profile.executable_sha256.into(),
            capabilities: profile.capabilities,
        }
    }
}
