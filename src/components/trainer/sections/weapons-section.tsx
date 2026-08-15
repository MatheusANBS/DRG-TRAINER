import {
  Crosshair,
  Infinity as InfinityIcon,
  KeyRound,
  Sparkles,
  Wrench,
  Zap,
} from "lucide-react";

import { ActionControl, SemanticsBadge } from "@/components/trainer/action-control";
import {
  DisabledOption,
  HotkeyButton,
  OptionRow,
  StatePill,
  TrainerSection,
  ValuePair,
} from "@/components/trainer/primitives";
import {
  NO_VALUE,
  cleanWeaponName,
  formatClip,
  formatDamage,
} from "@/lib/weapon-format";
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { ProgressionActions } from "@/hooks/useProgressionActions";
import type { TrainerHotkeys } from "@/hooks/useTrainerHotkeys";
import type { WeaponRuntime } from "@/hooks/useWeaponRuntime";
import type { ClipStatus, DamageStatus } from "@/types/trainer";

type WeaponsSectionProps = {
  compact: boolean;
  clip: ClipStatus | null;
  damage: DamageStatus | null;
  runtime: WeaponRuntime;
  hotkeys: TrainerHotkeys;
  progression: ProgressionActions;
};

/** Switch com explicação obrigatória quando está desabilitado (SPEC-019). */
function RuntimeToggle({
  checked,
  disabled,
  blockedReason,
  label,
  onChange,
}: {
  checked: boolean;
  disabled: boolean;
  blockedReason: string | null;
  label: string;
  onChange: (enabled: boolean) => void;
}) {
  const control = (
    <Switch
      checked={checked}
      disabled={disabled}
      onCheckedChange={onChange}
      aria-label={`Toggle ${label}`}
    />
  );
  if (!blockedReason) return control;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex">{control}</span>
      </TooltipTrigger>
      <TooltipContent side="top" className="trainer-tooltip max-w-64">
        {blockedReason}
      </TooltipContent>
    </Tooltip>
  );
}

export function WeaponsSection({
  compact,
  clip,
  damage,
  runtime,
  hotkeys,
  progression,
}: WeaponsSectionProps) {
  const weaponName = damage?.weaponName ?? clip?.weaponName;

  return (
    <TrainerSection icon={Crosshair} title="Weapons" compact={compact}>
      <div className="option-grid">
        <OptionRow
          icon={InfinityIcon}
          title="Infinite Magazine"
          info="Follows EquippedActor and keeps ClipCount equal to ClipSize every 35 ms. Reserve ammunition is not changed. Turning it off restores the game's own behaviour."
          active={runtime.clipEnabled}
        >
          <StatePill active={runtime.clipEnabled} label={runtime.clipEnabled ? "On" : "Off"} />
          <RuntimeToggle
            checked={runtime.clipEnabled}
            disabled={runtime.togglingClip || runtime.clipBlocked !== null}
            blockedReason={runtime.clipBlocked}
            label="Infinite Magazine"
            onChange={runtime.toggleClip}
          />
          <HotkeyButton
            value={hotkeys.clipHotkey}
            busy={hotkeys.clipBusy}
            label="Infinite Magazine"
            onCapture={hotkeys.saveClipHotkey}
          />
        </OptionRow>

        <OptionRow
          icon={Zap}
          title="Weapon Damage"
          info="Follows the equipped weapon through Unreal reflection and keeps active direct or radial DamageComponent fields at 9999. Hitscan and projectile class defaults are resolved automatically."
          active={runtime.damageEnabled}
        >
          <ValuePair
            label="Damage"
            value={runtime.damageEnabled ? "9,999" : formatDamage(damage)}
          />
          <StatePill active={runtime.damageEnabled} label={runtime.damageEnabled ? "On" : "Off"} />
          <RuntimeToggle
            checked={runtime.damageEnabled}
            disabled={
              runtime.togglingDamage ||
              runtime.damageBlocked !== null ||
              (damage != null && !damage.available && !runtime.damageEnabled)
            }
            blockedReason={
              runtime.damageBlocked ??
              (damage != null && !damage.available && !runtime.damageEnabled
                ? damage.message
                : null)
            }
            label="Weapon Damage"
            onChange={runtime.toggleDamage}
          />
          <HotkeyButton
            value={hotkeys.damageHotkey}
            busy={hotkeys.damageBusy}
            label="Weapon Damage"
            onCapture={hotkeys.saveDamageHotkey}
          />
        </OptionRow>

        <OptionRow
          icon={Crosshair}
          title="Equipped weapon"
          info="Resolved dynamically through PlayerController → Pawn → InventoryComponent → EquippedActor."
        >
          <ValuePair
            label="Weapon"
            value={weaponName ? cleanWeaponName(weaponName) : NO_VALUE}
          />
          <ValuePair label="Clip" value={formatClip(clip)} />
          <ValuePair
            label="Damage fields"
            value={damage?.available ? String(damage.targetCount) : NO_VALUE}
          />
        </OptionRow>

        <OptionRow
          icon={KeyRound}
          title="Unlock All Weapons"
          info="Permanently unlocks all 26 primary and secondary weapons through the game's native routine. Space Rig only. A save backup is created first."
        >
          <SemanticsBadge semantics="persistent" />
          <ActionControl
            action={progression.weapons}
            semantics="persistent"
            label="Unlock all"
            pendingLabel="Unlocking"
            icon={KeyRound}
            onActivate={() => progression.setConfirming("weapons")}
          />
        </OptionRow>

        <OptionRow
          icon={Wrench}
          title="Unlock All Gear Modifications"
          info="Permanently acquires every purchasable weapon and equipment modification through Cheat_UnlockAllUpgrades. Equipped builds are preserved. Space Rig only; a save backup is created first."
        >
          <SemanticsBadge semantics="persistent" />
          <ActionControl
            action={progression.gear}
            semantics="persistent"
            label="Unlock mods"
            pendingLabel="Unlocking"
            icon={Wrench}
            onActivate={() => progression.setConfirming("gear")}
          />
        </OptionRow>

        <OptionRow
          icon={Sparkles}
          title="Unlock All Overclocks & Cosmetics"
          info="Permanently grants every weapon overclock, skin, vanity cosmetic and victory pose, including repair of schematics already marked as forged. The unsafe analytics path is bypassed. Space Rig only; a save backup is created first."
        >
          <SemanticsBadge semantics="persistent" />
          <ActionControl
            action={progression.schematics}
            semantics="persistent"
            label="Unlock all"
            pendingLabel="Unlocking"
            icon={Sparkles}
            onActivate={() => progression.setConfirming("schematics")}
          />
        </OptionRow>
      </div>

      {!compact && (
        <div className="option-grid option-grid-border">
          <DisabledOption
            title="Reserve ammunition"
            info="AmmoCount monitoring and an independent freeze will be mapped here."
          />
        </div>
      )}
    </TrainerSection>
  );
}
