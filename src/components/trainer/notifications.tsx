import { CheckCircle2 } from "lucide-react";

import { formatNumber } from "@/lib/format";
import type {
  MaxClassLevelResult,
  PermanentUnlockResult,
  PromoteAllResult,
  ResourceWriteResult,
  SchematicUnlockResult,
  UnlockAllResult,
  WriteResult,
} from "@/types/trainer";

type TrainerNotificationsProps = {
  error: string | null;
  creditWrite: WriteResult | null;
  weaponUnlock: UnlockAllResult | null;
  resourceWrite: ResourceWriteResult | null;
  maxLevel: MaxClassLevelResult | null;
  perkUnlock: PermanentUnlockResult | null;
  gearUnlock: PermanentUnlockResult | null;
  schematicUnlock: SchematicUnlockResult | null;
  promotion: PromoteAllResult | null;
};

function SuccessToast({ children }: { children: React.ReactNode }) {
  return (
    <div className="trainer-toast">
      <CheckCircle2 className="size-3.5" />
      {children}
    </div>
  );
}

export function TrainerNotifications({
  error,
  creditWrite,
  weaponUnlock,
  resourceWrite,
  maxLevel,
  perkUnlock,
  gearUnlock,
  schematicUnlock,
  promotion,
}: TrainerNotificationsProps) {
  return (
    <>
      {error && <div className="trainer-error">{error}</div>}
      {creditWrite && (
        <SuccessToast>
          Credits verified: {formatNumber(creditWrite.previous)} → {formatNumber(creditWrite.current)}
        </SuccessToast>
      )}
      {weaponUnlock && (
        <SuccessToast>{weaponUnlock.weaponCount} weapons processed. Save backup created before unlock.</SuccessToast>
      )}
      {resourceWrite && (
        <SuccessToast>{resourceWrite.message} Values verified; save backup created.</SuccessToast>
      )}
      {maxLevel && (
        <SuccessToast>{maxLevel.classesUpdated} classes set to level 25. Promotions preserved; backup created.</SuccessToast>
      )}
      {perkUnlock && (
        <SuccessToast>Perks acquired: {perkUnlock.itemsBefore} → {perkUnlock.itemsAfter}. Backup created.</SuccessToast>
      )}
      {gearUnlock && (
        <SuccessToast>Gear modifications acquired: {gearUnlock.itemsBefore} → {gearUnlock.itemsAfter}. Backup created.</SuccessToast>
      )}
      {schematicUnlock && (
        <SuccessToast>
          Rewards applied: {schematicUnlock.overclocksProcessed} overclocks, {schematicUnlock.cosmeticsProcessed} cosmetics. Schematics: {schematicUnlock.forgedAfter} forged. Backup created.
        </SuccessToast>
      )}
      {promotion && (
        <SuccessToast>{promotion.classesPromoted} classes promoted once. Native progression saved; backup created.</SuccessToast>
      )}
    </>
  );
}
