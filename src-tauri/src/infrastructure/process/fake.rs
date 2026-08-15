//! Backend de memoria falso para testes offline (SPEC-006).
//!
//! Deliberadamente simples: um mapa de regioes sobre buffers reais, com as
//! mesmas validacoes de intervalo e de escrita parcial do backend Win32. Nao e
//! um mock de expectativas — e uma memoria de verdade, so que em processo.
//!
//! Chamadas nativas nao sao simuladas: elas sao registradas e retornam o exit
//! code configurado, para que os testes verifiquem *que* uma rotina seria
//! chamada sem fingir que o jogo executou algo.

use std::cell::RefCell;
use std::collections::BTreeMap;

use super::access::{MemoryReader, MemoryWriter, NativeCall, NativeCaller, validate_range};
use crate::shared::error::{ErrorCode, Result, TrainerError};

/// Registro de uma chamada nativa que o codigo sob teste tentou executar.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordedCall {
    pub address: usize,
    pub label: String,
    pub call: NativeCallRecord,
}

/// Copia comparavel de `NativeCall` (o original nao deriva `PartialEq` por
/// conter float).
#[derive(Debug, Clone, PartialEq)]
pub enum NativeCallRecord {
    Direct {
        argument: usize,
    },
    TwoArgs {
        first: usize,
        second: usize,
    },
    ThreeArgs {
        first: usize,
        second: usize,
        third: usize,
    },
    ResourceDelta {
        save: usize,
        resource: usize,
        amount: f32,
    },
}

impl From<NativeCall> for NativeCallRecord {
    fn from(call: NativeCall) -> Self {
        match call {
            NativeCall::Direct { argument } => Self::Direct { argument },
            NativeCall::TwoArgs { first, second } => Self::TwoArgs { first, second },
            NativeCall::ThreeArgs {
                first,
                second,
                third,
            } => Self::ThreeArgs {
                first,
                second,
                third,
            },
            NativeCall::ResourceDelta {
                save,
                resource,
                amount,
            } => Self::ResourceDelta {
                save,
                resource,
                amount,
            },
        }
    }
}

/// Memoria de processo simulada por regioes contiguas.
pub struct FakeMemory {
    /// base -> bytes da regiao.
    regions: RefCell<BTreeMap<usize, Vec<u8>>>,
    writable: bool,
    calls: RefCell<Vec<RecordedCall>>,
    native_exit_code: RefCell<u32>,
    /// Quando definido, toda chamada nativa falha com esta mensagem.
    native_failure: RefCell<Option<String>>,
}

impl FakeMemory {
    pub fn new_writable() -> Self {
        Self::new(true)
    }

    pub fn new_read_only() -> Self {
        Self::new(false)
    }

    fn new(writable: bool) -> Self {
        Self {
            regions: RefCell::new(BTreeMap::new()),
            writable,
            calls: RefCell::new(Vec::new()),
            native_exit_code: RefCell::new(0),
            native_failure: RefCell::new(None),
        }
    }

    /// Mapeia uma regiao com o conteudo informado.
    pub fn map(&self, base: usize, bytes: impl Into<Vec<u8>>) -> &Self {
        self.regions.borrow_mut().insert(base, bytes.into());
        self
    }

    /// Mapeia uma regiao zerada de `size` bytes.
    pub fn map_zeroed(&self, base: usize, size: usize) -> &Self {
        self.map(base, vec![0_u8; size])
    }

    /// Grava um `u64` no endereco, expandindo a regiao existente se necessario.
    pub fn poke_u64(&self, address: usize, value: u64) -> &Self {
        self.poke(address, &value.to_le_bytes())
    }

    pub fn poke_u32(&self, address: usize, value: u32) -> &Self {
        self.poke(address, &value.to_le_bytes())
    }

    pub fn poke_i32(&self, address: usize, value: i32) -> &Self {
        self.poke(address, &value.to_le_bytes())
    }

    pub fn poke_f32(&self, address: usize, value: f32) -> &Self {
        self.poke(address, &value.to_le_bytes())
    }

    /// Escreve bytes ignorando o modo somente-leitura (preparacao de cenario).
    pub fn poke(&self, address: usize, bytes: &[u8]) -> &Self {
        let mut regions = self.regions.borrow_mut();
        if let Some((base, region)) = Self::region_for_mut(&mut regions, address, bytes.len()) {
            let offset = address - base;
            region[offset..offset + bytes.len()].copy_from_slice(bytes);
            return self;
        }
        drop(regions);
        self.map(address, bytes.to_vec())
    }

    pub fn set_native_exit_code(&self, code: u32) -> &Self {
        *self.native_exit_code.borrow_mut() = code;
        self
    }

    pub fn fail_native_calls(&self, message: impl Into<String>) -> &Self {
        *self.native_failure.borrow_mut() = Some(message.into());
        self
    }

    /// Chamadas nativas registradas, na ordem em que foram tentadas.
    pub fn recorded_calls(&self) -> Vec<RecordedCall> {
        self.calls.borrow().clone()
    }

    fn region_for_mut(
        regions: &mut BTreeMap<usize, Vec<u8>>,
        address: usize,
        size: usize,
    ) -> Option<(usize, &mut Vec<u8>)> {
        let base = *regions
            .range(..=address)
            .next_back()
            .filter(|(base, region)| address - **base + size <= region.len())
            .map(|(base, _)| base)?;
        regions.get_mut(&base).map(|region| (base, region))
    }
}

impl MemoryReader for FakeMemory {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        validate_range(address, size)?;
        let regions = self.regions.borrow();
        let (base, region) = regions
            .range(..=address)
            .next_back()
            .filter(|(base, region)| address - **base + size <= region.len())
            .ok_or_else(|| {
                TrainerError::new(
                    ErrorCode::MemoryReadFailed,
                    format!("Falha ao ler 0x{address:X} ({size} bytes): regiao nao mapeada"),
                )
            })?;
        let offset = address - base;
        Ok(region[offset..offset + size].to_vec())
    }
}

impl MemoryWriter for FakeMemory {
    fn write_bytes(&self, address: usize, bytes: &[u8]) -> Result<()> {
        if !self.writable {
            return Err(TrainerError::new(
                ErrorCode::ReadOnlyHandle,
                "O processo foi aberto em modo somente leitura.",
            ));
        }
        validate_range(address, bytes.len())?;
        let mut regions = self.regions.borrow_mut();
        let (base, region) =
            Self::region_for_mut(&mut regions, address, bytes.len()).ok_or_else(|| {
                TrainerError::new(
                    ErrorCode::MemoryWriteFailed,
                    format!("Falha ao escrever em 0x{address:X}: regiao nao mapeada"),
                )
            })?;
        let offset = address - base;
        region[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }
}

impl NativeCaller for FakeMemory {
    fn call_native(&self, address: usize, call: NativeCall, label: &str) -> Result<u32> {
        if !self.writable {
            return Err(TrainerError::new(
                ErrorCode::ReadOnlyHandle,
                "O processo foi aberto em modo somente leitura.",
            ));
        }
        self.calls.borrow_mut().push(RecordedCall {
            address,
            label: label.to_string(),
            call: call.into(),
        });
        if let Some(message) = self.native_failure.borrow().as_ref() {
            return Err(TrainerError::new(
                ErrorCode::NativeCallFailed,
                format!("{label}: {message}"),
            ));
        }
        Ok(*self.native_exit_code.borrow())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: usize = 0x10_0000;

    #[test]
    fn reads_back_what_was_mapped() {
        let memory = FakeMemory::new_writable();
        memory.map(BASE, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(memory.read_bytes(BASE, 4).unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(memory.read_u32(BASE).unwrap(), 0x0403_0201);
        assert_eq!(memory.read_bytes(BASE + 4, 4).unwrap(), vec![5, 6, 7, 8]);
    }

    #[test]
    fn unmapped_reads_fail_instead_of_returning_zeroes() {
        let memory = FakeMemory::new_writable();
        memory.map_zeroed(BASE, 16);
        let error = memory
            .read_u32(BASE + 0x1000)
            .expect_err("regiao nao mapeada deve falhar");
        assert_eq!(error.code(), ErrorCode::MemoryReadFailed);
    }

    #[test]
    fn reads_that_overflow_the_region_fail() {
        let memory = FakeMemory::new_writable();
        memory.map_zeroed(BASE, 8);
        assert!(memory.read_bytes(BASE, 8).is_ok());
        assert!(memory.read_bytes(BASE, 9).is_err());
    }

    #[test]
    fn read_only_backend_refuses_writes_and_native_calls() {
        let memory = FakeMemory::new_read_only();
        memory.map_zeroed(BASE, 8);
        assert_eq!(
            memory
                .write_i32(BASE, 7)
                .expect_err("escrita deve falhar")
                .code(),
            ErrorCode::ReadOnlyHandle
        );
        assert_eq!(
            memory
                .call_native(BASE, NativeCall::Direct { argument: 1 }, "teste")
                .expect_err("chamada deve falhar")
                .code(),
            ErrorCode::ReadOnlyHandle
        );
        assert!(memory.recorded_calls().is_empty());
    }

    #[test]
    fn verified_write_reports_the_previous_value() {
        let memory = FakeMemory::new_writable();
        memory.map_zeroed(BASE, 8);
        memory.poke_i32(BASE, 42);
        let previous = memory
            .write_i32_verified(BASE, 99)
            .expect("escrita verificada");
        assert_eq!(previous, 42);
        assert_eq!(memory.read_i32(BASE).unwrap(), 99);
    }

    #[test]
    fn native_calls_are_recorded_in_order() {
        let memory = FakeMemory::new_writable();
        memory.set_native_exit_code(3);
        let code = memory
            .call_native(
                0x4000,
                NativeCall::TwoArgs {
                    first: 1,
                    second: 2,
                },
                "Promote",
            )
            .expect("chamada deve ser registrada");
        assert_eq!(code, 3);
        assert_eq!(
            memory.recorded_calls(),
            vec![RecordedCall {
                address: 0x4000,
                label: "Promote".into(),
                call: NativeCallRecord::TwoArgs {
                    first: 1,
                    second: 2
                },
            }]
        );
    }

    #[test]
    fn configured_native_failure_still_records_the_attempt() {
        let memory = FakeMemory::new_writable();
        memory.fail_native_calls("rotina recusada");
        let error = memory
            .call_native(0x4000, NativeCall::Direct { argument: 8 }, "SaveToDisk")
            .expect_err("chamada deve falhar");
        assert_eq!(error.code(), ErrorCode::NativeCallFailed);
        assert_eq!(memory.recorded_calls().len(), 1);
    }

    #[test]
    fn enforces_the_same_address_validation_as_the_real_backend() {
        let memory = FakeMemory::new_writable();
        assert_eq!(
            memory.read_u32(0).expect_err("nulo deve falhar").code(),
            ErrorCode::InvalidGameState
        );
    }

    #[test]
    fn validates_code_prefix_against_mapped_bytes() {
        let memory = FakeMemory::new_writable();
        memory.map(BASE, vec![0x48, 0x83, 0xEC, 0x28, 0x90]);
        assert!(
            memory
                .validate_code_prefix(BASE, &[0x48, 0x83, 0xEC, 0x28], "Rotina")
                .is_ok()
        );
        let error = memory
            .validate_code_prefix(BASE, &[0x48, 0x83, 0xEC, 0x20], "Rotina")
            .expect_err("prefixo divergente deve falhar");
        assert_eq!(error.code(), ErrorCode::SignatureMismatch);
    }

    #[test]
    fn empty_signature_is_treated_as_a_profile_error() {
        let memory = FakeMemory::new_writable();
        memory.map_zeroed(BASE, 8);
        assert_eq!(
            memory
                .validate_code_prefix(BASE, &[], "Rotina")
                .expect_err("assinatura vazia deve falhar")
                .code(),
            ErrorCode::InvalidArgument
        );
    }
}
