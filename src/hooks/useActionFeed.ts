/**
 * Fila de feedback de ações (SPEC-019).
 *
 * Um único lugar publica sucesso e erro, em vez de um par de estados por ação.
 * Entradas somem sozinhas; o estado persistente importante (build, conexão,
 * toggles) vive no status bar e não depende daqui.
 */

import { useCallback, useEffect, useRef, useState } from "react";

import { errorDetail, errorMessage } from "@/lib/errors";
import type { TrainerError } from "@/types/trainer";

export type FeedTone = "success" | "error";

export type FeedEntry = {
  id: number;
  tone: FeedTone;
  message: string;
  detail?: string;
};

export type ActionFeed = {
  entries: FeedEntry[];
  pushSuccess: (message: string, detail?: string) => void;
  pushError: (error: TrainerError) => void;
  dismiss: (id: number) => void;
  clear: () => void;
};

/** Quantas entradas ficam visíveis ao mesmo tempo. */
const MAX_ENTRIES = 3;
const SUCCESS_TIMEOUT_MS = 6_000;
const ERROR_TIMEOUT_MS = 9_000;

export function useActionFeed(): ActionFeed {
  const [entries, setEntries] = useState<FeedEntry[]>([]);
  const nextId = useRef(1);
  const timers = useRef(new Map<number, number>());

  const dismiss = useCallback((id: number) => {
    setEntries((current) => current.filter((entry) => entry.id !== id));
    const timer = timers.current.get(id);
    if (timer !== undefined) {
      window.clearTimeout(timer);
      timers.current.delete(id);
    }
  }, []);

  const push = useCallback(
    (tone: FeedTone, message: string, detail?: string) => {
      const id = nextId.current++;
      setEntries((current) => [...current.slice(-(MAX_ENTRIES - 1)), { id, tone, message, detail }]);
      const timeout = tone === "error" ? ERROR_TIMEOUT_MS : SUCCESS_TIMEOUT_MS;
      timers.current.set(
        id,
        window.setTimeout(() => dismiss(id), timeout),
      );
    },
    [dismiss],
  );

  const pushSuccess = useCallback(
    (message: string, detail?: string) => push("success", message, detail),
    [push],
  );

  const pushError = useCallback(
    (error: TrainerError) => push("error", errorMessage(error), errorDetail(error) ?? undefined),
    [push],
  );

  const clear = useCallback(() => {
    timers.current.forEach((timer) => window.clearTimeout(timer));
    timers.current.clear();
    setEntries([]);
  }, []);

  // Nenhum timer sobrevive ao unmount.
  useEffect(() => {
    const pending = timers.current;
    return () => {
      pending.forEach((timer) => window.clearTimeout(timer));
      pending.clear();
    };
  }, []);

  return { entries, pushSuccess, pushError, dismiss, clear };
}
