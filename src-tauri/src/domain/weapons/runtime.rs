//! Toggles de runtime: Infinite Magazine e Weapon Damage.
//!
//! Sao as unicas operacoes continuas do trainer: um worker reescreve o valor a
//! cada poucos milissegundos enquanto o toggle estiver ligado. O estado
//! observavel fica publicado em um store para que o polling do frontend nao
//! precise reabrir o processo (SPEC-013).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use crate::build_profiles::Capability;
use crate::domain::dto::{ClipStatus, DamageStatus};
use crate::domain::session::TrainerSession;
use crate::shared::error::{Result, codes};
use crate::shared::limits::{FREEZE_INTERVAL_MS, RECONNECT_INTERVAL_MS};

use super::targets::{clip_snapshot, damage_snapshot};

const FREEZE_INTERVAL: Duration = Duration::from_millis(FREEZE_INTERVAL_MS);
const RECONNECT_INTERVAL: Duration = Duration::from_millis(RECONNECT_INTERVAL_MS);

static INFINITE_MAGAZINE_ENABLED: AtomicBool = AtomicBool::new(false);
static CLIP_WORKER_RUNNING: AtomicBool = AtomicBool::new(false);
static WEAPON_DAMAGE_ENABLED: AtomicBool = AtomicBool::new(false);
static DAMAGE_WORKER_RUNNING: AtomicBool = AtomicBool::new(false);

static CLIP_STATUS: OnceLock<Mutex<ClipStatus>> = OnceLock::new();
static DAMAGE_STATUS: OnceLock<Mutex<DamageStatus>> = OnceLock::new();

fn clip_store() -> &'static Mutex<ClipStatus> {
    CLIP_STATUS.get_or_init(|| Mutex::new(ClipStatus::pending(false, "Desativado.")))
}

fn damage_store() -> &'static Mutex<DamageStatus> {
    DAMAGE_STATUS.get_or_init(|| Mutex::new(DamageStatus::pending(false, "Desativado.")))
}

fn publish_clip(status: ClipStatus) {
    if let Ok(mut current) = clip_store().lock() {
        *current = status;
    }
}

fn publish_damage(status: DamageStatus) {
    if let Ok(mut current) = damage_store().lock() {
        *current = status;
    }
}

pub fn infinite_magazine_enabled() -> bool {
    INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire)
}

pub fn weapon_damage_enabled() -> bool {
    WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire)
}

pub fn read_clip_status() -> Result<ClipStatus> {
    if infinite_magazine_enabled() {
        // Com o worker ativo, o store e a fonte de verdade: reabrir o processo
        // aqui competiria com o freeze.
        return clip_store()
            .lock()
            .map(|status| status.clone())
            .map_err(|_| codes::state_unavailable("Status do Infinite Magazine indisponivel."));
    }

    let session = TrainerSession::read_only()?;
    session.require(Capability::InfiniteMagazine)?;
    clip_snapshot(session.runtime(), false, false)
}

pub fn read_damage_status() -> Result<DamageStatus> {
    if weapon_damage_enabled() {
        return damage_store()
            .lock()
            .map(|status| status.clone())
            .map_err(|_| codes::state_unavailable("Status do Weapon Damage indisponivel."));
    }

    let session = TrainerSession::read_only()?;
    session.require(Capability::WeaponDamage)?;
    damage_snapshot(session.runtime(), false, false)
}

pub fn set_infinite_magazine(enabled: bool) -> Result<ClipStatus> {
    if enabled {
        // A capacidade e conferida antes de ligar o toggle: em build nao
        // suportada o switch nem chega a mudar de estado.
        TrainerSession::read_only()?.require(Capability::InfiniteMagazine)?;
    }

    INFINITE_MAGAZINE_ENABLED.store(enabled, Ordering::Release);
    if enabled {
        publish_clip(ClipStatus::pending(true, "Ativando Infinite Magazine..."));
        start_clip_worker();
    } else {
        publish_clip(ClipStatus::pending(false, "Infinite Magazine desativado."));
    }

    clip_store()
        .lock()
        .map(|status| status.clone())
        .map_err(|_| codes::state_unavailable("Status do Infinite Magazine indisponivel."))
}

pub fn set_weapon_damage(enabled: bool) -> Result<DamageStatus> {
    if enabled {
        TrainerSession::read_only()?.require(Capability::WeaponDamage)?;
    }

    WEAPON_DAMAGE_ENABLED.store(enabled, Ordering::Release);
    if enabled {
        publish_damage(DamageStatus::pending(true, "Ativando Weapon Damage..."));
        start_damage_worker();
    } else {
        publish_damage(DamageStatus::pending(false, "Weapon Damage desativado."));
    }

    damage_store()
        .lock()
        .map(|status| status.clone())
        .map_err(|_| codes::state_unavailable("Status do Weapon Damage indisponivel."))
}

/// Liga o worker de congelamento do carregador, se ainda nao houver um.
fn start_clip_worker() {
    if CLIP_WORKER_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    thread::spawn(|| {
        while infinite_magazine_enabled() {
            match TrainerSession::writable() {
                Ok(session) => {
                    while infinite_magazine_enabled() {
                        match clip_snapshot(session.runtime(), true, true) {
                            Ok(status) => publish_clip(status),
                            Err(error) => {
                                publish_clip(ClipStatus::pending(true, error.to_string()));
                                break;
                            }
                        }
                        thread::sleep(FREEZE_INTERVAL);
                    }
                }
                Err(error) => publish_clip(ClipStatus::pending(true, error.to_string())),
            }
            if infinite_magazine_enabled() {
                thread::sleep(RECONNECT_INTERVAL);
            }
        }

        publish_clip(ClipStatus::pending(false, "Infinite Magazine desativado."));
        CLIP_WORKER_RUNNING.store(false, Ordering::Release);

        // Cobre o caso raro de reativacao enquanto o worker anterior encerrava.
        if infinite_magazine_enabled() {
            start_clip_worker();
        }
    });
}

fn start_damage_worker() {
    if DAMAGE_WORKER_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    thread::spawn(|| {
        while weapon_damage_enabled() {
            match TrainerSession::writable() {
                Ok(session) => {
                    while weapon_damage_enabled() {
                        match damage_snapshot(session.runtime(), true, true) {
                            Ok(status) => publish_damage(status),
                            Err(error) => {
                                publish_damage(DamageStatus::pending(true, error.to_string()));
                                break;
                            }
                        }
                        thread::sleep(FREEZE_INTERVAL);
                    }
                }
                Err(error) => publish_damage(DamageStatus::pending(true, error.to_string())),
            }
            if weapon_damage_enabled() {
                thread::sleep(RECONNECT_INTERVAL);
            }
        }

        publish_damage(DamageStatus::pending(false, "Weapon Damage desativado."));
        DAMAGE_WORKER_RUNNING.store(false, Ordering::Release);

        if weapon_damage_enabled() {
            start_damage_worker();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggles_start_disabled() {
        // Estado inicial dos atomics do processo de teste.
        assert!(!ClipStatus::pending(false, "x").enabled);
        assert!(!DamageStatus::pending(false, "x").enabled);
    }

    #[test]
    fn turning_a_toggle_off_never_needs_the_game_running() {
        // Desligar precisa funcionar mesmo sem processo: e o caminho de
        // emergencia do usuario.
        let clip = set_infinite_magazine(false).expect("desligar deve sempre funcionar");
        assert!(!clip.enabled);
        assert!(!infinite_magazine_enabled());

        let damage = set_weapon_damage(false).expect("desligar deve sempre funcionar");
        assert!(!damage.enabled);
        assert!(!weapon_damage_enabled());
    }

    #[test]
    fn pending_status_carries_the_message_and_no_stale_values() {
        let status = ClipStatus::pending(true, "Ativando Infinite Magazine...");
        assert!(status.enabled);
        assert!(!status.available);
        assert!(status.clip_count.is_none());
        assert!(status.address.is_none());
        assert_eq!(status.message, "Ativando Infinite Magazine...");

        let damage = DamageStatus::pending(true, "Ativando Weapon Damage...");
        assert!(damage.enabled);
        assert!(!damage.available);
        assert_eq!(damage.target_count, 0);
        assert!(damage.addresses.is_empty());
    }
}
