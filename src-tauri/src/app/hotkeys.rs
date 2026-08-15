//! Atalhos globais dos toggles de runtime.
//!
//! Registrar um atalho global e uma operacao com efeito colateral no sistema:
//! o registro novo so substitui o anterior depois de confirmado, para nunca
//! deixar o usuario sem atalho por causa de um texto invalido.

use std::sync::{Mutex, OnceLock};

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::domain::dto::HotkeyResult;
use crate::domain::weapons;
use crate::shared::error::{ErrorCode, Result, TrainerError};

static CLIP_HOTKEY: OnceLock<Mutex<Option<Shortcut>>> = OnceLock::new();
static DAMAGE_HOTKEY: OnceLock<Mutex<Option<Shortcut>>> = OnceLock::new();

fn slot(storage: &'static OnceLock<Mutex<Option<Shortcut>>>) -> &'static Mutex<Option<Shortcut>> {
    storage.get_or_init(|| Mutex::new(None))
}

fn parse(hotkey: Option<&str>) -> Result<Option<Shortcut>> {
    hotkey
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value.parse::<Shortcut>().map_err(|error| {
                TrainerError::new(
                    ErrorCode::HotkeyInvalid,
                    format!("Atalho invalido: {error}"),
                )
            })
        })
        .transpose()
}

fn apply(
    app: &tauri::AppHandle,
    storage: &'static OnceLock<Mutex<Option<Shortcut>>>,
    hotkey: Option<String>,
) -> Result<HotkeyResult> {
    let requested = parse(hotkey.as_deref())?;
    let store = slot(storage);
    let mut current = store.lock().map_err(|_| {
        TrainerError::new(
            ErrorCode::StateUnavailable,
            "Configuracao de hotkey indisponivel.",
        )
    })?;

    if *current == requested {
        return Ok(HotkeyResult { hotkey });
    }
    // Registra o novo antes de soltar o antigo: se o registro falhar, o atalho
    // anterior continua valendo.
    if let Some(shortcut) = requested.as_ref() {
        app.global_shortcut().register(*shortcut).map_err(|error| {
            TrainerError::new(
                ErrorCode::HotkeyRegistrationFailed,
                format!("Nao foi possivel registrar o hotkey: {error}"),
            )
        })?;
    }
    if let Some(previous) = current.take() {
        let _ = app.global_shortcut().unregister(previous);
    }
    *current = requested;
    Ok(HotkeyResult { hotkey })
}

#[tauri::command]
pub fn set_clip_hotkey(
    app: tauri::AppHandle,
    hotkey: Option<String>,
) -> std::result::Result<HotkeyResult, TrainerError> {
    apply(&app, &CLIP_HOTKEY, hotkey)
}

#[tauri::command]
pub fn set_damage_hotkey(
    app: tauri::AppHandle,
    hotkey: Option<String>,
) -> std::result::Result<HotkeyResult, TrainerError> {
    apply(&app, &DAMAGE_HOTKEY, hotkey)
}

/// Verifica se o atalho disparado e o registrado para um slot.
fn matches(storage: &'static OnceLock<Mutex<Option<Shortcut>>>, shortcut: &Shortcut) -> bool {
    slot(storage)
        .lock()
        .ok()
        .and_then(|registered| registered.as_ref().map(|value| value == shortcut))
        .unwrap_or(false)
}

/// Handler global: alterna o toggle correspondente ao atalho pressionado.
pub fn handle(shortcut: &Shortcut, state: ShortcutState) {
    if state != ShortcutState::Pressed {
        return;
    }
    if matches(&CLIP_HOTKEY, shortcut) {
        let enabled = !weapons::infinite_magazine_enabled();
        let _ = weapons::set_infinite_magazine(enabled);
    }
    if matches(&DAMAGE_HOTKEY, shortcut) {
        let enabled = !weapons::weapon_damage_enabled();
        let _ = weapons::set_weapon_damage(enabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_global_hotkeys() {
        for value in ["F8", "F9", "Control+Alt+K", "Control+Shift+F12"] {
            assert!(
                parse(Some(value)).expect("atalho valido").is_some(),
                "{value} deveria ser aceito"
            );
        }
    }

    #[test]
    fn blank_input_clears_the_hotkey_instead_of_failing() {
        assert!(parse(None).unwrap().is_none());
        assert!(parse(Some("")).unwrap().is_none());
        assert!(parse(Some("   ")).unwrap().is_none());
    }

    #[test]
    fn invalid_text_reports_a_stable_error_code() {
        let error = parse(Some("NaoEUmAtalho")).expect_err("texto invalido deve falhar");
        assert_eq!(error.code(), ErrorCode::HotkeyInvalid);
    }

    #[test]
    fn surrounding_whitespace_is_tolerated() {
        assert_eq!(
            parse(Some("  F8  ")).unwrap(),
            parse(Some("F8")).unwrap(),
            "espacos em volta nao devem mudar o atalho"
        );
    }
}
