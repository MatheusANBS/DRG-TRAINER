import { Activity, Boxes, Coins, Gem, Loader2, PackagePlus, RefreshCw } from "lucide-react";
import type { FormEvent } from "react";

import { ActionControl, SemanticsBadge } from "@/components/trainer/action-control";
import { OptionRow, TrainerSection, ValuePair } from "@/components/trainer/primitives";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { InventoryActions } from "@/hooks/useInventoryActions";
import { formatNumber } from "@/lib/format";
import { NO_VALUE } from "@/lib/weapon-format";
import type { CreditSnapshot, ResourceSnapshot } from "@/types/trainer";

const RESOURCE_CATEGORIES = ["Minerals", "Brewing", "Special"] as const;

type InventorySectionProps = {
  compact: boolean;
  credits: CreditSnapshot | null;
  resources: ResourceSnapshot[];
  inventory: InventoryActions;
  onOpenInventory: () => void;
};

export function InventorySection({
  compact,
  credits,
  resources,
  inventory,
  onOpenInventory,
}: InventorySectionProps) {
  const submitCredits = (event: FormEvent) => {
    event.preventDefault();
    if (!inventory.writeCredits.disabled) inventory.setConfirming("credits");
  };

  return (
    <TrainerSection icon={Boxes} title="Inventory" compact={compact}>
      <div className="option-grid">
        <OptionRow
          icon={Coins}
          title="Credits"
          info="Reads and writes FSDSaveGame.Credits. The value is verified immediately after each write. This changes the loaded save in memory; the game decides when to persist it."
        >
          <span className="option-value">
            {credits ? formatNumber(credits.credits) : NO_VALUE}
          </span>
          {compact ? (
            <Button size="xs" variant="secondary" onClick={onOpenInventory}>
              Edit
            </Button>
          ) : (
            <form onSubmit={submitCredits} className="inline-editor">
              <Input
                aria-label="Credits value"
                type="number"
                min="0"
                max="2147483647"
                value={inventory.creditsInput}
                // O campo continua editável mesmo com valor inválido: quem
                // bloqueia a gravação é o botão, não a digitação.
                disabled={!credits || inventory.writeCredits.pending}
                onChange={(event) => inventory.editCredits(event.target.value)}
              />
              <Button
                type="submit"
                size="xs"
                disabled={inventory.writeCredits.disabled}
                data-phase={inventory.writeCredits.phase}
              >
                {inventory.writeCredits.pending ? (
                  <Loader2 className="animate-spin" />
                ) : (
                  <RefreshCw />
                )}
                Apply
              </Button>
            </form>
          )}
        </OptionRow>

        <OptionRow
          icon={Activity}
          title="Save object"
          info="The active FSDSaveGame object is resolved through GUObjectArray and revalidated before every access."
        >
          <ValuePair label="Class" value="FSDSaveGame" />
          <ValuePair label="Address" value={credits?.address ?? NO_VALUE} wide />
        </OptionRow>

        <OptionRow
          icon={Gem}
          title="Stored resources"
          info="Tracks the 14 persistent inventory resources in FSDSaveGame.Resources: crafting minerals, brewing ingredients, Phazyonite, Blank Matrix Cores, Error Cubes and Data Cells."
        >
          <ValuePair
            label="Catalog"
            value={resources.length ? `${resources.length} items` : NO_VALUE}
          />
          {compact && (
            <Button size="xs" variant="secondary" onClick={onOpenInventory}>
              Manage
            </Button>
          )}
        </OptionRow>
      </div>

      {!compact && (
        <div className="option-grid option-grid-border">
          <OptionRow
            icon={PackagePlus}
            title="Add resource"
            info="Adds the chosen amount to one resource through the game's native FSDSaveGame::AddResource routine. Missing map entries are created by the game; a save backup is made before writing."
          >
            <ValuePair
              label="Current"
              value={
                inventory.selectedResource
                  ? formatNumber(inventory.selectedResource.amount)
                  : NO_VALUE
              }
            />
            <div className="resource-editor">
              <Select
                value={inventory.selectedResourceId}
                onValueChange={inventory.setSelectedResourceId}
                disabled={inventory.busy}
              >
                <SelectTrigger aria-label="Resource" className="resource-select">
                  <SelectValue placeholder="Select resource" />
                </SelectTrigger>
                <SelectContent>
                  {RESOURCE_CATEGORIES.map((category) => (
                    <SelectGroup key={category}>
                      <SelectLabel>{category}</SelectLabel>
                      {resources
                        .filter((resource) => resource.category === category)
                        .map((resource) => (
                          <SelectItem key={resource.id} value={resource.id}>
                            {resource.name} · {formatNumber(resource.amount)}
                          </SelectItem>
                        ))}
                    </SelectGroup>
                  ))}
                </SelectContent>
              </Select>
              <Input
                aria-label="Amount for selected resource"
                type="number"
                min="1"
                max="1000000"
                value={inventory.resourceAmount}
                disabled={inventory.busy}
                onChange={(event) => inventory.setResourceAmount(event.target.value)}
              />
              <SemanticsBadge semantics="persistent" />
              <ActionControl
                action={inventory.addResource}
                semantics="persistent"
                label="Add"
                pendingLabel="Adding"
                icon={PackagePlus}
                onActivate={() => inventory.setConfirming("single")}
              />
            </div>
          </OptionRow>

          <OptionRow
            icon={Gem}
            title="Add all resources"
            info="Adds the same quantity to all 14 persistent resources in one operation. Each result is verified, one save backup is created, and the game only saves to disk after the complete batch succeeds."
          >
            <ValuePair label="Targets" value={`${resources.length || 14} items`} />
            <div className="resource-editor resource-editor-all">
              <Input
                aria-label="Amount for all resources"
                type="number"
                min="1"
                max="1000000"
                value={inventory.allResourcesAmount}
                disabled={inventory.busy}
                onChange={(event) => inventory.setAllResourcesAmount(event.target.value)}
              />
              <SemanticsBadge semantics="persistent" />
              <ActionControl
                action={inventory.addAllResources}
                semantics="persistent"
                label="Add to all"
                pendingLabel="Adding"
                icon={Gem}
                onActivate={() => inventory.setConfirming("all")}
              />
            </div>
          </OptionRow>
        </div>
      )}
    </TrainerSection>
  );
}
