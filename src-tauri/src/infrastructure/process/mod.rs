//! Fronteira Win32 de processo e memoria (SPEC-001).
//!
//! Contrato arquitetural: nenhum modulo fora de `infrastructure` chama APIs
//! Win32 de processo, memoria, alocacao ou execucao remota. Tudo passa pelos
//! traits de `access` e pelas implementacoes deste modulo.

pub mod access;
pub mod discovery;
mod handle;
pub mod memory;

#[cfg(test)]
pub mod fake;

pub use access::{
    MAX_TRANSFER_SIZE, MIN_USER_ADDRESS, MemoryReader, MemoryWriter, NativeCall, NativeCaller,
    ProcessAccess, validate_range,
};
pub use discovery::{LocatedProcess, executable_sha256, invalidate_hash_cache, locate};
pub use memory::{Access, ProcessMemory};
