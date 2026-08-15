//! Dominio: as regras do DRG (SPEC-004).
//!
//! Depende de `infrastructure`, `build_profiles` e `shared`. Nao conhece Tauri
//! nem Win32 — o que permite testar as regras sem o jogo e sem a WebView.

pub mod dto;
pub mod inventory;
pub mod player;
pub mod progression;
pub mod save_game;
pub mod session;
pub mod status;
pub mod weapons;

#[cfg(test)]
pub mod testing;

/// Testes que exigem o jogo em execucao. Ver `docs/testing.md`.
#[cfg(all(test, feature = "live-tests"))]
mod live_tests;
