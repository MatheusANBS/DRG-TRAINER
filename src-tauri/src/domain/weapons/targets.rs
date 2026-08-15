//! Descoberta da arma equipada e dos campos de dano.
//!
//! Conhecimento especifico do DRG: nomes de propriedades, componentes de dano e
//! lancadores de projetil. O runtime Unreal continua generico; aqui usamos a
//! reflexao dele para achar os enderecos certos sem offsets fixos.

use crate::domain::dto::{ClipStatus, DamageStatus};
use crate::infrastructure::process::{MemoryReader, MemoryWriter};
use crate::infrastructure::unreal::UnrealRuntime;
use crate::shared::error::{Result, codes};
use crate::shared::limits::{DAMAGE_VALUE, MAX_DAMAGE_TARGETS};

/// Teto plausivel para um campo de dano; acima disso o ponteiro e lixo.
const MAX_DAMAGE_FIELD: f32 = 100_000_000.0;
/// Intervalo aceito para o tamanho do carregador.
const CLIP_SIZE_RANGE: std::ops::RangeInclusive<i32> = 0..=1_000_000;

/// Propriedades que apontam direto para um componente de dano.
const DAMAGE_PROPERTIES: [&str; 21] = [
    "Damage",
    "DamageComponent",
    "DamageComp",
    "ShockWaveDamageComponent",
    "AoEDamageComponent",
    "ExplosionDamage",
    "CritcalOverheatDamage",
    "BurstFireBonusDamage",
    "FireExplosionDamage",
    "OverchargeDamageComponent",
    "WeaponBlastDamage",
    "ShotwaveBonusDamage",
    "MoleBonusDamage",
    "AoEHeatDamageComponent",
    "AoEColdDamageComponent",
    "ExplodingTargetsDamageComponent",
    "RadiantSuperheaterFrostShock",
    "BarrelProximityDamageComponent",
    "MainDamageComponent",
    "SimpleDamageComponent",
    "InitialDamageComponent",
];

/// Componentes de disparo hitscan que carregam o dano em si.
const FIRE_PROPERTIES: [&str; 8] = [
    "WeaponFire",
    "HitScan",
    "Hitscan",
    "HitscanComponent",
    "MultiHitscan",
    "AllPiercingHitscan",
    "CapsuleHitscanComp",
    "ReflectionHitscanComponent",
];

/// Lancadores de projetil, cujo dano vive no CDO da classe do projetil.
const LAUNCHER_PROPERTIES: [&str; 3] = [
    "projectileLauncher",
    "ProjectileLancher",
    "ChargedProjectileLauncher",
];

/// Classes de projetil referenciadas por um lancador.
const PROJECTILE_CLASS_PROPERTIES: [&str; 3] = [
    "ProjectileClass",
    "NormalProjectileClass",
    "ChargedProjectileClass",
];

/// Um valor so e considerado campo de dano se for finito, positivo e dentro da
/// faixa plausivel — ou se ja for o valor congelado pelo trainer.
fn is_damage_field(value: f32) -> bool {
    value.is_finite()
        && (value > 0.0 || (value - DAMAGE_VALUE).abs() < f32::EPSILON)
        && value < MAX_DAMAGE_FIELD
}

/// Ator atualmente equipado pelo jogador.
pub fn equipped_actor<M: MemoryReader>(runtime: &UnrealRuntime<M>) -> Result<usize> {
    let player = &runtime.profile().offsets.player;
    let memory = runtime.memory();

    let controller = runtime.active_controller()?;
    let pawn = memory.read_pointer(controller + player.controller_pawn)?;
    if pawn == 0 {
        return Err(codes::object_not_found(
            "O player ainda nao possui Pawn ativo.",
        ));
    }
    let inventory = memory.read_pointer(pawn + player.player_inventory)?;
    if inventory == 0 {
        return Err(codes::object_not_found(
            "InventoryComponent do player ainda nao foi carregado.",
        ));
    }
    let equipped = memory.read_pointer(inventory + player.inventory_equipped_actor)?;
    if equipped == 0 {
        return Err(codes::object_not_found("Nenhum item esta equipado."));
    }
    Ok(equipped)
}

pub fn is_ammo_weapon<M: MemoryReader>(runtime: &UnrealRuntime<M>, object: usize) -> Result<bool> {
    runtime.is_instance_of(object, runtime.profile().catalog.ammo_weapon_class)
}

/// Coleta os enderecos de dano de um componente, se ele for um DamageComponent.
fn push_component_targets<M: MemoryReader>(
    runtime: &UnrealRuntime<M>,
    component: usize,
    targets: &mut Vec<usize>,
) -> Result<()> {
    if !runtime.is_instance_of(component, "DamageComponent")? {
        return Ok(());
    }
    for field_name in ["Damage", "RadialDamage"] {
        let Some(offset) = runtime.find_property_offset(component, field_name)? else {
            continue;
        };
        let address = component + offset;
        if is_damage_field(runtime.memory().read_f32(address)?) {
            targets.push(address);
        }
    }
    Ok(())
}

/// Percorre um objeto atras de todos os campos de dano alcancaveis.
fn targets_from_object<M: MemoryReader>(
    runtime: &UnrealRuntime<M>,
    object: usize,
    follow_projectiles: bool,
) -> Result<Vec<usize>> {
    let memory = runtime.memory();
    let mut targets = Vec::new();

    for property in DAMAGE_PROPERTIES {
        if let Some(component) = runtime.object_property(object, property)? {
            push_component_targets(runtime, component, &mut targets)?;
        }
    }

    for property in FIRE_PROPERTIES {
        let Some(fire) = runtime.object_property(object, property)? else {
            continue;
        };
        if !runtime.is_instance_of(fire, "HitscanBaseComponent")? {
            continue;
        }
        let before = targets.len();
        if let Some(component) = runtime.object_property(fire, "DamageComponent")? {
            push_component_targets(runtime, component, &mut targets)?;
        }
        // Algumas armas guardam o dano direto no componente de disparo.
        if targets.len() == before
            && let Some(offset) = runtime.find_property_offset(fire, "Damage")?
        {
            let address = fire + offset;
            if is_damage_field(memory.read_f32(address)?) {
                targets.push(address);
            }
        }
    }

    if follow_projectiles {
        for launcher_name in LAUNCHER_PROPERTIES {
            let Some(launcher) = runtime.object_property(object, launcher_name)? else {
                continue;
            };
            if !runtime.is_instance_of(launcher, "ProjectileLauncherBaseComponent")? {
                continue;
            }
            for class_name in PROJECTILE_CLASS_PROPERTIES {
                let Some(projectile_class) = runtime.object_property(launcher, class_name)? else {
                    continue;
                };
                let Some(default_object) = runtime.class_default_object(projectile_class)? else {
                    continue;
                };
                // Um nivel apenas: projeteis nao lancam outros projeteis.
                targets.extend(targets_from_object(runtime, default_object, false)?);
            }
        }
    }

    targets.sort_unstable();
    targets.dedup();
    if targets.len() > MAX_DAMAGE_TARGETS {
        return Err(codes::invalid_state(format!(
            "Quantidade inesperada de campos de dano: {}.",
            targets.len()
        )));
    }
    Ok(targets)
}

pub fn damage_targets<M: MemoryReader>(
    runtime: &UnrealRuntime<M>,
    weapon: usize,
) -> Result<Vec<usize>> {
    targets_from_object(runtime, weapon, true)
}

/// Estado do carregador da arma equipada; congela quando `freeze`.
pub fn clip_snapshot<M: MemoryWriter>(
    runtime: &UnrealRuntime<M>,
    enabled: bool,
    freeze: bool,
) -> Result<ClipStatus> {
    let player = &runtime.profile().offsets.player;
    let memory = runtime.memory();

    let equipped = equipped_actor(runtime)?;
    let weapon_name = runtime
        .read_object_name(equipped)
        .unwrap_or_else(|_| "Unknown weapon".into());

    if !is_ammo_weapon(runtime, equipped)? {
        return Ok(ClipStatus {
            enabled,
            available: false,
            pid: Some(runtime.pid()),
            weapon_name: Some(weapon_name),
            clip_count: None,
            clip_size: None,
            address: None,
            message: "O item equipado nao usa AmmoDrivenWeapon.".into(),
        });
    }

    let clip_size = memory.read_i32(equipped + player.clip_size)?;
    if !CLIP_SIZE_RANGE.contains(&clip_size) {
        return Err(codes::invalid_state(format!(
            "ClipSize fora do intervalo seguro: {clip_size}."
        )));
    }

    let address = equipped + player.clip_count;
    let mut clip_count = memory.read_i32(address)?;
    if freeze && clip_count != clip_size {
        // `write_i32_verified` releitura e confere: uma escrita que nao pode
        // ser confirmada vira erro em vez de sucesso silencioso.
        memory
            .write_i32_verified(address, clip_size)
            .map_err(|error| error.context("A verificacao do freeze de ClipCount falhou"))?;
        clip_count = clip_size;
    }

    Ok(ClipStatus {
        enabled,
        available: true,
        pid: Some(runtime.pid()),
        weapon_name: Some(weapon_name),
        clip_count: Some(clip_count),
        clip_size: Some(clip_size),
        address: Some(format!("0x{address:X}")),
        message: if freeze {
            "Infinite Magazine ativo.".into()
        } else {
            "Arma equipada encontrada.".into()
        },
    })
}

/// Estado dos campos de dano da arma equipada; congela quando `freeze`.
pub fn damage_snapshot<M: MemoryWriter>(
    runtime: &UnrealRuntime<M>,
    enabled: bool,
    freeze: bool,
) -> Result<DamageStatus> {
    let memory = runtime.memory();
    let equipped = equipped_actor(runtime)?;
    let weapon_name = runtime
        .read_object_name(equipped)
        .unwrap_or_else(|_| "Unknown weapon".into());

    if !is_ammo_weapon(runtime, equipped)? {
        return Ok(DamageStatus {
            enabled,
            available: false,
            pid: Some(runtime.pid()),
            weapon_name: Some(weapon_name),
            value: None,
            target_count: 0,
            addresses: Vec::new(),
            message: "O item equipado nao usa AmmoDrivenWeapon.".into(),
        });
    }

    let targets = damage_targets(runtime, equipped)?;
    if targets.is_empty() {
        return Ok(DamageStatus {
            enabled,
            available: false,
            pid: Some(runtime.pid()),
            weapon_name: Some(weapon_name),
            value: None,
            target_count: 0,
            addresses: Vec::new(),
            message: "Nenhum DamageComponent ativo foi encontrado nesta arma.".into(),
        });
    }

    let mut value = memory.read_f32(targets[0])?;
    if freeze {
        for address in &targets {
            if (memory.read_f32(*address)? - DAMAGE_VALUE).abs() >= f32::EPSILON {
                memory.write_f32(*address, DAMAGE_VALUE)?;
            }
            let verified = memory.read_f32(*address)?;
            if (verified - DAMAGE_VALUE).abs() >= f32::EPSILON {
                return Err(codes::invalid_state(format!(
                    "A verificacao do dano falhou em 0x{address:X}: {verified}."
                )));
            }
        }
        value = DAMAGE_VALUE;
    }

    Ok(DamageStatus {
        enabled,
        available: true,
        pid: Some(runtime.pid()),
        weapon_name: Some(weapon_name),
        value: Some(value),
        target_count: targets.len(),
        addresses: targets
            .iter()
            .map(|address| format!("0x{address:X}"))
            .collect(),
        message: if freeze {
            format!("Weapon Damage ativo em {} campo(s).", targets.len())
        } else {
            format!("{} campo(s) de dano encontrado(s).", targets.len())
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::testing::profile;
    use crate::infrastructure::unreal::testing::TestWorld;
    use crate::shared::ErrorCode;

    /// Constroi controller -> pawn -> inventory -> arma equipada.
    struct WeaponWorld {
        world: TestWorld,
        weapon: usize,
    }

    impl WeaponWorld {
        fn new(equip_ammo_weapon: bool) -> Self {
            let world = TestWorld::new();
            let player = &profile().offsets.player;
            let controller_name = profile().catalog.player_controller_names[0];

            let controller_class = world.spawn_class(controller_name, None);
            let controller = world.spawn_object(controller_name, controller_class);
            let pawn_class = world.spawn_class("BP_PlayerCharacter_C", None);
            let pawn = world.spawn_object("BP_PlayerCharacter_C", pawn_class);
            let inventory_class = world.spawn_class("InventoryComponent", None);
            let inventory = world.spawn_object("Inventory", inventory_class);

            let weapon_base = world.spawn_class(profile().catalog.ammo_weapon_class, None);
            let weapon_class = if equip_ammo_weapon {
                world.spawn_class("WPN_AssaultRifle_C", Some(weapon_base))
            } else {
                world.spawn_class("BP_Flare_C", None)
            };
            let weapon = world.spawn_object("WPN_AssaultRifle_C", weapon_class);

            world
                .memory()
                .poke_u64(controller + player.controller_pawn, pawn as u64);
            world
                .memory()
                .poke_u64(pawn + player.player_inventory, inventory as u64);
            world
                .memory()
                .poke_u64(inventory + player.inventory_equipped_actor, weapon as u64);
            world.memory().poke_i32(weapon + player.clip_size, 30);
            world.memory().poke_i32(weapon + player.clip_count, 12);

            crate::infrastructure::unreal::cache::invalidate_all();
            Self { world, weapon }
        }
    }

    #[test]
    fn resolves_the_equipped_weapon_through_controller_pawn_and_inventory() {
        let scenario = WeaponWorld::new(true);
        let runtime = scenario.world.runtime();
        assert_eq!(equipped_actor(&runtime).unwrap(), scenario.weapon);
        assert!(is_ammo_weapon(&runtime, scenario.weapon).unwrap());
    }

    #[test]
    fn reports_clip_state_without_writing_when_not_freezing() {
        let scenario = WeaponWorld::new(true);
        let runtime = scenario.world.runtime();
        let status = clip_snapshot(&runtime, false, false).unwrap();

        assert!(status.available);
        assert_eq!(status.clip_count, Some(12));
        assert_eq!(status.clip_size, Some(30));
        assert_eq!(status.weapon_name.as_deref(), Some("WPN_AssaultRifle_C"));
        assert_eq!(status.message, "Arma equipada encontrada.");
    }

    #[test]
    fn freezing_refills_the_magazine_and_verifies_the_write() {
        let scenario = WeaponWorld::new(true);
        let runtime = scenario.world.runtime();
        let status = clip_snapshot(&runtime, true, true).unwrap();

        assert_eq!(status.clip_count, Some(30));
        assert!(status.enabled);
        assert_eq!(
            scenario
                .world
                .memory()
                .read_i32(scenario.weapon + profile().offsets.player.clip_count)
                .unwrap(),
            30
        );
    }

    #[test]
    fn a_non_ammo_item_is_reported_as_unavailable_instead_of_failing() {
        let scenario = WeaponWorld::new(false);
        let runtime = scenario.world.runtime();
        let status = clip_snapshot(&runtime, false, false).unwrap();

        assert!(!status.available);
        assert!(status.clip_count.is_none());
        assert!(status.message.contains("AmmoDrivenWeapon"));
    }

    #[test]
    fn an_implausible_clip_size_is_refused() {
        let scenario = WeaponWorld::new(true);
        scenario
            .world
            .memory()
            .poke_i32(scenario.weapon + profile().offsets.player.clip_size, -1);

        let runtime = scenario.world.runtime();
        let error =
            clip_snapshot(&runtime, false, false).expect_err("ClipSize invalido deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
    }

    #[test]
    fn an_empty_inventory_slot_reports_object_not_found() {
        let scenario = WeaponWorld::new(true);
        let player = &profile().offsets.player;
        let runtime = scenario.world.runtime();
        let controller = runtime.active_controller().unwrap();
        let pawn = scenario
            .world
            .memory()
            .read_u64(controller + player.controller_pawn)
            .unwrap() as usize;
        let inventory = scenario
            .world
            .memory()
            .read_u64(pawn + player.player_inventory)
            .unwrap() as usize;
        scenario
            .world
            .memory()
            .poke_u64(inventory + player.inventory_equipped_actor, 0);

        let error = equipped_actor(&runtime).expect_err("sem item equipado deve falhar");
        assert_eq!(error.code(), ErrorCode::ObjectNotFound);
        assert!(error.message().contains("equipado"));
    }

    #[test]
    fn damage_field_filter_matches_the_documented_rule() {
        assert!(is_damage_field(16.0));
        assert!(is_damage_field(DAMAGE_VALUE));
        assert!(!is_damage_field(0.0));
        assert!(!is_damage_field(-3.0));
        assert!(!is_damage_field(f32::NAN));
        assert!(!is_damage_field(f32::INFINITY));
        assert!(!is_damage_field(MAX_DAMAGE_FIELD));
    }

    #[test]
    fn a_weapon_without_damage_components_is_unavailable_rather_than_broken() {
        let scenario = WeaponWorld::new(true);
        let runtime = scenario.world.runtime();
        let status = damage_snapshot(&runtime, false, false).unwrap();

        assert!(!status.available);
        assert_eq!(status.target_count, 0);
        assert!(status.message.contains("DamageComponent"));
    }
}
