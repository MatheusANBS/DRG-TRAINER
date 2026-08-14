import { invoke } from "@tauri-apps/api/core";

import type { ResourceSnapshot } from "@/types/trainer";

let mockCredits = 49_995;
let mockInfiniteMagazine = false;
let mockWeaponDamage = false;
let mockWeaponsUnlocked = false;
let mockPerksUnlocked = false;
let mockGearUnlocked = false;
let mockSchematicsUnlocked = false;
let mockResources: ResourceSnapshot[] = [
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
].map(([id, name, category, amount]) => ({
  id: String(id),
  name: String(name),
  category: String(category),
  amount: Number(amount),
}));

export async function invokeApp<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const isBrowserPreview = import.meta.env.DEV && !("__TAURI_INTERNALS__" in window);
  if (!isBrowserPreview) return invoke<T>(command, args);

  if (command === "set_credits") {
    const previous = mockCredits;
    mockCredits = Number(args?.value ?? mockCredits);
    return { previous, current: mockCredits, pid: 26248, address: "0x2C5A759EC10" } as T;
  }
  if (command === "get_resources") return mockResources as T;
  if (command === "add_resource" || command === "add_all_resources") {
    const amount = Number(args?.amount ?? 0);
    const resourceId = String(args?.resourceId ?? "");
    mockResources = mockResources.map((resource) =>
      command === "add_all_resources" || resource.id === resourceId
        ? { ...resource, amount: resource.amount + amount }
        : resource,
    );
    return {
      pid: 26248,
      amountAdded: amount,
      resources: mockResources,
      backupPath: "F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\add-resources-preview",
      message: `${amount} resources added.`,
    } as T;
  }
  if (command === "set_infinite_magazine") mockInfiniteMagazine = Boolean(args?.enabled);
  if (command === "set_weapon_damage") mockWeaponDamage = Boolean(args?.enabled);
  if (command === "set_clip_hotkey" || command === "set_damage_hotkey") {
    return { hotkey: args?.hotkey ?? null } as T;
  }
  if (command === "unlock_all_weapons") {
    const result = {
      pid: 26248,
      weaponCount: 26,
      unlockedBefore: mockWeaponsUnlocked ? 26 : 0,
      unlockedAfter: 26,
      ownedBefore: mockWeaponsUnlocked ? 26 : 0,
      ownedAfter: 26,
      backupPath: "F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\unlock-all-preview",
      message: mockWeaponsUnlocked ? "All weapons were already unlocked." : "All weapon licenses were unlocked.",
    };
    mockWeaponsUnlocked = true;
    return result as T;
  }
  if (command === "max_class_level") {
    return {
      pid: 26248,
      classesUpdated: 4,
      changes: ["Driller", "Engineer", "Gunner", "Scout"].map((className) => ({
        className,
        previousXp: className === "Scout" ? 3_048 : 0,
        currentXp: 315_000,
      })),
      backupPath: "F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\max-class-level-preview",
      message: "All four playable classes are now level 25.",
    } as T;
  }
  if (command === "unlock_all_perks" || command === "unlock_all_gear_modifications") {
    const perks = command === "unlock_all_perks";
    const already = perks ? mockPerksUnlocked : mockGearUnlocked;
    if (perks) mockPerksUnlocked = true;
    else mockGearUnlocked = true;
    return {
      pid: 26248,
      itemsBefore: already ? (perks ? 23 : 148) : 0,
      itemsAfter: perks ? 23 : 148,
      backupPath: `F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\${perks ? "unlock-all-perks" : "unlock-all-gear-modifications"}-preview`,
      message: perks ? "All perks unlocked." : "All gear modifications acquired.",
    } as T;
  }
  if (command === "unlock_all_overclocks_and_cosmetics") {
    const already = mockSchematicsUnlocked;
    mockSchematicsUnlocked = true;
    return {
      pid: 26248,
      overclocksProcessed: already ? 0 : 160,
      cosmeticsProcessed: 348,
      forgedBefore: already ? 515 : 0,
      forgedAfter: 515,
      ownedBefore: already ? 0 : 30,
      ownedAfter: 0,
      backupPath: "F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\unlock-all-schematics-preview",
      message: "All overclocks and cosmetic schematics unlocked and forged.",
    } as T;
  }
  if (command === "promote_all_classes") {
    return {
      pid: 26248,
      classesPromoted: 4,
      changes: ["Driller", "Engineer", "Gunner", "Scout"].map((className) => ({
        className,
        previousPromotions: 0,
        currentPromotions: 1,
      })),
      backupPath: "F:\\Deep Rock Galactic\\FSD\\Saved\\SaveGames\\DRGTrainerBackups\\promote-all-classes-preview",
      message: "All four classes received one promotion.",
    } as T;
  }
  if (command === "get_clip_status" || command === "set_infinite_magazine") {
    return {
      enabled: mockInfiniteMagazine,
      available: true,
      pid: 26248,
      weaponName: "WPN_AssaultRifle_C",
      clipCount: 30,
      clipSize: 30,
      address: "0x2C5B823C780",
      message: mockInfiniteMagazine ? "Infinite Magazine ativo." : "Arma equipada encontrada.",
    } as T;
  }
  if (command === "get_damage_status" || command === "set_weapon_damage") {
    return {
      enabled: mockWeaponDamage,
      available: true,
      pid: 26248,
      weaponName: "WPN_AssaultRifle_C",
      value: mockWeaponDamage ? 9999 : 16,
      targetCount: 2,
      addresses: ["0x2C5B823C898", "0x2C5B823C924"],
      message: mockWeaponDamage ? "Weapon Damage ativo em 2 campos." : "2 campos de dano encontrados.",
    } as T;
  }
  return {
    credits: mockCredits,
    pid: 26248,
    address: "0x2C5A759EC10",
    moduleBase: "0x7FF6A2C30000",
    offset: "0x590",
    buildValid: true,
  } as T;
}
