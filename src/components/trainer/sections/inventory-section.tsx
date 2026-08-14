import { Activity, Boxes, Coins, Gem, Loader2, PackagePlus, RefreshCw } from "lucide-react";
import type { FormEventHandler } from "react";

import { OptionRow, TrainerSection, ValuePair } from "@/components/trainer/primitives";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectGroup, SelectItem, SelectLabel, SelectTrigger, SelectValue } from "@/components/ui/select";
import { formatNumber } from "@/lib/format";
import type { CreditSnapshot, ResourceSnapshot, ResourceWriteMode } from "@/types/trainer";

type InventorySectionProps = {
  compact: boolean;
  snapshot: CreditSnapshot | null;
  resources: ResourceSnapshot[];
  selectedResource: ResourceSnapshot | undefined;
  selectedResourceId: string;
  creditsAmount: string;
  resourceAmount: string;
  allResourcesAmount: string;
  writing: boolean;
  resourceBusy: ResourceWriteMode | null;
  creditsAmountValid: boolean;
  resourceAmountValid: boolean;
  allResourcesAmountValid: boolean;
  onOpenInventory: () => void;
  onCreditsSubmit: FormEventHandler<HTMLFormElement>;
  onCreditsAmountChange: (value: string) => void;
  onResourceChange: (id: string) => void;
  onResourceAmountChange: (value: string) => void;
  onAllResourcesAmountChange: (value: string) => void;
  onAddResource: () => void;
  onAddAllResources: () => void;
};

export function InventorySection(props: InventorySectionProps) {
  const { compact, snapshot, resources, selectedResource, resourceBusy } = props;
  return (
    <TrainerSection icon={Boxes} title="Inventory" compact={compact}>
      <div className="option-grid">
        <OptionRow icon={Coins} title="Credits" info="Reads and writes FSDSaveGame.Credits at offset 0x590. The value is verified immediately after each write.">
          <span className="option-value">{snapshot ? formatNumber(snapshot.credits) : "--"}</span>
          {compact ? (
            <Button size="xs" variant="secondary" onClick={props.onOpenInventory}>Edit</Button>
          ) : (
            <form onSubmit={props.onCreditsSubmit} className="inline-editor">
              <Input aria-label="Credits value" type="number" min="0" max="2147483647" value={props.creditsAmount} disabled={!snapshot || props.writing} onChange={(event) => props.onCreditsAmountChange(event.target.value)} />
              <Button type="submit" size="xs" disabled={!snapshot || !props.creditsAmountValid || props.writing}>
                {props.writing ? <Loader2 className="animate-spin" /> : <RefreshCw />} Apply
              </Button>
            </form>
          )}
        </OptionRow>
        <OptionRow icon={Activity} title="Save object" info="The active FSDSaveGame object is resolved through GUObjectArray and checked before every access.">
          <ValuePair label="Class" value="FSDSaveGame" />
          <ValuePair label="Address" value={snapshot?.address ?? "--"} wide />
        </OptionRow>
        <OptionRow icon={Gem} title="Stored resources" info="Tracks the 14 persistent inventory resources in FSDSaveGame.Resources: crafting minerals, brewing ingredients, Phazyonite, Blank Matrix Cores, Error Cubes and Data Cells.">
          <ValuePair label="Catalog" value={resources.length ? `${resources.length} items` : "--"} />
          {compact && <Button size="xs" variant="secondary" onClick={props.onOpenInventory}>Manage</Button>}
        </OptionRow>
      </div>
      {!compact && (
        <div className="option-grid option-grid-border">
          <OptionRow icon={PackagePlus} title="Add resource" info="Adds the chosen amount to one resource through the game's native FSDSaveGame::AddResource routine. Missing map entries are created by the game; a backup is made before saving.">
            <ValuePair label="Current" value={selectedResource ? formatNumber(selectedResource.amount) : "--"} />
            <div className="resource-editor">
              <Select value={props.selectedResourceId} onValueChange={props.onResourceChange} disabled={!snapshot || resourceBusy !== null}>
                <SelectTrigger aria-label="Resource" className="resource-select"><SelectValue placeholder="Select resource" /></SelectTrigger>
                <SelectContent>
                  {["Minerals", "Brewing", "Special"].map((category) => (
                    <SelectGroup key={category}>
                      <SelectLabel>{category}</SelectLabel>
                      {resources.filter((resource) => resource.category === category).map((resource) => (
                        <SelectItem key={resource.id} value={resource.id}>{resource.name} · {formatNumber(resource.amount)}</SelectItem>
                      ))}
                    </SelectGroup>
                  ))}
                </SelectContent>
              </Select>
              <Input aria-label="Amount for selected resource" type="number" min="1" max="1000000" value={props.resourceAmount} disabled={!snapshot || resourceBusy !== null} onChange={(event) => props.onResourceAmountChange(event.target.value)} />
              <Button type="button" size="xs" disabled={!snapshot || !selectedResource || !props.resourceAmountValid || resourceBusy !== null} onClick={props.onAddResource}>
                {resourceBusy === "single" ? <Loader2 className="animate-spin" /> : <PackagePlus />}
                {resourceBusy === "single" ? "Adding" : "Add"}
              </Button>
            </div>
          </OptionRow>
          <OptionRow icon={Gem} title="Add all resources" info="Adds the same quantity to all 14 persistent resources in one operation. Each result is verified, one save backup is created, and SaveToDisk runs only after the complete batch succeeds.">
            <ValuePair label="Targets" value="14 items" />
            <div className="resource-editor resource-editor-all">
              <Input aria-label="Amount for all resources" type="number" min="1" max="1000000" value={props.allResourcesAmount} disabled={!snapshot || resourceBusy !== null} onChange={(event) => props.onAllResourcesAmountChange(event.target.value)} />
              <Button type="button" size="xs" disabled={!snapshot || !props.allResourcesAmountValid || resourceBusy !== null} onClick={props.onAddAllResources}>
                {resourceBusy === "all" ? <Loader2 className="animate-spin" /> : <Gem />}
                {resourceBusy === "all" ? "Adding" : "Add to all"}
              </Button>
            </div>
          </OptionRow>
        </div>
      )}
    </TrainerSection>
  );
}
