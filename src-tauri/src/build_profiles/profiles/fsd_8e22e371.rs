//! Perfil da build shipping do DRG com SHA-256 `8e22e371...`.
//!
//! Todos os valores abaixo foram extraidos e validados nessa build especifica.
//! Para suportar uma build nova, copie este arquivo, troque a hash, marque o
//! status como `Draft`, zere as capacidades ainda nao verificadas e siga
//! `docs/build-support.md`. Nada aqui deve ser reaproveitado "por parecer
//! igual" em outra build.

use crate::build_profiles::types::{
    BuildProfile, Capabilities, Catalog, NativeFunction, NativeFunctions, Offsets, PlayableClass,
    PlayerOffsets, ProfileStatus, ResourceDefinition, SaveOffsets, UnrealOffsets,
};

pub const PROFILE: BuildProfile = BuildProfile {
    id: "fsd-8e22e371",
    display_name: "Deep Rock Galactic (shipping 8e22e371)",
    status: ProfileStatus::Supported,
    executable_name: "FSD-Win64-Shipping.exe",
    executable_sha256: "8e22e3710c607e811e0319a4378d2b7ab2e46c6b46406629a6657b1662c77a41",
    offsets: OFFSETS,
    natives: NATIVES,
    catalog: CATALOG,
    capabilities: Capabilities::ALL,
    resources: &RESOURCES,
};

const OFFSETS: Offsets = Offsets {
    unreal: UnrealOffsets {
        guobject_array_rva: 0x0660_1040,
        fname_pool_rva: 0x065C_4A80,
        objects_member: 0x10,
        num_elements: 0x24,
        uobject_item_size: 0x18,
        objects_per_chunk: 65_536,
        uobject_internal_index: 0x0C,
        uobject_class: 0x10,
        uobject_fname: 0x18,
        uobject_fname_number: 0x1C,
        ustruct_super: 0x40,
        ustruct_child_properties: 0x50,
        uclass_default_object: 0x118,
        ffield_next: 0x20,
        ffield_fname: 0x28,
        fproperty_element_size: 0x3C,
        fproperty_offset_internal: 0x4C,
        fname_block_size: 0x20_000,
        schematic_reward_vtable_slot: 0x270,
    },
    save: SaveOffsets {
        credits: 0x590,
        resources_map: 0x8E8,
        resource_savegame_id: 0xE0,
        resource_map_element_size: 0x1C,
        character_saves: 0x580,
        character_save_size: 0x2F8,
        character_xp: 0x10,
        character_promotions: 0x18,
        owned_perks: 0x160,
        purchased_item_upgrades: 0x620,
        unlocked_items: 0x630,
        owned_items: 0x640,
        schematic_save: 0x218,
        forged_schematics: 0x18,
        owned_schematics: 0x28,
        schematic_savegame_id: 0x3C,
        schematic_settings_all_schematics: 0x348,
        schematic_item: 0xA8,
        overclock_item_overclock: 0x30,
        tset_allocation_flags: 0x20,
        tset_max_index: 0x28,
        tset_element_size: 0x10,
        fsd_savegame_class_index: 1662,
        active_savegame_index: 40754,
    },
    player: PlayerOffsets {
        controller_pawn: 0x250,
        player_inventory: 0xB18,
        inventory_equipped_actor: 0xE0,
        clip_size: 0x684,
        clip_count: 0x6A0,
    },
};

// Implementacoes nativas resolvidas a partir das UFunctions refletidas na build
// verificada. O prefixo e conferido byte a byte antes de qualquer chamada.
const NATIVES: NativeFunctions = NativeFunctions {
    unlock_all_weapons: NativeFunction {
        rva: 0x0169_1540,
        prefix: &[
            0x41, 0x54, 0x41, 0x57, 0x48, 0x81, 0xEC, 0x88, 0x00, 0x00, 0x00, 0x48, 0x8B, 0xD1,
        ],
        label: "Cheat_UnlockAllWeapons",
    },
    unlock_all_perks: NativeFunction {
        rva: 0x0168_E670,
        prefix: &[
            0x48, 0x89, 0x6C, 0x24, 0x18, 0x56, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x8B, 0xF1,
        ],
        label: "C_UnlockAll_Perks",
    },
    unlock_all_upgrades: NativeFunction {
        rva: 0x0169_1300,
        prefix: &[
            0x40, 0x55, 0x41, 0x54, 0x48, 0x81, 0xEC, 0x98, 0x00, 0x00, 0x00, 0x48, 0x8B, 0xD1,
        ],
        label: "Cheat_UnlockAllUpgrades",
    },
    forge_schematic_save: NativeFunction {
        rva: 0x016D_1660,
        prefix: &[
            0x48, 0x89, 0x5C, 0x24, 0x08, 0x48, 0x89, 0x6C, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24,
            0x18, 0x57, 0x48, 0x83, 0xEC, 0x20,
        ],
        label: "SchematicSave::ForgeSchematic",
    },
    overclock_reward: NativeFunction {
        rva: 0x0180_5EB0,
        prefix: &[
            0x48, 0x89, 0x6C, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18, 0x57, 0x48, 0x83, 0xEC,
            0x70, 0x48, 0x8B, 0xFA,
        ],
        label: "OverclockShematicItem::GrantReward",
    },
    skin_reward: NativeFunction {
        rva: 0x0180_60A0,
        prefix: &[
            0x48, 0x89, 0x5C, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24, 0x20, 0x57, 0x48, 0x83, 0xEC,
            0x20, 0x49, 0x8B, 0xF0,
        ],
        label: "SkinSchematicItem::GrantReward",
    },
    vanity_reward: NativeFunction {
        rva: 0x0180_6190,
        prefix: &[0x49, 0x8B, 0xC0, 0x4C, 0x8B, 0x41, 0x28, 0x4D, 0x85, 0xC0],
        label: "VanitySchematicItem::GrantReward",
    },
    victory_pose_reward: NativeFunction {
        rva: 0x0180_61B0,
        prefix: &[0x49, 0x8B, 0xC0, 0x4C, 0x8B, 0x41, 0x28, 0x4D, 0x85, 0xC0],
        label: "VictoryPoseSchematicItem::GrantReward",
    },
    retire_character: NativeFunction {
        rva: 0x016E_BB10,
        prefix: &[
            0x40, 0x55, 0x56, 0x41, 0x56, 0x48, 0x83, 0xEC, 0x30, 0x48, 0x8B, 0x42, 0x30,
        ],
        label: "FSDSaveGame::RetireCharacter",
    },
    save_to_disk: NativeFunction {
        rva: 0x016E_C0F0,
        prefix: &[
            0x48, 0x83, 0xEC, 0x28, 0x80, 0xB9, 0x68, 0x0C, 0x00, 0x00, 0x00,
        ],
        label: "FSDSaveGame::SaveToDisk",
    },
    add_resource: NativeFunction {
        rva: 0x016C_63D0,
        prefix: &[
            0x48, 0x89, 0x74, 0x24, 0x18, 0x57, 0x48, 0x83, 0xEC, 0x40, 0x0F, 0x29, 0x74, 0x24,
            0x30,
        ],
        label: "FSDSaveGame::AddResource",
    },
};

const CATALOG: Catalog = Catalog {
    weapon_count: 26,
    overclock_schematic_count: 160,
    cosmetic_schematic_count: 348,
    max_class_xp: 315_000,
    playable_classes: &PLAYABLE_CLASSES,
    player_controller_names: &[
        "BP_PlayerController_SpaceRig_C",
        "BP_NetworkPlayerController_C",
    ],
    ammo_weapon_class: "AmmoDrivenWeapon",
};

const PLAYABLE_CLASSES: [PlayableClass; 4] = [
    PlayableClass {
        name: "Driller",
        character_id_object: "DrillerID",
        savegame_id: [
            0x9E, 0xDD, 0x56, 0xF1, 0xEE, 0xBC, 0xC5, 0x48, 0x8D, 0x5B, 0x5E, 0x5B, 0x80, 0xB6,
            0x2D, 0xB4,
        ],
    },
    PlayableClass {
        name: "Engineer",
        character_id_object: "EngineerID",
        savegame_id: [
            0x85, 0xEF, 0x62, 0x6C, 0x65, 0xF1, 0x02, 0x4A, 0x8D, 0xFE, 0xB5, 0xD0, 0xF3, 0x90,
            0x9D, 0x2E,
        ],
    },
    PlayableClass {
        name: "Gunner",
        character_id_object: "GunnerID",
        savegame_id: [
            0xAE, 0x56, 0xE1, 0x80, 0xFE, 0xC0, 0xC4, 0x4D, 0x96, 0xFA, 0x29, 0xC2, 0x83, 0x66,
            0xB9, 0x7B,
        ],
    },
    PlayableClass {
        name: "Scout",
        character_id_object: "ScoutID",
        savegame_id: [
            0x30, 0xD8, 0xEA, 0x17, 0xD8, 0xFB, 0xBA, 0x4C, 0x95, 0x30, 0x6D, 0xE9, 0x65, 0x5C,
            0x2F, 0x8C,
        ],
    },
];

const RESOURCES: [ResourceDefinition; 14] = [
    ResourceDefinition {
        id: "bismor",
        name: "Bismor",
        category: "Minerals",
        object_name: "RES_CARVED_Bismor",
        savegame_id: [
            0xAF, 0x0D, 0xC4, 0xFE, 0x83, 0x61, 0xBB, 0x48, 0xB3, 0x2C, 0x92, 0xCC, 0x97, 0xE2,
            0x1D, 0xE7,
        ],
    },
    ResourceDefinition {
        id: "croppa",
        name: "Croppa",
        category: "Minerals",
        object_name: "RES_VEIN_Croppa",
        savegame_id: [
            0x8A, 0xA7, 0xFB, 0x43, 0x29, 0x3A, 0x0B, 0x49, 0xB8, 0xBE, 0x42, 0xFF, 0xE0, 0x68,
            0xA4, 0x4C,
        ],
    },
    ResourceDefinition {
        id: "enor_pearl",
        name: "Enor Pearl",
        category: "Minerals",
        object_name: "RES_EMBED_Enor",
        savegame_id: [
            0x48, 0x8D, 0x05, 0x14, 0x6F, 0x5F, 0x75, 0x4B, 0xA3, 0xD4, 0x61, 0x0D, 0x08, 0xC0,
            0x60, 0x3E,
        ],
    },
    ResourceDefinition {
        id: "jadiz",
        name: "Jadiz",
        category: "Minerals",
        object_name: "RES_EMBED_Jadiz",
        savegame_id: [
            0x22, 0xBC, 0x4F, 0x7D, 0x07, 0xD1, 0x3E, 0x43, 0xBF, 0xCA, 0x81, 0xBD, 0x9C, 0x14,
            0xB1, 0xAF,
        ],
    },
    ResourceDefinition {
        id: "magnite",
        name: "Magnite",
        category: "Minerals",
        object_name: "RES_CARVED_Magnite",
        savegame_id: [
            0xAA, 0xDE, 0xD8, 0x76, 0x6C, 0x22, 0x7D, 0x40, 0x80, 0x32, 0xAF, 0xD1, 0x8D, 0x63,
            0x56, 0x1E,
        ],
    },
    ResourceDefinition {
        id: "umanite",
        name: "Umanite",
        category: "Minerals",
        object_name: "RES_CARVED_Umanite",
        savegame_id: [
            0x5F, 0x2B, 0xCF, 0x83, 0x47, 0x76, 0x0A, 0x42, 0xA2, 0x3B, 0x6E, 0xDC, 0x07, 0xC0,
            0x94, 0x1D,
        ],
    },
    ResourceDefinition {
        id: "phazyonite",
        name: "Phazyonite",
        category: "Minerals",
        object_name: "RES_CARVED_Phazyonite",
        savegame_id: [
            0x67, 0x66, 0x8A, 0xAE, 0x82, 0x8F, 0xDB, 0x48, 0xA9, 0x11, 0x1E, 0x1B, 0x91, 0x2D,
            0xBF, 0xA4,
        ],
    },
    ResourceDefinition {
        id: "barley_bulb",
        name: "Barley Bulb",
        category: "Brewing",
        object_name: "RES_COLLECT_Barley1",
        savegame_id: [
            0x22, 0xDA, 0xA7, 0x57, 0xAD, 0x7A, 0x80, 0x49, 0x89, 0x1B, 0x17, 0xED, 0xCC, 0x2F,
            0xE0, 0x98,
        ],
    },
    ResourceDefinition {
        id: "malt_star",
        name: "Malt Star",
        category: "Brewing",
        object_name: "RES_COLLECT_Barley3",
        savegame_id: [
            0x41, 0xEA, 0x55, 0x0C, 0x1D, 0x46, 0xC5, 0x4B, 0xBE, 0x2E, 0x9C, 0xA5, 0xA7, 0xAC,
            0xCB, 0x06,
        ],
    },
    ResourceDefinition {
        id: "starch_nut",
        name: "Starch Nut",
        category: "Brewing",
        object_name: "RES_COLLECT_Barley4",
        savegame_id: [
            0x72, 0x31, 0x22, 0x04, 0xE2, 0x87, 0xBC, 0x41, 0x81, 0x55, 0x40, 0xA0, 0xCF, 0x88,
            0x12, 0x80,
        ],
    },
    ResourceDefinition {
        id: "yeast_cone",
        name: "Yeast Cone",
        category: "Brewing",
        object_name: "RES_COLLECT_Barley2",
        savegame_id: [
            0x07, 0x85, 0x48, 0xB9, 0x32, 0x32, 0xC0, 0x40, 0x85, 0xF8, 0x92, 0xE0, 0x84, 0xA7,
            0x41, 0x00,
        ],
    },
    ResourceDefinition {
        id: "blank_matrix_core",
        name: "Blank Matrix Core",
        category: "Special",
        object_name: "RES_BlankSchematic",
        savegame_id: [
            0xA1, 0x0C, 0xB2, 0x85, 0x38, 0x71, 0xFB, 0x49, 0x9A, 0xC8, 0x54, 0xA1, 0xCD, 0xE2,
            0x20, 0x2C,
        ],
    },
    ResourceDefinition {
        id: "error_cube",
        name: "Error Cube",
        category: "Special",
        object_name: "RES_EMBED_UnknownArtifact",
        savegame_id: [
            0x58, 0x28, 0x65, 0x2C, 0x9A, 0x5D, 0xE8, 0x45, 0xA9, 0xE2, 0xE1, 0xB8, 0xB4, 0x63,
            0xC5, 0x16,
        ],
    },
    ResourceDefinition {
        id: "data_cell",
        name: "Data Cell",
        category: "Special",
        object_name: "RES_DataCell",
        savegame_id: [
            0x99, 0xFA, 0x52, 0x6A, 0xD8, 0x77, 0x48, 0x45, 0x94, 0x98, 0x90, 0x5A, 0x27, 0x86,
            0x93, 0xF6,
        ],
    },
];
