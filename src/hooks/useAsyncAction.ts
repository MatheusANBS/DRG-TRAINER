/**
 * Ciclo de vida de uma ação assíncrona (SPEC-019).
 *
 * Garante três coisas que a UI antes repetia em cada handler:
 * fase observável (`idle`/`pending`/`success`/`error`/`blocked`), impossibilidade
 * de disparo duplicado, e publicação do resultado no feed.
 */

import { useCallback, useEffect, useRef, useState } from "react";

import type { ActionFeed } from "@/hooks/useActionFeed";
import { toTrainerError } from "@/lib/errors";
import type { ActionPhase } from "@/types/actions";
import type { TrainerError } from "@/types/trainer";

export type AsyncAction = {
  phase: ActionPhase;
  pending: boolean;
  /** Motivo do bloqueio, ou `null` quando a ação está liberada. */
  blockedReason: string | null;
  /** `true` quando o controle deve ficar desabilitado. */
  disabled: boolean;
  error: TrainerError | null;
  run: () => void;
};

type Options<T> = {
  perform: () => Promise<T>;
  /** Motivo de bloqueio vindo do estado central; `null` libera a ação. */
  blockedReason: string | null;
  /** Mensagem publicada no feed em caso de sucesso. */
  successMessage: (result: T) => string;
  /** Detalhe secundário opcional (ex.: caminho do backup). */
  successDetail?: (result: T) => string | undefined;
  feed: ActionFeed;
  onSuccess?: (result: T) => void;
};

export function useAsyncAction<T>({
  perform,
  blockedReason,
  successMessage,
  successDetail,
  feed,
  onSuccess,
}: Options<T>): AsyncAction {
  const [phase, setPhase] = useState<ActionPhase>("idle");
  const [error, setError] = useState<TrainerError | null>(null);
  const inFlight = useRef(false);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  // Refs mantêm `run` estável sem congelar closures antigas. A atualização
  // acontece após o commit; `run` só é chamado a partir de eventos, portanto
  // sempre lê a versão mais recente.
  const latest = useRef({ perform, successMessage, successDetail, feed, onSuccess });
  useEffect(() => {
    latest.current = { perform, successMessage, successDetail, feed, onSuccess };
  });

  const run = useCallback(() => {
    // Duplo clique, Enter repetido e hotkey simultânea caem todos aqui.
    if (inFlight.current) return;
    inFlight.current = true;
    setPhase("pending");
    setError(null);

    void (async () => {
      try {
        const result = await latest.current.perform();
        if (!mounted.current) return;
        setPhase("success");
        latest.current.feed.pushSuccess(
          latest.current.successMessage(result),
          latest.current.successDetail?.(result),
        );
        latest.current.onSuccess?.(result);
      } catch (raw) {
        const trainerError = toTrainerError(raw);
        if (!mounted.current) return;
        setPhase("error");
        setError(trainerError);
        latest.current.feed.pushError(trainerError);
      } finally {
        inFlight.current = false;
      }
    })();
  }, []);

  const effectivePhase: ActionPhase = blockedReason ? "blocked" : phase;

  return {
    phase: effectivePhase,
    pending: phase === "pending",
    blockedReason,
    disabled: blockedReason !== null || phase === "pending",
    error,
    run,
  };
}
