import { Award, Gauge, KeyRound } from "lucide-react";

import { ActionControl, SemanticsBadge } from "@/components/trainer/action-control";
import {
  DisabledOption,
  OptionRow,
  TrainerSection,
  ValuePair,
} from "@/components/trainer/primitives";
import type { ProgressionActions } from "@/hooks/useProgressionActions";
import { NO_VALUE } from "@/lib/weapon-format";
import type { TrainerConnection } from "@/state/trainer-state";

type PlayerSectionProps = {
  compact: boolean;
  connection: TrainerConnection;
  progression: ProgressionActions;
};

export function PlayerSection({ compact, connection, progression }: PlayerSectionProps) {
  const profile = connection.kind === "attached" ? connection.profileId : NO_VALUE;

  return (
    <TrainerSection icon={Gauge} title="Player" compact={compact}>
      <div className="option-grid">
        <OptionRow
          icon={Gauge}
          title="Max Class Level"
          info="Permanently sets Driller, Engineer, Gunner and Scout to level 25 (315,000 XP). Promotions are preserved and Bosco is ignored. Space Rig only; a save backup is created first."
        >
          <ValuePair label="Target" value="Level 25" />
          <SemanticsBadge semantics="persistent" />
          <ActionControl
            action={progression.maxLevel}
            semantics="persistent"
            label="Max all"
            pendingLabel="Applying"
            icon={Gauge}
            onActivate={() => progression.setConfirming("maxLevel")}
          />
        </OptionRow>

        <OptionRow
          icon={Award}
          title="Promote All Classes"
          info="Uses the game's native RetireCharacter routine to promote Driller, Engineer, Gunner and Scout once. Each class returns to level 1 and receives the normal promotion progression. A save backup is created first."
        >
          <ValuePair label="Effect" value="+1 each" />
          <SemanticsBadge semantics="persistent" />
          <ActionControl
            action={progression.promote}
            semantics="persistent"
            label="Promote all"
            pendingLabel="Promoting"
            icon={Award}
            onActivate={() => progression.setConfirming("promote")}
          />
        </OptionRow>

        <OptionRow
          icon={KeyRound}
          title="Unlock All Perks"
          info="Permanently acquires every available perk rank through the game's native perk routine. Equipped perk loadouts are preserved. Space Rig only; a save backup is created first."
        >
          <ValuePair label="Profile" value={profile} wide />
          <SemanticsBadge semantics="persistent" />
          <ActionControl
            action={progression.perks}
            semantics="persistent"
            label="Unlock perks"
            pendingLabel="Unlocking"
            icon={KeyRound}
            onActivate={() => progression.setConfirming("perks")}
          />
        </OptionRow>
      </div>

      {!compact && (
        <div className="option-grid option-grid-border">
          <DisabledOption
            title="Health & shield"
            info="Reserved for the upcoming PlayerCharacter mapping."
          />
          <DisabledOption
            title="Movement"
            info="Reserved for movement speed and traversal controls."
          />
        </div>
      )}
    </TrainerSection>
  );
}
