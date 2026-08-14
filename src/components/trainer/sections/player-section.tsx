import { Activity, Award, Gauge, KeyRound, Loader2, ShieldCheck } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  DisabledOption,
  OptionRow,
  StatePill,
  TrainerSection,
  ValuePair,
} from "@/components/trainer/primitives";
import type { ConnectionState, CreditSnapshot } from "@/types/trainer";

type PlayerSectionProps = {
  compact: boolean;
  snapshot: CreditSnapshot | null;
  connection: ConnectionState;
  leveling: boolean;
  promoteBusy: boolean;
  perksBusy: boolean;
  onMaxLevel: () => void;
  onPromote: () => void;
  onUnlockPerks: () => void;
};

export function PlayerSection(props: PlayerSectionProps) {
  const { compact, snapshot, connection, leveling, promoteBusy, perksBusy } = props;
  return (
    <TrainerSection icon={Gauge} title="Player" compact={compact}>
      <div className="option-grid">
        <OptionRow icon={Activity} title="Runtime session" info="Read-only connection status for FSD-Win64-Shipping.exe. No memory is written from this option.">
          <ValuePair label="Process" value={snapshot ? `${snapshot.pid}` : "--"} />
          <StatePill active={connection === "connected"} label={connection === "connected" ? "Live" : "Waiting"} />
        </OptionRow>
        <OptionRow icon={ShieldCheck} title="Build validation" info="The executable SHA-256 must match the mapped game build. Writes are blocked after an update until offsets are verified again.">
          <ValuePair label="Module" value={snapshot?.moduleBase ?? "--"} wide />
          <StatePill active={snapshot?.buildValid ?? false} label="Verified" />
        </OptionRow>
        <OptionRow icon={Gauge} title="Max Class Level" info="Permanently sets Driller, Engineer, Gunner and Scout to level 25 (315,000 XP). Promotions are preserved and Bosco is ignored. Space Rig only; a save backup is created first.">
          <ValuePair label="Target" value="Level 25" />
          <Button type="button" size="xs" disabled={!snapshot || leveling} onClick={props.onMaxLevel}>
            {leveling ? <Loader2 className="animate-spin" /> : <Gauge />}
            {leveling ? "Applying" : "Max all"}
          </Button>
        </OptionRow>
        <OptionRow icon={Award} title="Promote All Classes" info="Uses the game's native RetireCharacter routine to promote Driller, Engineer, Gunner and Scout once. Each class returns to level 1 and receives the normal promotion progression. A save backup is created first.">
          <ValuePair label="Effect" value="+1 each" />
          <Button type="button" size="xs" disabled={!snapshot || promoteBusy} onClick={props.onPromote}>
            {promoteBusy ? <Loader2 className="animate-spin" /> : <Award />}
            {promoteBusy ? "Promoting" : "Promote all"}
          </Button>
        </OptionRow>
        <OptionRow icon={KeyRound} title="Unlock All Perks" info="Permanently acquires every available perk rank through the game's native perk routine. Equipped perk loadouts are preserved. Space Rig only; a save backup is created first.">
          <StatePill active={false} label="Permanent" />
          <Button type="button" size="xs" disabled={!snapshot || perksBusy} onClick={props.onUnlockPerks}>
            {perksBusy ? <Loader2 className="animate-spin" /> : <KeyRound />}
            {perksBusy ? "Unlocking" : "Unlock perks"}
          </Button>
        </OptionRow>
      </div>
      {!compact && (
        <div className="option-grid option-grid-border">
          <DisabledOption title="Health & shield" info="Reserved for the upcoming PlayerCharacter mapping." />
          <DisabledOption title="Movement" info="Reserved for movement speed and traversal controls." />
        </div>
      )}
    </TrainerSection>
  );
}
