//! Comandos Tauri.
//!
//! Contrato (SPEC-004): um comando valida entrada, delega ao dominio e traduz o
//! erro. Nenhuma regra de negocio mora aqui — se um comando comecar a decidir
//! algo, a decisao pertence ao dominio.

use crate::domain::dto::{
    ClipStatus, CreditSnapshot, DamageStatus, MaxClassLevelResult, PermanentUnlockResult,
    PromoteAllResult, ResourceSnapshot, ResourceWriteResult, SchematicUnlockResult, TrainerStatus,
    UnlockAllResult, WriteResult,
};
use crate::domain::{inventory, player, progression, status, weapons};
use crate::shared::error::{ErrorCode, TrainerError};

type CommandResult<T> = std::result::Result<T, TrainerError>;

/// Executa trabalho bloqueante fora da thread da WebView.
///
/// Toda operacao de memoria bloqueia; sem isso a UI congelaria durante um
/// unlock ou um backup.
async fn blocking<T, F>(label: &'static str, task: F) -> CommandResult<T>
where
    F: FnOnce() -> CommandResult<T> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|error| {
            TrainerError::new(ErrorCode::TaskFailed, format!("Falha em {label}: {error}"))
        })?
}

#[tauri::command]
pub async fn get_trainer_status() -> CommandResult<TrainerStatus> {
    blocking("status do trainer", || Ok(status::status())).await
}

#[tauri::command]
pub async fn read_credits() -> CommandResult<CreditSnapshot> {
    blocking("leitura de creditos", inventory::read_credits).await
}

#[tauri::command]
pub async fn set_credits(value: i32) -> CommandResult<WriteResult> {
    blocking("escrita de creditos", move || inventory::set_credits(value)).await
}

#[tauri::command]
pub async fn get_resources() -> CommandResult<Vec<ResourceSnapshot>> {
    blocking("leitura de recursos", inventory::read_resources).await
}

#[tauri::command]
pub async fn add_resource(resource_id: String, amount: i32) -> CommandResult<ResourceWriteResult> {
    blocking("adicao de recurso", move || {
        inventory::add_resource(&resource_id, amount)
    })
    .await
}

#[tauri::command]
pub async fn add_all_resources(amount: i32) -> CommandResult<ResourceWriteResult> {
    blocking("adicao de recursos", move || {
        inventory::add_all_resources(amount)
    })
    .await
}

#[tauri::command]
pub async fn get_clip_status() -> CommandResult<ClipStatus> {
    blocking("leitura da arma equipada", weapons::read_clip_status).await
}

#[tauri::command]
pub async fn set_infinite_magazine(enabled: bool) -> CommandResult<ClipStatus> {
    blocking("Infinite Magazine", move || {
        weapons::set_infinite_magazine(enabled)
    })
    .await
}

#[tauri::command]
pub async fn get_damage_status() -> CommandResult<DamageStatus> {
    blocking("leitura do dano da arma", weapons::read_damage_status).await
}

#[tauri::command]
pub async fn set_weapon_damage(enabled: bool) -> CommandResult<DamageStatus> {
    blocking("Weapon Damage", move || weapons::set_weapon_damage(enabled)).await
}

#[tauri::command]
pub async fn unlock_all_weapons() -> CommandResult<UnlockAllResult> {
    blocking("unlock de armas", progression::unlock_all_weapons).await
}

#[tauri::command]
pub async fn unlock_all_perks() -> CommandResult<PermanentUnlockResult> {
    blocking("unlock de perks", progression::unlock_all_perks).await
}

#[tauri::command]
pub async fn unlock_all_gear_modifications() -> CommandResult<PermanentUnlockResult> {
    blocking(
        "unlock de modificacoes",
        progression::unlock_all_gear_modifications,
    )
    .await
}

#[tauri::command]
pub async fn unlock_all_overclocks_and_cosmetics() -> CommandResult<SchematicUnlockResult> {
    blocking(
        "unlock de esquemas",
        progression::unlock_all_overclocks_and_cosmetics,
    )
    .await
}

#[tauri::command]
pub async fn promote_all_classes() -> CommandResult<PromoteAllResult> {
    blocking("promocao de classes", player::promote_all_classes).await
}

#[tauri::command]
pub async fn max_class_level() -> CommandResult<MaxClassLevelResult> {
    blocking("nivel maximo de classes", player::max_class_level).await
}
