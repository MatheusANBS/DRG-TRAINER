/**
 * Tradução de erros do backend (SPEC-019).
 *
 * O backend emite `{ code, message, recoverable }`. O código é o contrato
 * estável; a mensagem é o detalhe técnico. Aqui cada código conhecido vira
 * um texto claro para o usuário, e a mensagem original fica como detalhe.
 */

import type { TrainerError, TrainerErrorCode } from "@/types/trainer";

const MESSAGES: Record<TrainerErrorCode, string> = {
  PROCESS_NOT_FOUND: "Waiting for Deep Rock Galactic to start.",
  MODULE_NOT_FOUND: "The game is starting up. Waiting for its main module.",
  PROCESS_OPEN_FAILED:
    "Could not open the game process. Try running the trainer as administrator.",
  BUILD_UNSUPPORTED:
    "This game build is not supported yet. Memory-dependent actions are disabled.",
  BUILD_HASH_FAILED: "Could not read the game executable to identify the build.",
  CAPABILITY_UNAVAILABLE: "This action has not been verified on the current build.",
  MEMORY_READ_FAILED: "A memory read failed. The game may have changed state.",
  MEMORY_WRITE_FAILED: "A memory write failed. Nothing was changed.",
  WRITE_VERIFICATION_FAILED:
    "The write could not be confirmed. Check the value before trying again.",
  READ_ONLY_HANDLE: "The trainer is attached read-only and cannot write.",
  SIGNATURE_MISMATCH:
    "A native routine does not match the verified build. The action was blocked.",
  NATIVE_CALL_FAILED: "The game routine did not complete.",
  NATIVE_CALL_TIMEOUT: "The game routine timed out. Try again in a moment.",
  WORLD_NOT_READY: "Enter the Space Rig and wait for your character to load.",
  OBJECT_NOT_FOUND: "The game object is not loaded yet.",
  SAVE_NOT_FOUND: "No active save was found. Load a character first.",
  SAVE_STATE_CHANGED: "The active save changed during the operation. Nothing was persisted.",
  BACKUP_FAILED: "The save backup could not be created, so nothing was changed.",
  INVALID_ARGUMENT: "The value is out of the accepted range.",
  INVALID_GAME_STATE: "The data read from the game is inconsistent. The action was blocked.",
  STATE_UNAVAILABLE: "Internal trainer state is unavailable. Restart the trainer.",
  HOTKEY_INVALID: "That key combination cannot be used as a global hotkey.",
  HOTKEY_REGISTRATION_FAILED:
    "Windows refused the hotkey. It may already be taken by another app.",
  TASK_FAILED: "The operation did not finish. Try again.",
};

/** Erro genérico usado quando o backend devolve algo fora do contrato. */
export const UNKNOWN_ERROR: TrainerError = {
  code: "TASK_FAILED",
  message: "Unexpected error.",
  recoverable: true,
};

function isTrainerErrorCode(value: unknown): value is TrainerErrorCode {
  return typeof value === "string" && value in MESSAGES;
}

/**
 * Erro lançável que preserva o contrato do backend.
 *
 * É um `Error` de verdade — logs e ferramentas mantêm o stack trace — mas
 * carrega `code` e `recoverable`, então `toTrainerError` o reconhece pelo
 * mesmo caminho de um erro serializado vindo do IPC.
 */
export class TrainerFailure extends Error implements TrainerError {
  readonly code: TrainerErrorCode;
  readonly recoverable: boolean;

  constructor(error: TrainerError) {
    super(error.message);
    this.name = "TrainerFailure";
    this.code = error.code;
    this.recoverable = error.recoverable;
  }
}

/**
 * Normaliza qualquer rejeição em um `TrainerError`.
 *
 * O backend serializa erros tipados, mas a fronteira Tauri também pode
 * rejeitar com string (falha de IPC) — e o preview de browser lança `Error`.
 */
export function toTrainerError(value: unknown): TrainerError {
  if (typeof value === "object" && value !== null && "code" in value) {
    const candidate = value as { code: unknown; message?: unknown; recoverable?: unknown };
    if (isTrainerErrorCode(candidate.code)) {
      return {
        code: candidate.code,
        message:
          typeof candidate.message === "string" && candidate.message.length > 0
            ? candidate.message
            : MESSAGES[candidate.code],
        recoverable: candidate.recoverable === true,
      };
    }
  }

  if (value instanceof Error) {
    return { ...UNKNOWN_ERROR, message: value.message };
  }
  if (typeof value === "string" && value.length > 0) {
    return { ...UNKNOWN_ERROR, message: value };
  }
  return UNKNOWN_ERROR;
}

/** Texto exibido ao usuário para um erro conhecido. */
export function errorMessage(error: TrainerError): string {
  return MESSAGES[error.code] ?? error.message;
}

/** Detalhe técnico, exibido apenas quando acrescenta informação. */
export function errorDetail(error: TrainerError): string | null {
  const friendly = errorMessage(error);
  const detail = error.message.trim();
  return detail.length > 0 && detail !== friendly ? detail : null;
}

/**
 * Estados de espera não são falhas: o jogo simplesmente ainda não está lá.
 * A UI mostra "waiting", nunca um erro vermelho.
 */
export function isWaiting(error: TrainerError): boolean {
  return error.code === "PROCESS_NOT_FOUND" || error.code === "MODULE_NOT_FOUND";
}
