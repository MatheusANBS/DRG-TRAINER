import { Gauge, Info, Keyboard, Zap } from "lucide-react";
import { type KeyboardEvent, type ReactNode, useState } from "react";

import { Badge } from "@/components/ui/badge";
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import type { ClipStatus, ConnectionState, DamageStatus } from "@/types/trainer";

export function TrainerSection({
  icon: Icon,
  title,
  children,
  compact,
}: {
  icon: typeof Gauge;
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
  icon: typeof Gauge;
  title: string;
  info: string;
  children: ReactNode;
  active?: boolean;
}) {
  return (
    <div className={cn("option-row", active && "option-row-active")}>
      <div className="option-name">
        <Icon className="size-3 text-muted-foreground" />
        <span>{title}</span>
        <OptionInfo text={info} />
      </div>
      <div className="option-controls">{children}</div>
    </div>
  );
}

function OptionInfo({ text }: { text: string }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button type="button" className="info-trigger" aria-label="More information">
          <Info className="size-3" />
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
  label = "Infinite Magazine",
  onCapture,
}: {
  value: string;
  busy: boolean;
  label?: string;
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
          aria-label={`Configure ${label} hotkey`}
        >
          <Keyboard className="size-3" />
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
      <Badge variant="outline" className="text-[8px] font-normal text-muted-foreground">
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

export function StatusDot({ state }: { state: ConnectionState }) {
  const color = state === "connected" ? "bg-emerald-400" : state === "error" ? "bg-red-400" : "bg-amber-400";
  return <span className={cn("size-1.5 rounded-full", color)} />;
}

export function cleanWeaponName(name: string) {
  return name.replace(/^WPN_/, "").replace(/_C$/, "").replaceAll("_", " ");
}

export function formatClip(clip: ClipStatus | null) {
  if (clip?.clipCount == null || clip.clipSize == null) return "-- / --";
  return `${clip.clipCount} / ${clip.clipSize}`;
}

export function formatDamage(damage: DamageStatus | null) {
  if (!damage?.available || damage.value == null) return "--";
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 1 }).format(damage.value);
}

function displayHotkey(value: string) {
  return value.replaceAll("Control", "Ctrl").replaceAll("+", " + ");
}

function keyboardEventToShortcut(event: KeyboardEvent<HTMLButtonElement>) {
  if (["Control", "Shift", "Alt", "Meta"].includes(event.key)) return null;
  const modifiers: string[] = [];
  if (event.ctrlKey) modifiers.push("Control");
  if (event.altKey) modifiers.push("Alt");
  if (event.shiftKey) modifiers.push("Shift");
  if (event.metaKey) modifiers.push("Super");

  let key: string;
  if (/^F([1-9]|1[0-2])$/.test(event.key)) key = event.key;
  else if (/^Key[A-Z]$/.test(event.code)) key = event.code.slice(3);
  else if (/^Digit[0-9]$/.test(event.code)) key = event.code.slice(5);
  else return null;

  if (!key.startsWith("F") && modifiers.length === 0) return null;
  return [...modifiers, key].join("+");
}
