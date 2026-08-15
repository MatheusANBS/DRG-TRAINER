//! Resolucao de perfil por hash do executavel (SPEC-002).
//!
//! Este e o unico ponto do backend autorizado a decidir "esta build e
//! suportada". Qualquer operacao dependente de offset passa por aqui antes.

use super::profiles;
use super::types::{BuildProfile, ProfileStatus};

/// Resultado da resolucao de uma build.
#[derive(Debug, Clone)]
pub enum BuildResolution {
    /// A hash corresponde a um perfil promovido a `Supported`.
    Supported(&'static BuildProfile),
    /// A hash corresponde a um perfil conhecido que ainda nao foi promovido.
    /// Ele e reportado ao usuario, mas nao habilita operacoes de memoria.
    Unverified(&'static BuildProfile),
    /// Nenhum perfil corresponde a esta hash.
    Unsupported { sha256: String },
}

impl BuildResolution {
    /// Perfil utilizavel para operacoes dependentes de offset. `Unverified` e
    /// `Unsupported` retornam `None` de proposito.
    pub fn supported_profile(&self) -> Option<&'static BuildProfile> {
        match self {
            Self::Supported(profile) => Some(profile),
            Self::Unverified(_) | Self::Unsupported { .. } => None,
        }
    }

    /// Perfil correspondente a hash, mesmo que ainda nao promovido. Serve para
    /// exibir informacao no frontend, nunca para liberar escrita.
    pub fn matched_profile(&self) -> Option<&'static BuildProfile> {
        match self {
            Self::Supported(profile) | Self::Unverified(profile) => Some(profile),
            Self::Unsupported { .. } => None,
        }
    }
}

/// Resolve a build a partir da hash SHA-256 do executavel.
///
/// A comparacao e case-insensitive para tolerar hashes coladas em maiusculas
/// durante o onboarding de uma build nova.
pub fn resolve(sha256: &str) -> BuildResolution {
    let normalized = sha256.trim().to_ascii_lowercase();
    let matched = profiles::ALL
        .iter()
        .copied()
        .find(|profile| profile.executable_sha256.eq_ignore_ascii_case(&normalized));

    match matched {
        Some(profile) if profile.status.allows_memory_operations() => {
            BuildResolution::Supported(profile)
        }
        Some(profile) => BuildResolution::Unverified(profile),
        None => BuildResolution::Unsupported { sha256: normalized },
    }
}

/// Perfis expostos ao usuario. Builds `Deprecated` continuam registradas para
/// diagnostico, mas nao aparecem como suportadas.
pub fn supported_profiles() -> impl Iterator<Item = &'static BuildProfile> {
    profiles::ALL
        .iter()
        .copied()
        .filter(|profile| profile.status == ProfileStatus::Supported)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_profiles::types::Capability;

    fn any_profile() -> &'static BuildProfile {
        profiles::ALL[0]
    }

    #[test]
    fn resolves_known_hash_to_supported_profile() {
        let profile = any_profile();
        let resolution = resolve(profile.executable_sha256);
        assert!(matches!(resolution, BuildResolution::Supported(_)));
        assert_eq!(
            resolution.supported_profile().map(|entry| entry.id),
            Some(profile.id)
        );
    }

    #[test]
    fn hash_comparison_is_case_insensitive() {
        let profile = any_profile();
        let upper = profile.executable_sha256.to_ascii_uppercase();
        assert!(matches!(resolve(&upper), BuildResolution::Supported(_)));
    }

    #[test]
    fn unknown_hash_is_blocked_and_reports_the_hash() {
        let hash = "0".repeat(64);
        let resolution = resolve(&hash);
        match &resolution {
            BuildResolution::Unsupported { sha256 } => assert_eq!(sha256, &hash),
            other => panic!("build desconhecida deveria ser bloqueada: {other:?}"),
        }
        assert!(resolution.supported_profile().is_none());
        assert!(resolution.matched_profile().is_none());
    }

    #[test]
    fn every_registered_profile_has_a_well_formed_hash() {
        for profile in profiles::ALL {
            assert_eq!(
                profile.executable_sha256.len(),
                64,
                "{} tem hash com tamanho invalido",
                profile.id
            );
            assert!(
                profile
                    .executable_sha256
                    .chars()
                    .all(|character| character.is_ascii_hexdigit()
                        && !character.is_ascii_uppercase()),
                "{} deve usar hash hexadecimal minuscula",
                profile.id
            );
        }
    }

    #[test]
    fn profile_ids_and_hashes_are_unique() {
        for (index, profile) in profiles::ALL.iter().enumerate() {
            for other in &profiles::ALL[index + 1..] {
                assert_ne!(profile.id, other.id, "ids de perfil duplicados");
                assert_ne!(
                    profile.executable_sha256, other.executable_sha256,
                    "hashes de perfil duplicadas"
                );
            }
        }
    }

    #[test]
    fn native_signatures_are_never_empty() {
        for profile in profiles::ALL {
            let natives = &profile.natives;
            for native in [
                natives.unlock_all_weapons,
                natives.unlock_all_perks,
                natives.unlock_all_upgrades,
                natives.forge_schematic_save,
                natives.overclock_reward,
                natives.skin_reward,
                natives.vanity_reward,
                natives.victory_pose_reward,
                natives.retire_character,
                natives.save_to_disk,
                natives.add_resource,
            ] {
                assert!(
                    !native.prefix.is_empty(),
                    "{} / {} precisa de prefixo de validacao",
                    profile.id,
                    native.label
                );
                assert!(
                    native.rva != 0,
                    "{} / {} precisa de RVA",
                    profile.id,
                    native.label
                );
            }
        }
    }

    #[test]
    fn resource_ids_are_unique_within_each_profile() {
        for profile in profiles::ALL {
            for (index, resource) in profile.resources.iter().enumerate() {
                for other in &profile.resources[index + 1..] {
                    assert_ne!(
                        resource.id, other.id,
                        "{} tem recursos com id repetido",
                        profile.id
                    );
                    assert_ne!(
                        resource.savegame_id, other.savegame_id,
                        "{} tem recursos com SavegameID repetido",
                        profile.id
                    );
                }
                assert_ne!(resource.savegame_id, [0; 16]);
            }
        }
    }

    #[test]
    fn playable_classes_are_complete_and_distinct() {
        for profile in profiles::ALL {
            let classes = profile.catalog.playable_classes;
            assert_eq!(
                classes.len(),
                4,
                "{} deve ter 4 classes jogaveis",
                profile.id
            );
            for (index, class) in classes.iter().enumerate() {
                assert_ne!(class.savegame_id, [0; 16]);
                for other in &classes[index + 1..] {
                    assert_ne!(class.name, other.name);
                    assert_ne!(class.savegame_id, other.savegame_id);
                }
            }
        }
    }

    #[test]
    fn unsupported_capability_is_refused_before_any_memory_access() {
        let profile = any_profile();
        // Todas as capacidades do perfil atual estao verificadas.
        assert!(profile.require(Capability::Credits).is_ok());

        let mut blocked = *profile;
        blocked.capabilities.credits = false;
        let error = blocked
            .require(Capability::Credits)
            .expect_err("capacidade nao verificada deve bloquear");
        assert_eq!(
            error.code(),
            crate::shared::ErrorCode::CapabilityUnavailable
        );
    }

    #[test]
    fn draft_profiles_never_allow_memory_operations() {
        assert!(!ProfileStatus::Draft.allows_memory_operations());
        assert!(!ProfileStatus::Verified.allows_memory_operations());
        assert!(!ProfileStatus::Deprecated.allows_memory_operations());
        assert!(ProfileStatus::Supported.allows_memory_operations());
    }

    #[test]
    fn schematic_catalog_totals_match_the_documented_count() {
        let profile = any_profile();
        assert_eq!(profile.catalog.total_schematic_count(), 508);
    }
}
