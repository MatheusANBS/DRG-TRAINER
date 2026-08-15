//! Status operacional do trainer (SPEC-016).
//!
//! Responde a pergunta que o usuario faz antes de qualquer acao: "e seguro
//! operar agora?". Nunca abre o processo para escrita e nunca falha por
//! ausencia do jogo — ausencia e um estado, nao um erro.

use crate::build_profiles::{BuildResolution, Capabilities, registry};
use crate::infrastructure::process::discovery;
use crate::infrastructure::unreal::cache;

use super::dto::{BuildState, ProcessState, TrainerStatus};
use super::session::candidate_executables;

/// Le o estado atual de processo e build.
pub fn status() -> TrainerStatus {
    let expected = candidate_executables();
    let expected_executables = expected.iter().map(|name| (*name).to_string()).collect();

    let Ok(process) = discovery::locate(&expected) else {
        // Jogo fechado: nenhum endereco resolvido continua valido.
        cache::invalidate_all();
        return TrainerStatus {
            process: ProcessState::Detached,
            build: BuildState::Unknown,
            capabilities: Capabilities::NONE,
            expected_executables,
        };
    };

    let attached = ProcessState::Attached {
        pid: process.pid,
        executable: process.executable_name.clone(),
    };

    // O processo existe; a hash decide se as acoes ficam liberadas.
    let Ok(sha256) = discovery::executable_sha256(&process.executable) else {
        return TrainerStatus {
            process: attached,
            build: BuildState::Unknown,
            capabilities: Capabilities::NONE,
            expected_executables,
        };
    };

    let (build, capabilities) = match registry::resolve(&sha256) {
        BuildResolution::Supported(profile) => (
            BuildState::Verified {
                profile_id: profile.id.into(),
                display_name: profile.display_name.into(),
                sha256,
                status: profile.status,
            },
            profile.capabilities,
        ),
        BuildResolution::Unverified(profile) => (
            BuildState::Unverified {
                profile_id: profile.id.into(),
                display_name: profile.display_name.into(),
                sha256,
                status: profile.status,
            },
            // Perfil conhecido mas nao promovido nao habilita nada.
            Capabilities::NONE,
        ),
        BuildResolution::Unsupported { sha256 } => {
            (BuildState::Unsupported { sha256 }, Capabilities::NONE)
        }
    };

    TrainerStatus {
        process: attached,
        build,
        capabilities,
        expected_executables,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_a_first_class_state_instead_of_failing_without_the_game() {
        let status = status();
        assert!(!status.expected_executables.is_empty());
        match status.process {
            // Em CI o jogo nao esta aberto.
            ProcessState::Detached => {
                assert!(matches!(status.build, BuildState::Unknown));
                assert_eq!(status.capabilities, Capabilities::NONE);
                assert!(!status.build.is_operable());
            }
            // Em maquina de desenvolvimento com o jogo aberto.
            ProcessState::Attached { pid, .. } => {
                assert!(pid > 0);
                if !status.build.is_operable() {
                    assert_eq!(status.capabilities, Capabilities::NONE);
                }
            }
        }
    }

    #[test]
    fn expected_executables_match_the_registered_profiles() {
        let status = status();
        assert!(
            status
                .expected_executables
                .contains(&"FSD-Win64-Shipping.exe".to_string())
        );
    }
}
