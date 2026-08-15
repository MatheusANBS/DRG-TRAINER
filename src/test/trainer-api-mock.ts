/**
 * Mocks centralizados da fronteira Tauri (SPEC-008).
 *
 * Todo teste passa por aqui em vez de mockar `@tauri-apps/api` diretamente:
 * a fronteira real é `services/trainer-api`, e é ela que precisa ser
 * substituída para o teste refletir o que o app realmente chama.
 */

import { vi } from "vitest";

import type {
  Capabilities,
  ClipStatus,
  CreditSnapshot,
  DamageStatus,
  ResourceSnapshot,
  TrainerError,
  TrainerErrorCode,
  TrainerStatus,
} from "@/types/trainer";

export const ALL_CAPABILITIES: Capabilities = {
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
};

export const NO_CAPABILITIES: Capabilities = Object.fromEntries(
  Object.keys(ALL_CAPABILITIES).map((key) => [key, false]),
) as Capabilities;

export const SHA256 = "8e22e3710c607e811e0319a4378d2b7ab2e46c6b46406629a6657b1662c77a41";

export function attachedStatus(overrides: Partial<TrainerStatus> = {}): TrainerStatus {
  return {
    process: { kind: "attached", pid: 26248, executable: "FSD-Win64-Shipping.exe" },
    build: {
      kind: "verified",
      profileId: "fsd-8e22e371",
      displayName: "Deep Rock Galactic (shipping 8e22e371)",
      sha256: SHA256,
      status: "supported",
    },
    capabilities: ALL_CAPABILITIES,
    expectedExecutables: ["FSD-Win64-Shipping.exe"],
    ...overrides,
  };
}

export function detachedStatus(): TrainerStatus {
  return {
    process: { kind: "detached" },
    build: { kind: "unknown" },
    capabilities: NO_CAPABILITIES,
    expectedExecutables: ["FSD-Win64-Shipping.exe"],
  };
}

export function unsupportedStatus(): TrainerStatus {
  return {
    process: { kind: "attached", pid: 26248, executable: "FSD-Win64-Shipping.exe" },
    build: { kind: "unsupported", sha256: "a".repeat(64) },
    capabilities: NO_CAPABILITIES,
    expectedExecutables: ["FSD-Win64-Shipping.exe"],
  };
}

export function creditSnapshot(credits = 49_995): CreditSnapshot {
  return {
    credits,
    pid: 26248,
    address: "0x2C5A759EC10",
    moduleBase: "0x7FF6A2C30000",
    offset: "0x590",
  };
}

export function resources(): ResourceSnapshot[] {
  return [
    { id: "bismor", name: "Bismor", category: "Minerals", amount: 50 },
    { id: "croppa", name: "Croppa", category: "Minerals", amount: 120 },
    { id: "malt_star", name: "Malt Star", category: "Brewing", amount: 7 },
    { id: "error_cube", name: "Error Cube", category: "Special", amount: 1 },
  ];
}

export function clipStatus(overrides: Partial<ClipStatus> = {}): ClipStatus {
  return {
    enabled: false,
    available: true,
    pid: 26248,
    weaponName: "WPN_AssaultRifle_C",
    clipCount: 12,
    clipSize: 30,
    address: "0x2C5B823C780",
    message: "Arma equipada encontrada.",
    ...overrides,
  };
}

export function damageStatus(overrides: Partial<DamageStatus> = {}): DamageStatus {
  return {
    enabled: false,
    available: true,
    pid: 26248,
    weaponName: "WPN_AssaultRifle_C",
    value: 16,
    targetCount: 2,
    addresses: ["0x2C5B823C898", "0x2C5B823C924"],
    message: "2 campos de dano encontrados.",
    ...overrides,
  };
}

export function backup(operation = "unlock-all") {
  return { path: `C:\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\${operation}-1`, files: 2, totalBackups: 3 };
}

export function trainerError(
  code: TrainerErrorCode,
  message = "detalhe técnico",
  recoverable = false,
): TrainerError {
  return { code, message, recoverable };
}

/**
 * Cria um duplo completo da API, com todos os comandos resolvendo valores
 * plausíveis. Cada teste sobrescreve apenas o que exercita.
 */
export function createApiMock(overrides: Partial<Record<string, unknown>> = {}) {
  const api = {
    status: vi.fn().mockResolvedValue(attachedStatus()),
    readCredits: vi.fn().mockResolvedValue(creditSnapshot()),
    setCredits: vi.fn().mockResolvedValue({
      previous: 49_995,
      current: 1_000,
      pid: 26248,
      address: "0x2C5A759EC10",
    }),
    readResources: vi.fn().mockResolvedValue(resources()),
    addResource: vi.fn().mockResolvedValue({
      pid: 26248,
      amountAdded: 1_000,
      resources: resources(),
      backup: backup("add-resource-bismor"),
      message: "Bismor adicionado: +1000.",
    }),
    addAllResources: vi.fn().mockResolvedValue({
      pid: 26248,
      amountAdded: 1_000,
      resources: resources(),
      backup: backup("add-all-resources"),
      message: "1000 unidades adicionadas a 14 recursos.",
    }),
    readClipStatus: vi.fn().mockResolvedValue(clipStatus()),
    setInfiniteMagazine: vi
      .fn()
      .mockImplementation((enabled: boolean) =>
        Promise.resolve(clipStatus({ enabled, clipCount: enabled ? 30 : 12 })),
      ),
    readDamageStatus: vi.fn().mockResolvedValue(damageStatus()),
    setWeaponDamage: vi
      .fn()
      .mockImplementation((enabled: boolean) =>
        Promise.resolve(damageStatus({ enabled, value: enabled ? 9_999 : 16 })),
      ),
    unlockAllWeapons: vi.fn().mockResolvedValue({
      pid: 26248,
      weaponCount: 26,
      unlockedBefore: 0,
      unlockedAfter: 26,
      ownedBefore: 0,
      ownedAfter: 26,
      backup: backup("unlock-all"),
      message: "Unlock concluido.",
    }),
    unlockAllPerks: vi.fn().mockResolvedValue({
      pid: 26248,
      itemsBefore: 0,
      itemsAfter: 23,
      backup: backup("unlock-all-perks"),
      message: "Perks liberados.",
    }),
    unlockAllGearModifications: vi.fn().mockResolvedValue({
      pid: 26248,
      itemsBefore: 0,
      itemsAfter: 148,
      backup: backup("unlock-all-gear-modifications"),
      message: "Modificacoes adquiridas.",
    }),
    unlockAllOverclocksAndCosmetics: vi.fn().mockResolvedValue({
      pid: 26248,
      overclocksProcessed: 160,
      cosmeticsProcessed: 348,
      forgedBefore: 0,
      forgedAfter: 515,
      ownedBefore: 30,
      ownedAfter: 0,
      backup: backup("unlock-all-schematics"),
      message: "Recompensas processadas.",
    }),
    maxClassLevel: vi.fn().mockResolvedValue({
      pid: 26248,
      classesUpdated: 4,
      changes: [],
      backup: backup("max-class-level"),
      message: "Classes no nivel 25.",
    }),
    promoteAllClasses: vi.fn().mockResolvedValue({
      pid: 26248,
      classesPromoted: 4,
      changes: [],
      backup: backup("promote-all-classes"),
      message: "Classes promovidas.",
    }),
    setClipHotkey: vi
      .fn()
      .mockImplementation((hotkey: string | null) => Promise.resolve({ hotkey })),
    setDamageHotkey: vi
      .fn()
      .mockImplementation((hotkey: string | null) => Promise.resolve({ hotkey })),
    ...overrides,
  };
  return api;
}

export type TrainerApiMock = ReturnType<typeof createApiMock>;
