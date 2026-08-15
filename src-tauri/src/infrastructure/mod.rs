//! Infraestrutura: tudo que fala com o sistema operacional ou com o processo
//! do jogo (SPEC-004).
//!
//! - `process` — fronteira Win32 de processo e memoria (SPEC-001);
//! - `unreal`  — runtime Unreal generico (SPEC-005);
//! - `save`    — transacoes de mutacao permanente de save (SPEC-020).
//!
//! A infraestrutura depende de `shared` e `build_profiles`. Ela nunca depende
//! de `domain` nem de Tauri.

pub mod process;
pub mod save;
pub mod unreal;
