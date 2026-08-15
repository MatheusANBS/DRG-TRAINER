//! Testes de integracao offline (SPEC-006, nivel 2).
//!
//! Rodam contra a API publica do crate, sem o jogo instalado. O objetivo e
//! travar os contratos que o frontend e o processo de release dependem:
//! resolucao de build, status operacional e serializacao dos erros.

use drg_credits_lib::build_profiles::{
    BuildResolution, Capabilities, ProfileStatus, profiles, registry,
};
use drg_credits_lib::domain::dto::{BuildState, ProcessState};
use drg_credits_lib::domain::status;
use drg_credits_lib::shared::{ErrorCode, TrainerError};

#[test]
fn an_unknown_build_is_blocked_and_reports_its_hash() {
    let hash = "f".repeat(64);
    match registry::resolve(&hash) {
        BuildResolution::Unsupported { sha256 } => assert_eq!(sha256, hash),
        other => panic!("build desconhecida deveria ser bloqueada, veio {other:?}"),
    }
}

#[test]
fn every_shipped_profile_is_supported_and_self_consistent() {
    assert!(
        !profiles::ALL.is_empty(),
        "o binario precisa de ao menos um perfil"
    );

    for profile in profiles::ALL {
        assert_eq!(
            profile.status,
            ProfileStatus::Supported,
            "{} nao deveria ser distribuido antes de promovido",
            profile.id
        );
        assert_eq!(profile.executable_sha256.len(), 64);
        assert!(!profile.resources.is_empty());
        assert_eq!(profile.catalog.playable_classes.len(), 4);

        // Um perfil promovido precisa ter a hash resolvivel pelo registry.
        assert!(
            matches!(
                registry::resolve(profile.executable_sha256),
                BuildResolution::Supported(_)
            ),
            "{} nao e resolvido pela propria hash",
            profile.id
        );
    }
}

#[test]
fn supported_profiles_are_exactly_the_promoted_ones() {
    let promoted = registry::supported_profiles().count();
    let declared = profiles::ALL
        .iter()
        .filter(|profile| profile.status == ProfileStatus::Supported)
        .count();
    assert_eq!(promoted, declared);
}

#[test]
fn status_never_fails_and_disables_everything_without_a_verified_build() {
    let status = status::status();
    assert!(!status.expected_executables.is_empty());

    match (&status.process, &status.build) {
        (ProcessState::Detached, build) => {
            assert!(matches!(build, BuildState::Unknown));
            assert_eq!(status.capabilities, Capabilities::NONE);
        }
        (ProcessState::Attached { pid, executable }, build) => {
            assert!(*pid > 0);
            assert!(!executable.is_empty());
            if !build.is_operable() {
                assert_eq!(
                    status.capabilities,
                    Capabilities::NONE,
                    "build nao verificada nao pode habilitar capacidades"
                );
            }
        }
    }
}

#[test]
fn errors_serialize_with_a_stable_code_and_recoverability() {
    let error = TrainerError::new(ErrorCode::BuildUnsupported, "hash desconhecida");
    let json = serde_json::to_value(&error).expect("erro deve serializar");

    assert_eq!(json["code"], "BUILD_UNSUPPORTED");
    assert_eq!(json["recoverable"], false);
    assert_eq!(json["message"], "hash desconhecida");
}

#[test]
fn waiting_errors_are_marked_recoverable_for_the_frontend() {
    for code in [ErrorCode::ProcessNotFound, ErrorCode::ModuleNotFound] {
        let json = serde_json::to_value(TrainerError::new(code, "aguardando"))
            .expect("erro deve serializar");
        assert_eq!(
            json["recoverable"], true,
            "{code:?} deveria ser recuperavel"
        );
    }
}
