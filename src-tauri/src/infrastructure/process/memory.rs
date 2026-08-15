//! Implementacao Win32 do acesso a memoria (SPEC-001).
//!
//! Este e o unico arquivo do backend que chama `ReadProcessMemory`,
//! `WriteProcessMemory`, `VirtualAllocEx` e `CreateRemoteThread`. Qualquer
//! codigo de dominio que precise disso passa pelos traits em `access.rs`.

use std::ffi::c_void;

use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, GetExitCodeThread, OpenProcess, PROCESS_CREATE_THREAD,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    WaitForSingleObject,
};

use super::access::{MemoryReader, MemoryWriter, NativeCall, NativeCaller, validate_range};
use super::handle::OwnedHandle;
use crate::shared::error::{ErrorCode, Result, TrainerError};
use crate::shared::limits::REMOTE_CALL_TIMEOUT_MS;

/// Handle de processo aberto com o menor conjunto de direitos necessario.
///
/// Invariantes:
/// - abre somente leitura por padrao;
/// - direitos de escrita/execucao remota exigem `Access::ReadWrite` explicito;
/// - o handle e liberado por RAII;
/// - toda leitura e escrita valida o intervalo e a quantidade de bytes.
pub struct ProcessMemory {
    handle: OwnedHandle,
    pid: u32,
    writable: bool,
}

/// Nivel de acesso pedido ao abrir o processo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// Leitura apenas. Padrao para polling e diagnostico.
    ReadOnly,
    /// Leitura, escrita e execucao remota. Exigido para qualquer mutacao.
    ReadWrite,
}

impl Access {
    fn is_writable(self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

impl ProcessMemory {
    pub fn open(pid: u32, access: Access) -> Result<Self> {
        let mut rights = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
        if access.is_writable() {
            rights |= PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_CREATE_THREAD;
        }
        let handle = unsafe { OpenProcess(rights, false, pid) }.map_err(|error| {
            TrainerError::new(
                ErrorCode::ProcessOpenFailed,
                format!("Nao foi possivel abrir o processo: {error}"),
            )
        })?;
        Ok(Self {
            handle: OwnedHandle::new(handle),
            pid,
            writable: access.is_writable(),
        })
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn is_writable(&self) -> bool {
        self.writable
    }

    fn ensure_writable(&self) -> Result<()> {
        if !self.writable {
            return Err(TrainerError::new(
                ErrorCode::ReadOnlyHandle,
                "O processo foi aberto em modo somente leitura.",
            ));
        }
        Ok(())
    }

    /// Aguarda a thread remota e devolve o exit code, ou erro em timeout.
    fn wait_for_thread(thread: &OwnedHandle, label: &str) -> Result<u32> {
        let wait = unsafe { WaitForSingleObject(thread.raw(), REMOTE_CALL_TIMEOUT_MS) };
        if wait != WAIT_OBJECT_0 {
            return Err(TrainerError::new(
                ErrorCode::NativeCallTimeout,
                format!("Tempo limite excedido ao executar {label}."),
            ));
        }
        let mut exit_code = 0_u32;
        unsafe { GetExitCodeThread(thread.raw(), &mut exit_code) }.map_err(|error| {
            TrainerError::new(
                ErrorCode::NativeCallFailed,
                format!("Falha ao confirmar {label}: {error}"),
            )
        })?;
        Ok(exit_code)
    }

    /// Chamada direta: a propria rotina do jogo vira start routine da thread.
    fn call_direct(&self, address: usize, argument: usize, label: &str) -> Result<u32> {
        let start: unsafe extern "system" fn(*mut c_void) -> u32 = unsafe {
            std::mem::transmute::<usize, unsafe extern "system" fn(*mut c_void) -> u32>(address)
        };
        let thread = unsafe {
            CreateRemoteThread(
                self.handle.raw(),
                None,
                0,
                Some(start),
                Some(argument as *const c_void),
                0,
                None,
            )
        }
        .map_err(|error| {
            TrainerError::new(
                ErrorCode::NativeCallFailed,
                format!("Falha ao iniciar {label}: {error}"),
            )
        })?;
        Self::wait_for_thread(&OwnedHandle::new(thread), label)
    }

    /// Chamada via stub: aloca o trampolim, executa e libera sempre.
    fn call_with_stub(&self, stub: &[u8], label: &str) -> Result<u32> {
        let remote = unsafe {
            VirtualAllocEx(
                self.handle.raw(),
                None,
                stub.len(),
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE,
            )
        };
        if remote.is_null() {
            return Err(TrainerError::new(
                ErrorCode::NativeCallFailed,
                format!("Nao foi possivel preparar a chamada de {label}."),
            ));
        }

        let result = (|| -> Result<u32> {
            self.write_bytes(remote as usize, stub)?;
            let start: unsafe extern "system" fn(*mut c_void) -> u32 = unsafe {
                std::mem::transmute::<*mut c_void, unsafe extern "system" fn(*mut c_void) -> u32>(
                    remote,
                )
            };
            let thread = unsafe {
                CreateRemoteThread(self.handle.raw(), None, 0, Some(start), None, 0, None)
            }
            .map_err(|error| {
                TrainerError::new(
                    ErrorCode::NativeCallFailed,
                    format!("Falha ao iniciar {label}: {error}"),
                )
            })?;
            Self::wait_for_thread(&OwnedHandle::new(thread), label)
        })();

        // A liberacao acontece mesmo quando a chamada falha ou expira.
        let _ = unsafe { VirtualFreeEx(self.handle.raw(), remote, 0, MEM_RELEASE) };
        result
    }
}

/// Monta o trampolim x64 para a variante pedida.
///
/// Convencao: reserva shadow space de 0x28, carrega os registradores de
/// argumento, chama a rotina verificada e retorna.
pub(super) fn build_stub(address: usize, call: NativeCall) -> Vec<u8> {
    let mut stub = Vec::with_capacity(56);
    stub.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]); // sub rsp, 0x28

    match call {
        NativeCall::Direct { .. } => unreachable!("chamada direta nao usa stub"),
        NativeCall::TwoArgs { first, second } => {
            stub.extend_from_slice(&[0x48, 0xB9]); // mov rcx, imm64
            stub.extend_from_slice(&(first as u64).to_le_bytes());
            stub.extend_from_slice(&[0x48, 0xBA]); // mov rdx, imm64
            stub.extend_from_slice(&(second as u64).to_le_bytes());
            stub.extend_from_slice(&[0x48, 0xB8]); // mov rax, imm64
            stub.extend_from_slice(&(address as u64).to_le_bytes());
            stub.extend_from_slice(&[0xFF, 0xD0]); // call rax
        }
        NativeCall::ThreeArgs {
            first,
            second,
            third,
        } => {
            stub.extend_from_slice(&[0x48, 0xB9]);
            stub.extend_from_slice(&(first as u64).to_le_bytes());
            stub.extend_from_slice(&[0x48, 0xBA]);
            stub.extend_from_slice(&(second as u64).to_le_bytes());
            stub.extend_from_slice(&[0x49, 0xB8]); // mov r8, imm64
            stub.extend_from_slice(&(third as u64).to_le_bytes());
            stub.extend_from_slice(&[0x48, 0xB8]);
            stub.extend_from_slice(&(address as u64).to_le_bytes());
            stub.extend_from_slice(&[0xFF, 0xD0]);
        }
        NativeCall::ResourceDelta {
            save,
            resource,
            amount,
        } => {
            stub.extend_from_slice(&[0x48, 0xB9]);
            stub.extend_from_slice(&(save as u64).to_le_bytes());
            stub.extend_from_slice(&[0x48, 0xBA]);
            stub.extend_from_slice(&(resource as u64).to_le_bytes());
            stub.push(0xB8); // mov eax, imm32
            stub.extend_from_slice(&amount.to_bits().to_le_bytes());
            stub.extend_from_slice(&[0x66, 0x0F, 0x6E, 0xD0]); // movd xmm2, eax
            stub.extend_from_slice(&[0x49, 0xBB]); // mov r11, imm64
            stub.extend_from_slice(&(address as u64).to_le_bytes());
            stub.extend_from_slice(&[0x41, 0xFF, 0xD3]); // call r11
        }
    }

    stub.extend_from_slice(&[0x48, 0x83, 0xC4, 0x28, 0xC3]); // add rsp, 0x28 / ret
    stub
}

impl MemoryReader for ProcessMemory {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        validate_range(address, size)?;
        let mut buffer = vec![0_u8; size];
        let mut received = 0_usize;
        unsafe {
            ReadProcessMemory(
                self.handle.raw(),
                address as *const c_void,
                buffer.as_mut_ptr().cast(),
                size,
                Some(&mut received),
            )
        }
        .map_err(|error| {
            TrainerError::new(
                ErrorCode::MemoryReadFailed,
                format!("Falha ao ler 0x{address:X} ({size} bytes): {error}"),
            )
        })?;
        if received != size {
            return Err(TrainerError::new(
                ErrorCode::MemoryReadFailed,
                format!("Leitura incompleta em 0x{address:X}: {received}/{size} bytes"),
            ));
        }
        Ok(buffer)
    }
}

impl MemoryWriter for ProcessMemory {
    fn write_bytes(&self, address: usize, bytes: &[u8]) -> Result<()> {
        self.ensure_writable()?;
        validate_range(address, bytes.len())?;
        let mut written = 0_usize;
        unsafe {
            WriteProcessMemory(
                self.handle.raw(),
                address as *const c_void,
                bytes.as_ptr().cast(),
                bytes.len(),
                Some(&mut written),
            )
        }
        .map_err(|error| {
            TrainerError::new(
                ErrorCode::MemoryWriteFailed,
                format!("Falha ao escrever em 0x{address:X}: {error}"),
            )
        })?;
        if written != bytes.len() {
            return Err(TrainerError::new(
                ErrorCode::MemoryWriteFailed,
                format!(
                    "Escrita incompleta em 0x{address:X}: {written}/{} bytes",
                    bytes.len()
                ),
            ));
        }
        Ok(())
    }
}

impl NativeCaller for ProcessMemory {
    fn call_native(&self, address: usize, call: NativeCall, label: &str) -> Result<u32> {
        self.ensure_writable()?;
        if address < super::access::MIN_USER_ADDRESS {
            return Err(TrainerError::new(
                ErrorCode::InvalidGameState,
                format!("Endereco invalido ao executar {label}."),
            ));
        }
        for argument in call.required_arguments() {
            if argument == 0 {
                return Err(TrainerError::new(
                    ErrorCode::InvalidArgument,
                    format!("Parametros invalidos ao executar {label}."),
                ));
            }
        }
        if let NativeCall::ResourceDelta { amount, .. } = call
            && (!amount.is_finite() || amount == 0.0)
        {
            return Err(TrainerError::new(
                ErrorCode::InvalidArgument,
                format!("Parametros invalidos ao executar {label}."),
            ));
        }

        match call {
            NativeCall::Direct { argument } => self.call_direct(address, argument, label),
            other => self.call_with_stub(&build_stub(address, other), label),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_arg_stub_matches_the_documented_encoding() {
        let stub = build_stub(
            0x1122_3344_5566_7788,
            NativeCall::TwoArgs {
                first: 0xAABB_CCDD_EEFF_0011,
                second: 0x0102_0304_0506_0708,
            },
        );
        let mut expected = vec![0x48, 0x83, 0xEC, 0x28, 0x48, 0xB9];
        expected.extend_from_slice(&0xAABB_CCDD_EEFF_0011_u64.to_le_bytes());
        expected.extend_from_slice(&[0x48, 0xBA]);
        expected.extend_from_slice(&0x0102_0304_0506_0708_u64.to_le_bytes());
        expected.extend_from_slice(&[0x48, 0xB8]);
        expected.extend_from_slice(&0x1122_3344_5566_7788_u64.to_le_bytes());
        expected.extend_from_slice(&[0xFF, 0xD0, 0x48, 0x83, 0xC4, 0x28, 0xC3]);
        assert_eq!(stub, expected);
    }

    #[test]
    fn three_arg_stub_loads_r8() {
        let stub = build_stub(
            0x1000,
            NativeCall::ThreeArgs {
                first: 1,
                second: 2,
                third: 3,
            },
        );
        // 49 B8 = mov r8, imm64
        assert!(
            stub.windows(2).any(|pair| pair == [0x49, 0xB8]),
            "stub de tres argumentos deve carregar R8"
        );
        assert_eq!(&stub[stub.len() - 5..], &[0x48, 0x83, 0xC4, 0x28, 0xC3]);
    }

    #[test]
    fn resource_delta_stub_moves_the_float_into_xmm2() {
        let amount = 1_000.0_f32;
        let stub = build_stub(
            0x2000,
            NativeCall::ResourceDelta {
                save: 0x3000,
                resource: 0x4000,
                amount,
            },
        );
        // movd xmm2, eax
        assert!(
            stub.windows(4)
                .any(|window| window == [0x66, 0x0F, 0x6E, 0xD0]),
            "stub de recurso deve mover o float para XMM2"
        );
        assert!(
            stub.windows(4)
                .any(|window| window == amount.to_bits().to_le_bytes()),
            "os bits do float devem estar embutidos no stub"
        );
        // call r11
        assert!(stub.windows(3).any(|window| window == [0x41, 0xFF, 0xD3]));
    }

    #[test]
    fn every_stub_balances_the_shadow_space() {
        for call in [
            NativeCall::TwoArgs {
                first: 1,
                second: 2,
            },
            NativeCall::ThreeArgs {
                first: 1,
                second: 2,
                third: 0,
            },
            NativeCall::ResourceDelta {
                save: 1,
                resource: 2,
                amount: 5.0,
            },
        ] {
            let stub = build_stub(0x9000, call);
            assert_eq!(&stub[..4], &[0x48, 0x83, 0xEC, 0x28], "prologo ausente");
            assert_eq!(
                &stub[stub.len() - 5..],
                &[0x48, 0x83, 0xC4, 0x28, 0xC3],
                "epilogo ausente"
            );
        }
    }

    #[test]
    fn required_arguments_ignore_the_optional_third_slot() {
        let call = NativeCall::ThreeArgs {
            first: 0x10,
            second: 0x20,
            third: 0,
        };
        assert_eq!(call.required_arguments(), [0x10, 0x20]);
    }
}
