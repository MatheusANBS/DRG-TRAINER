//! Dominio de armas: descoberta da arma equipada e toggles de runtime.

pub mod runtime;
pub mod targets;

pub use runtime::{
    infinite_magazine_enabled, read_clip_status, read_damage_status, set_infinite_magazine,
    set_weapon_damage, weapon_damage_enabled,
};
