#[derive(Clone, Copy)]
pub(super) struct ResourceDefinition {
    pub(super) id: &'static str,
    pub(super) name: &'static str,
    pub(super) category: &'static str,
    pub(super) object_name: &'static str,
    pub(super) savegame_id: [u8; 16],
}

pub(super) const RESOURCE_DEFINITIONS: [ResourceDefinition; 14] = [
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
