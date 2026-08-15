//! Limites de seguranca da aplicacao.
//!
//! Diferente dos dados em `build_profiles`, estes valores sao politica do
//! trainer e nao mudam quando a build do jogo muda. Eles existem para impedir
//! que uma leitura corrompida vire uma escrita destrutiva.

/// Maior delta aceito em uma unica operacao de recurso.
pub const MAX_RESOURCE_DELTA: i32 = 1_000_000;

/// Teto absoluto do total de um recurso apos a operacao.
pub const MAX_RESOURCE_TOTAL: f32 = 100_000_000.0;

/// Valor gravado nos campos de dano quando Weapon Damage esta ativo.
pub const DAMAGE_VALUE: f32 = 9_999.0;

/// Numero maximo de campos de dano aceitos para uma unica arma. Acima disso a
/// travessia provavelmente encontrou lixo e a operacao e abortada.
pub const MAX_DAMAGE_TARGETS: usize = 64;

/// Intervalo entre reescritas dos valores congelados (Infinite Magazine/Damage).
pub const FREEZE_INTERVAL_MS: u64 = 35;

/// Espera antes de tentar reconectar quando o processo some durante um freeze.
pub const RECONNECT_INTERVAL_MS: u64 = 500;

/// Tempo limite para uma chamada nativa remota concluir.
pub const REMOTE_CALL_TIMEOUT_MS: u32 = 15_000;

/// Teto de itens/contadores lidos do save antes de considerarmos o dado corrompido.
pub const MAX_SAVE_COLLECTION: i32 = 10_000;
