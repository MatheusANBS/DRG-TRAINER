//! Nucleo do runtime Unreal: o handle que amarra memoria, perfil e modulo.
//!
//! As capacidades sao adicionadas por `impl` nos modulos irmaos (`fname`,
//! `object`, `validation`, `native_call`), um eixo de responsabilidade por
//! arquivo (SPEC-005). Este arquivo so conhece a estrutura, nunca regras
//! especificas do DRG.

use crate::build_profiles::{BuildProfile, UnrealOffsets};
use crate::infrastructure::process::MemoryReader;

use super::cache::CacheKey;

/// Acesso ao runtime Unreal de um processo ja identificado e validado.
///
/// Generico sobre o backend de memoria para que os testes offline usem
/// `FakeMemory` sem que o codigo de producao conheca mocks (SPEC-006).
pub struct UnrealRuntime<M> {
    pub(super) memory: M,
    pub(super) profile: &'static BuildProfile,
    pub(super) module_base: usize,
    pub(super) pid: u32,
}

impl<M: MemoryReader> UnrealRuntime<M> {
    pub fn new(memory: M, profile: &'static BuildProfile, module_base: usize, pid: u32) -> Self {
        Self {
            memory,
            profile,
            module_base,
            pid,
        }
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn profile(&self) -> &'static BuildProfile {
        self.profile
    }

    pub fn module_base(&self) -> usize {
        self.module_base
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub(super) fn unreal(&self) -> &'static UnrealOffsets {
        &self.profile.offsets.unreal
    }

    /// Endereco da `GUObjectArray` nesta instancia do processo.
    pub(super) fn guobject_array(&self) -> usize {
        self.module_base + self.unreal().guobject_array_rva
    }

    /// Endereco da `FNamePool` nesta instancia do processo.
    pub(super) fn fname_pool(&self) -> usize {
        self.module_base + self.unreal().fname_pool_rva
    }

    /// Chave usada pelos caches: qualquer troca de processo ou de perfil
    /// invalida tudo automaticamente.
    pub(crate) fn cache_key(&self) -> CacheKey {
        CacheKey {
            pid: self.pid,
            profile_id: self.profile.id,
        }
    }
}
