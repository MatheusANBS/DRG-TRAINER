/**
 * Contrato de dados com o backend.
 *
 * Espelha `src-tauri/src/domain/dto.rs`. Mudou lá, mude aqui — os testes de
 * contrato em `src/services/trainer-api.test.ts` existem para essa checagem
 * não depender de memória.
 */

/** Códigos estáveis emitidos pelo backend (`shared::error::ErrorCode`). */
export type TrainerErrorCode =
  | "PROCESS_NOT_FOUND"
  | "MODULE_NOT_FOUND"
  | "PROCESS_OPEN_FAILED"
  | "BUILD_UNSUPPORTED"
  | "BUILD_HASH_FAILED"
  | "CAPABILITY_UNAVAILABLE"
  | "MEMORY_READ_FAILED"
  | "MEMORY_WRITE_FAILED"
  | "WRITE_VERIFICATION_FAILED"
  | "READ_ONLY_HANDLE"
  | "SIGNATURE_MISMATCH"
  | "NATIVE_CALL_FAILED"
  | "NATIVE_CALL_TIMEOUT"
  | "WORLD_NOT_READY"
  | "OBJECT_NOT_FOUND"
  | "SAVE_NOT_FOUND"
  | "SAVE_STATE_CHANGED"
  | "BACKUP_FAILED"
  | "INVALID_ARGUMENT"
  | "INVALID_GAME_STATE"
  | "STATE_UNAVAILABLE"
  | "HOTKEY_INVALID"
  | "HOTKEY_REGISTRATION_FAILED"
  | "TASK_FAILED";

export type TrainerError = {
  code: TrainerErrorCode;
  message: string;
  recoverable: boolean;
};

export type ProfileStatus = "draft" | "verified" | "supported" | "deprecated";

export type ProcessState =
  | { kind: "detached" }
  | { kind: "attached"; pid: number; executable: string };

export type BuildState =
  | { kind: "unknown" }
  | {
      kind: "verified";
      profileId: string;
      displayName: string;
      sha256: string;
      status: ProfileStatus;
    }
  | {
      kind: "unverified";
      profileId: string;
      displayName: string;
      sha256: string;
      status: ProfileStatus;
    }
  | { kind: "unsupported"; sha256: string };

/** Capacidades verificadas do perfil ativo. Tudo falso em build desconhecida. */
export type Capabilities = {
  credits: boolean;
  resources: boolean;
  weaponUnlock: boolean;
  perkUnlock: boolean;
  gearUnlock: boolean;
  schematicUnlock: boolean;
  classLevel: boolean;
  promotion: boolean;
  infiniteMagazine: boolean;
  weaponDamage: boolean;
};

export type CapabilityId = keyof Capabilities;

export type TrainerStatus = {
  process: ProcessState;
  build: BuildState;
  capabilities: Capabilities;
  expectedExecutables: string[];
};

/** Backup criado antes de uma mutação permanente. */
export type BackupSet = {
  path: string;
  files: number;
  totalBackups: number;
};

export type CreditSnapshot = {
  credits: number;
  pid: number;
  address: string;
  moduleBase: string;
  offset: string;
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
  backup: BackupSet;
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
  backup: BackupSet;
  message: string;
};

export type PermanentUnlockResult = {
  pid: number;
  itemsBefore: number;
  itemsAfter: number;
  backup: BackupSet;
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
  backup: BackupSet;
  message: string;
};

export type MaxClassLevelResult = {
  pid: number;
  classesUpdated: number;
  changes: Array<{ className: string; previousXp: number; currentXp: number }>;
  backup: BackupSet;
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
  backup: BackupSet;
  message: string;
};

export type HotkeyResult = { hotkey: string | null };

export type ModuleId = "all" | "player" | "inventory" | "weapons";
export type ResourceWriteMode = "single" | "all";
