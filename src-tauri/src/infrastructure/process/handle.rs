//! Handles Win32 com liberacao garantida por RAII (SPEC-001).

use windows::Win32::Foundation::{CloseHandle, HANDLE};

/// Handle proprietario: fecha em `Drop`, sem excecao e sem caminho manual.
///
/// Nao existe API publica para "vazar" o handle de proposito; qualquer codigo
/// que precise do valor bruto usa `raw()` e nao assume posse.
pub struct OwnedHandle(HANDLE);

impl OwnedHandle {
    /// # Safety
    ///
    /// O chamador transfere a posse do handle. Ele nao pode ser fechado por
    /// outro caminho depois desta chamada.
    pub(super) fn new(handle: HANDLE) -> Self {
        Self(handle)
    }

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
