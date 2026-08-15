//! Sessao do trainer: attach, identificacao de build e acesso compartilhado.
//!
//! Este e o unico caminho para obter acesso a memoria do jogo. Ele garante a
//! ordem correta (processo -> modulo -> hash -> perfil -> handle) e recusa
//! qualquer operacao dependente de offset em build nao suportada (SPEC-002).

use std::path::{Path, PathBuf};

use crate::build_profiles::{BuildProfile, BuildResolution, Capability, profiles, registry};
use crate::infrastructure::process::{Access, LocatedProcess, ProcessMemory, discovery};
use crate::infrastructure::save::SaveTransaction;
use crate::infrastructure::unreal::{UnrealRuntime, cache};
use crate::shared::error::{ErrorCode, Result, TrainerError};

/// Nomes de executavel aceitos, derivados dos perfis registrados.
pub fn candidate_executables() -> Vec<&'static str> {
    let mut names = profiles::ALL
        .iter()
        .map(|profile| profile.executable_name)
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

/// Processo localizado com a build ja identificada, sem handle aberto.
///
/// Serve ao status operacional: reportar "attached mas build desconhecida" nao
/// deve exigir abrir o processo para escrita.
pub struct IdentifiedProcess {
    pub process: LocatedProcess,
    pub sha256: String,
    pub resolution: BuildResolution,
}

/// Encontra o processo e resolve a build. Nao abre handle.
pub fn identify() -> Result<IdentifiedProcess> {
    let candidates = candidate_executables();
    let process = match discovery::locate(&candidates) {
        Ok(process) => process,
        Err(error) => {
            // O jogo saiu: nenhum endereco cacheado continua valido.
            if error.code().is_waiting() {
                cache::invalidate_all();
            }
            return Err(error);
        }
    };
    let sha256 = discovery::executable_sha256(&process.executable)?;
    let resolution = registry::resolve(&sha256);
    Ok(IdentifiedProcess {
        process,
        sha256,
        resolution,
    })
}

/// Sessao ativa contra uma build suportada.
pub struct TrainerSession {
    runtime: UnrealRuntime<ProcessMemory>,
    executable: PathBuf,
    sha256: String,
}

impl TrainerSession {
    /// Abre uma sessao. Falha antes de qualquer leitura se a build nao for
    /// suportada.
    pub fn attach(access: Access) -> Result<Self> {
        let identified = identify()?;
        let profile = identified.resolution.supported_profile().ok_or_else(|| {
            TrainerError::new(
                ErrorCode::BuildUnsupported,
                match identified.resolution.matched_profile() {
                    Some(profile) => format!(
                        "O perfil {} ainda nao foi promovido a suportado (status {:?}). As operacoes dependentes de offset estao bloqueadas.",
                        profile.id, profile.status
                    ),
                    None => format!(
                        "A build do jogo mudou. SHA-256 atual: {}. Os offsets foram bloqueados.",
                        identified.sha256
                    ),
                },
            )
        })?;

        let memory = ProcessMemory::open(identified.process.pid, access)?;
        Ok(Self {
            runtime: UnrealRuntime::new(
                memory,
                profile,
                identified.process.module_base,
                identified.process.pid,
            ),
            executable: identified.process.executable,
            sha256: identified.sha256,
        })
    }

    /// Sessao somente leitura: polling, status e diagnostico.
    pub fn read_only() -> Result<Self> {
        Self::attach(Access::ReadOnly)
    }

    /// Sessao com escrita: exigida por qualquer mutacao.
    pub fn writable() -> Result<Self> {
        Self::attach(Access::ReadWrite)
    }

    pub fn runtime(&self) -> &UnrealRuntime<ProcessMemory> {
        &self.runtime
    }

    pub fn profile(&self) -> &'static BuildProfile {
        self.runtime.profile()
    }

    pub fn pid(&self) -> u32 {
        self.runtime.pid()
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    /// Gate de capacidade: recusa antes de tocar na memoria.
    pub fn require(&self, capability: Capability) -> Result<()> {
        self.profile().require(capability)
    }

    /// Abre uma transacao de mutacao permanente (backup obrigatorio, SPEC-020).
    pub fn begin_save_transaction(&self, operation: &str) -> Result<SaveTransaction> {
        SaveTransaction::begin(&self.executable, operation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_list_is_deduplicated_and_non_empty() {
        let candidates = candidate_executables();
        assert!(!candidates.is_empty());
        let mut sorted = candidates.clone();
        sorted.dedup();
        assert_eq!(candidates, sorted);
        assert!(candidates.contains(&"FSD-Win64-Shipping.exe"));
    }

    #[test]
    fn attaching_without_the_game_reports_a_waiting_state() {
        // Em CI o jogo nunca esta aberto: o erro precisa ser "aguardando", nao
        // uma falha dura, para o frontend nao mostrar erro vermelho.
        if let Err(error) = TrainerSession::read_only() {
            assert!(
                error.code().is_waiting()
                    || error.code() == ErrorCode::BuildUnsupported
                    || error.code() == ErrorCode::ProcessOpenFailed,
                "codigo inesperado ao anexar sem o jogo: {:?}",
                error.code()
            );
        }
    }
}
