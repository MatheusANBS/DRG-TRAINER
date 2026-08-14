use super::*;

pub(super) struct OwnedHandle(pub(super) HANDLE);

impl OwnedHandle {
    pub(super) fn raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

pub(super) struct ProcessMemory {
    handle: OwnedHandle,
    writable: bool,
}

impl ProcessMemory {
    pub(super) fn open(pid: u32, writable: bool) -> Result<Self> {
        let mut access = PROCESS_VM_READ | PROCESS_QUERY_INFORMATION;
        if writable {
            access |= PROCESS_VM_WRITE | PROCESS_VM_OPERATION | PROCESS_CREATE_THREAD;
        }
        let handle = unsafe { OpenProcess(access, false, pid) }
            .map_err(|error| MemoryError(format!("Nao foi possivel abrir o processo: {error}")))?;
        Ok(Self {
            handle: OwnedHandle(handle),
            writable,
        })
    }

    pub(super) fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>> {
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
            MemoryError(format!(
                "Falha ao ler 0x{address:X} ({size} bytes): {error}"
            ))
        })?;
        if received != size {
            return Err(MemoryError(format!(
                "Leitura incompleta em 0x{address:X}: {received}/{size} bytes"
            )));
        }
        Ok(buffer)
    }

    pub(super) fn read_u16(&self, address: usize) -> Result<u16> {
        let bytes = self.read_bytes(address, 2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub(super) fn read_u32(&self, address: usize) -> Result<u32> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub(super) fn read_i32(&self, address: usize) -> Result<i32> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub(super) fn read_f32(&self, address: usize) -> Result<f32> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(f32::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub(super) fn read_u64(&self, address: usize) -> Result<u64> {
        let bytes = self.read_bytes(address, 8)?;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    pub(super) fn write_i32(&self, address: usize, value: i32) -> Result<()> {
        self.write_bytes(address, &value.to_le_bytes())
    }

    pub(super) fn write_bytes(&self, address: usize, bytes: &[u8]) -> Result<()> {
        if !self.writable {
            return Err(MemoryError(
                "O processo foi aberto em modo somente leitura.".into(),
            ));
        }
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
        .map_err(|error| MemoryError(format!("Falha ao escrever em 0x{address:X}: {error}")))?;
        if written != bytes.len() {
            return Err(MemoryError(format!(
                "Escrita incompleta em 0x{address:X}: {written}/{} bytes",
                bytes.len()
            )));
        }
        Ok(())
    }

    pub(super) fn write_f32(&self, address: usize, value: f32) -> Result<()> {
        if !self.writable {
            return Err(MemoryError(
                "O processo foi aberto em modo somente leitura.".into(),
            ));
        }
        let bytes = value.to_le_bytes();
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
        .map_err(|error| MemoryError(format!("Falha ao escrever em 0x{address:X}: {error}")))?;
        if written != bytes.len() {
            return Err(MemoryError(format!(
                "Escrita incompleta em 0x{address:X}: {written}/{} bytes",
                bytes.len()
            )));
        }
        Ok(())
    }

    pub(super) fn validate_code_prefix(
        &self,
        address: usize,
        expected: &[u8],
        label: &str,
    ) -> Result<()> {
        let actual = self.read_bytes(address, expected.len())?;
        if actual != expected {
            return Err(MemoryError(format!(
                "A assinatura nativa de {label} nao corresponde a build verificada."
            )));
        }
        Ok(())
    }

    pub(super) fn call_remote(&self, address: usize, parameter: usize, label: &str) -> Result<()> {
        if !self.writable {
            return Err(MemoryError(
                "O processo foi aberto em modo somente leitura.".into(),
            ));
        }
        if address == 0 || parameter == 0 {
            return Err(MemoryError(format!(
                "Endereco invalido ao executar {label}."
            )));
        }

        let start: unsafe extern "system" fn(*mut c_void) -> u32 =
            unsafe { std::mem::transmute(address) };
        let thread = unsafe {
            CreateRemoteThread(
                self.handle.raw(),
                None,
                0,
                Some(start),
                Some(parameter as *const c_void),
                0,
                None,
            )
        }
        .map_err(|error| MemoryError(format!("Falha ao iniciar {label}: {error}")))?;
        let thread = OwnedHandle(thread);
        let wait = unsafe { WaitForSingleObject(thread.raw(), REMOTE_CALL_TIMEOUT_MS) };
        if wait != WAIT_OBJECT_0 {
            return Err(MemoryError(format!(
                "Tempo limite excedido ao executar {label}."
            )));
        }
        Ok(())
    }

    pub(super) fn call_remote_two_args(
        &self,
        address: usize,
        first: usize,
        second: usize,
        label: &str,
    ) -> Result<u32> {
        if !self.writable {
            return Err(MemoryError(
                "O processo foi aberto em modo somente leitura.".into(),
            ));
        }
        if address == 0 || first == 0 || second == 0 {
            return Err(MemoryError(format!(
                "Endereco invalido ao executar {label}."
            )));
        }

        // Windows x64: reserve shadow space, load RCX/RDX, call the verified native routine.
        let mut stub = Vec::with_capacity(38);
        stub.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]);
        stub.extend_from_slice(&[0x48, 0xB9]);
        stub.extend_from_slice(&(first as u64).to_le_bytes());
        stub.extend_from_slice(&[0x48, 0xBA]);
        stub.extend_from_slice(&(second as u64).to_le_bytes());
        stub.extend_from_slice(&[0x48, 0xB8]);
        stub.extend_from_slice(&(address as u64).to_le_bytes());
        stub.extend_from_slice(&[0xFF, 0xD0, 0x48, 0x83, 0xC4, 0x28, 0xC3]);

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
            return Err(MemoryError(format!(
                "Nao foi possivel preparar a chamada de {label}."
            )));
        }

        let result = (|| -> Result<u32> {
            self.write_bytes(remote as usize, &stub)?;
            let start: unsafe extern "system" fn(*mut c_void) -> u32 =
                unsafe { std::mem::transmute(remote) };
            let thread = unsafe {
                CreateRemoteThread(self.handle.raw(), None, 0, Some(start), None, 0, None)
            }
            .map_err(|error| MemoryError(format!("Falha ao iniciar {label}: {error}")))?;
            let thread = OwnedHandle(thread);
            let wait = unsafe { WaitForSingleObject(thread.raw(), REMOTE_CALL_TIMEOUT_MS) };
            if wait != WAIT_OBJECT_0 {
                return Err(MemoryError(format!(
                    "Tempo limite excedido ao executar {label}."
                )));
            }
            let mut exit_code = 0_u32;
            unsafe { GetExitCodeThread(thread.raw(), &mut exit_code) }
                .map_err(|error| MemoryError(format!("Falha ao confirmar {label}: {error}")))?;
            Ok(exit_code)
        })();

        let _ = unsafe { VirtualFreeEx(self.handle.raw(), remote, 0, MEM_RELEASE) };
        result
    }

    pub(super) fn call_remote_three_args(
        &self,
        address: usize,
        first: usize,
        second: usize,
        third: usize,
        label: &str,
    ) -> Result<u32> {
        if !self.writable {
            return Err(MemoryError(
                "O processo foi aberto em modo somente leitura.".into(),
            ));
        }
        if address == 0 || first == 0 || second == 0 {
            return Err(MemoryError(format!(
                "Endereco invalido ao executar {label}."
            )));
        }

        let mut stub = Vec::with_capacity(48);
        stub.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]);
        stub.extend_from_slice(&[0x48, 0xB9]);
        stub.extend_from_slice(&(first as u64).to_le_bytes());
        stub.extend_from_slice(&[0x48, 0xBA]);
        stub.extend_from_slice(&(second as u64).to_le_bytes());
        stub.extend_from_slice(&[0x49, 0xB8]);
        stub.extend_from_slice(&(third as u64).to_le_bytes());
        stub.extend_from_slice(&[0x48, 0xB8]);
        stub.extend_from_slice(&(address as u64).to_le_bytes());
        stub.extend_from_slice(&[0xFF, 0xD0, 0x48, 0x83, 0xC4, 0x28, 0xC3]);

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
            return Err(MemoryError(format!(
                "Nao foi possivel preparar a chamada de {label}."
            )));
        }

        let result = (|| -> Result<u32> {
            self.write_bytes(remote as usize, &stub)?;
            let start: unsafe extern "system" fn(*mut c_void) -> u32 =
                unsafe { std::mem::transmute(remote) };
            let thread = unsafe {
                CreateRemoteThread(self.handle.raw(), None, 0, Some(start), None, 0, None)
            }
            .map_err(|error| MemoryError(format!("Falha ao iniciar {label}: {error}")))?;
            let thread = OwnedHandle(thread);
            let wait = unsafe { WaitForSingleObject(thread.raw(), REMOTE_CALL_TIMEOUT_MS) };
            if wait != WAIT_OBJECT_0 {
                return Err(MemoryError(format!(
                    "Tempo limite excedido ao executar {label}."
                )));
            }
            let mut exit_code = 0_u32;
            unsafe { GetExitCodeThread(thread.raw(), &mut exit_code) }
                .map_err(|error| MemoryError(format!("Falha ao confirmar {label}: {error}")))?;
            Ok(exit_code)
        })();

        let _ = unsafe { VirtualFreeEx(self.handle.raw(), remote, 0, MEM_RELEASE) };
        result
    }

    pub(super) fn call_remote_resource_delta(
        &self,
        address: usize,
        save: usize,
        resource: usize,
        amount: f32,
        label: &str,
    ) -> Result<()> {
        if !self.writable {
            return Err(MemoryError(
                "O processo foi aberto em modo somente leitura.".into(),
            ));
        }
        if address == 0 || save == 0 || resource == 0 || !amount.is_finite() || amount == 0.0 {
            return Err(MemoryError(format!(
                "Parametros invalidos ao executar {label}."
            )));
        }

        // FSDSaveGame::AddResource(ResourceData*, float): RCX, RDX e XMM2 no ABI x64.
        let mut stub = Vec::with_capacity(48);
        stub.extend_from_slice(&[0x48, 0x83, 0xEC, 0x28]);
        stub.extend_from_slice(&[0x48, 0xB9]);
        stub.extend_from_slice(&(save as u64).to_le_bytes());
        stub.extend_from_slice(&[0x48, 0xBA]);
        stub.extend_from_slice(&(resource as u64).to_le_bytes());
        stub.push(0xB8);
        stub.extend_from_slice(&amount.to_bits().to_le_bytes());
        stub.extend_from_slice(&[0x66, 0x0F, 0x6E, 0xD0]);
        stub.extend_from_slice(&[0x49, 0xBB]);
        stub.extend_from_slice(&(address as u64).to_le_bytes());
        stub.extend_from_slice(&[0x41, 0xFF, 0xD3, 0x48, 0x83, 0xC4, 0x28, 0xC3]);

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
            return Err(MemoryError(format!(
                "Nao foi possivel preparar a chamada de {label}."
            )));
        }

        let result = (|| -> Result<()> {
            self.write_bytes(remote as usize, &stub)?;
            let start: unsafe extern "system" fn(*mut c_void) -> u32 =
                unsafe { std::mem::transmute(remote) };
            let thread = unsafe {
                CreateRemoteThread(self.handle.raw(), None, 0, Some(start), None, 0, None)
            }
            .map_err(|error| MemoryError(format!("Falha ao iniciar {label}: {error}")))?;
            let thread = OwnedHandle(thread);
            let wait = unsafe { WaitForSingleObject(thread.raw(), REMOTE_CALL_TIMEOUT_MS) };
            if wait != WAIT_OBJECT_0 {
                return Err(MemoryError(format!(
                    "Tempo limite excedido ao executar {label}."
                )));
            }
            Ok(())
        })();

        let _ = unsafe { VirtualFreeEx(self.handle.raw(), remote, 0, MEM_RELEASE) };
        result
    }
}
