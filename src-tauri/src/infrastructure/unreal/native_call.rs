//! Execucao de rotinas nativas do jogo (SPEC-005).
//!
//! Regra inegociavel (SPEC-001): nenhuma chamada acontece sem validar o
//! prefixo de codigo declarado no perfil. Qual rotina chamar e decisao do
//! dominio; *como* chama-la com seguranca e responsabilidade deste modulo.

use crate::build_profiles::NativeFunction;
use crate::infrastructure::process::{MemoryReader, NativeCall, NativeCaller};
use crate::shared::error::Result;

use super::types::UnrealRuntime;

impl<M: MemoryReader> UnrealRuntime<M> {
    /// Confere que a rotina no perfil corresponde ao codigo carregado e devolve
    /// seu endereco absoluto. Usado tambem por testes de resolucao que nao
    /// devem executar nada.
    pub fn verify_native(&self, native: NativeFunction) -> Result<usize> {
        let address = native.address(self.module_base);
        self.memory
            .validate_code_prefix(address, native.prefix, native.label)?;
        Ok(address)
    }
}

impl<M: MemoryReader + NativeCaller> UnrealRuntime<M> {
    /// Valida e executa uma rotina nativa.
    pub fn call_verified(&self, native: NativeFunction, call: NativeCall) -> Result<u32> {
        let address = self.verify_native(native)?;
        self.memory.call_native(address, call, native.label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::process::fake::NativeCallRecord;
    use crate::infrastructure::unreal::testing::{MODULE_BASE, TestWorld, profile};
    use crate::shared::ErrorCode;

    fn map_routine(world: &TestWorld, native: NativeFunction, bytes: &[u8]) {
        world
            .memory()
            .map(native.address(MODULE_BASE), bytes.to_vec());
    }

    #[test]
    fn refuses_to_call_when_the_signature_does_not_match() {
        let world = TestWorld::new();
        let native = profile().natives.save_to_disk;
        // Codigo diferente do prefixo declarado no perfil.
        map_routine(&world, native, &vec![0x90; native.prefix.len()]);

        let error = world
            .runtime()
            .call_verified(native, NativeCall::Direct { argument: 0x5000 })
            .expect_err("assinatura divergente deve bloquear a chamada");
        assert_eq!(error.code(), ErrorCode::SignatureMismatch);
        assert!(
            world.memory().recorded_calls().is_empty(),
            "nenhuma chamada deve ser emitida quando a assinatura falha"
        );
    }

    #[test]
    fn calls_the_routine_when_the_signature_matches() {
        let world = TestWorld::new();
        let native = profile().natives.save_to_disk;
        map_routine(&world, native, native.prefix);
        world.memory().set_native_exit_code(1);

        let exit_code = world
            .runtime()
            .call_verified(native, NativeCall::Direct { argument: 0x5000 })
            .expect("assinatura valida deve permitir a chamada");

        assert_eq!(exit_code, 1);
        let calls = world.memory().recorded_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].address, native.address(MODULE_BASE));
        assert_eq!(calls[0].label, native.label);
        assert_eq!(calls[0].call, NativeCallRecord::Direct { argument: 0x5000 });
    }

    #[test]
    fn verify_native_does_not_execute_anything() {
        let world = TestWorld::new();
        let native = profile().natives.unlock_all_weapons;
        map_routine(&world, native, native.prefix);

        let address = world
            .runtime()
            .verify_native(native)
            .expect("rotina valida deve ser resolvida");
        assert_eq!(address, MODULE_BASE + native.rva);
        assert!(world.memory().recorded_calls().is_empty());
    }

    #[test]
    fn unreadable_routine_fails_before_any_call() {
        let world = TestWorld::new();
        let native = profile().natives.add_resource;
        // Nada mapeado nesse endereco.
        let error = world
            .runtime()
            .verify_native(native)
            .expect_err("rotina ilegivel deve falhar");
        assert_eq!(error.code(), ErrorCode::MemoryReadFailed);
    }
}
