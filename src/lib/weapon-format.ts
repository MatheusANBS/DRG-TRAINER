/**
 * Formatação e leitura de dados de arma.
 *
 * Funções puras, fora dos componentes: elas são testadas diretamente e não
 * dependem de render.
 */

import type { ClipStatus, DamageStatus } from "@/types/trainer";

/** Placeholder de leitura indisponível, usado em toda a interface. */
export const NO_VALUE = "--";

/** `WPN_AssaultRifle_C` → `AssaultRifle`. */
export function cleanWeaponName(name: string): string {
  return name.replace(/^WPN_/, "").replace(/_C$/, "").replaceAll("_", " ");
}

export function formatClip(clip: ClipStatus | null): string {
  if (clip?.clipCount == null || clip.clipSize == null) return `${NO_VALUE} / ${NO_VALUE}`;
  return `${clip.clipCount} / ${clip.clipSize}`;
}

export function formatDamage(damage: DamageStatus | null): string {
  if (!damage?.available || damage.value == null) return NO_VALUE;
  return new Intl.NumberFormat("en-US", { maximumFractionDigits: 1 }).format(damage.value);
}

/** Formato legível de um atalho salvo (`Control+F8` → `Ctrl + F8`). */
export function displayHotkey(value: string): string {
  return value.replaceAll("Control", "Ctrl").replaceAll("+", " + ");
}

type ShortcutSource = {
  key: string;
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
};

/**
 * Converte um evento de teclado no formato aceito pelo backend.
 *
 * Devolve `null` quando a combinação não serve como atalho global — teclas
 * modificadoras sozinhas, ou uma letra sem modificador, que roubaria a digitação
 * do jogo.
 */
export function keyboardEventToShortcut(event: ShortcutSource): string | null {
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
