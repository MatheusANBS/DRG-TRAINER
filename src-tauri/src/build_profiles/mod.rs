//! Perfis de build do DRG (SPEC-002).
//!
//! Fronteira de compatibilidade: hash esperada, offsets, assinaturas nativas e
//! catalogos ficam aqui e em nenhum outro lugar. Modulos genericos de leitura,
//! escrita e travessia Unreal recebem o perfil como parametro.

pub mod profiles;
pub mod registry;
pub mod types;

pub use registry::{BuildResolution, resolve, supported_profiles};
pub use types::{
    BuildProfile, Capabilities, Capability, Catalog, NativeFunction, NativeFunctions, Offsets,
    PlayableClass, PlayerOffsets, ProfileDescriptor, ProfileStatus, ResourceDefinition,
    SaveOffsets, UnrealOffsets,
};
