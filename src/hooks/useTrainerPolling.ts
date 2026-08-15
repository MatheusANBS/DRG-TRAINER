/**
 * Primitiva de polling (SPEC-013).
 *
 * `setInterval` permitia dois polls concorrentes quando um ciclo demorava mais
 * que o intervalo. Aqui o próximo tick só é agendado depois que o anterior
 * termina, o que elimina overlap por construção.
 *
 * Além disso: cancelamento no unmount, backoff leve em erro, e intervalo maior
 * quando a janela está oculta — um trainer minimizado não precisa ler memória
 * duas vezes por segundo.
 */

import { useEffect, useRef } from "react";

export type PollingOptions = {
  /** Intervalo normal entre ciclos bem-sucedidos. */
  intervalMs: number;
  /** Intervalo após uma falha. Padrão: 4x o normal. */
  errorIntervalMs?: number;
  /** Intervalo com a janela oculta. Padrão: 8x o normal. */
  hiddenIntervalMs?: number;
  /** Desliga o polling sem desmontar o componente. */
  enabled?: boolean;
};

/**
 * `run` deve devolver `true` em sucesso e `false` em falha esperada; exceções
 * também contam como falha e acionam o backoff.
 */
export function useTrainerPolling(run: () => Promise<boolean>, options: PollingOptions) {
  const {
    intervalMs,
    errorIntervalMs = intervalMs * 4,
    hiddenIntervalMs = intervalMs * 8,
    enabled = true,
  } = options;

  // A referência é atualizada depois do commit: o tick lê `runRef.current` na
  // hora em que dispara, então sempre vê a versão mais recente.
  const runRef = useRef(run);
  useEffect(() => {
    runRef.current = run;
  });

  useEffect(() => {
    if (!enabled) return;

    let cancelled = false;
    let timer: number | undefined;

    const schedule = (delay: number) => {
      if (cancelled) return;
      timer = window.setTimeout(() => void tick(), delay);
    };

    const tick = async () => {
      if (cancelled) return;
      let ok = false;
      try {
        ok = await runRef.current();
      } catch {
        ok = false;
      }
      if (cancelled) return;
      if (document.visibilityState === "hidden") schedule(hiddenIntervalMs);
      else schedule(ok ? intervalMs : errorIntervalMs);
    };

    // Ao voltar para a janela, atualiza imediatamente em vez de esperar o
    // intervalo longo do modo oculto terminar.
    const onVisibilityChange = () => {
      if (document.visibilityState !== "visible" || cancelled) return;
      if (timer !== undefined) window.clearTimeout(timer);
      void tick();
    };

    void tick();
    document.addEventListener("visibilitychange", onVisibilityChange);

    return () => {
      cancelled = true;
      if (timer !== undefined) window.clearTimeout(timer);
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, [enabled, intervalMs, errorIntervalMs, hiddenIntervalMs]);
}
