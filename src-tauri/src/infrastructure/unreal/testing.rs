//! Mundo Unreal sintetico para testes offline (SPEC-006).
//!
//! Constroi uma `FNamePool` e uma `GUObjectArray` reais sobre `FakeMemory`,
//! usando os offsets do perfil de producao. Isso permite exercitar resolvers,
//! validacao e travessia sem o jogo aberto — e faz um offset errado no perfil
//! quebrar o teste, em vez de passar despercebido.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use crate::build_profiles::{BuildProfile, profiles::fsd_8e22e371};
use crate::infrastructure::process::fake::FakeMemory;

use super::types::UnrealRuntime;

pub const PID: u32 = 4242;
pub const MODULE_BASE: usize = 0x1400_0000;
pub const FNAME_BLOCK: usize = 0x3000_0000;
pub const CHUNK_TABLE: usize = 0x4000_0000;
pub const CHUNK: usize = 0x4100_0000;
/// Base da arena de objetos. Cada objeto e mapeado sob demanda, entao indices
/// altos (como o do save ativo) nao custam memoria.
pub const OBJECT_ARENA: usize = 0x8000_0000;
/// Espaco por objeto: precisa cobrir o maior offset usado (inventario do pawn).
pub const OBJECT_STRIDE: usize = 0x2000;
/// Arena para alocacoes auxiliares (arrays, mapas, structs).
pub const HEAP: usize = 0x7000_0000;

/// Slots do chunk 0 para um mundo pequeno (testes de resolucao e travessia).
pub const DEFAULT_SLOTS: u32 = 4_096;

pub fn profile() -> &'static BuildProfile {
    &fsd_8e22e371::PROFILE
}

pub const FNAME_POOL: usize = MODULE_BASE + 0x065C_4A80;
pub const GUOBJECT_ARRAY: usize = MODULE_BASE + 0x0660_1040;

/// Mundo de teste: memoria falsa + pool de nomes + array de objetos.
pub struct TestWorld {
    memory: FakeMemory,
    fname_cursor: Cell<usize>,
    /// Nomes ja internados: como na pool real, o mesmo texto tem um unico
    /// `ComparisonIndex`.
    interned: RefCell<HashMap<String, u32>>,
    next_index: Cell<u32>,
    live_count: Cell<u32>,
    heap_cursor: Cell<usize>,
    /// Meta-classe compartilhada. No jogo real uma `UClass` nao e instancia de
    /// si mesma, e a validacao de controller depende disso.
    meta_class: Cell<usize>,
}

/// Indice global da meta-classe criada por `TestWorld::new`.
pub const META_CLASS_INDEX: u32 = 0;

impl TestWorld {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_SLOTS)
    }

    /// `slots` define quantos indices a `GUObjectArray` pode enderecar. Mundos
    /// que precisam do indice real do save ativo usam uma capacidade maior.
    pub fn with_capacity(slots: u32) -> Self {
        let memory = FakeMemory::new_writable();
        let offsets = &profile().offsets.unreal;

        // FNamePool: header + tabela de blocos, com o bloco 0 mapeado.
        memory.map_zeroed(FNAME_POOL, 0x200);
        memory.map_zeroed(FNAME_BLOCK, offsets.fname_block_size);
        memory.poke_u32(FNAME_POOL + 0x08, 0); // bloco corrente
        memory.poke_u32(FNAME_POOL + 0x0C, 0); // cursor corrente
        memory.poke_u64(FNAME_POOL + 0x10, FNAME_BLOCK as u64);

        // GUObjectArray: ponteiro para a tabela de chunks + contador.
        memory.map_zeroed(GUOBJECT_ARRAY, 0x40);
        memory.map_zeroed(CHUNK_TABLE, 0x100);
        memory.map_zeroed(CHUNK, slots as usize * offsets.uobject_item_size);
        memory.poke_u64(GUOBJECT_ARRAY + offsets.objects_member, CHUNK_TABLE as u64);
        memory.poke_u32(GUOBJECT_ARRAY + offsets.num_elements, 0);
        memory.poke_u64(CHUNK_TABLE, CHUNK as u64);

        memory.map_zeroed(HEAP, 0x40_000);

        let world = Self {
            memory,
            // O slot 0 fica reservado: indice de FName 0 seria ambiguo.
            fname_cursor: Cell::new(2),
            interned: RefCell::new(HashMap::new()),
            next_index: Cell::new(0),
            live_count: Cell::new(0),
            heap_cursor: Cell::new(HEAP),
            meta_class: Cell::new(0),
        };

        // `UClass` de todas as classes. Ela e a unica auto-referente.
        let meta = world.spawn_object("Class", 0);
        world
            .memory
            .poke_u64(meta + offsets.uobject_class, meta as u64);
        world.meta_class.set(meta);
        world
    }

    pub fn memory(&self) -> &FakeMemory {
        &self.memory
    }

    pub fn runtime(&self) -> UnrealRuntime<&FakeMemory> {
        UnrealRuntime::new(&self.memory, profile(), MODULE_BASE, PID)
    }

    /// Reserva `size` bytes na heap auxiliar e devolve o endereco.
    pub fn alloc(&self, size: usize) -> usize {
        let address = self.heap_cursor.get();
        // Alinhamento de 16 bytes, como as alocacoes reais do UE.
        self.heap_cursor.set((address + size + 15) & !15);
        address
    }

    /// Registra um nome ASCII na pool e devolve seu `ComparisonIndex`.
    pub fn intern(&self, name: &str) -> u32 {
        if let Some(index) = self.interned.borrow().get(name) {
            return *index;
        }
        let offset = self.fname_cursor.get();
        let bytes = name.as_bytes();
        let header = (bytes.len() as u16) << 6; // bit 0 = 0 -> ASCII
        self.memory
            .poke(FNAME_BLOCK + offset, &header.to_le_bytes());
        self.memory.poke(FNAME_BLOCK + offset + 2, bytes);

        let advance = (2 + bytes.len() + 1) & !1;
        self.fname_cursor.set(offset + advance);
        self.memory
            .poke_u32(FNAME_POOL + 0x0C, self.fname_cursor.get() as u32);

        let index = offset as u32 >> 1;
        self.interned.borrow_mut().insert(name.to_string(), index);
        index
    }

    /// Cria um `UClass` com nome proprio e super opcional.
    pub fn spawn_class(&self, name: &str, super_class: Option<usize>) -> usize {
        let offsets = &profile().offsets.unreal;
        let class = self.spawn_object(name, self.meta_class.get());
        self.memory.poke_u64(
            class + offsets.ustruct_super,
            super_class.unwrap_or(0) as u64,
        );
        class
    }

    /// Cria um `UObject` nomeado no proximo indice livre.
    pub fn spawn_object(&self, name: &str, class: usize) -> usize {
        let index = self.next_index.get();
        self.next_index.set(index + 1);
        self.spawn_object_at(index, name, class)
    }

    /// Cria um `UObject` em um indice global especifico.
    ///
    /// Indices nao usados permanecem nulos no chunk, exatamente como slots
    /// livres do jogo real.
    pub fn spawn_object_at(&self, index: u32, name: &str, class: usize) -> usize {
        let offsets = &profile().offsets.unreal;
        let object = OBJECT_ARENA + index as usize * OBJECT_STRIDE;
        self.memory.map_zeroed(object, OBJECT_STRIDE);

        self.memory
            .poke_u32(object + offsets.uobject_internal_index, index);
        self.memory
            .poke_u64(object + offsets.uobject_class, class as u64);
        self.memory
            .poke_u32(object + offsets.uobject_fname, self.intern(name));
        self.memory
            .poke_u32(object + offsets.uobject_fname_number, 0);

        self.memory.poke_u64(
            CHUNK + index as usize * offsets.uobject_item_size,
            object as u64,
        );
        if index >= self.live_count.get() {
            self.live_count.set(index + 1);
            self.memory
                .poke_u32(GUOBJECT_ARRAY + offsets.num_elements, index + 1);
        }
        if index >= self.next_index.get() {
            self.next_index.set(index + 1);
        }
        object
    }

    /// Marca o objeto como instancia numerada (FName Number != 0), usado para
    /// testar o descarte de copias geradas em runtime.
    pub fn set_fname_number(&self, object: usize, number: u32) {
        self.memory.poke_u32(
            object + profile().offsets.unreal.uobject_fname_number,
            number,
        );
    }

    /// Troca o nome de um objeto ja criado.
    pub fn rename(&self, object: usize, name: &str) {
        self.memory.poke_u32(
            object + profile().offsets.unreal.uobject_fname,
            self.intern(name),
        );
    }
}

impl Default for TestWorld {
    fn default() -> Self {
        Self::new()
    }
}
