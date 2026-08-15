/**
 * Confirmação de mutações (SPEC-015 / SPEC-019).
 *
 * Só ações com semântica `standard` ou `persistent` chegam aqui. O texto sempre
 * diz o que muda, se há backup e o que é preciso para a operação funcionar.
 */

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
import type { InventoryActions } from "@/hooks/useInventoryActions";
import type { ProgressionActionId, ProgressionActions } from "@/hooks/useProgressionActions";
import { formatNumber } from "@/lib/format";
import type { CreditSnapshot } from "@/types/trainer";

type Copy = { title: string; description: string; action: string };

const PROGRESSION_COPY: Record<ProgressionActionId, Copy> = {
  maxLevel: {
    title: "Set every class to level 25?",
    description:
      "This permanently sets the XP of all four playable classes to 315,000. Existing promotions remain unchanged, Bosco is ignored, and a save backup is created first. Stay in the Space Rig.",
    action: "Create backup & apply",
  },
  promote: {
    title: "Promote all four classes?",
    description:
      "Driller, Engineer, Gunner and Scout will each receive one native promotion and return to level 1, including normal promotion rewards and rank progression. Stay in the Space Rig; a save backup is created first.",
    action: "Create backup & promote",
  },
  perks: {
    title: "Unlock every perk?",
    description:
      "This permanently acquires every perk rank using the game's native routine. Equipped perk loadouts remain unchanged. Stay in the Space Rig; a save backup is created first.",
    action: "Create backup & unlock",
  },
  weapons: {
    title: "Unlock every weapon?",
    description:
      "This is a permanent save change. Stay in the Space Rig; the trainer will create a backup, run the game's native unlock routine for all 26 weapons, and save the result.",
    action: "Create backup & unlock",
  },
  gear: {
    title: "Unlock every gear modification?",
    description:
      "This permanently acquires all purchasable weapon and equipment modifications. Current loadouts are preserved. Stay in the Space Rig; a save backup is created first.",
    action: "Create backup & unlock",
  },
  schematics: {
    title: "Unlock all overclocks and cosmetics?",
    description:
      "This permanently grants every weapon overclock and forge cosmetic, repairs rewards missing from already-forged schematics, and completes remaining schematics. Stay in the Space Rig; a save backup is created first.",
    action: "Create backup & unlock",
  },
};

function ConfirmDialog({
  open,
  onOpenChange,
  copy,
  onConfirm,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  copy: Copy;
  onConfirm: () => void;
}) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{copy.title}</AlertDialogTitle>
          <AlertDialogDescription>{copy.description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm}>{copy.action}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export function TrainerDialogs({
  credits,
  inventory,
  progression,
}: {
  credits: CreditSnapshot | null;
  inventory: InventoryActions;
  progression: ProgressionActions;
}) {
  const pendingProgression = progression.confirming;
  const inventoryPending = inventory.confirming;

  const inventoryCopy: Copy | null =
    inventoryPending === "credits"
      ? {
          title: "Apply credits value?",
          description: `Memory will change from ${
            credits ? formatNumber(credits.credits) : "the current balance"
          } to ${formatNumber(Number(inventory.creditsInput))}. This writes to the save loaded in memory; the game decides when to persist it.`,
          action: "Write value",
        }
      : inventoryPending === "all"
        ? {
            title: "Add to all resources?",
            description: `${formatNumber(
              Number(inventory.allResourcesAmount),
            )} will be added to each persistent resource. The trainer will create a save backup, use the game's native resource routine, verify every new balance, and persist it. Stay in the Space Rig.`,
            action: "Create backup & add",
          }
        : inventoryPending === "single"
          ? {
              title: `Add ${inventory.selectedResource?.name ?? "resource"}?`,
              description: `${formatNumber(Number(inventory.resourceAmount))} will be added to ${
                inventory.selectedResource?.name ?? "the selected resource"
              }. The trainer will create a save backup, use the game's native resource routine, verify the new balance, and persist it. Stay in the Space Rig.`,
              action: "Create backup & add",
            }
          : null;

  return (
    <>
      {inventoryCopy && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && inventory.setConfirming(null)}
          copy={inventoryCopy}
          onConfirm={inventory.confirm}
        />
      )}
      {pendingProgression && (
        <ConfirmDialog
          open
          onOpenChange={(open) => !open && progression.setConfirming(null)}
          copy={PROGRESSION_COPY[pendingProgression]}
          onConfirm={progression.confirm}
        />
      )}
    </>
  );
}
