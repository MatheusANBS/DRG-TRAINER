//! Tipos de erro compartilhados por toda a aplicacao.
//!
//! Contrato (SPEC-001 / SPEC-019): todo erro que atravessa a fronteira Tauri
//! carrega um `code` estavel. O frontend mapeia o codigo para uma mensagem
//! propria e usa `recoverable` para decidir se oferece retry automatico.
//! A `message` continua sendo o detalhe tecnico, util para diagnostico.

use serde::Serialize;

/// Codigos estaveis de erro. Mudar um valor existente e uma quebra de contrato
/// com o frontend; adicionar variantes novas nao e.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// O processo do jogo ainda nao esta em execucao.
    ProcessNotFound,
    /// O processo existe mas o modulo principal ainda nao foi mapeado.
    ModuleNotFound,
    /// `OpenProcess` falhou (normalmente permissao insuficiente).
    ProcessOpenFailed,
    /// A hash do executavel nao corresponde a nenhum perfil suportado.
    BuildUnsupported,
    /// Nao foi possivel calcular a hash do executavel.
    BuildHashFailed,
    /// O perfil resolvido nao declara essa capacidade como verificada.
    CapabilityUnavailable,
    /// Falha em `ReadProcessMemory` ou leitura incompleta.
    MemoryReadFailed,
    /// Falha em `WriteProcessMemory` ou escrita incompleta.
    MemoryWriteFailed,
    /// A escrita ocorreu mas a releitura nao confirmou o valor.
    WriteVerificationFailed,
    /// Tentativa de escrita em um handle aberto somente para leitura.
    ReadOnlyHandle,
    /// O prefixo de codigo nativo nao corresponde ao perfil.
    SignatureMismatch,
    /// A execucao remota falhou ao iniciar ou ao confirmar o resultado.
    NativeCallFailed,
    /// A thread remota nao retornou dentro do tempo limite.
    NativeCallTimeout,
    /// O mundo/controller do jogador ainda nao foi carregado.
    WorldNotReady,
    /// O objeto Unreal esperado nao foi encontrado ou e invalido.
    ObjectNotFound,
    /// Nenhuma instancia ativa de save foi localizada.
    SaveNotFound,
    /// O save ativo mudou no meio de uma operacao transacional.
    SaveStateChanged,
    /// O diretorio de saves nao foi encontrado ou a copia falhou.
    BackupFailed,
    /// Parametro invalido vindo do frontend.
    InvalidArgument,
    /// Os dados lidos do jogo estao fora dos limites plausiveis.
    InvalidGameState,
    /// Um `Mutex` interno foi envenenado ou o estado ficou indisponivel.
    StateUnavailable,
    /// O texto do atalho global nao pode ser interpretado.
    HotkeyInvalid,
    /// O sistema operacional recusou o registro do atalho global.
    HotkeyRegistrationFailed,
    /// A tarefa em background falhou antes de produzir um resultado.
    TaskFailed,
}

impl ErrorCode {
    /// Indica se repetir a mesma operacao mais tarde pode ter sucesso sem
    /// intervencao do usuario. O frontend usa isso para escolher entre
    /// "aguardando" e "erro".
    pub fn is_recoverable(self) -> bool {
        matches!(
            self,
            Self::ProcessNotFound
                | Self::ModuleNotFound
                | Self::WorldNotReady
                | Self::ObjectNotFound
                | Self::SaveNotFound
                | Self::NativeCallTimeout
                | Self::TaskFailed
        )
    }

    /// Indica se o erro representa ausencia do jogo, e nao uma falha real.
    /// Estados de espera nao devem ser exibidos como erro no frontend.
    pub fn is_waiting(self) -> bool {
        matches!(self, Self::ProcessNotFound | Self::ModuleNotFound)
    }
}

/// Erro tipado do backend. Sempre carrega codigo + detalhe.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainerError {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
}

impl TrainerError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            recoverable: code.is_recoverable(),
        }
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    /// Acrescenta contexto preservando o codigo original.
    pub fn context(self, prefix: impl std::fmt::Display) -> Self {
        Self {
            message: format!("{prefix}: {}", self.message),
            ..self
        }
    }
}

impl std::fmt::Display for TrainerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TrainerError {}

impl From<std::io::Error> for TrainerError {
    fn from(error: std::io::Error) -> Self {
        Self::new(ErrorCode::BackupFailed, error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TrainerError>;

/// Atalhos de construcao usados com frequencia pelos modulos de dominio.
pub mod codes {
    use super::{ErrorCode, TrainerError};

    pub fn invalid_argument(message: impl Into<String>) -> TrainerError {
        TrainerError::new(ErrorCode::InvalidArgument, message)
    }

    pub fn invalid_state(message: impl Into<String>) -> TrainerError {
        TrainerError::new(ErrorCode::InvalidGameState, message)
    }

    pub fn object_not_found(message: impl Into<String>) -> TrainerError {
        TrainerError::new(ErrorCode::ObjectNotFound, message)
    }

    pub fn state_unavailable(message: impl Into<String>) -> TrainerError {
        TrainerError::new(ErrorCode::StateUnavailable, message)
    }

    pub fn save_state_changed(message: impl Into<String>) -> TrainerError {
        TrainerError::new(ErrorCode::SaveStateChanged, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_codes_are_also_recoverable() {
        for code in [ErrorCode::ProcessNotFound, ErrorCode::ModuleNotFound] {
            assert!(code.is_waiting());
            assert!(code.is_recoverable());
        }
    }

    #[test]
    fn hard_failures_are_not_recoverable() {
        for code in [
            ErrorCode::BuildUnsupported,
            ErrorCode::SignatureMismatch,
            ErrorCode::WriteVerificationFailed,
            ErrorCode::ReadOnlyHandle,
        ] {
            assert!(!code.is_recoverable());
            assert!(!code.is_waiting());
        }
    }

    #[test]
    fn serializes_code_as_screaming_snake_case() {
        let error = TrainerError::new(ErrorCode::BuildUnsupported, "detalhe");
        let json = serde_json::to_string(&error).expect("serializacao deve funcionar");
        assert!(json.contains("\"code\":\"BUILD_UNSUPPORTED\""));
        assert!(json.contains("\"recoverable\":false"));
    }

    #[test]
    fn context_preserves_code_and_recoverability() {
        let error = TrainerError::new(ErrorCode::ProcessNotFound, "sem processo").context("attach");
        assert_eq!(error.code(), ErrorCode::ProcessNotFound);
        assert!(error.recoverable);
        assert_eq!(error.message(), "attach: sem processo");
    }
}
