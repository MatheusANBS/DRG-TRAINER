//! Contrato de dados entre backend e frontend.
//!
//! Um unico arquivo com tudo que atravessa a fronteira Tauri, para que uma
//! mudanca de contrato seja visivel em um diff so. Todos os campos usam
//! `camelCase` na serializacao.

use serde::Serialize;

use crate::build_profiles::{Capabilities, ProfileStatus};
use crate::infrastructure::save::BackupSet;

/// Estado do processo do jogo.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProcessState {
    /// O jogo nao esta em execucao.
    Detached,
    /// O processo foi localizado.
    #[serde(rename_all = "camelCase")]
    Attached { pid: u32, executable: String },
}

/// Estado de suporte da build carregada (SPEC-016).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BuildState {
    /// Ainda nao foi possivel identificar a build.
    Unknown,
    /// A hash corresponde a um perfil promovido: e seguro operar.
    #[serde(rename_all = "camelCase")]
    Verified {
        profile_id: String,
        display_name: String,
        sha256: String,
        status: ProfileStatus,
    },
    /// A hash corresponde a um perfil conhecido, mas ainda nao promovido.
    #[serde(rename_all = "camelCase")]
    Unverified {
        profile_id: String,
        display_name: String,
        sha256: String,
        status: ProfileStatus,
    },
    /// Nenhum perfil corresponde: operacoes dependentes de offset bloqueadas.
    #[serde(rename_all = "camelCase")]
    Unsupported { sha256: String },
}

impl BuildState {
    /// Somente `Verified` libera acoes dependentes de memoria.
    pub fn is_operable(&self) -> bool {
        matches!(self, Self::Verified { .. })
    }
}

/// Status operacional completo: o que a UI precisa para decidir o que habilitar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerStatus {
    pub process: ProcessState,
    pub build: BuildState,
    pub capabilities: Capabilities,
    /// Nomes de executavel que o trainer procura, para exibir enquanto aguarda.
    pub expected_executables: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditSnapshot {
    pub credits: i32,
    pub pid: u32,
    pub address: String,
    pub module_base: String,
    pub offset: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    pub previous: i32,
    pub current: i32,
    pub pid: u32,
    pub address: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSnapshot {
    pub id: String,
    pub name: String,
    pub category: String,
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceWriteResult {
    pub pid: u32,
    pub amount_added: i32,
    pub resources: Vec<ResourceSnapshot>,
    pub backup: BackupSet,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockAllResult {
    pub pid: u32,
    pub weapon_count: usize,
    pub unlocked_before: i32,
    pub unlocked_after: i32,
    pub owned_before: i32,
    pub owned_after: i32,
    pub backup: BackupSet,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermanentUnlockResult {
    pub pid: u32,
    pub items_before: i32,
    pub items_after: i32,
    pub backup: BackupSet,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicUnlockResult {
    pub pid: u32,
    pub overclocks_processed: usize,
    pub cosmetics_processed: usize,
    pub forged_before: i32,
    pub forged_after: i32,
    pub owned_before: i32,
    pub owned_after: i32,
    pub backup: BackupSet,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassLevelChange {
    pub class_name: String,
    pub previous_xp: i32,
    pub current_xp: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxClassLevelResult {
    pub pid: u32,
    pub classes_updated: usize,
    pub changes: Vec<ClassLevelChange>,
    pub backup: BackupSet,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionChange {
    pub class_name: String,
    pub previous_promotions: i32,
    pub current_promotions: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoteAllResult {
    pub pid: u32,
    pub classes_promoted: usize,
    pub changes: Vec<PromotionChange>,
    pub backup: BackupSet,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipStatus {
    pub enabled: bool,
    pub available: bool,
    pub pid: Option<u32>,
    pub weapon_name: Option<String>,
    pub clip_count: Option<i32>,
    pub clip_size: Option<i32>,
    pub address: Option<String>,
    pub message: String,
}

impl ClipStatus {
    pub fn pending(enabled: bool, message: impl Into<String>) -> Self {
        Self {
            enabled,
            available: false,
            pid: None,
            weapon_name: None,
            clip_count: None,
            clip_size: None,
            address: None,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageStatus {
    pub enabled: bool,
    pub available: bool,
    pub pid: Option<u32>,
    pub weapon_name: Option<String>,
    pub value: Option<f32>,
    pub target_count: usize,
    pub addresses: Vec<String>,
    pub message: String,
}

impl DamageStatus {
    pub fn pending(enabled: bool, message: impl Into<String>) -> Self {
        Self {
            enabled,
            available: false,
            pid: None,
            weapon_name: None,
            value: None,
            target_count: 0,
            addresses: Vec::new(),
            message: message.into(),
        }
    }
}

/// Resultado do registro de um atalho global.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyResult {
    pub hotkey: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_state_only_reports_operable_when_verified() {
        assert!(
            BuildState::Verified {
                profile_id: "x".into(),
                display_name: "X".into(),
                sha256: "a".repeat(64),
                status: ProfileStatus::Supported,
            }
            .is_operable()
        );
        assert!(!BuildState::Unknown.is_operable());
        assert!(
            !BuildState::Unsupported {
                sha256: "b".repeat(64)
            }
            .is_operable()
        );
        assert!(
            !BuildState::Unverified {
                profile_id: "x".into(),
                display_name: "X".into(),
                sha256: "c".repeat(64),
                status: ProfileStatus::Draft,
            }
            .is_operable()
        );
    }

    #[test]
    fn states_serialize_with_a_discriminating_kind() {
        let detached = serde_json::to_string(&ProcessState::Detached).unwrap();
        assert_eq!(detached, "{\"kind\":\"detached\"}");

        let unsupported = serde_json::to_string(&BuildState::Unsupported {
            sha256: "d".repeat(64),
        })
        .unwrap();
        assert!(unsupported.contains("\"kind\":\"unsupported\""));
        assert!(unsupported.contains("\"sha256\""));
    }

    #[test]
    fn attached_process_exposes_pid_and_executable() {
        let json = serde_json::to_string(&ProcessState::Attached {
            pid: 26248,
            executable: "FSD-Win64-Shipping.exe".into(),
        })
        .unwrap();
        assert!(json.contains("\"pid\":26248"));
        assert!(json.contains("FSD-Win64-Shipping.exe"));
    }
}
