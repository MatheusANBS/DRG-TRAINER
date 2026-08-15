//! DRG Runtime Trainer — backend.
//!
//! Camadas (SPEC-004), de fora para dentro:
//!
//! ```text
//! app             comandos Tauri e atalhos globais
//! domain          regras do DRG (inventario, progressao, jogador, armas)
//! infrastructure  Win32, runtime Unreal e transacoes de save
//! build_profiles  dados especificos de cada build do jogo
//! shared          erros tipados e limites de seguranca
//! ```
//!
//! Regras de dependencia:
//!
//! - `app` conhece `domain`; `domain` conhece `infrastructure`;
//!   `infrastructure` conhece `build_profiles` e `shared`.
//! - Nenhuma seta aponta para tras. Em particular, `domain` nao conhece Tauri e
//!   `infrastructure` nao conhece `domain`.
//! - Chamadas Win32 de processo/memoria existem apenas em
//!   `infrastructure::process` (SPEC-001).

pub mod app;
pub mod build_profiles;
pub mod domain;
pub mod infrastructure;
pub mod shared;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::run();
}
