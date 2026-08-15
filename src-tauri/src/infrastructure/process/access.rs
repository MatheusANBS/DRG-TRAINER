//! Contratos de acesso a memoria de processo (SPEC-001 / SPEC-006).
//!
//! O dominio e o runtime Unreal conversam com estes traits, nunca com Win32.
//! Isso mantem a fronteira de seguranca em um unico lugar e permite substituir
//! o backend real por um fake baseado em buffer nos testes offline.

use crate::shared::error::{ErrorCode, Result, TrainerError};

/// Menor endereco de usuario plausivel no Windows x64. Qualquer coisa abaixo
/// disso e ponteiro nulo ou lixo, e nunca deve virar uma leitura.
pub const MIN_USER_ADDRESS: usize = 0x1_0000;

/// Teto de bytes por operacao. Existe para transformar um tamanho corrompido em
/// erro em vez de uma alocacao gigante.
pub const MAX_TRANSFER_SIZE: usize = 16 * 1024 * 1024;

/// Valida um intervalo antes de qualquer dereference.
///
/// Chamado por toda implementacao de leitura/escrita: nenhuma delas confia em
/// enderecos vindos da travessia de objetos do jogo.
pub fn validate_range(address: usize, size: usize) -> Result<()> {
    if size == 0 {
        return Err(TrainerError::new(
            ErrorCode::InvalidArgument,
            "Operacao de memoria com tamanho zero.",
        ));
    }
    if size > MAX_TRANSFER_SIZE {
        return Err(TrainerError::new(
            ErrorCode::InvalidArgument,
            format!("Tamanho de transferencia fora do limite seguro: {size} bytes."),
        ));
    }
    if address < MIN_USER_ADDRESS {
        return Err(TrainerError::new(
            ErrorCode::InvalidGameState,
            format!("Endereco invalido: 0x{address:X}."),
        ));
    }
    if address.checked_add(size).is_none() {
        return Err(TrainerError::new(
            ErrorCode::InvalidGameState,
            format!("Intervalo 0x{address:X}+{size} extrapola o espaco de enderecamento."),
        ));
    }
    Ok(())
}

/// Leitura de memoria de outro processo.
///
/// Implementacoes devem falhar quando a leitura for parcial: retornar bytes
/// zerados de uma pagina inacessivel seria pior do que um erro.
pub trait MemoryReader {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>>;

    fn read_array<const N: usize>(&self, address: usize) -> Result<[u8; N]> {
        let bytes = self.read_bytes(address, N)?;
        bytes.try_into().map_err(|_| {
            TrainerError::new(
                ErrorCode::MemoryReadFailed,
                format!("Leitura de {N} bytes em 0x{address:X} retornou tamanho inesperado."),
            )
        })
    }

    fn read_u16(&self, address: usize) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_array::<2>(address)?))
    }

    fn read_u32(&self, address: usize) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_array::<4>(address)?))
    }

    fn read_i32(&self, address: usize) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_array::<4>(address)?))
    }

    fn read_f32(&self, address: usize) -> Result<f32> {
        Ok(f32::from_le_bytes(self.read_array::<4>(address)?))
    }

    fn read_u64(&self, address: usize) -> Result<u64> {
        Ok(u64::from_le_bytes(self.read_array::<8>(address)?))
    }

    /// Le um ponteiro e ja o converte para `usize`.
    fn read_pointer(&self, address: usize) -> Result<usize> {
        Ok(self.read_u64(address)? as usize)
    }

    /// Le um GUID de 16 bytes (SavegameID).
    fn read_guid(&self, address: usize) -> Result<[u8; 16]> {
        self.read_array::<16>(address)
    }

    /// Confere que os bytes iniciais de uma rotina correspondem ao perfil.
    ///
    /// Nenhuma chamada nativa acontece sem passar por aqui.
    fn validate_code_prefix(&self, address: usize, expected: &[u8], label: &str) -> Result<()> {
        if expected.is_empty() {
            return Err(TrainerError::new(
                ErrorCode::InvalidArgument,
                format!("{label} nao possui assinatura de validacao no perfil."),
            ));
        }
        let actual = self.read_bytes(address, expected.len())?;
        if actual != expected {
            return Err(TrainerError::new(
                ErrorCode::SignatureMismatch,
                format!("A assinatura nativa de {label} nao corresponde a build verificada."),
            ));
        }
        Ok(())
    }
}

/// Escrita de memoria. Sempre exige um handle aberto em modo de escrita e
/// sempre valida a quantidade exata de bytes gravados.
pub trait MemoryWriter: MemoryReader {
    fn write_bytes(&self, address: usize, bytes: &[u8]) -> Result<()>;

    fn write_i32(&self, address: usize, value: i32) -> Result<()> {
        self.write_bytes(address, &value.to_le_bytes())
    }

    fn write_f32(&self, address: usize, value: f32) -> Result<()> {
        self.write_bytes(address, &value.to_le_bytes())
    }

    /// Escreve e confirma relendo. Uma escrita que nao verifica e um bug.
    fn write_i32_verified(&self, address: usize, value: i32) -> Result<i32> {
        let previous = self.read_i32(address)?;
        self.write_bytes(address, &value.to_le_bytes())?;
        let current = self.read_i32(address)?;
        if current != value {
            return Err(TrainerError::new(
                ErrorCode::WriteVerificationFailed,
                format!("A verificacao da escrita falhou: esperado {value}, lido {current}."),
            ));
        }
        Ok(previous)
    }
}

/// Argumentos de uma chamada nativa remota, no ABI x64 do Windows.
#[derive(Debug, Clone, Copy)]
pub enum NativeCall {
    /// A propria funcao vira start routine da thread; um argumento em RCX.
    Direct { argument: usize },
    /// Stub que carrega RCX e RDX.
    TwoArgs { first: usize, second: usize },
    /// Stub que carrega RCX, RDX e R8. `third` pode ser zero por design.
    ThreeArgs {
        first: usize,
        second: usize,
        third: usize,
    },
    /// Stub que carrega RCX, RDX e XMM2 (`AddResource(save, resource, float)`).
    ResourceDelta {
        save: usize,
        resource: usize,
        amount: f32,
    },
}

impl NativeCall {
    /// Argumentos que nao podem ser nulos para a chamada fazer sentido.
    pub(super) fn required_arguments(&self) -> [usize; 2] {
        match *self {
            Self::Direct { argument } => [argument, argument],
            Self::TwoArgs { first, second } => [first, second],
            Self::ThreeArgs { first, second, .. } => [first, second],
            Self::ResourceDelta { save, resource, .. } => [save, resource],
        }
    }
}

/// Execucao de rotinas nativas dentro do processo alvo.
///
/// Separado de leitura/escrita porque so faz sentido contra um processo real:
/// o backend fake de testes recusa a chamada em vez de simula-la.
pub trait NativeCaller {
    fn call_native(&self, address: usize, call: NativeCall, label: &str) -> Result<u32>;
}

/// Acesso completo exigido pelas operacoes de mutacao.
pub trait ProcessAccess: MemoryWriter + NativeCaller {}

impl<T: MemoryWriter + NativeCaller> ProcessAccess for T {}

// Referencias tambem satisfazem os contratos. Isso permite que uma sessao
// empreste o backend para varios adaptadores (runtime Unreal, leitor de save)
// sem duplicar handles do processo.
impl<T: MemoryReader> MemoryReader for &T {
    fn read_bytes(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        (**self).read_bytes(address, size)
    }
}

impl<T: MemoryWriter> MemoryWriter for &T {
    fn write_bytes(&self, address: usize, bytes: &[u8]) -> Result<()> {
        (**self).write_bytes(address, bytes)
    }
}

impl<T: NativeCaller> NativeCaller for &T {
    fn call_native(&self, address: usize, call: NativeCall, label: &str) -> Result<u32> {
        (**self).call_native(address, call, label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_null_and_low_addresses() {
        for address in [0, 1, MIN_USER_ADDRESS - 1] {
            let error = validate_range(address, 4).expect_err("endereco baixo deve falhar");
            assert_eq!(error.code(), ErrorCode::InvalidGameState);
        }
        assert!(validate_range(MIN_USER_ADDRESS, 4).is_ok());
    }

    #[test]
    fn rejects_zero_and_oversized_transfers() {
        assert_eq!(
            validate_range(MIN_USER_ADDRESS, 0)
                .expect_err("tamanho zero deve falhar")
                .code(),
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            validate_range(MIN_USER_ADDRESS, MAX_TRANSFER_SIZE + 1)
                .expect_err("tamanho acima do teto deve falhar")
                .code(),
            ErrorCode::InvalidArgument
        );
        assert!(validate_range(MIN_USER_ADDRESS, MAX_TRANSFER_SIZE).is_ok());
    }

    #[test]
    fn rejects_ranges_that_wrap_the_address_space() {
        let error =
            validate_range(usize::MAX - 1, 16).expect_err("intervalo que estoura deve falhar");
        assert_eq!(error.code(), ErrorCode::InvalidGameState);
    }
}
