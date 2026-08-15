/**
 * Ações de inventário: créditos e recursos (SPEC-012).
 *
 * O hook concentra validação de entrada, confirmação e execução. Os componentes
 * de seção recebem apenas o que precisam renderizar.
 */

import { useCallback, useMemo, useState } from "react";

import type { ActionFeed } from "@/hooks/useActionFeed";
import { useAsyncAction } from "@/hooks/useAsyncAction";
import { formatNumber } from "@/lib/format";
import { trainerApi } from "@/services/trainer-api";
import type { TrainerState } from "@/state/trainer-state";
import { blockedReason } from "@/state/trainer-state";
import type { ResourceSnapshot, ResourceWriteMode } from "@/types/trainer";

const MAX_CREDITS = 2_147_483_647;
const MAX_RESOURCE_DELTA = 1_000_000;

function isIntegerInRange(value: string, min: number, max: number): boolean {
  if (value.trim() === "") return false;
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= min && parsed <= max;
}

type Options = {
  state: TrainerState;
  feed: ActionFeed;
  resources: ResourceSnapshot[];
  onResourcesChanged: (resources: ResourceSnapshot[]) => void;
  onCreditsChanged: () => void;
};

export function useInventoryActions({
  state,
  feed,
  resources,
  onResourcesChanged,
  onCreditsChanged,
}: Options) {
  const [creditsInput, setCreditsInput] = useState("0");
  const [creditsDirty, setCreditsDirty] = useState(false);
  const [selectedResourceId, setSelectedResourceId] = useState("bismor");
  const [resourceAmount, setResourceAmount] = useState("1000");
  const [allResourcesAmount, setAllResourcesAmount] = useState("1000");
  const [confirming, setConfirming] = useState<"credits" | ResourceWriteMode | null>(null);

  const selectedResource = useMemo(
    () => resources.find((resource) => resource.id === selectedResourceId),
    [resources, selectedResourceId],
  );

  const creditsValid = isIntegerInRange(creditsInput, 0, MAX_CREDITS);
  const resourceAmountValid = isIntegerInRange(resourceAmount, 1, MAX_RESOURCE_DELTA);
  const allResourcesAmountValid = isIntegerInRange(allResourcesAmount, 1, MAX_RESOURCE_DELTA);

  /** Enquanto o usuário não digita, o campo acompanha o valor lido do jogo. */
  const syncCreditsInput = useCallback(
    (value: number) => {
      if (!creditsDirty) setCreditsInput(String(value));
    },
    [creditsDirty],
  );

  const editCredits = useCallback((value: string) => {
    setCreditsDirty(true);
    setCreditsInput(value);
  }, []);

  const writeCredits = useAsyncAction({
    perform: () => trainerApi.setCredits(Number(creditsInput)),
    blockedReason: creditsValid
      ? blockedReason(state, "credits")
      : "Enter a whole number between 0 and 2,147,483,647.",
    successMessage: (result) =>
      `Credits verified: ${formatNumber(result.previous)} → ${formatNumber(result.current)}`,
    feed,
    onSuccess: () => {
      setCreditsDirty(false);
      onCreditsChanged();
    },
  });

  const addResource = useAsyncAction({
    perform: () => trainerApi.addResource(selectedResourceId, Number(resourceAmount)),
    blockedReason:
      resourceAmountValid && selectedResource
        ? blockedReason(state, "resources")
        : "Enter a whole number between 1 and 1,000,000.",
    successMessage: (result) => result.message,
    successDetail: (result) => `Backup: ${result.backup.path}`,
    feed,
    onSuccess: (result) => onResourcesChanged(result.resources),
  });

  const addAllResources = useAsyncAction({
    perform: () => trainerApi.addAllResources(Number(allResourcesAmount)),
    blockedReason: allResourcesAmountValid
      ? blockedReason(state, "resources")
      : "Enter a whole number between 1 and 1,000,000.",
    successMessage: (result) => result.message,
    successDetail: (result) => `Backup: ${result.backup.path}`,
    feed,
    onSuccess: (result) => onResourcesChanged(result.resources),
  });

  const busy = writeCredits.pending || addResource.pending || addAllResources.pending;

  const confirm = useCallback(() => {
    const pending = confirming;
    setConfirming(null);
    if (pending === "credits") writeCredits.run();
    else if (pending === "single") addResource.run();
    else if (pending === "all") addAllResources.run();
  }, [confirming, writeCredits, addResource, addAllResources]);

  return {
    creditsInput,
    creditsValid,
    editCredits,
    syncCreditsInput,
    selectedResourceId,
    setSelectedResourceId,
    selectedResource,
    resourceAmount,
    setResourceAmount,
    resourceAmountValid,
    allResourcesAmount,
    setAllResourcesAmount,
    allResourcesAmountValid,
    writeCredits,
    addResource,
    addAllResources,
    busy,
    confirming,
    setConfirming,
    confirm,
  };
}

export type InventoryActions = ReturnType<typeof useInventoryActions>;
