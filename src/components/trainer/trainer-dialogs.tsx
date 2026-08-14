import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { formatNumber } from "@/lib/format";
import type { CreditSnapshot, ResourceSnapshot, ResourceWriteMode } from "@/types/trainer";

type ConfirmDialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: React.ReactNode;
  action: string;
  onConfirm: () => void;
};

function ConfirmDialog({ open, onOpenChange, title, description, action, onConfirm }: ConfirmDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>{action}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

type TrainerDialogsProps = {
  creditsOpen: boolean;
  resourceMode: ResourceWriteMode | null;
  weaponsOpen: boolean;
  maxLevelOpen: boolean;
  perksOpen: boolean;
  gearOpen: boolean;
  schematicsOpen: boolean;
  promotionOpen: boolean;
  snapshot: CreditSnapshot | null;
  selectedResource: ResourceSnapshot | null | undefined;
  creditsAmount: number;
  resourceAmount: number;
  allResourcesAmount: number;
  onCreditsOpenChange: (open: boolean) => void;
  onResourceModeChange: (mode: ResourceWriteMode | null) => void;
  onWeaponsOpenChange: (open: boolean) => void;
  onMaxLevelOpenChange: (open: boolean) => void;
  onPerksOpenChange: (open: boolean) => void;
  onGearOpenChange: (open: boolean) => void;
  onSchematicsOpenChange: (open: boolean) => void;
  onPromotionOpenChange: (open: boolean) => void;
  onWriteCredits: () => void;
  onWriteResource: () => void;
  onUnlockWeapons: () => void;
  onMaxLevel: () => void;
  onUnlockPerks: () => void;
  onUnlockGear: () => void;
  onUnlockSchematics: () => void;
  onPromote: () => void;
};

export function TrainerDialogs(props: TrainerDialogsProps) {
  const resourceTitle = props.resourceMode === "all"
    ? "Add to all resources?"
    : `Add ${props.selectedResource?.name ?? "resource"}?`;
  const resourceDescription = props.resourceMode === "all"
    ? `${formatNumber(props.allResourcesAmount)} will be added to each of the 14 persistent resources.`
    : `${formatNumber(props.resourceAmount)} will be added to ${props.selectedResource?.name ?? "the selected resource"}.`;

  return (
    <>
      <ConfirmDialog
        open={props.creditsOpen}
        onOpenChange={props.onCreditsOpenChange}
        title="Apply credits value?"
        description={`Memory will change from ${props.snapshot ? formatNumber(props.snapshot.credits) : "--"} to ${formatNumber(props.creditsAmount)}.`}
        action="Write value"
        onConfirm={props.onWriteCredits}
      />
      <ConfirmDialog
        open={props.resourceMode !== null}
        onOpenChange={(open) => !open && props.onResourceModeChange(null)}
        title={resourceTitle}
        description={`${resourceDescription} The trainer will create a save backup, use the game's native resource routine, verify the new balance, and persist it. Stay in the Space Rig.`}
        action="Create backup & add"
        onConfirm={props.onWriteResource}
      />
      <ConfirmDialog
        open={props.weaponsOpen}
        onOpenChange={props.onWeaponsOpenChange}
        title="Unlock every weapon?"
        description="This is a permanent save change. Stay in the Space Rig; the trainer will create a backup, run the game's native unlock routine for all 26 weapons, and save the result."
        action="Create backup & unlock"
        onConfirm={props.onUnlockWeapons}
      />
      <ConfirmDialog
        open={props.maxLevelOpen}
        onOpenChange={props.onMaxLevelOpenChange}
        title="Set every class to level 25?"
        description="This permanently sets the XP of all four playable classes to 315,000. Existing promotions remain unchanged, Bosco is ignored, and a save backup is created first. Stay in the Space Rig."
        action="Create backup & apply"
        onConfirm={props.onMaxLevel}
      />
      <ConfirmDialog
        open={props.perksOpen}
        onOpenChange={props.onPerksOpenChange}
        title="Unlock every perk?"
        description="This permanently acquires every perk rank using the game's native routine. Equipped perk loadouts remain unchanged. Stay in the Space Rig; a save backup is created first."
        action="Create backup & unlock"
        onConfirm={props.onUnlockPerks}
      />
      <ConfirmDialog
        open={props.gearOpen}
        onOpenChange={props.onGearOpenChange}
        title="Unlock every gear modification?"
        description="This permanently acquires all purchasable weapon and equipment modifications. Current loadouts are preserved. Stay in the Space Rig; a save backup is created first."
        action="Create backup & unlock"
        onConfirm={props.onUnlockGear}
      />
      <ConfirmDialog
        open={props.schematicsOpen}
        onOpenChange={props.onSchematicsOpenChange}
        title="Unlock all overclocks and cosmetics?"
        description="This permanently grants every weapon overclock and forge cosmetic, repairs rewards missing from already-forged schematics, and completes remaining schematics. Stay in the Space Rig; a save backup is created first."
        action="Create backup & unlock"
        onConfirm={props.onUnlockSchematics}
      />
      <ConfirmDialog
        open={props.promotionOpen}
        onOpenChange={props.onPromotionOpenChange}
        title="Promote all four classes?"
        description="Driller, Engineer, Gunner and Scout will each receive one native promotion and return to level 1, including normal promotion rewards and rank progression. Stay in the Space Rig; a save backup is created first."
        action="Create backup & promote"
        onConfirm={props.onPromote}
      />
    </>
  );
}
