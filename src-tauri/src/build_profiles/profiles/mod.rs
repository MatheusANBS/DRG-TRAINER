//! Perfis de build conhecidos.
//!
//! Cada build suportada mora em seu proprio arquivo. Adicionar uma build e
//! criar um modulo novo e registra-lo em `ALL`; nada mais no backend deve
//! precisar de alteracao (SPEC-002).

pub mod fsd_8e22e371;

use crate::build_profiles::types::BuildProfile;

/// Todos os perfis compilados no binario, na ordem em que sao consultados.
pub const ALL: &[&BuildProfile] = &[&fsd_8e22e371::PROFILE];
