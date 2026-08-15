//! Tipos compartilhados entre dominio, infraestrutura e camada de aplicacao.
//!
//! Este modulo nao depende de Tauri nem de Win32: ele existe para que
//! infraestrutura e dominio conversem sem se conhecerem (SPEC-004).

pub mod error;
pub mod limits;

pub use error::{ErrorCode, Result, TrainerError};
