/**
 * Única fronteira com o backend (SPEC-012).
 *
 * Nenhum componente ou hook chama `invoke` diretamente. Todo comando aparece
 * aqui com assinatura tipada, e toda rejeição sai normalizada como
 * `TrainerError` — de modo que nenhuma tela precise interpretar strings.
 */

import { invoke } from "@tauri-apps/api/core";

import { TrainerFailure, toTrainerError } from "@/lib/errors";
import { mockInvoke, shouldUseMockBackend } from "@/services/mock-backend";
import type {
  ClipStatus,
  CreditSnapshot,
  DamageStatus,
  HotkeyResult,
  MaxClassLevelResult,
  PermanentUnlockResult,
  PromoteAllResult,
  ResourceSnapshot,
  ResourceWriteResult,
  SchematicUnlockResult,
  TrainerStatus,
  UnlockAllResult,
  WriteResult,
} from "@/types/trainer";

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    if (shouldUseMockBackend()) return await mockInvoke<T>(command, args);
    return await invoke<T>(command, args);
  } catch (error) {
    // Sempre um `Error` real, para não perder stack trace, mas carregando o
    // código estável do backend.
    throw new TrainerFailure(toTrainerError(error));
  }
}

export const trainerApi = {
  /** Estado operacional: processo, build e capacidades (SPEC-016). */
  status: () => call<TrainerStatus>("get_trainer_status"),

  readCredits: () => call<CreditSnapshot>("read_credits"),
  setCredits: (value: number) => call<WriteResult>("set_credits", { value }),

  readResources: () => call<ResourceSnapshot[]>("get_resources"),
  addResource: (resourceId: string, amount: number) =>
    call<ResourceWriteResult>("add_resource", { resourceId, amount }),
  addAllResources: (amount: number) =>
    call<ResourceWriteResult>("add_all_resources", { amount }),

  readClipStatus: () => call<ClipStatus>("get_clip_status"),
  setInfiniteMagazine: (enabled: boolean) =>
    call<ClipStatus>("set_infinite_magazine", { enabled }),
  readDamageStatus: () => call<DamageStatus>("get_damage_status"),
  setWeaponDamage: (enabled: boolean) => call<DamageStatus>("set_weapon_damage", { enabled }),

  unlockAllWeapons: () => call<UnlockAllResult>("unlock_all_weapons"),
  unlockAllPerks: () => call<PermanentUnlockResult>("unlock_all_perks"),
  unlockAllGearModifications: () =>
    call<PermanentUnlockResult>("unlock_all_gear_modifications"),
  unlockAllOverclocksAndCosmetics: () =>
    call<SchematicUnlockResult>("unlock_all_overclocks_and_cosmetics"),

  maxClassLevel: () => call<MaxClassLevelResult>("max_class_level"),
  promoteAllClasses: () => call<PromoteAllResult>("promote_all_classes"),

  setClipHotkey: (hotkey: string | null) => call<HotkeyResult>("set_clip_hotkey", { hotkey }),
  setDamageHotkey: (hotkey: string | null) =>
    call<HotkeyResult>("set_damage_hotkey", { hotkey }),
} as const;

export type TrainerApi = typeof trainerApi;
