/**
 * Composition root (SPEC-012).
 *
 * `App` liga hooks a seções e não faz mais nada: sem polling, sem chamadas ao
 * backend, sem lógica de habilitação. Cada responsabilidade vive em um hook
 * próprio, e as seções são majoritariamente apresentacionais.
 */

import { Boxes, Crosshair, Gauge, Zap } from "lucide-react";
import { useEffect, useState } from "react";

import { TrainerNotifications } from "@/components/trainer/notifications";
import { InventorySection } from "@/components/trainer/sections/inventory-section";
import { PlayerSection } from "@/components/trainer/sections/player-section";
import { WeaponsSection } from "@/components/trainer/sections/weapons-section";
import { TrainerDialogs } from "@/components/trainer/trainer-dialogs";
import { TrainerStatusBar } from "@/components/trainer/trainer-status";
import { useActionFeed } from "@/hooks/useActionFeed";
import { useInventoryActions } from "@/hooks/useInventoryActions";
import { useProgressionActions } from "@/hooks/useProgressionActions";
import { useTrainerConnection } from "@/hooks/useTrainerConnection";
import { useTrainerHotkeys } from "@/hooks/useTrainerHotkeys";
import { useTrainerSnapshot } from "@/hooks/useTrainerSnapshot";
import { useWeaponRuntime } from "@/hooks/useWeaponRuntime";
import { cn } from "@/lib/utils";
import type { ModuleId } from "@/types/trainer";

const NAVIGATION = [
  { id: "all", label: "All", icon: Zap },
  { id: "player", label: "Player", icon: Gauge },
  { id: "inventory", label: "Inventory", icon: Boxes },
  { id: "weapons", label: "Weapons", icon: Crosshair },
] as const satisfies ReadonlyArray<{ id: ModuleId; label: string; icon: typeof Zap }>;

export default function App() {
  const [activeModule, setActiveModule] = useState<ModuleId>("all");

  const feed = useActionFeed();
  const state = useTrainerConnection();
  const progression = useProgressionActions(state, feed);

  // Ler o save enquanto uma mutação roda competiria com a própria operação.
  const snapshot = useTrainerSnapshot(state, progression.busy);

  const inventory = useInventoryActions({
    state,
    feed,
    resources: snapshot.resources,
    onResourcesChanged: snapshot.setResources,
    onCreditsChanged: () => void snapshot.refresh(),
  });

  const runtime = useWeaponRuntime({
    state,
    feed,
    clip: snapshot.clip,
    damage: snapshot.damage,
    setClip: snapshot.setClip,
    setDamage: snapshot.setDamage,
  });

  const hotkeys = useTrainerHotkeys(feed);

  // O campo de créditos acompanha o jogo até o usuário começar a editar.
  const currentCredits = snapshot.credits?.credits;
  const { syncCreditsInput } = inventory;
  useEffect(() => {
    if (currentCredits !== undefined) syncCreditsInput(currentCredits);
  }, [currentCredits, syncCreditsInput]);

  const showPlayer = activeModule === "all" || activeModule === "player";
  const showInventory = activeModule === "all" || activeModule === "inventory";
  const showWeapons = activeModule === "all" || activeModule === "weapons";

  return (
    <div className="dense-trainer min-h-screen bg-background text-foreground">
      <header className="trainer-header">
        <div className="trainer-topbar">
          <div className="trainer-brand">
            <span className="logo-mark" aria-hidden>
              DRG
            </span>
            <span className="trainer-wordmark">Runtime Trainer</span>
          </div>
          <nav className="module-tabs" aria-label="Trainer modules">
            {NAVIGATION.map((item) => {
              const Icon = item.icon;
              return (
                <button
                  key={item.id}
                  type="button"
                  aria-current={activeModule === item.id ? "page" : undefined}
                  className={cn("module-tab", activeModule === item.id && "module-tab-active")}
                  onClick={() => setActiveModule(item.id)}
                >
                  <Icon className="size-3" aria-hidden />
                  {item.label}
                </button>
              );
            })}
          </nav>
        </div>
        <TrainerStatusBar connection={state.connection} />
      </header>

      <main className="trainer-content">
        {showPlayer && (
          <PlayerSection
            compact={activeModule === "all"}
            connection={state.connection}
            progression={progression}
          />
        )}

        {showInventory && (
          <InventorySection
            compact={activeModule === "all"}
            credits={snapshot.credits}
            resources={snapshot.resources}
            inventory={inventory}
            onOpenInventory={() => setActiveModule("inventory")}
          />
        )}

        {showWeapons && (
          <WeaponsSection
            compact={activeModule === "all"}
            clip={snapshot.clip}
            damage={snapshot.damage}
            runtime={runtime}
            hotkeys={hotkeys}
            progression={progression}
          />
        )}

        <TrainerNotifications entries={feed.entries} onDismiss={feed.dismiss} />
      </main>

      <TrainerDialogs
        credits={snapshot.credits}
        inventory={inventory}
        progression={progression}
      />
    </div>
  );
}
