//! Caches do runtime Unreal com invalidacao explicita (SPEC-005).
//!
//! Regra: todo cache e chaveado por `(pid, profile_id)`. Trocar de processo
//! (restart do jogo) ou de perfil (build diferente) descarta tudo antes do
//! primeiro uso, sem depender de ninguem lembrar de limpar.

use std::sync::{Mutex, OnceLock};

use crate::shared::error::{Result, codes};

/// Identidade da sessao a que os caches pertencem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheKey {
    pub pid: u32,
    pub profile_id: &'static str,
}

/// Valores derivados caros de recalcular a cada poll.
#[derive(Debug, Default)]
pub struct RuntimeCaches {
    key: Option<CacheKey>,
    /// Indices de FName das classes de PlayerController aceitas.
    pub controller_name_indices: Option<Vec<u32>>,
    /// Ultimo PlayerController resolvido (revalidado antes de reutilizar).
    pub player_controller: Option<usize>,
    /// Ultima instancia ativa de FSDSaveGame (revalidada antes de reutilizar).
    pub active_save: Option<usize>,
}

impl RuntimeCaches {
    fn reset_to(&mut self, key: CacheKey) {
        *self = Self {
            key: Some(key),
            ..Default::default()
        };
    }
}

static CACHES: OnceLock<Mutex<RuntimeCaches>> = OnceLock::new();

fn store() -> &'static Mutex<RuntimeCaches> {
    CACHES.get_or_init(|| Mutex::new(RuntimeCaches::default()))
}

/// Executa `action` com os caches da sessao, descartando-os primeiro se a
/// sessao mudou.
pub fn with<T>(key: CacheKey, action: impl FnOnce(&mut RuntimeCaches) -> T) -> Result<T> {
    let mut guard = store()
        .lock()
        .map_err(|_| codes::state_unavailable("Cache do runtime indisponivel."))?;
    if guard.key != Some(key) {
        guard.reset_to(key);
    }
    Ok(action(&mut guard))
}

/// Descarta todos os caches. Chamado em detach e ao trocar de build.
pub fn invalidate_all() {
    if let Some(store) = CACHES.get()
        && let Ok(mut guard) = store.lock()
    {
        *guard = RuntimeCaches::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = "perfil-teste";

    #[test]
    fn changing_pid_discards_cached_values() {
        invalidate_all();
        let first = CacheKey {
            pid: 100,
            profile_id: PROFILE,
        };
        with(first, |caches| caches.player_controller = Some(0xDEAD)).unwrap();
        let kept = with(first, |caches| caches.player_controller).unwrap();
        assert_eq!(kept, Some(0xDEAD));

        let restarted = CacheKey {
            pid: 101,
            profile_id: PROFILE,
        };
        let after_restart = with(restarted, |caches| caches.player_controller).unwrap();
        assert_eq!(
            after_restart, None,
            "reiniciar o jogo deve invalidar o controller"
        );
    }

    #[test]
    fn changing_profile_discards_cached_values() {
        invalidate_all();
        let first = CacheKey {
            pid: 200,
            profile_id: "build-a",
        };
        with(first, |caches| caches.active_save = Some(0xBEEF)).unwrap();

        let other_build = CacheKey {
            pid: 200,
            profile_id: "build-b",
        };
        let after_build_change = with(other_build, |caches| caches.active_save).unwrap();
        assert_eq!(
            after_build_change, None,
            "trocar de build deve invalidar o save ativo"
        );
    }

    #[test]
    fn explicit_invalidation_clears_everything() {
        let key = CacheKey {
            pid: 300,
            profile_id: PROFILE,
        };
        with(key, |caches| {
            caches.active_save = Some(1);
            caches.player_controller = Some(2);
            caches.controller_name_indices = Some(vec![3]);
        })
        .unwrap();

        invalidate_all();

        let cleared = with(key, |caches| {
            (
                caches.active_save,
                caches.player_controller,
                caches.controller_name_indices.clone(),
            )
        })
        .unwrap();
        assert_eq!(cleared, (None, None, None));
    }
}
