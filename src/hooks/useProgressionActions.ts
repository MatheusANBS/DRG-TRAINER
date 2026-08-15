/**
 * Ações permanentes de progressão (SPEC-012 / SPEC-015).
 *
 * Todas são `persistent`: gravam no save em disco e exigem confirmação. O hook
 * mantém uma única confirmação ativa por vez, para que dois diálogos não possam
 * disparar duas mutações concorrentes.
 */

import { useCallback, useState } from "react";

import type { ActionFeed } from "@/hooks/useActionFeed";
import { type AsyncAction, useAsyncAction } from "@/hooks/useAsyncAction";
import { trainerApi } from "@/services/trainer-api";
import type { TrainerState } from "@/state/trainer-state";
import { blockedReason } from "@/state/trainer-state";
import type { BackupSet } from "@/types/trainer";

export type ProgressionActionId =
  | "maxLevel"
  | "promote"
  | "perks"
  | "weapons"
  | "gear"
  | "schematics";

function backupDetail(result: { backup: BackupSet }): string {
  return `Backup: ${result.backup.path}`;
}

export function useProgressionActions(state: TrainerState, feed: ActionFeed) {
  const [confirming, setConfirming] = useState<ProgressionActionId | null>(null);

  const maxLevel = useAsyncAction({
    perform: trainerApi.maxClassLevel,
    blockedReason: blockedReason(state, "classLevel"),
    successMessage: (result) =>
      `${result.classesUpdated} classes set to level 25. Promotions preserved.`,
    successDetail: backupDetail,
    feed,
  });

  const promote = useAsyncAction({
    perform: trainerApi.promoteAllClasses,
    blockedReason: blockedReason(state, "promotion"),
    successMessage: (result) =>
      `${result.classesPromoted} classes promoted once through the game's own progression.`,
    successDetail: backupDetail,
    feed,
  });

  const perks = useAsyncAction({
    perform: trainerApi.unlockAllPerks,
    blockedReason: blockedReason(state, "perkUnlock"),
    successMessage: (result) => `Perks acquired: ${result.itemsBefore} → ${result.itemsAfter}.`,
    successDetail: backupDetail,
    feed,
  });

  const weapons = useAsyncAction({
    perform: trainerApi.unlockAllWeapons,
    blockedReason: blockedReason(state, "weaponUnlock"),
    successMessage: (result) => `${result.weaponCount} weapons processed.`,
    successDetail: backupDetail,
    feed,
  });

  const gear = useAsyncAction({
    perform: trainerApi.unlockAllGearModifications,
    blockedReason: blockedReason(state, "gearUnlock"),
    successMessage: (result) =>
      `Gear modifications acquired: ${result.itemsBefore} → ${result.itemsAfter}.`,
    successDetail: backupDetail,
    feed,
  });

  const schematics = useAsyncAction({
    perform: trainerApi.unlockAllOverclocksAndCosmetics,
    blockedReason: blockedReason(state, "schematicUnlock"),
    successMessage: (result) =>
      `Rewards applied: ${result.overclocksProcessed} overclocks, ${result.cosmeticsProcessed} cosmetics. ${result.forgedAfter} schematics forged.`,
    successDetail: backupDetail,
    feed,
  });

  const actions: Record<ProgressionActionId, AsyncAction> = {
    maxLevel,
    promote,
    perks,
    weapons,
    gear,
    schematics,
  };

  const confirm = useCallback(() => {
    const pending = confirming;
    setConfirming(null);
    if (pending) actions[pending].run();
    // `actions` é recriado a cada render; ler o pendente e disparar na hora
    // evita capturar uma versão antiga.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [confirming, maxLevel, promote, perks, weapons, gear, schematics]);

  const busy = Object.values(actions).some((action) => action.pending);

  return { ...actions, actions, confirming, setConfirming, confirm, busy };
}

export type ProgressionActions = ReturnType<typeof useProgressionActions>;
