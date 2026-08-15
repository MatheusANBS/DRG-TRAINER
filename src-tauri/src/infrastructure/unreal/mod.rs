//! Runtime Unreal generico (SPEC-005).
//!
//! Decomposto por eixo de responsabilidade:
//!
//! - `types`    — o handle `UnrealRuntime` e seus enderecos derivados;
//! - `fname`    — leitura e busca na `FNamePool`;
//! - `object`   — resolucao e travessia da `GUObjectArray`;
//! - `validation` — checagem de instancia e reflexao de propriedades;
//! - `native_call` — execucao de rotinas nativas, sempre com assinatura validada;
//! - `cache`    — memoizacao com invalidacao explicita.
//!
//! Nada aqui conhece regras do DRG: nomes de classes, offsets e catalogos
//! chegam pelo `BuildProfile`. Codigo especifico do jogo mora em `domain`.

pub mod cache;
pub mod fname;
pub mod native_call;
pub mod object;
pub mod types;
pub mod validation;

#[cfg(test)]
pub mod testing;

pub use cache::invalidate_all as invalidate_caches;
pub use object::Scan;
pub use types::UnrealRuntime;
