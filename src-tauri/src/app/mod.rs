//! Camada de aplicacao: a fronteira Tauri (SPEC-004).
//!
//! Unico modulo que conhece `tauri`. Ele registra comandos, atalhos globais e
//! monta o app; toda regra de negocio vive em `domain`.

pub mod commands;
pub mod hotkeys;

/// Monta e executa o aplicativo.
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, shortcut, event| {
                    hotkeys::handle(shortcut, event.state());
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::get_trainer_status,
            commands::read_credits,
            commands::set_credits,
            commands::get_resources,
            commands::add_resource,
            commands::add_all_resources,
            commands::get_clip_status,
            commands::set_infinite_magazine,
            commands::get_damage_status,
            commands::set_weapon_damage,
            commands::unlock_all_weapons,
            commands::unlock_all_perks,
            commands::unlock_all_gear_modifications,
            commands::unlock_all_overclocks_and_cosmetics,
            commands::promote_all_classes,
            commands::max_class_level,
            hotkeys::set_clip_hotkey,
            hotkeys::set_damage_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("falha ao iniciar o aplicativo Tauri");
}
