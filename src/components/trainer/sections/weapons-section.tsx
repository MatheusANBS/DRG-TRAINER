import { Crosshair, Infinity as InfinityIcon, KeyRound, Loader2, Sparkles, Wrench, Zap } from "lucide-react";

import { cleanWeaponName, DisabledOption, formatClip, formatDamage, HotkeyButton, OptionRow, StatePill, TrainerSection, ValuePair } from "@/components/trainer/primitives";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import type { ClipStatus, CreditSnapshot, DamageStatus } from "@/types/trainer";

type WeaponsSectionProps = {
  compact: boolean;
  snapshot: CreditSnapshot | null;
  clip: ClipStatus | null;
  damage: DamageStatus | null;
  togglingClip: boolean;
  togglingDamage: boolean;
  clipHotkey: string;
  damageHotkey: string;
  hotkeyBusy: boolean;
  damageHotkeyBusy: boolean;
  unlocking: boolean;
  gearBusy: boolean;
  schematicBusy: boolean;
  onToggleClip: (enabled: boolean) => void;
  onToggleDamage: (enabled: boolean) => void;
  onClipHotkey: (hotkey: string) => Promise<void>;
  onDamageHotkey: (hotkey: string) => Promise<void>;
  onUnlockWeapons: () => void;
  onUnlockGear: () => void;
  onUnlockSchematics: () => void;
};

export function WeaponsSection(props: WeaponsSectionProps) {
  const { compact, snapshot, clip, damage } = props;
  return (
    <TrainerSection icon={Crosshair} title="Weapons" compact={compact}>
      <div className="option-grid">
        <OptionRow icon={InfinityIcon} title="Infinite Magazine" info="Follows EquippedActor and keeps ClipCount equal to ClipSize every 35 ms. Reserve ammunition is not changed." active={clip?.enabled}>
          <StatePill active={clip?.enabled ?? false} label={clip?.enabled ? "On" : "Off"} />
          <Switch checked={clip?.enabled ?? false} disabled={props.togglingClip} onCheckedChange={props.onToggleClip} aria-label="Toggle Infinite Magazine" />
          <HotkeyButton value={props.clipHotkey} busy={props.hotkeyBusy} onCapture={props.onClipHotkey} />
        </OptionRow>
        <OptionRow icon={Zap} title="Weapon Damage" info="Follows the equipped weapon through Unreal reflection and keeps active direct or radial DamageComponent fields at 9999. Hitscan and projectile class defaults are resolved automatically." active={damage?.enabled}>
          <ValuePair label="Damage" value={damage?.enabled ? "9,999" : formatDamage(damage)} />
          <StatePill active={damage?.enabled ?? false} label={damage?.enabled ? "On" : "Off"} />
          <Switch checked={damage?.enabled ?? false} disabled={props.togglingDamage || (damage != null && !damage.available && !damage.enabled)} onCheckedChange={props.onToggleDamage} aria-label="Toggle Weapon Damage" />
          <HotkeyButton value={props.damageHotkey} busy={props.damageHotkeyBusy} label="Weapon Damage" onCapture={props.onDamageHotkey} />
        </OptionRow>
        <OptionRow icon={Crosshair} title="Equipped weapon" info="Resolved dynamically through PlayerController → Pawn → InventoryComponent → EquippedActor.">
          <ValuePair label="Weapon" value={(damage?.weaponName ?? clip?.weaponName) ? cleanWeaponName((damage?.weaponName ?? clip?.weaponName)!) : "--"} />
          <ValuePair label="Clip" value={formatClip(clip)} />
          <ValuePair label="Damage fields" value={damage?.available ? String(damage.targetCount) : "--"} />
        </OptionRow>
        <OptionRow icon={KeyRound} title="Unlock All Weapons" info="Permanently unlocks all 26 primary and secondary weapons through the game's native routine. Space Rig only. A save backup is created first.">
          <StatePill active={false} label="Permanent" />
          <Button type="button" size="xs" disabled={!snapshot || props.unlocking} onClick={props.onUnlockWeapons}>
            {props.unlocking ? <Loader2 className="animate-spin" /> : <KeyRound />}{props.unlocking ? "Unlocking" : "Unlock all"}
          </Button>
        </OptionRow>
        <OptionRow icon={Wrench} title="Unlock All Gear Modifications" info="Permanently acquires every purchasable weapon and equipment modification through Cheat_UnlockAllUpgrades. Equipped builds are preserved. Space Rig only; a save backup is created first.">
          <StatePill active={false} label="Permanent" />
          <Button type="button" size="xs" disabled={!snapshot || props.gearBusy} onClick={props.onUnlockGear}>
            {props.gearBusy ? <Loader2 className="animate-spin" /> : <Wrench />}{props.gearBusy ? "Unlocking" : "Unlock mods"}
          </Button>
        </OptionRow>
        <OptionRow icon={Sparkles} title="Unlock All Overclocks & Cosmetics" info="Permanently grants every weapon overclock, skin, vanity cosmetic and victory pose, including repair of schematics already marked as forged. The unsafe analytics path is bypassed. Space Rig only; a save backup is created first.">
          <StatePill active={false} label="Permanent" />
          <Button type="button" size="xs" disabled={!snapshot || props.schematicBusy} onClick={props.onUnlockSchematics}>
            {props.schematicBusy ? <Loader2 className="animate-spin" /> : <Sparkles />}{props.schematicBusy ? "Unlocking" : "Unlock all"}
          </Button>
        </OptionRow>
      </div>
      {!compact && <div className="option-grid option-grid-border"><DisabledOption title="Reserve ammunition" info="AmmoCount monitoring and an independent freeze will be mapped here." /></div>}
    </TrainerSection>
  );
}
