mod memory;

use memory::{
    ClipStatus, CreditSnapshot, DamageStatus, MaxClassLevelResult, PermanentUnlockResult,
    PromoteAllResult, ResourceSnapshot, ResourceWriteResult, SchematicUnlockResult,
    UnlockAllResult, WriteResult,
};
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

static CLIP_HOTKEY: OnceLock<Mutex<Option<Shortcut>>> = OnceLock::new();
static DAMAGE_HOTKEY: OnceLock<Mutex<Option<Shortcut>>> = OnceLock::new();

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HotkeyResult {
    hotkey: Option<String>,
}

#[tauri::command]
async fn read_credits() -> Result<CreditSnapshot, String> {
    tauri::async_runtime::spawn_blocking(memory::read_credits)
        .await
        .map_err(|error| format!("Falha na tarefa de leitura: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_credits(value: i32) -> Result<WriteResult, String> {
    if value < 0 {
        return Err("O saldo nao pode ser negativo.".to_string());
    }
    tauri::async_runtime::spawn_blocking(move || memory::set_credits(value))
        .await
        .map_err(|error| format!("Falha na tarefa de escrita: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_resources() -> Result<Vec<ResourceSnapshot>, String> {
    tauri::async_runtime::spawn_blocking(memory::read_resources)
        .await
        .map_err(|error| format!("Falha na leitura dos recursos: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn add_resource(resource_id: String, amount: i32) -> Result<ResourceWriteResult, String> {
    tauri::async_runtime::spawn_blocking(move || memory::add_resource(&resource_id, amount))
        .await
        .map_err(|error| format!("Falha ao adicionar o recurso: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn add_all_resources(amount: i32) -> Result<ResourceWriteResult, String> {
    tauri::async_runtime::spawn_blocking(move || memory::add_all_resources(amount))
        .await
        .map_err(|error| format!("Falha ao adicionar os recursos: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_clip_status() -> Result<ClipStatus, String> {
    tauri::async_runtime::spawn_blocking(memory::read_clip_status)
        .await
        .map_err(|error| format!("Falha na leitura da arma equipada: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_infinite_magazine(enabled: bool) -> Result<ClipStatus, String> {
    tauri::async_runtime::spawn_blocking(move || memory::set_infinite_magazine(enabled))
        .await
        .map_err(|error| format!("Falha ao alterar Infinite Magazine: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn get_damage_status() -> Result<DamageStatus, String> {
    tauri::async_runtime::spawn_blocking(memory::read_damage_status)
        .await
        .map_err(|error| format!("Falha na leitura do dano da arma: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn set_weapon_damage(enabled: bool) -> Result<DamageStatus, String> {
    tauri::async_runtime::spawn_blocking(move || memory::set_weapon_damage(enabled))
        .await
        .map_err(|error| format!("Falha ao alterar Weapon Damage: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn unlock_all_weapons() -> Result<UnlockAllResult, String> {
    tauri::async_runtime::spawn_blocking(memory::unlock_all_weapons)
        .await
        .map_err(|error| format!("Falha na tarefa de unlock: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn unlock_all_perks() -> Result<PermanentUnlockResult, String> {
    tauri::async_runtime::spawn_blocking(memory::unlock_all_perks)
        .await
        .map_err(|error| format!("Falha na tarefa de perks: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn unlock_all_gear_modifications() -> Result<PermanentUnlockResult, String> {
    tauri::async_runtime::spawn_blocking(memory::unlock_all_gear_modifications)
        .await
        .map_err(|error| format!("Falha na tarefa de modificacoes: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn unlock_all_overclocks_and_cosmetics() -> Result<SchematicUnlockResult, String> {
    tauri::async_runtime::spawn_blocking(memory::unlock_all_overclocks_and_cosmetics)
        .await
        .map_err(|error| format!("Falha na tarefa de esquemas: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn promote_all_classes() -> Result<PromoteAllResult, String> {
    tauri::async_runtime::spawn_blocking(memory::promote_all_classes)
        .await
        .map_err(|error| format!("Falha na tarefa de promocao: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn max_class_level() -> Result<MaxClassLevelResult, String> {
    tauri::async_runtime::spawn_blocking(memory::max_class_level)
        .await
        .map_err(|error| format!("Falha na tarefa de nivel maximo: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn set_clip_hotkey(app: tauri::AppHandle, hotkey: Option<String>) -> Result<HotkeyResult, String> {
    set_hotkey(&app, &CLIP_HOTKEY, hotkey)
}

#[tauri::command]
fn set_damage_hotkey(
    app: tauri::AppHandle,
    hotkey: Option<String>,
) -> Result<HotkeyResult, String> {
    set_hotkey(&app, &DAMAGE_HOTKEY, hotkey)
}

fn set_hotkey(
    app: &tauri::AppHandle,
    storage: &'static OnceLock<Mutex<Option<Shortcut>>>,
    hotkey: Option<String>,
) -> Result<HotkeyResult, String> {
    let requested = hotkey
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse::<Shortcut>()
                .map_err(|error| format!("Atalho invalido: {error}"))
        })
        .transpose()?;
    let store = storage.get_or_init(|| Mutex::new(None));
    let mut current = store
        .lock()
        .map_err(|_| "Configuracao de hotkey indisponivel.".to_string())?;

    if *current == requested {
        return Ok(HotkeyResult { hotkey });
    }
    if let Some(shortcut) = requested.as_ref() {
        app.global_shortcut()
            .register(*shortcut)
            .map_err(|error| format!("Nao foi possivel registrar o hotkey: {error}"))?;
    }
    if let Some(previous) = current.take() {
        let _ = app.global_shortcut().unregister(previous);
    }
    *current = requested;
    Ok(HotkeyResult { hotkey })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let triggered = CLIP_HOTKEY
                        .get_or_init(|| Mutex::new(None))
                        .lock()
                        .ok()
                        .and_then(|registered| registered.as_ref().map(|value| value == shortcut))
                        .unwrap_or(false);
                    if triggered {
                        let enabled = !memory::infinite_magazine_enabled();
                        let _ = memory::set_infinite_magazine(enabled);
                    }
                    let damage_triggered = DAMAGE_HOTKEY
                        .get_or_init(|| Mutex::new(None))
                        .lock()
                        .ok()
                        .and_then(|registered| registered.as_ref().map(|value| value == shortcut))
                        .unwrap_or(false);
                    if damage_triggered {
                        let enabled = !memory::weapon_damage_enabled();
                        let _ = memory::set_weapon_damage(enabled);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            read_credits,
            set_credits,
            get_resources,
            add_resource,
            add_all_resources,
            get_clip_status,
            set_infinite_magazine,
            set_clip_hotkey,
            get_damage_status,
            set_weapon_damage,
            set_damage_hotkey,
            unlock_all_weapons,
            unlock_all_perks,
            unlock_all_gear_modifications,
            unlock_all_overclocks_and_cosmetics,
            promote_all_classes,
            max_class_level
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o aplicativo Tauri");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_global_hotkeys() {
        assert!("F8".parse::<Shortcut>().is_ok());
        assert!("Control+Alt+K".parse::<Shortcut>().is_ok());
        assert!("Control+Shift+F12".parse::<Shortcut>().is_ok());
    }
}
