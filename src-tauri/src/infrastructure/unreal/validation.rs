//! Validacao de objetos e reflexao de propriedades (SPEC-005).
//!
//! Responsabilidade unica: responder "este ponteiro e mesmo o que eu acho que
//! e?" antes de qualquer leitura dependente de layout. Nenhuma travessia daqui
//! escreve na memoria do jogo.

use crate::infrastructure::process::MemoryReader;
use crate::shared::error::{Result, codes};

use super::types::UnrealRuntime;

/// Profundidade maxima da cadeia de heranca percorrida.
const MAX_CLASS_DEPTH: usize = 32;
/// Numero maximo de propriedades inspecionadas por classe.
const MAX_FIELDS_PER_CLASS: usize = 1_024;
/// Tamanho/offset maximos plausiveis para uma `FProperty`.
const MAX_PROPERTY_EXTENT: usize = 0x10_000;

impl<M: MemoryReader> UnrealRuntime<M> {
    /// Verifica se `object` e instancia de `expected_class` ou de um descendente.
    pub fn is_instance_of(&self, object: usize, expected_class: &str) -> Result<bool> {
        if object == 0 {
            return Ok(false);
        }
        let offsets = self.unreal();
        let mut class = self.memory.read_pointer(object + offsets.uobject_class)?;
        for _ in 0..MAX_CLASS_DEPTH {
            if class == 0 {
                return Ok(false);
            }
            if self.read_object_name(class)? == expected_class {
                return Ok(true);
            }
            class = self.memory.read_pointer(class + offsets.ustruct_super)?;
        }
        Ok(false)
    }

    /// Confere se um candidato a PlayerController ainda e coerente.
    ///
    /// Um objeto pode ter sido reciclado entre dois polls; revalidar e mais
    /// barato do que varrer o array inteiro e mais seguro do que confiar no
    /// cache. Falha de leitura conta como invalido.
    pub(super) fn controller_is_valid(
        &self,
        object: usize,
        name_indices: &[u32],
        require_pawn: bool,
    ) -> bool {
        let offsets = self.unreal();
        let player = &self.profile.offsets.player;

        let check = (|| -> Result<bool> {
            let object_name = self.memory.read_u32(object + offsets.uobject_fname)?;
            if !name_indices.contains(&object_name) {
                return Ok(false);
            }
            let class = self.memory.read_pointer(object + offsets.uobject_class)?;
            if class == 0 || self.memory.read_u32(class + offsets.uobject_fname)? != object_name {
                return Ok(false);
            }
            if require_pawn && self.memory.read_pointer(object + player.controller_pawn)? == 0 {
                return Ok(false);
            }
            Ok(true)
        })();

        check.unwrap_or(false)
    }

    /// Descobre o offset de uma propriedade percorrendo a reflexao do UE.
    ///
    /// Usar reflexao em vez de offset fixo e o que mantem as acoes de arma
    /// funcionando entre variacoes menores de layout.
    pub fn find_property_offset(&self, object: usize, target: &str) -> Result<Option<usize>> {
        let offsets = self.unreal();
        let mut class = self.memory.read_pointer(object + offsets.uobject_class)?;

        for _ in 0..MAX_CLASS_DEPTH {
            if class == 0 {
                return Ok(None);
            }
            let mut field = self
                .memory
                .read_pointer(class + offsets.ustruct_child_properties)?;

            for _ in 0..MAX_FIELDS_PER_CLASS {
                if field == 0 {
                    break;
                }
                let comparison_index = self.memory.read_u32(field + offsets.ffield_fname)?;
                if self.read_fname(comparison_index)? == target {
                    let element_size = self
                        .memory
                        .read_u32(field + offsets.fproperty_element_size)?
                        as usize;
                    let offset = self
                        .memory
                        .read_u32(field + offsets.fproperty_offset_internal)?
                        as usize;
                    if element_size == 0
                        || element_size > MAX_PROPERTY_EXTENT
                        || offset > MAX_PROPERTY_EXTENT
                    {
                        return Err(codes::invalid_state(format!(
                            "Metadata invalida para {target}: size=0x{element_size:X}, offset=0x{offset:X}."
                        )));
                    }
                    return Ok(Some(offset));
                }
                field = self.memory.read_pointer(field + offsets.ffield_next)?;
            }
            class = self.memory.read_pointer(class + offsets.ustruct_super)?;
        }
        Ok(None)
    }

    /// Le uma propriedade do tipo ponteiro, ja descartando nulos.
    pub fn object_property(&self, object: usize, name: &str) -> Result<Option<usize>> {
        let Some(offset) = self.find_property_offset(object, name)? else {
            return Ok(None);
        };
        let value = self.memory.read_pointer(object + offset)?;
        Ok((value != 0).then_some(value))
    }

    /// Objeto padrao (CDO) de uma `UClass`, com checagem de coerencia.
    pub fn class_default_object(&self, class: usize) -> Result<Option<usize>> {
        let offsets = self.unreal();
        let default_object = self
            .memory
            .read_pointer(class + offsets.uclass_default_object)?;
        if default_object == 0 {
            return Ok(None);
        }
        // O CDO precisa apontar de volta para a propria classe.
        if self
            .memory
            .read_pointer(default_object + offsets.uobject_class)?
            != class
        {
            return Ok(None);
        }
        Ok(Some(default_object))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::unreal::testing::{TestWorld, profile};
    use crate::shared::ErrorCode;

    /// Cria uma `FProperty` encadeada na classe informada.
    fn add_property(
        world: &TestWorld,
        class: usize,
        name: &str,
        offset: u32,
        element_size: u32,
    ) -> usize {
        let offsets = &profile().offsets.unreal;
        let field = world.alloc(0x60);
        world
            .memory()
            .poke_u32(field + offsets.ffield_fname, world.intern(name));
        world
            .memory()
            .poke_u32(field + offsets.fproperty_element_size, element_size);
        world
            .memory()
            .poke_u32(field + offsets.fproperty_offset_internal, offset);
        world.memory().poke_u64(field + offsets.ffield_next, 0);

        // Encadeia no fim da lista de propriedades da classe.
        let head = world
            .memory()
            .read_u64(class + offsets.ustruct_child_properties)
            .unwrap() as usize;
        if head == 0 {
            world
                .memory()
                .poke_u64(class + offsets.ustruct_child_properties, field as u64);
        } else {
            let mut current = head;
            loop {
                let next = world
                    .memory()
                    .read_u64(current + offsets.ffield_next)
                    .unwrap() as usize;
                if next == 0 {
                    break;
                }
                current = next;
            }
            world
                .memory()
                .poke_u64(current + offsets.ffield_next, field as u64);
        }
        field
    }

    #[test]
    fn recognizes_a_direct_instance() {
        let world = TestWorld::new();
        let class = world.spawn_class("AmmoDrivenWeapon", None);
        let weapon = world.spawn_object("WPN_AssaultRifle_C", class);
        assert!(
            world
                .runtime()
                .is_instance_of(weapon, "AmmoDrivenWeapon")
                .unwrap()
        );
    }

    #[test]
    fn walks_the_inheritance_chain() {
        let world = TestWorld::new();
        let base = world.spawn_class("AmmoDrivenWeapon", None);
        let middle = world.spawn_class("BurstWeapon", Some(base));
        let leaf = world.spawn_class("WPN_AssaultRifle_C", Some(middle));
        let weapon = world.spawn_object("Instancia", leaf);

        let runtime = world.runtime();
        assert!(runtime.is_instance_of(weapon, "AmmoDrivenWeapon").unwrap());
        assert!(runtime.is_instance_of(weapon, "BurstWeapon").unwrap());
        assert!(!runtime.is_instance_of(weapon, "Grenade").unwrap());
    }

    #[test]
    fn null_object_is_never_an_instance() {
        let world = TestWorld::new();
        assert!(!world.runtime().is_instance_of(0, "Qualquer").unwrap());
    }

    #[test]
    fn a_cyclic_class_chain_terminates() {
        let world = TestWorld::new();
        let offsets = &profile().offsets.unreal;
        let first = world.spawn_class("ClasseA", None);
        let second = world.spawn_class("ClasseB", Some(first));
        // Fecha o ciclo: A -> B -> A.
        world
            .memory()
            .poke_u64(first + offsets.ustruct_super, second as u64);
        let object = world.spawn_object("Instancia", second);

        // Termina por limite de profundidade em vez de rodar para sempre.
        assert!(
            !world
                .runtime()
                .is_instance_of(object, "Inexistente")
                .unwrap()
        );
    }

    #[test]
    fn finds_a_property_offset_through_reflection() {
        let world = TestWorld::new();
        let class = world.spawn_class("DamageComponent", None);
        add_property(&world, class, "Damage", 0x120, 4);
        let component = world.spawn_object("Componente", class);

        assert_eq!(
            world
                .runtime()
                .find_property_offset(component, "Damage")
                .unwrap(),
            Some(0x120)
        );
    }

    #[test]
    fn finds_a_property_declared_on_a_parent_class() {
        let world = TestWorld::new();
        let base = world.spawn_class("BaseComponent", None);
        add_property(&world, base, "RadialDamage", 0x40, 4);
        let derived = world.spawn_class("DamageComponent", Some(base));
        let component = world.spawn_object("Componente", derived);

        assert_eq!(
            world
                .runtime()
                .find_property_offset(component, "RadialDamage")
                .unwrap(),
            Some(0x40)
        );
    }

    #[test]
    fn unknown_property_returns_none_instead_of_failing() {
        let world = TestWorld::new();
        let class = world.spawn_class("DamageComponent", None);
        add_property(&world, class, "Damage", 0x120, 4);
        let component = world.spawn_object("Componente", class);

        assert_eq!(
            world
                .runtime()
                .find_property_offset(component, "NaoExiste")
                .unwrap(),
            None
        );
    }

    #[test]
    fn implausible_property_metadata_is_rejected() {
        let world = TestWorld::new();
        let class = world.spawn_class("DamageComponent", None);
        add_property(&world, class, "Damage", 0x120, 0);
        let component = world.spawn_object("Componente", class);

        let error = world
            .runtime()
            .find_property_offset(component, "Damage")
            .expect_err("element size zero deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
    }

    #[test]
    fn object_property_discards_null_pointers() {
        let world = TestWorld::new();
        let class = world.spawn_class("Weapon", None);
        add_property(&world, class, "DamageComponent", 0x200, 8);
        let weapon = world.spawn_object("Arma", class);

        let runtime = world.runtime();
        assert_eq!(
            runtime.object_property(weapon, "DamageComponent").unwrap(),
            None
        );

        world.memory().poke_u64(weapon + 0x200, 0xABCD_0000);
        assert_eq!(
            runtime.object_property(weapon, "DamageComponent").unwrap(),
            Some(0xABCD_0000)
        );
    }

    #[test]
    fn class_default_object_requires_a_back_reference() {
        let world = TestWorld::new();
        let offsets = &profile().offsets.unreal;
        let class = world.spawn_class("ProjectileClass", None);
        let cdo = world.spawn_object("Default__Projectile", class);
        world
            .memory()
            .poke_u64(class + offsets.uclass_default_object, cdo as u64);

        let runtime = world.runtime();
        assert_eq!(runtime.class_default_object(class).unwrap(), Some(cdo));

        // CDO apontando para outra classe e descartado.
        let other = world.spawn_class("Outra", None);
        world
            .memory()
            .poke_u64(cdo + offsets.uobject_class, other as u64);
        assert_eq!(runtime.class_default_object(class).unwrap(), None);
    }
}
