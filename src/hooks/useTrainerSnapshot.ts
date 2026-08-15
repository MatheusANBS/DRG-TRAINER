/**
 * Leitura periódica dos dados do jogo (SPEC-013).
 *
 * Um ciclo lê tudo em paralelo e cada resultado é tratado isoladamente: uma
 * arma não equipada não pode apagar o saldo de créditos da tela.
 *
 * O polling pausa enquanto uma mutação está em andamento — ler o save no meio
 * de um unlock competiria com a própria operação.
 */

import { useCallback, useState } from "react";

import { useTrainerPolling } from "@/hooks/useTrainerPolling";
import { trainerApi } from "@/services/trainer-api";
import type { TrainerState } from "@/state/trainer-state";
import { isOperable } from "@/state/trainer-state";
import type {
  ClipStatus,
  CreditSnapshot,
  DamageStatus,
  ResourceSnapshot,
} from "@/types/trainer";

const SNAPSHOT_INTERVAL_MS = 500;

export type TrainerSnapshot = {
  credits: CreditSnapshot | null;
  resources: ResourceSnapshot[];
  clip: ClipStatus | null;
  damage: DamageStatus | null;
  /** Força uma leitura imediata (usado após uma mutação). */
  refresh: () => Promise<void>;
  setResources: (resources: ResourceSnapshot[]) => void;
  setCredits: (credits: CreditSnapshot | null) => void;
  setClip: (clip: ClipStatus | null) => void;
  setDamage: (damage: DamageStatus | null) => void;
};

export function useTrainerSnapshot(state: TrainerState, paused: boolean): TrainerSnapshot {
  const [credits, setCredits] = useState<CreditSnapshot | null>(null);
  const [resources, setResources] = useState<ResourceSnapshot[]>([]);
  const [clip, setClip] = useState<ClipStatus | null>(null);
  const [damage, setDamage] = useState<DamageStatus | null>(null);

  const operable = isOperable(state);
  const { capabilities } = state;

  const read = useCallback(async () => {
    if (!operable) {
      // Sem build verificada não há dado confiável: limpar é mais honesto do
      // que manter o último valor lido de outra sessão.
      setCredits(null);
      setResources([]);
      setClip(null);
      setDamage(null);
      return false;
    }

    const [creditsResult, resourcesResult, clipResult, damageResult] = await Promise.allSettled([
      capabilities.credits ? trainerApi.readCredits() : Promise.resolve(null),
      capabilities.resources ? trainerApi.readResources() : Promise.resolve([]),
      capabilities.infiniteMagazine ? trainerApi.readClipStatus() : Promise.resolve(null),
      capabilities.weaponDamage ? trainerApi.readDamageStatus() : Promise.resolve(null),
    ]);

    if (creditsResult.status === "fulfilled") setCredits(creditsResult.value);
    if (resourcesResult.status === "fulfilled") setResources(resourcesResult.value);
    // Arma e dano falham com frequência normal (fora de missão, item sem dano);
    // manter o último estado evita piscar a UI a cada 500 ms.
    if (clipResult.status === "fulfilled") setClip(clipResult.value);
    if (damageResult.status === "fulfilled") setDamage(damageResult.value);

    return creditsResult.status === "fulfilled";
  }, [
    operable,
    capabilities.credits,
    capabilities.resources,
    capabilities.infiniteMagazine,
    capabilities.weaponDamage,
  ]);

  useTrainerPolling(read, { intervalMs: SNAPSHOT_INTERVAL_MS, enabled: !paused });

  const refresh = useCallback(async () => {
    await read();
  }, [read]);

  return {
    credits,
    resources,
    clip,
    damage,
    refresh,
    setResources,
    setCredits,
    setClip,
    setDamage,
  };
}
