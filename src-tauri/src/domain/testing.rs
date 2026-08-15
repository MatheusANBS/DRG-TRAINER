//! Save sintetico do DRG para testes offline (SPEC-006).
//!
//! Monta um `FSDSaveGame` plausivel sobre o mundo Unreal de teste, usando os
//! offsets e o catalogo do perfil real. Serve para exercitar leitura de save,
//! validacoes e as transacoes de mutacao sem o jogo aberto.

use crate::build_profiles::BuildProfile;
use crate::infrastructure::process::MemoryReader;
use crate::infrastructure::unreal::testing::TestWorld;

pub use crate::infrastructure::unreal::testing::{MODULE_BASE, PID, profile};

/// Capacidade da `GUObjectArray` de teste: precisa alcancar o indice real do
/// save ativo para exercitar o caminho rapido de resolucao.
fn required_slots() -> u32 {
    profile().offsets.save.active_savegame_index + 8
}

/// Mundo com um save ativo montado e pronto para leitura.
pub struct SaveWorld {
    pub world: TestWorld,
    pub save_class: usize,
    pub save: usize,
    resource_elements: usize,
    character_saves: usize,
}

impl SaveWorld {
    pub fn new() -> Self {
        let profile = profile();
        let offsets = &profile.offsets.save;
        let unreal = &profile.offsets.unreal;
        let world = TestWorld::with_capacity(required_slots());

        // A UClass do save vive no indice declarado no perfil.
        let save_class = world.spawn_object_at(offsets.fsd_savegame_class_index, "FSDSaveGame", 0);
        world
            .memory()
            .poke_u64(save_class + unreal.uobject_class, save_class as u64);

        // A instancia ativa vive no indice de atalho, com o mesmo FName.
        let save = world.spawn_object_at(offsets.active_savegame_index, "FSDSaveGame", save_class);
        world.memory().poke_i32(save + offsets.credits, 49_995);

        // Mapa de recursos: uma entrada por recurso do catalogo, zeradas.
        let resource_count = profile.resources.len();
        let resource_elements = world.alloc(resource_count * offsets.resource_map_element_size);
        let map = save + offsets.resources_map;
        world.memory().poke_u64(map, resource_elements as u64);
        world.memory().poke_i32(map + 8, resource_count as i32);
        world.memory().poke_i32(map + 12, 128);
        world.memory().poke_i32(map + 0x28, resource_count as i32);
        world.memory().poke_i32(map + 0x2C, 128);
        // Bits de alocacao: todas as entradas ocupadas.
        for index in 0..resource_count {
            let word_address = map + 0x10 + (index / 32) * 4;
            let current = world.memory().read_u32(word_address).unwrap_or(0);
            world
                .memory()
                .poke_u32(word_address, current | (1 << (index % 32)));
        }
        for (index, resource) in profile.resources.iter().enumerate() {
            let entry = resource_elements + index * offsets.resource_map_element_size;
            world.memory().poke(entry, &resource.savegame_id);
            world.memory().poke_f32(entry + 16, 0.0);
        }

        // CharacterSaves: uma entrada por classe jogavel, na ordem do catalogo.
        let classes = profile.catalog.playable_classes;
        let character_saves = world.alloc(classes.len() * offsets.character_save_size);
        world
            .memory()
            .poke_u64(save + offsets.character_saves, character_saves as u64);
        world
            .memory()
            .poke_i32(save + offsets.character_saves + 8, classes.len() as i32);
        world
            .memory()
            .poke_i32(save + offsets.character_saves + 12, classes.len() as i32);
        for (index, class) in classes.iter().enumerate() {
            let entry = character_saves + index * offsets.character_save_size;
            world.memory().poke(entry, &class.savegame_id);
            world.memory().poke_i32(entry + offsets.character_xp, 0);
            world
                .memory()
                .poke_i32(entry + offsets.character_promotions, 0);
        }

        Self {
            world,
            save_class,
            save,
            resource_elements,
            character_saves,
        }
    }

    pub fn profile(&self) -> &'static BuildProfile {
        profile()
    }

    /// Mapeia os bytes de todas as rotinas nativas do perfil, para que a
    /// validacao de assinatura passe.
    pub fn map_native_routines(&self) -> &Self {
        for native in native_list(profile()) {
            self.world
                .memory()
                .map(native.address(MODULE_BASE), native.prefix.to_vec());
        }
        self
    }

    /// Corrompe a assinatura de uma rotina especifica.
    pub fn corrupt_native(&self, native: crate::build_profiles::NativeFunction) -> &Self {
        self.world
            .memory()
            .map(native.address(MODULE_BASE), vec![0x90; native.prefix.len()]);
        self
    }

    /// Cria o PlayerController da sessao, com ou sem Pawn ativo.
    pub fn spawn_controller(&self, with_pawn: bool) -> usize {
        let name = profile().catalog.player_controller_names[0];
        let class = self.world.spawn_class(name, None);
        let controller = self.world.spawn_object(name, class);
        // `controller_is_valid` exige que o FName do objeto e o da classe batam.
        if with_pawn {
            let pawn_class = self.world.spawn_class("BP_PlayerCharacter_C", None);
            let pawn = self.world.spawn_object("BP_PlayerCharacter_C", pawn_class);
            self.world.memory().poke_u64(
                controller + profile().offsets.player.controller_pawn,
                pawn as u64,
            );
        }
        controller
    }

    /// Define a quantidade de um recurso no mapa do save.
    pub fn set_resource(&self, id: &str, amount: f32) -> &Self {
        let offsets = &profile().offsets.save;
        let index = profile()
            .resources
            .iter()
            .position(|resource| resource.id == id)
            .expect("recurso deve existir no catalogo");
        let entry = self.resource_elements + index * offsets.resource_map_element_size;
        self.world.memory().poke_f32(entry + 16, amount);
        self
    }

    /// Define XP e promocoes de uma classe.
    pub fn set_class_progress(&self, class_name: &str, xp: i32, promotions: i32) -> &Self {
        let offsets = &profile().offsets.save;
        let index = self.class_index(class_name);
        let entry = self.character_saves + index * offsets.character_save_size;
        self.world
            .memory()
            .poke_i32(entry + offsets.character_xp, xp);
        self.world
            .memory()
            .poke_i32(entry + offsets.character_promotions, promotions);
        self
    }

    /// Endereco do XP de uma classe, para assertivas diretas.
    pub fn class_xp_address(&self, class_name: &str) -> usize {
        let offsets = &profile().offsets.save;
        self.character_saves
            + self.class_index(class_name) * offsets.character_save_size
            + offsets.character_xp
    }

    /// Endereco das promocoes de uma classe.
    pub fn class_promotions_address(&self, class_name: &str) -> usize {
        let offsets = &profile().offsets.save;
        self.character_saves
            + self.class_index(class_name) * offsets.character_save_size
            + offsets.character_promotions
    }

    /// Zera o GUID de uma classe, simulando um save incompleto.
    pub fn corrupt_class_guid(&self, class_name: &str) -> &Self {
        let offsets = &profile().offsets.save;
        let entry =
            self.character_saves + self.class_index(class_name) * offsets.character_save_size;
        self.world.memory().poke(entry, &[0_u8; 16]);
        self
    }

    /// Troca o nome da UClass do save, quebrando a validacao de identidade.
    pub fn rename_save_class(&self, name: &str) -> &Self {
        self.world.rename(self.save_class, name);
        self
    }

    /// Escreve um `TArray<FGuid>` em um offset do save.
    pub fn write_guid_array(&self, offset: usize, ids: &[[u8; 16]]) -> &Self {
        let data = self.world.alloc(ids.len().max(1) * 16);
        for (index, id) in ids.iter().enumerate() {
            self.world.memory().poke(data + index * 16, id);
        }
        self.world
            .memory()
            .poke_u64(self.save + offset, data as u64);
        self.world
            .memory()
            .poke_i32(self.save + offset + 8, ids.len() as i32);
        self.world
            .memory()
            .poke_i32(self.save + offset + 12, ids.len() as i32);
        self
    }

    /// Define os contadores de itens desbloqueados/adquiridos.
    pub fn set_item_counts(&self, unlocked: i32, owned: i32) -> &Self {
        let offsets = &profile().offsets.save;
        self.world
            .memory()
            .poke_i32(self.save + offsets.unlocked_items + 8, unlocked);
        self.world
            .memory()
            .poke_i32(self.save + offsets.owned_items + 8, owned);
        self
    }

    /// Define o tamanho de um `TArray` de progressao sem preencher os dados.
    pub fn set_progression_count(&self, offset: usize, count: i32) -> &Self {
        self.world.memory().poke_i32(self.save + offset + 8, count);
        self.world
            .memory()
            .poke_i32(self.save + offset + 12, count.max(1));
        if count > 0 {
            let data = self.world.alloc(count as usize * 16);
            self.world
                .memory()
                .poke_u64(self.save + offset, data as u64);
        }
        self
    }

    fn class_index(&self, class_name: &str) -> usize {
        profile()
            .catalog
            .playable_classes
            .iter()
            .position(|class| class.name == class_name)
            .expect("classe deve existir no catalogo")
    }
}

impl Default for SaveWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Todas as rotinas nativas declaradas no perfil.
pub fn native_list(profile: &'static BuildProfile) -> Vec<crate::build_profiles::NativeFunction> {
    let natives = &profile.natives;
    vec![
        natives.unlock_all_weapons,
        natives.unlock_all_perks,
        natives.unlock_all_upgrades,
        natives.forge_schematic_save,
        natives.overclock_reward,
        natives.skin_reward,
        natives.vanity_reward,
        natives.victory_pose_reward,
        natives.retire_character,
        natives.save_to_disk,
        natives.add_resource,
    ]
}
