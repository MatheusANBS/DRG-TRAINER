import { Info, Keyboard, type LucideIcon, Zap } from "lucide-react";
import { type KeyboardEvent, type ReactNode, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { displayHotkey, keyboardEventToShortcut } from "@/lib/weapon-format";

export function TrainerSection({
  icon: Icon,
  title,
  children,
  compact,
}: {
  icon: LucideIcon;
  title: string;
  children: ReactNode;
  compact?: boolean;
}) {
  return (
    <section className={cn("trainer-section", compact && "trainer-section-compact")}>
      <div className="section-heading">
        <Icon className="size-3.5" />
        <h2>{title}</h2>
      </div>
      {children}
    </section>
  );
}

export function OptionRow({
  icon: Icon,
  title,
  info,
  children,
  active,
}: {
  icon: LucideIcon;
  title: string;
  info: string;
  children: ReactNode;
  active?: boolean;
}) {
  return (
    <div className={cn("option-row", active && "option-row-active")}>
      <div className="option-name">
        <Icon className="size-3 text-muted-foreground" aria-hidden />
        <span>{title}</span>
        <OptionInfo text={info} label={title} />
      </div>
      <div className="option-controls">{children}</div>
    </div>
  );
}

function OptionInfo({ text, label }: { text: string; label: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button type="button" className="info-trigger" aria-label={`About ${label}`}>
          <Info className="size-3" aria-hidden />
        </button>
      </TooltipTrigger>
      <TooltipContent side="top" sideOffset={7} className="trainer-tooltip max-w-64 leading-relaxed">
        {text}
      </TooltipContent>
    </Tooltip>
  );
}

export function HotkeyButton({
  value,
  busy,
  label,
  onCapture,
}: {
  value: string;
  busy: boolean;
  label: string;
  onCapture: (hotkey: string) => Promise<void>;
}) {
  const [recording, setRecording] = useState(false);

  const handleKeyDown = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (!recording) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      setRecording(false);
      return;
    }
    const hotkey = keyboardEventToShortcut(event);
    if (!hotkey) return;
    setRecording(false);
    void onCapture(hotkey);
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          className={cn("hotkey-button", recording && "hotkey-recording")}
          disabled={busy}
          onClick={() => setRecording(true)}
          onKeyDown={handleKeyDown}
          onBlur={() => setRecording(false)}
          aria-label={`Configure ${label} hotkey`}
        >
          <Keyboard className="size-3" aria-hidden />
          {recording ? "Press keys" : displayHotkey(value)}
        </button>
      </TooltipTrigger>
      {!recording && (
        <TooltipContent side="top" className="trainer-tooltip">
          Click and press a new global hotkey.
        </TooltipContent>
      )}
    </Tooltip>
  );
}

export function DisabledOption({ title, info }: { title: string; info: string }) {
  return (
    <OptionRow icon={Zap} title={title} info={info}>
      <Badge variant="outline" className="badge-meta font-normal text-muted-foreground">
        Not mapped
      </Badge>
      <Switch disabled aria-label={`${title} unavailable`} />
    </OptionRow>
  );
}

export function ValuePair({ label, value, wide }: { label: string; value: string; wide?: boolean }) {
  return (
    <span className={cn("value-pair", wide && "value-pair-wide")}>
      <small>{label}</small>
      <strong>{value}</strong>
    </span>
  );
}

export function StatePill({ active, label }: { active: boolean; label: string }) {
  return <span className={cn("state-pill", active && "state-pill-active")}>{label}</span>;
}
