/**
 * Modelo de estado do trainer (SPEC-013).
 *
 * Em vez de booleanos espalhados (`connected`, `buildValid`, `waiting`), há um
 * único estado discriminado derivado do status do backend. Toda desabilitação
 * de ação sai daqui — nenhuma tela decide sozinha se pode operar.
 */

import type {
  Capabilities,
  CapabilityId,
  TrainerError,
  TrainerStatus,
} from "@/types/trainer";
import { isWaiting } from "@/lib/errors";

export type TrainerConnection =
  /** Primeiro poll ainda não retornou. */
  | { kind: "connecting" }
  /** O jogo não está em execução. */
  | { kind: "detached"; expected: string[] }
  /** Processo encontrado, build sem perfil correspondente. */
  | { kind: "unsupported"; pid: number; executable: string; sha256: string }
  /** Perfil existe mas ainda não foi promovido a suportado. */
  | { kind: "unverified"; pid: number; executable: string; profileId: string; sha256: string }
  /** Tudo pronto: é seguro operar. */
  | {
      kind: "attached";
      pid: number;
      executable: string;
      profileId: string;
      displayName: string;
      sha256: string;
    }
  /** Falha real de leitura de status. */
  | { kind: "error"; error: TrainerError };

export const NO_CAPABILITIES: Capabilities = {
  credits: false,
  resources: false,
  weaponUnlock: false,
  perkUnlock: false,
  gearUnlock: false,
  schematicUnlock: false,
  classLevel: false,
  promotion: false,
  infiniteMagazine: false,
  weaponDamage: false,
};

export type TrainerState = {
  connection: TrainerConnection;
  capabilities: Capabilities;
};

export const INITIAL_STATE: TrainerState = {
  connection: { kind: "connecting" },
  capabilities: NO_CAPABILITIES,
};

/** Converte o status bruto do backend no estado observável da UI. */
export function deriveState(status: TrainerStatus): TrainerState {
  const capabilities = status.capabilities;

  if (status.process.kind === "detached") {
    return {
      connection: { kind: "detached", expected: status.expectedExecutables },
      capabilities: NO_CAPABILITIES,
    };
  }

  const { pid, executable } = status.process;

  switch (status.build.kind) {
    case "verified":
      return {
        connection: {
          kind: "attached",
          pid,
          executable,
          profileId: status.build.profileId,
          displayName: status.build.displayName,
          sha256: status.build.sha256,
        },
        capabilities,
      };
    case "unverified":
      return {
        connection: {
          kind: "unverified",
          pid,
          executable,
          profileId: status.build.profileId,
          sha256: status.build.sha256,
        },
        capabilities: NO_CAPABILITIES,
      };
    case "unsupported":
      return {
        connection: { kind: "unsupported", pid, executable, sha256: status.build.sha256 },
        capabilities: NO_CAPABILITIES,
      };
    case "unknown":
      // Processo achado, build ainda não identificada: tratado como não operável.
      return {
        connection: { kind: "unsupported", pid, executable, sha256: "" },
        capabilities: NO_CAPABILITIES,
      };
  }
}

/** Estado a partir de uma falha de leitura de status. */
export function errorState(error: TrainerError, expected: string[]): TrainerState {
  if (isWaiting(error)) {
    return { connection: { kind: "detached", expected }, capabilities: NO_CAPABILITIES };
  }
  return { connection: { kind: "error", error }, capabilities: NO_CAPABILITIES };
}

/** É seguro executar ações dependentes de memória? */
export function isOperable(state: TrainerState): boolean {
  return state.connection.kind === "attached";
}

/** PID atual, quando houver processo. */
export function currentPid(connection: TrainerConnection): number | null {
  return "pid" in connection ? connection.pid : null;
}

/**
 * Motivo pelo qual uma ação está bloqueada, ou `null` se liberada.
 *
 * Uma única função responde por todos os botões: ninguém recalcula `disabled`.
 */
export function blockedReason(
  state: TrainerState,
  capability: CapabilityId,
): string | null {
  switch (state.connection.kind) {
    case "connecting":
      return "Connecting to the game.";
    case "detached":
      return "Deep Rock Galactic is not running.";
    case "unsupported":
      return "Unsupported game build — memory-dependent actions are disabled.";
    case "unverified":
      return "This build profile has not been verified yet.";
    case "error":
      return state.connection.error.message;
    case "attached":
      return state.capabilities[capability]
        ? null
        : "Not verified on the current build profile.";
  }
}

/** Rótulo curto do estado operacional, para o status bar (SPEC-016). */
export function connectionLabel(connection: TrainerConnection): string {
  switch (connection.kind) {
    case "connecting":
      return "CONNECTING";
    case "detached":
      return "GAME NOT RUNNING";
    case "unsupported":
    case "unverified":
    case "attached":
      return "GAME ATTACHED";
    case "error":
      return "TRAINER ERROR";
  }
}

/** Rótulo do estado da build, para o status bar (SPEC-016). */
export function buildLabel(connection: TrainerConnection): string {
  switch (connection.kind) {
    case "attached":
      return "BUILD VERIFIED";
    case "unsupported":
      return "BUILD UNSUPPORTED";
    case "unverified":
      return "BUILD UNVERIFIED";
    case "detached":
    case "connecting":
      return "BUILD UNKNOWN";
    case "error":
      return "BUILD UNKNOWN";
  }
}
