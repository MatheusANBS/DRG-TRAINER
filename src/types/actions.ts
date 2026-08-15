/**
 * Semântica de ação (SPEC-015).
 *
 * Cada controle do trainer declara o impacto do que faz. A semântica governa
 * rótulo, badge, confirmação e tratamento visual — de modo que um toggle de
 * runtime nunca pareça tão grave quanto uma mutação permanente de save, e
 * vice-versa.
 */

export type ActionSemantics =
  /** Efeito só enquanto o trainer está ativo; reversível desligando. */
  | "runtime"
  /** Escreve em memória, mas não grava no save. Reversível editando de novo. */
  | "standard"
  /** Grava no save em disco. Irreversível sem restaurar o backup. */
  | "persistent";

export type ActionSemanticsMeta = {
  /** Badge exibido antes do clique. `null` quando não acrescenta informação. */
  badge: string | null;
  /** Texto acessível que descreve o impacto. */
  impact: string;
  /** Se a ação exige confirmação explícita antes de executar. */
  requiresConfirmation: boolean;
};

export const ACTION_SEMANTICS: Record<ActionSemantics, ActionSemanticsMeta> = {
  runtime: {
    badge: null,
    impact: "Runtime only — turning it off restores the game's own value.",
    requiresConfirmation: false,
  },
  standard: {
    badge: null,
    impact: "Writes to the loaded save in memory. Not saved to disk by the trainer.",
    requiresConfirmation: true,
  },
  persistent: {
    badge: "PERMANENT",
    impact: "Writes to the save file on disk. A backup is created first.",
    requiresConfirmation: true,
  },
};

/** Fases observáveis de uma ação assíncrona (SPEC-019). */
export type ActionPhase = "idle" | "pending" | "success" | "error" | "blocked";
