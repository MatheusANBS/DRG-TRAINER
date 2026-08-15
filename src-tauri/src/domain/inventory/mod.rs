//! Dominio de inventario: creditos e recursos.

pub mod credits;
pub mod resources;

pub use credits::{read_credits, set_credits};
pub use resources::{add_all_resources, add_resource, read_resources};
