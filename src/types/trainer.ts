export type CreditSnapshot = {
  credits: number;
  pid: number;
  address: string;
  moduleBase: string;
  offset: string;
  buildValid: boolean;
};

export type WriteResult = {
  previous: number;
  current: number;
  pid: number;
  address: string;
};

export type ResourceSnapshot = {
  id: string;
  name: string;
  category: string;
  amount: number;
};

export type ResourceWriteResult = {
  pid: number;
  amountAdded: number;
  resources: ResourceSnapshot[];
  backupPath: string;
  message: string;
};

export type ClipStatus = {
  enabled: boolean;
  available: boolean;
  pid: number | null;
  weaponName: string | null;
  clipCount: number | null;
  clipSize: number | null;
  address: string | null;
  message: string;
};

export type DamageStatus = {
  enabled: boolean;
  available: boolean;
  pid: number | null;
  weaponName: string | null;
  value: number | null;
  targetCount: number;
  addresses: string[];
  message: string;
};

export type UnlockAllResult = {
  pid: number;
  weaponCount: number;
  unlockedBefore: number;
  unlockedAfter: number;
  ownedBefore: number;
  ownedAfter: number;
  backupPath: string;
  message: string;
};

export type MaxClassLevelResult = {
  pid: number;
  classesUpdated: number;
  changes: Array<{
    className: string;
    previousXp: number;
    currentXp: number;
  }>;
  backupPath: string;
  message: string;
};

export type PermanentUnlockResult = {
  pid: number;
  itemsBefore: number;
  itemsAfter: number;
  backupPath: string;
  message: string;
};

export type SchematicUnlockResult = {
  pid: number;
  overclocksProcessed: number;
  cosmeticsProcessed: number;
  forgedBefore: number;
  forgedAfter: number;
  ownedBefore: number;
  ownedAfter: number;
  backupPath: string;
  message: string;
};

export type PromoteAllResult = {
  pid: number;
  classesPromoted: number;
  changes: Array<{
    className: string;
    previousPromotions: number;
    currentPromotions: number;
  }>;
  backupPath: string;
  message: string;
};

export type ConnectionState = "connecting" | "connected" | "waiting" | "error";
export type ModuleId = "all" | "player" | "inventory" | "weapons";
export type ResourceWriteMode = "single" | "all";
