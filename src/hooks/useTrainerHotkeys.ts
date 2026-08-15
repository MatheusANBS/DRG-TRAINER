/**
 * Atalhos globais dos toggles de runtime (SPEC-012).
 *
 * O registro acontece no backend; aqui ficam apenas a persistência local e o
 * fluxo de captura. Um atalho recusado pelo sistema não sobrescreve o anterior
 * salvo — o backend só troca depois de confirmar o registro.
 */

import { useCallback, useEffect, useRef, useState } from "react";

import type { ActionFeed } from "@/hooks/useActionFeed";
import { toTrainerError } from "@/lib/errors";
import { trainerApi } from "@/services/trainer-api";

const CLIP_STORAGE_KEY = "drg.clipHotkey";
const DAMAGE_STORAGE_KEY = "drg.damageHotkey";
export const DEFAULT_CLIP_HOTKEY = "F8";
export const DEFAULT_DAMAGE_HOTKEY = "F9";

function readStored(key: string, fallback: string): string {
  try {
    return localStorage.getItem(key) ?? fallback;
  } catch {
    // localStorage pode estar indisponível em ambiente de teste restrito.
    return fallback;
  }
}

function persist(key: string, value: string): void {
  try {
    localStorage.setItem(key, value);
  } catch {
    // Falhar em persistir não deve impedir o atalho de funcionar na sessão.
  }
}

export function useTrainerHotkeys(feed: ActionFeed) {
  const [clipHotkey, setClipHotkey] = useState(() =>
    readStored(CLIP_STORAGE_KEY, DEFAULT_CLIP_HOTKEY),
  );
  const [damageHotkey, setDamageHotkey] = useState(() =>
    readStored(DAMAGE_STORAGE_KEY, DEFAULT_DAMAGE_HOTKEY),
  );
  const [clipBusy, setClipBusy] = useState(false);
  const [damageBusy, setDamageBusy] = useState(false);
  const registered = useRef(false);

  // Registra os atalhos salvos uma única vez, na montagem.
  useEffect(() => {
    if (registered.current) return;
    registered.current = true;

    void (async () => {
      const results = await Promise.allSettled([
        trainerApi.setClipHotkey(clipHotkey),
        trainerApi.setDamageHotkey(damageHotkey),
      ]);
      for (const result of results) {
        if (result.status === "rejected") feed.pushError(toTrainerError(result.reason));
      }
    })();
    // Executa apenas na montagem: os valores iniciais vêm do storage.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const saveClipHotkey = useCallback(
    async (hotkey: string) => {
      setClipBusy(true);
      try {
        await trainerApi.setClipHotkey(hotkey);
        setClipHotkey(hotkey);
        persist(CLIP_STORAGE_KEY, hotkey);
      } catch (error) {
        feed.pushError(toTrainerError(error));
      } finally {
        setClipBusy(false);
      }
    },
    [feed],
  );

  const saveDamageHotkey = useCallback(
    async (hotkey: string) => {
      setDamageBusy(true);
      try {
        await trainerApi.setDamageHotkey(hotkey);
        setDamageHotkey(hotkey);
        persist(DAMAGE_STORAGE_KEY, hotkey);
      } catch (error) {
        feed.pushError(toTrainerError(error));
      } finally {
        setDamageBusy(false);
      }
    },
    [feed],
  );

  return {
    clipHotkey,
    damageHotkey,
    clipBusy,
    damageBusy,
    saveClipHotkey,
    saveDamageHotkey,
  };
}

export type TrainerHotkeys = ReturnType<typeof useTrainerHotkeys>;
