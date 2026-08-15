//! Dominio de progressao: desbloqueios permanentes de armas, perks,
//! modificacoes de equipamento e esquemas.
//!
//! Todas as operacoes aqui sao mutacoes permanentes: passam por
//! `SaveTransaction` e terminam com gravacao em disco (SPEC-020).

pub mod schematics;
pub mod unlocks;

pub use schematics::unlock_all_overclocks_and_cosmetics;
pub use unlocks::{unlock_all_gear_modifications, unlock_all_perks, unlock_all_weapons};

use std::time::Duration;

use crate::build_profiles::NativeFunction;
use crate::infrastructure::process::{NativeCall, NativeCaller, ProcessAccess};
use crate::infrastructure::save::SaveTransaction;
use crate::infrastructure::unreal::UnrealRuntime;
use crate::shared::error::Result;

/// Tempo que o jogo leva para propagar o efeito da rotina antes de podermos
/// reler os contadores com confianca.
pub(crate) const SETTLE_DELAY: Duration = Duration::from_millis(250);

/// Grava o save em disco encerrando uma mutacao permanente.
///
/// A assinatura ja deve ter sido validada antes do backup; aqui a chamada e
/// protegida pela transacao para que a mensagem aponte o caminho de restauracao.
pub(crate) fn persist_to_disk<M: ProcessAccess>(
    runtime: &UnrealRuntime<M>,
    save: usize,
    transaction: &SaveTransaction,
) -> Result<()> {
    let save_to_disk = runtime.profile().natives.save_to_disk;
    transaction.guard(
        runtime
            .call_verified(save_to_disk, NativeCall::Direct { argument: save })
            .map(|_| ()),
    )
}

/// Executa uma rotina de unlock com um unico argumento e aguarda o efeito.
pub(crate) fn invoke_and_settle<M: NativeCaller + crate::infrastructure::process::MemoryReader>(
    runtime: &UnrealRuntime<M>,
    native: NativeFunction,
    argument: usize,
    transaction: &SaveTransaction,
) -> Result<()> {
    transaction.guard(
        runtime
            .call_verified(native, NativeCall::Direct { argument })
            .map(|_| ()),
    )?;
    std::thread::sleep(SETTLE_DELAY);
    Ok(())
}
