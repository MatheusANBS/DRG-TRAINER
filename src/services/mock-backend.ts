/**
 * Backend simulado para o preview no browser (`npm run dev` fora do Tauri).
 *
 * Existe para permitir trabalho visual sem o jogo aberto. Não é usado em
 * produção nem nos testes: os testes injetam seus próprios stubs pela camada
 * `trainer-api`, para não depender deste cenário.
 */

import type {
  ClipStatus,
  DamageStatus,
  ResourceSnapshot,
  TrainerStatus,
} from "@/types/trainer";

const PID = 26248;
const BACKUP_ROOT = "F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups";

function backup(operation: string) {
  return { path: `${BACKUP_ROOT}\\${operation}-preview`, files: 2, totalBackups: 7 };
}

const state = {
  credits: 49_995,
  infiniteMagazine: false,
  weaponDamage: false,
  weaponsUnlocked: false,
  perksUnlocked: false,
  gearUnlocked: false,
  schematicsUnlocked: false,
  resources: (
    [
      ["bismor", "Bismor", "Minerals", 50],
      ["croppa", "Croppa", "Minerals", 120],
      ["enor_pearl", "Enor Pearl", "Minerals", 30],
      ["jadiz", "Jadiz", "Minerals", 30],
      ["magnite", "Magnite", "Minerals", 80],
      ["umanite", "Umanite", "Minerals", 60],
      ["phazyonite", "Phazyonite", "Minerals", 12],
      ["barley_bulb", "Barley Bulb", "Brewing", 4],
      ["malt_star", "Malt Star", "Brewing", 7],
      ["starch_nut", "Starch Nut", "Brewing", 5],
      ["yeast_cone", "Yeast Cone", "Brewing", 6],
      ["blank_matrix_core", "Blank Matrix Core", "Special", 15],
      ["error_cube", "Error Cube", "Special", 1],
      ["data_cell", "Data Cell", "Special", 0],
    ] as const
  ).map(([id, name, category, amount]) => ({
    id,
    name,
    category,
    amount,
  })) as ResourceSnapshot[],
};

const STATUS: TrainerStatus = {
  process: { kind: "attached", pid: PID, executable: "FSD-Win64-Shipping.exe" },
  build: {
    kind: "verified",
    profileId: "fsd-8e22e371",
    displayName: "Deep Rock Galactic (shipping 8e22e371)",
    sha256: "8e22e3710c607e811e0319a4378d2b7ab2e46c6b46406629a6657b1662c77a41",
    status: "supported",
  },
  capabilities: {
    credits: true,
    resources: true,
    weaponUnlock: true,
    perkUnlock: true,
    gearUnlock: true,
    schematicUnlock: true,
    classLevel: true,
    promotion: true,
    infiniteMagazine: true,
    weaponDamage: true,
  },
  expectedExecutables: ["FSD-Win64-Shipping.exe"],
};

function clipStatus(): ClipStatus {
  return {
    enabled: state.infiniteMagazine,
    available: true,
    pid: PID,
    weaponName: "WPN_AssaultRifle_C",
    clipCount: 30,
    clipSize: 30,
    address: "0x2C5B823C780",
    message: state.infiniteMagazine ? "Infinite Magazine ativo." : "Arma equipada encontrada.",
  };
}

function damageStatus(): DamageStatus {
  return {
    enabled: state.weaponDamage,
    available: true,
    pid: PID,
    weaponName: "WPN_AssaultRifle_C",
    value: state.weaponDamage ? 9_999 : 16,
    targetCount: 2,
    addresses: ["0x2C5B823C898", "0x2C5B823C924"],
    message: state.weaponDamage
      ? "Weapon Damage ativo em 2 campos."
      : "2 campos de dano encontrados.",
  };
}

/** Só no `npm run dev` de browser: dentro do Tauri o backend real responde. */
export function shouldUseMockBackend(): boolean {
  return import.meta.env.DEV && !("__TAURI_INTERNALS__" in window);
}

/** Lê um argumento textual sem depender da stringificação padrão de `unknown`. */
function text(value: unknown): string {
  return typeof value === "string" ? value : "";
}

export async function mockInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  // Resolve em microtask, como o IPC real: o preview não deve ser síncrono
  // onde a aplicação de verdade é assíncrona.
  await Promise.resolve();

  switch (command) {
    case "get_trainer_status":
      return STATUS as T;

    case "read_credits":
      return {
        credits: state.credits,
        pid: PID,
        address: "0x2C5A759EC10",
        moduleBase: "0x7FF6A2C30000",
        offset: "0x590",
      } as T;

    case "set_credits": {
      const previous = state.credits;
      state.credits = Number(args?.value ?? state.credits);
      return {
        previous,
        current: state.credits,
        pid: PID,
        address: "0x2C5A759EC10",
      } as T;
    }

    case "get_resources":
      return state.resources as T;

    case "add_resource":
    case "add_all_resources": {
      const amount = Number(args?.amount ?? 0);
      const resourceId = text(args?.resourceId);
      state.resources = state.resources.map((resource) =>
        command === "add_all_resources" || resource.id === resourceId
          ? { ...resource, amount: resource.amount + amount }
          : resource,
      );
      return {
        pid: PID,
        amountAdded: amount,
        resources: state.resources,
        backup: backup("add-resources"),
        message: `${amount} resources added.`,
      } as T;
    }

    case "get_clip_status":
      return clipStatus() as T;
    case "set_infinite_magazine":
      state.infiniteMagazine = Boolean(args?.enabled);
      return clipStatus() as T;
    case "get_damage_status":
      return damageStatus() as T;
    case "set_weapon_damage":
      state.weaponDamage = Boolean(args?.enabled);
      return damageStatus() as T;

    case "set_clip_hotkey":
    case "set_damage_hotkey":
      return { hotkey: (args?.hotkey as string | null) ?? null } as T;

    case "unlock_all_weapons": {
      const already = state.weaponsUnlocked;
      state.weaponsUnlocked = true;
      return {
        pid: PID,
        weaponCount: 26,
        unlockedBefore: already ? 26 : 0,
        unlockedAfter: 26,
        ownedBefore: already ? 26 : 0,
        ownedAfter: 26,
        backup: backup("unlock-all"),
        message: already
          ? "All weapons were already unlocked."
          : "All weapon licenses were unlocked.",
      } as T;
    }

    case "unlock_all_perks":
    case "unlock_all_gear_modifications": {
      const perks = command === "unlock_all_perks";
      const already = perks ? state.perksUnlocked : state.gearUnlocked;
      if (perks) state.perksUnlocked = true;
      else state.gearUnlocked = true;
      const total = perks ? 23 : 148;
      return {
        pid: PID,
        itemsBefore: already ? total : 0,
        itemsAfter: total,
        backup: backup(perks ? "unlock-all-perks" : "unlock-all-gear-modifications"),
        message: perks ? "All perks unlocked." : "All gear modifications acquired.",
      } as T;
    }

    case "unlock_all_overclocks_and_cosmetics": {
      const already = state.schematicsUnlocked;
      state.schematicsUnlocked = true;
      return {
        pid: PID,
        overclocksProcessed: already ? 0 : 160,
        cosmeticsProcessed: 348,
        forgedBefore: already ? 515 : 0,
        forgedAfter: 515,
        ownedBefore: already ? 0 : 30,
        ownedAfter: 0,
        backup: backup("unlock-all-schematics"),
        message: "All overclocks and cosmetic schematics unlocked and forged.",
      } as T;
    }

    case "max_class_level":
      return {
        pid: PID,
        classesUpdated: 4,
        changes: ["Driller", "Engineer", "Gunner", "Scout"].map((className) => ({
          className,
          previousXp: className === "Scout" ? 3_048 : 0,
          currentXp: 315_000,
        })),
        backup: backup("max-class-level"),
        message: "All four playable classes are now level 25.",
      } as T;

    case "promote_all_classes":
      return {
        pid: PID,
        classesPromoted: 4,
        changes: ["Driller", "Engineer", "Gunner", "Scout"].map((className) => ({
          className,
          previousPromotions: 0,
          currentPromotions: 1,
        })),
        backup: backup("promote-all-classes"),
        message: "All four classes received one promotion.",
      } as T;

    default:
      throw new Error(`Comando sem mock no preview: ${command}`);
  }
}
