import {
  Boxes,
  Crosshair,
  Gauge,
  Info,
  ShieldCheck,
  Zap,
} from "lucide-react";
import { FormEvent, useCallback, useEffect, useRef, useState } from "react";

import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { TrainerDialogs } from "@/components/trainer/trainer-dialogs";
import { TrainerNotifications } from "@/components/trainer/notifications";
import { StatusDot } from "@/components/trainer/primitives";
import { InventorySection } from "@/components/trainer/sections/inventory-section";
import { PlayerSection } from "@/components/trainer/sections/player-section";
import { WeaponsSection } from "@/components/trainer/sections/weapons-section";
import { cn } from "@/lib/utils";
import { invokeApp } from "@/lib/trainer-api";
import { errorText } from "@/lib/format";
import { useAutoDismiss } from "@/hooks/use-auto-dismiss";
import type {
  ClipStatus,
  ConnectionState,
  CreditSnapshot,
  DamageStatus,
  MaxClassLevelResult,
  ModuleId,
  PermanentUnlockResult,
  PromoteAllResult,
  ResourceSnapshot,
  ResourceWriteMode,
  ResourceWriteResult,
  SchematicUnlockResult,
  UnlockAllResult,
  WriteResult,
} from "@/types/trainer";

const navigation = [
  { id: "all" as const, label: "All", icon: Zap },
  { id: "player" as const, label: "Player", icon: Gauge },
  { id: "inventory" as const, label: "Inventory", icon: Boxes },
  { id: "weapons" as const, label: "Weapons", icon: Crosshair },
];

export default function App() {
  const [activeModule, setActiveModule] = useState<ModuleId>("all");
  const [snapshot, setSnapshot] = useState<CreditSnapshot | null>(null);
  const [clip, setClip] = useState<ClipStatus | null>(null);
  const [damage, setDamage] = useState<DamageStatus | null>(null);
  const [connection, setConnection] = useState<ConnectionState>("connecting");
  const [status, setStatus] = useState("Connecting");
  const [amount, setAmount] = useState("0");
  const [writing, setWriting] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [lastWrite, setLastWrite] = useState<WriteResult | null>(null);
  const [resources, setResources] = useState<ResourceSnapshot[]>([]);
  const [selectedResourceId, setSelectedResourceId] = useState("bismor");
  const [resourceAmount, setResourceAmount] = useState("1000");
  const [allResourcesAmount, setAllResourcesAmount] = useState("1000");
  const [resourceBusy, setResourceBusy] = useState<ResourceWriteMode | null>(null);
  const [resourceConfirming, setResourceConfirming] = useState<ResourceWriteMode | null>(null);
  const [resourceError, setResourceError] = useState<string | null>(null);
  const [resourceResult, setResourceResult] = useState<ResourceWriteResult | null>(null);
  const [togglingClip, setTogglingClip] = useState(false);
  const [clipError, setClipError] = useState<string | null>(null);
  const [clipHotkey, setClipHotkey] = useState(() => localStorage.getItem("drg.clipHotkey") ?? "F8");
  const [togglingDamage, setTogglingDamage] = useState(false);
  const [damageError, setDamageError] = useState<string | null>(null);
  const [damageHotkey, setDamageHotkey] = useState(() => localStorage.getItem("drg.damageHotkey") ?? "F9");
  const [damageHotkeyError, setDamageHotkeyError] = useState<string | null>(null);
  const [damageHotkeyBusy, setDamageHotkeyBusy] = useState(false);
  const [unlockConfirming, setUnlockConfirming] = useState(false);
  const [unlocking, setUnlocking] = useState(false);
  const [unlockError, setUnlockError] = useState<string | null>(null);
  const [unlockResult, setUnlockResult] = useState<UnlockAllResult | null>(null);
  const [levelConfirming, setLevelConfirming] = useState(false);
  const [leveling, setLeveling] = useState(false);
  const [levelError, setLevelError] = useState<string | null>(null);
  const [levelResult, setLevelResult] = useState<MaxClassLevelResult | null>(null);
  const [perksConfirming, setPerksConfirming] = useState(false);
  const [perksBusy, setPerksBusy] = useState(false);
  const [perksError, setPerksError] = useState<string | null>(null);
  const [perksResult, setPerksResult] = useState<PermanentUnlockResult | null>(null);
  const [gearConfirming, setGearConfirming] = useState(false);
  const [gearBusy, setGearBusy] = useState(false);
  const [gearError, setGearError] = useState<string | null>(null);
  const [gearResult, setGearResult] = useState<PermanentUnlockResult | null>(null);
  const [schematicConfirming, setSchematicConfirming] = useState(false);
  const [schematicBusy, setSchematicBusy] = useState(false);
  const [schematicError, setSchematicError] = useState<string | null>(null);
  const [schematicResult, setSchematicResult] = useState<SchematicUnlockResult | null>(null);
  const [promoteConfirming, setPromoteConfirming] = useState(false);
  const [promoteBusy, setPromoteBusy] = useState(false);
  const [promoteError, setPromoteError] = useState<string | null>(null);
  const [promoteResult, setPromoteResult] = useState<PromoteAllResult | null>(null);
  const [hotkeyError, setHotkeyError] = useState<string | null>(null);
  const [hotkeyBusy, setHotkeyBusy] = useState(false);
  const amountDirty = useRef(false);
  const pollBusy = useRef(false);
  const lastClipPollingError = useRef<string | null>(null);
  const lastDamagePollingError = useRef<string | null>(null);

  useAutoDismiss(lastWrite, setLastWrite);
  useAutoDismiss(resourceResult, setResourceResult);
  useAutoDismiss(unlockResult, setUnlockResult);
  useAutoDismiss(levelResult, setLevelResult);
  useAutoDismiss(perksResult, setPerksResult);
  useAutoDismiss(gearResult, setGearResult);
  useAutoDismiss(schematicResult, setSchematicResult);
  useAutoDismiss(promoteResult, setPromoteResult);

  const poll = useCallback(async () => {
    if (writing || resourceBusy || unlocking || leveling || perksBusy || gearBusy || schematicBusy || promoteBusy || pollBusy.current) return;
    pollBusy.current = true;
    try {
      const [creditsResult, resourcesResult, clipResult, damageResult] = await Promise.allSettled([
        invokeApp<CreditSnapshot>("read_credits"),
        invokeApp<ResourceSnapshot[]>("get_resources"),
        invokeApp<ClipStatus>("get_clip_status"),
        invokeApp<DamageStatus>("get_damage_status"),
      ]);

      if (creditsResult.status === "fulfilled") {
        setSnapshot(creditsResult.value);
        setConnection("connected");
        setStatus("Attached");
        if (!amountDirty.current) setAmount(String(creditsResult.value.credits));
      } else {
        const message = errorText(creditsResult.reason);
        setSnapshot(null);
        setConnection(message.startsWith("Aguardando") ? "waiting" : "error");
        setStatus(message.startsWith("Aguardando") ? "Waiting for process" : message);
      }

      if (resourcesResult.status === "fulfilled") {
        setResources(resourcesResult.value);
        setResourceError(null);
      } else if (creditsResult.status === "fulfilled") {
        setResourceError(errorText(resourcesResult.reason));
      }

      if (clipResult.status === "fulfilled") {
        setClip(clipResult.value);
        setClipError(null);
        lastClipPollingError.current = null;
      } else {
        const message = errorText(clipResult.reason);
        if (lastClipPollingError.current !== message) {
          lastClipPollingError.current = message;
          setClipError(message);
        }
      }

      if (damageResult.status === "fulfilled") {
        setDamage(damageResult.value);
        setDamageError(null);
        lastDamagePollingError.current = null;
      } else {
        const message = errorText(damageResult.reason);
        if (lastDamagePollingError.current !== message) {
          lastDamagePollingError.current = message;
          setDamageError(message);
        }
      }
    } finally {
      pollBusy.current = false;
    }
  }, [writing, resourceBusy, unlocking, leveling, perksBusy, gearBusy, schematicBusy, promoteBusy]);

  useEffect(() => {
    void poll();
    const interval = window.setInterval(() => void poll(), 500);
    return () => window.clearInterval(interval);
  }, [poll]);

  useEffect(() => {
    void invokeApp("set_clip_hotkey", { hotkey: clipHotkey }).catch((error) => {
      setHotkeyError(errorText(error));
    });
  }, []);

  useEffect(() => {
    void invokeApp("set_damage_hotkey", { hotkey: damageHotkey }).catch((error) => {
      setDamageHotkeyError(errorText(error));
    });
  }, []);

  const visibleError =
    resourceError ??
    promoteError ??
    schematicError ??
    gearError ??
    perksError ??
    levelError ??
    unlockError ??
    clipError ??
    hotkeyError ??
    damageError ??
    damageHotkeyError;

  useEffect(() => {
    if (!visibleError) return;
    const timer = window.setTimeout(() => {
      setPromoteError(null);
      setResourceError(null);
      setSchematicError(null);
      setGearError(null);
      setPerksError(null);
      setLevelError(null);
      setUnlockError(null);
      setClipError(null);
      setHotkeyError(null);
      setDamageError(null);
      setDamageHotkeyError(null);
    }, 6_000);
    return () => window.clearTimeout(timer);
  }, [visibleError]);

  const parsedAmount = Number(amount);
  const amountValid =
    amount.trim() !== "" &&
    Number.isInteger(parsedAmount) &&
    parsedAmount >= 0 &&
    parsedAmount <= 2_147_483_647;
  const parsedResourceAmount = Number(resourceAmount);
  const parsedAllResourcesAmount = Number(allResourcesAmount);
  const resourceAmountValid =
    resourceAmount.trim() !== "" &&
    Number.isInteger(parsedResourceAmount) &&
    parsedResourceAmount >= 1 &&
    parsedResourceAmount <= 1_000_000;
  const allResourcesAmountValid =
    allResourcesAmount.trim() !== "" &&
    Number.isInteger(parsedAllResourcesAmount) &&
    parsedAllResourcesAmount >= 1 &&
    parsedAllResourcesAmount <= 1_000_000;
  const selectedResource = resources.find((resource) => resource.id === selectedResourceId);

  const requestWrite = (event: FormEvent) => {
    event.preventDefault();
    if (snapshot && amountValid) setConfirming(true);
  };

  const applyWrite = async () => {
    setConfirming(false);
    setWriting(true);
    setLastWrite(null);
    try {
      const result = await invokeApp<WriteResult>("set_credits", { value: parsedAmount });
      setLastWrite(result);
      setSnapshot((current) => current ? { ...current, credits: result.current, address: result.address } : current);
      amountDirty.current = false;
      setAmount(String(result.current));
      setStatus("Write verified");
    } catch (error) {
      setConnection("error");
      setStatus(errorText(error));
    } finally {
      setWriting(false);
    }
  };

  const applyResourceWrite = async () => {
    const mode = resourceConfirming;
    if (!mode) return;
    setResourceConfirming(null);
    setResourceBusy(mode);
    setResourceError(null);
    setResourceResult(null);
    try {
      const result = mode === "all"
        ? await invokeApp<ResourceWriteResult>("add_all_resources", { amount: parsedAllResourcesAmount })
        : await invokeApp<ResourceWriteResult>("add_resource", {
            resourceId: selectedResourceId,
            amount: parsedResourceAmount,
          });
      setResources(result.resources);
      setResourceResult(result);
      setStatus(mode === "all" ? "Resources added" : `${selectedResource?.name ?? "Resource"} added`);
    } catch (error) {
      setResourceError(errorText(error));
    } finally {
      setResourceBusy(null);
    }
  };

  const toggleInfiniteMagazine = async (enabled: boolean) => {
    setTogglingClip(true);
    setClipError(null);
    setClip((current) => current ? { ...current, enabled } : current);
    try {
      setClip(await invokeApp<ClipStatus>("set_infinite_magazine", { enabled }));
    } catch (error) {
      setClipError(errorText(error));
      setClip((current) => current ? { ...current, enabled: !enabled } : current);
    } finally {
      setTogglingClip(false);
    }
  };

  const saveClipHotkey = async (hotkey: string) => {
    setHotkeyBusy(true);
    setHotkeyError(null);
    try {
      await invokeApp("set_clip_hotkey", { hotkey });
      setClipHotkey(hotkey);
      localStorage.setItem("drg.clipHotkey", hotkey);
    } catch (error) {
      setHotkeyError(errorText(error));
    } finally {
      setHotkeyBusy(false);
    }
  };

  const toggleWeaponDamage = async (enabled: boolean) => {
    setTogglingDamage(true);
    setDamageError(null);
    setDamage((current) => current ? { ...current, enabled } : current);
    try {
      setDamage(await invokeApp<DamageStatus>("set_weapon_damage", { enabled }));
    } catch (error) {
      setDamageError(errorText(error));
      setDamage((current) => current ? { ...current, enabled: !enabled } : current);
    } finally {
      setTogglingDamage(false);
    }
  };

  const saveDamageHotkey = async (hotkey: string) => {
    setDamageHotkeyBusy(true);
    setDamageHotkeyError(null);
    try {
      await invokeApp("set_damage_hotkey", { hotkey });
      setDamageHotkey(hotkey);
      localStorage.setItem("drg.damageHotkey", hotkey);
    } catch (error) {
      setDamageHotkeyError(errorText(error));
    } finally {
      setDamageHotkeyBusy(false);
    }
  };

  const applyUnlockAll = async () => {
    setUnlockConfirming(false);
    setUnlocking(true);
    setUnlockError(null);
    setUnlockResult(null);
    try {
      const result = await invokeApp<UnlockAllResult>("unlock_all_weapons");
      setUnlockResult(result);
      setStatus("Weapons unlocked");
    } catch (error) {
      setUnlockError(errorText(error));
    } finally {
      setUnlocking(false);
    }
  };

  const applyMaxClassLevel = async () => {
    setLevelConfirming(false);
    setLeveling(true);
    setLevelError(null);
    setLevelResult(null);
    try {
      const result = await invokeApp<MaxClassLevelResult>("max_class_level");
      setLevelResult(result);
      setStatus("Classes at level 25");
    } catch (error) {
      setLevelError(errorText(error));
    } finally {
      setLeveling(false);
    }
  };

  const applyUnlockPerks = async () => {
    setPerksConfirming(false);
    setPerksBusy(true);
    setPerksError(null);
    setPerksResult(null);
    try {
      setPerksResult(await invokeApp<PermanentUnlockResult>("unlock_all_perks"));
      setStatus("Perks unlocked");
    } catch (error) {
      setPerksError(errorText(error));
    } finally {
      setPerksBusy(false);
    }
  };

  const applyUnlockGear = async () => {
    setGearConfirming(false);
    setGearBusy(true);
    setGearError(null);
    setGearResult(null);
    try {
      setGearResult(await invokeApp<PermanentUnlockResult>("unlock_all_gear_modifications"));
      setStatus("Gear modifications unlocked");
    } catch (error) {
      setGearError(errorText(error));
    } finally {
      setGearBusy(false);
    }
  };

  const applyUnlockSchematics = async () => {
    setSchematicConfirming(false);
    setSchematicBusy(true);
    setSchematicError(null);
    setSchematicResult(null);
    try {
      setSchematicResult(await invokeApp<SchematicUnlockResult>("unlock_all_overclocks_and_cosmetics"));
      setStatus("Overclocks & cosmetics unlocked");
    } catch (error) {
      setSchematicError(errorText(error));
    } finally {
      setSchematicBusy(false);
    }
  };

  const applyPromoteAll = async () => {
    setPromoteConfirming(false);
    setPromoteBusy(true);
    setPromoteError(null);
    setPromoteResult(null);
    try {
      setPromoteResult(await invokeApp<PromoteAllResult>("promote_all_classes"));
      setStatus("Classes promoted");
    } catch (error) {
      setPromoteError(errorText(error));
    } finally {
      setPromoteBusy(false);
    }
  };

  const showPlayer = activeModule === "all" || activeModule === "player";
  const showInventory = activeModule === "all" || activeModule === "inventory";
  const showWeapons = activeModule === "all" || activeModule === "weapons";

  return (
    <div className="dense-trainer min-h-screen bg-background text-foreground">
      <header className="trainer-topbar">
        <div className="flex min-w-0 items-center gap-3">
          <div className="logo-mark grid size-7 shrink-0 place-items-center rounded border font-mono text-[8px] font-black">DRG</div>
          <nav className="module-tabs" aria-label="Trainer modules">
            {navigation.map((item) => {
              const Icon = item.icon;
              return (
                <button
                  key={item.id}
                  type="button"
                  aria-current={activeModule === item.id ? "page" : undefined}
                  className={cn("module-tab", activeModule === item.id && "module-tab-active")}
                  onClick={() => setActiveModule(item.id)}
                >
                  <Icon className="size-3" />
                  {item.label}
                </button>
              );
            })}
          </nav>
        </div>

        <div className="flex shrink-0 items-center gap-3 text-[10px] text-muted-foreground">
          <span className="hidden font-mono md:inline">{snapshot ? `PID ${snapshot.pid}` : "FSD-Win64-Shipping.exe"}</span>
          <span className="flex items-center gap-1.5 text-foreground">
            <StatusDot state={connection} /> {status}
          </span>
          <Tooltip>
            <TooltipTrigger asChild>
              <ShieldCheck className="size-3.5 text-muted-foreground" aria-label="Build status" />
            </TooltipTrigger>
            <TooltipContent side="bottom" className="trainer-tooltip">Build hash verified before memory access.</TooltipContent>
          </Tooltip>
        </div>
      </header>

      <main className="trainer-content">
        {showPlayer && (
          <PlayerSection
            compact={activeModule === "all"}
            snapshot={snapshot}
            connection={connection}
            leveling={leveling}
            promoteBusy={promoteBusy}
            perksBusy={perksBusy}
            onMaxLevel={() => setLevelConfirming(true)}
            onPromote={() => setPromoteConfirming(true)}
            onUnlockPerks={() => setPerksConfirming(true)}
          />
        )}

        {showInventory && (
          <InventorySection
            compact={activeModule === "all"}
            snapshot={snapshot}
            resources={resources}
            selectedResource={selectedResource}
            selectedResourceId={selectedResourceId}
            creditsAmount={amount}
            resourceAmount={resourceAmount}
            allResourcesAmount={allResourcesAmount}
            writing={writing}
            resourceBusy={resourceBusy}
            creditsAmountValid={amountValid}
            resourceAmountValid={resourceAmountValid}
            allResourcesAmountValid={allResourcesAmountValid}
            onOpenInventory={() => setActiveModule("inventory")}
            onCreditsSubmit={requestWrite}
            onCreditsAmountChange={(value) => {
              amountDirty.current = true;
              setAmount(value);
              setLastWrite(null);
            }}
            onResourceChange={setSelectedResourceId}
            onResourceAmountChange={setResourceAmount}
            onAllResourcesAmountChange={setAllResourcesAmount}
            onAddResource={() => setResourceConfirming("single")}
            onAddAllResources={() => setResourceConfirming("all")}
          />
        )}

        {showWeapons && (
          <WeaponsSection
            compact={activeModule === "all"}
            snapshot={snapshot}
            clip={clip}
            damage={damage}
            togglingClip={togglingClip}
            togglingDamage={togglingDamage}
            clipHotkey={clipHotkey}
            damageHotkey={damageHotkey}
            hotkeyBusy={hotkeyBusy}
            damageHotkeyBusy={damageHotkeyBusy}
            unlocking={unlocking}
            gearBusy={gearBusy}
            schematicBusy={schematicBusy}
            onToggleClip={(enabled) => void toggleInfiniteMagazine(enabled)}
            onToggleDamage={(enabled) => void toggleWeaponDamage(enabled)}
            onClipHotkey={saveClipHotkey}
            onDamageHotkey={saveDamageHotkey}
            onUnlockWeapons={() => setUnlockConfirming(true)}
            onUnlockGear={() => setGearConfirming(true)}
            onUnlockSchematics={() => setSchematicConfirming(true)}
          />
        )}

        <TrainerNotifications
          error={visibleError}
          creditWrite={lastWrite}
          weaponUnlock={unlockResult}
          resourceWrite={resourceResult}
          maxLevel={levelResult}
          perkUnlock={perksResult}
          gearUnlock={gearResult}
          schematicUnlock={schematicResult}
          promotion={promoteResult}
        />
      </main>

      <TrainerDialogs
        creditsOpen={confirming}
        resourceMode={resourceConfirming}
        weaponsOpen={unlockConfirming}
        maxLevelOpen={levelConfirming}
        perksOpen={perksConfirming}
        gearOpen={gearConfirming}
        schematicsOpen={schematicConfirming}
        promotionOpen={promoteConfirming}
        snapshot={snapshot}
        selectedResource={selectedResource}
        creditsAmount={parsedAmount}
        resourceAmount={parsedResourceAmount}
        allResourcesAmount={parsedAllResourcesAmount}
        onCreditsOpenChange={setConfirming}
        onResourceModeChange={setResourceConfirming}
        onWeaponsOpenChange={setUnlockConfirming}
        onMaxLevelOpenChange={setLevelConfirming}
        onPerksOpenChange={setPerksConfirming}
        onGearOpenChange={setGearConfirming}
        onSchematicsOpenChange={setSchematicConfirming}
        onPromotionOpenChange={setPromoteConfirming}
        onWriteCredits={() => void applyWrite()}
        onWriteResource={() => void applyResourceWrite()}
        onUnlockWeapons={() => void applyUnlockAll()}
        onMaxLevel={() => void applyMaxClassLevel()}
        onUnlockPerks={() => void applyUnlockPerks()}
        onUnlockGear={() => void applyUnlockGear()}
        onUnlockSchematics={() => void applyUnlockSchematics()}
        onPromote={() => void applyPromoteAll()}
      />
    </div>
  );
}
