//! Leitura da `FNamePool` (SPEC-005).
//!
//! Responsabilidade unica: transformar `ComparisonIndex` em `String` e
//! localizar o indice de um nome conhecido. Nada aqui sabe o que e um
//! PlayerController ou um Schematic.

use crate::infrastructure::process::MemoryReader;
use crate::shared::error::{Result, codes};

use super::types::UnrealRuntime;

/// Comprimento maximo aceito para uma entrada da pool. Acima disso o header
/// lido e lixo, e insistir causaria uma leitura enorme.
const MAX_FNAME_LENGTH: usize = 1024;

/// Offsets internos da `FNamePool`, estaveis entre as builds UE4 do DRG.
const POOL_CURRENT_BLOCK: usize = 0x08;
const POOL_CURRENT_CURSOR: usize = 0x0C;
const POOL_BLOCKS: usize = 0x10;

impl<M: MemoryReader> UnrealRuntime<M> {
    /// Endereco do ponteiro do bloco `index`.
    fn fname_block(&self, index: u32) -> Result<usize> {
        let block = self
            .memory
            .read_pointer(self.fname_pool() + POOL_BLOCKS + index as usize * 8)?;
        if block == 0 {
            return Err(codes::invalid_state(format!(
                "Bloco {index} da FNamePool e nulo."
            )));
        }
        Ok(block)
    }

    /// Resolve um `ComparisonIndex` para texto.
    pub fn read_fname(&self, comparison_index: u32) -> Result<String> {
        let block_index = comparison_index >> 16;
        let offset_units = comparison_index & 0xFFFF;
        let entry = self.fname_block(block_index)? + offset_units as usize * 2;

        let header = self.memory.read_u16(entry)?;
        let length = (header >> 6) as usize;
        let is_wide = header & 1 != 0;
        if length == 0 || length > MAX_FNAME_LENGTH {
            return Err(codes::invalid_state(format!(
                "Comprimento FName invalido: {length}."
            )));
        }

        if is_wide {
            let bytes = self.memory.read_bytes(entry + 2, length * 2)?;
            let units = bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<_>>();
            String::from_utf16(&units).map_err(|error| codes::invalid_state(error.to_string()))
        } else {
            String::from_utf8(self.memory.read_bytes(entry + 2, length)?)
                .map_err(|error| codes::invalid_state(error.to_string()))
        }
    }

    /// Nome de um `UObject` a partir do seu `FName`.
    pub fn read_object_name(&self, object: usize) -> Result<String> {
        self.read_fname(self.memory.read_u32(object + self.unreal().uobject_fname)?)
    }

    /// Varre a pool procurando um nome ASCII conhecido e devolve seu indice.
    ///
    /// A varredura e sequencial de proposito: a pool nao expoe um indice
    /// reverso, e o resultado e cacheado pelas camadas superiores.
    pub fn find_fname_index(&self, target: &str) -> Result<u32> {
        let pool = self.fname_pool();
        let block_size = self.unreal().fname_block_size;
        let current_block = self.memory.read_u32(pool + POOL_CURRENT_BLOCK)?;
        let current_cursor = self.memory.read_u32(pool + POOL_CURRENT_CURSOR)? as usize;
        let target_bytes = target.as_bytes();

        for block_index in 0..=current_block {
            let Ok(block) = self.fname_block(block_index) else {
                continue;
            };
            let size = if block_index == current_block {
                current_cursor.min(block_size)
            } else {
                block_size
            };
            if size == 0 {
                continue;
            }
            let bytes = self.memory.read_bytes(block, size)?;

            let mut offset = 0_usize;
            while offset + 2 <= bytes.len() {
                let header = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]);
                let length = (header >> 6) as usize;
                let is_wide = header & 1 != 0;
                let payload = length.saturating_mul(if is_wide { 2 } else { 1 });
                if length == 0 || length > MAX_FNAME_LENGTH || offset + 2 + payload > bytes.len() {
                    offset += 2;
                    continue;
                }
                if !is_wide
                    && length == target_bytes.len()
                    && bytes[offset + 2..offset + 2 + length] == *target_bytes
                {
                    return Ok((block_index << 16) | (offset as u32 >> 1));
                }
                offset += (2 + payload + 1) & !1;
            }
        }

        Err(codes::object_not_found(format!(
            "FName {target:?} nao encontrado na build carregada."
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::unreal::testing::{FNAME_BLOCK, TestWorld};

    #[test]
    fn reads_a_narrow_fname() {
        let world = TestWorld::new();
        let index = world.intern("FSDSaveGame");
        assert_eq!(world.runtime().read_fname(index).unwrap(), "FSDSaveGame");
    }

    #[test]
    fn finds_the_index_of_an_interned_name() {
        let world = TestWorld::new();
        let expected = world.intern("BP_PlayerController_SpaceRig_C");
        world.intern("AmmoDrivenWeapon");
        let found = world
            .runtime()
            .find_fname_index("BP_PlayerController_SpaceRig_C")
            .expect("nome interno deve ser encontrado");
        assert_eq!(found, expected);
    }

    #[test]
    fn missing_name_reports_object_not_found() {
        let world = TestWorld::new();
        world.intern("Existe");
        let error = world
            .runtime()
            .find_fname_index("NaoExiste")
            .expect_err("nome ausente deve falhar");
        assert_eq!(error.code(), crate::shared::ErrorCode::ObjectNotFound);
    }

    #[test]
    fn rejects_entries_with_implausible_length() {
        let world = TestWorld::new();
        // Header com length = 0 no primeiro slot do bloco.
        world.memory().poke(FNAME_BLOCK, &[0x00, 0x00]);
        let error = world
            .runtime()
            .read_fname(0)
            .expect_err("comprimento zero deve falhar");
        assert_eq!(error.code(), crate::shared::ErrorCode::InvalidGameState);
    }

    #[test]
    fn null_block_is_reported_instead_of_dereferenced() {
        let world = TestWorld::new();
        // Bloco 5 nao foi mapeado: o ponteiro e zero.
        let error = world
            .runtime()
            .read_fname(5 << 16)
            .expect_err("bloco nulo deve falhar");
        assert_eq!(error.code(), crate::shared::ErrorCode::InvalidGameState);
    }
}
