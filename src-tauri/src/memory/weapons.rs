use super::*;

pub fn read_clip_status() -> Result<ClipStatus> {
    if INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire) {
        return clip_status_store()
            .lock()
            .map(|status| status.clone())
            .map_err(|_| MemoryError("Status do Infinite Magazine indisponivel.".into()));
    }
    clip_context(false)?.snapshot(false)
}

pub fn set_infinite_magazine(enabled: bool) -> Result<ClipStatus> {
    INFINITE_MAGAZINE_ENABLED.store(enabled, Ordering::Release);
    if enabled {
        if let Ok(mut status) = clip_status_store().lock() {
            status.enabled = true;
            status.message = "Ativando Infinite Magazine...".into();
        }
        start_clip_worker();
    } else if let Ok(mut status) = clip_status_store().lock() {
        status.enabled = false;
        status.message = "Infinite Magazine desativado.".into();
    }

    clip_status_store()
        .lock()
        .map(|status| status.clone())
        .map_err(|_| MemoryError("Status do Infinite Magazine indisponivel.".into()))
}

pub fn infinite_magazine_enabled() -> bool {
    INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire)
}

pub fn read_damage_status() -> Result<DamageStatus> {
    if WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire) {
        return damage_status_store()
            .lock()
            .map(|status| status.clone())
            .map_err(|_| MemoryError("Status do Weapon Damage indisponivel.".into()));
    }
    clip_context(false)?.damage_snapshot(false)
}

pub fn set_weapon_damage(enabled: bool) -> Result<DamageStatus> {
    WEAPON_DAMAGE_ENABLED.store(enabled, Ordering::Release);
    if enabled {
        if let Ok(mut status) = damage_status_store().lock() {
            status.enabled = true;
            status.message = "Ativando Weapon Damage...".into();
        }
        start_damage_worker();
    } else if let Ok(mut status) = damage_status_store().lock() {
        status.enabled = false;
        status.message = "Weapon Damage desativado.".into();
    }

    damage_status_store()
        .lock()
        .map(|status| status.clone())
        .map_err(|_| MemoryError("Status do Weapon Damage indisponivel.".into()))
}

pub fn weapon_damage_enabled() -> bool {
    WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire)
}
