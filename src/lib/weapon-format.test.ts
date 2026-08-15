import { describe, expect, it } from "vitest";

import {
  NO_VALUE,
  cleanWeaponName,
  displayHotkey,
  formatClip,
  formatDamage,
  keyboardEventToShortcut,
} from "@/lib/weapon-format";
import { clipStatus, damageStatus } from "@/test/trainer-api-mock";

describe("weapon name", () => {
  it("strips the engine prefix and suffix", () => {
    expect(cleanWeaponName("WPN_AssaultRifle_C")).toBe("AssaultRifle");
    expect(cleanWeaponName("WPN_Minigun_Heavy_C")).toBe("Minigun Heavy");
  });

  it("leaves an unrecognised name readable", () => {
    expect(cleanWeaponName("BP_Flare")).toBe("BP Flare");
  });
});

describe("clip and damage formatting", () => {
  it("shows a placeholder instead of a partial reading", () => {
    expect(formatClip(null)).toBe(`${NO_VALUE} / ${NO_VALUE}`);
    expect(formatClip(clipStatus({ clipCount: null }))).toBe(`${NO_VALUE} / ${NO_VALUE}`);
    expect(formatDamage(null)).toBe(NO_VALUE);
    expect(formatDamage(damageStatus({ available: false }))).toBe(NO_VALUE);
  });

  it("formats real readings", () => {
    expect(formatClip(clipStatus({ clipCount: 12, clipSize: 30 }))).toBe("12 / 30");
    expect(formatDamage(damageStatus({ value: 9_999 }))).toBe("9,999");
  });
});

describe("hotkey capture", () => {
  const base = { ctrlKey: false, altKey: false, shiftKey: false, metaKey: false };

  it("accepts a bare function key", () => {
    expect(keyboardEventToShortcut({ ...base, key: "F8", code: "F8" })).toBe("F8");
  });

  it("accepts a letter only with a modifier", () => {
    expect(keyboardEventToShortcut({ ...base, key: "k", code: "KeyK" })).toBeNull();
    expect(keyboardEventToShortcut({ ...base, ctrlKey: true, key: "k", code: "KeyK" })).toBe(
      "Control+K",
    );
  });

  it("keeps modifiers in the order the backend expects", () => {
    expect(
      keyboardEventToShortcut({
        key: "F12",
        code: "F12",
        ctrlKey: true,
        altKey: true,
        shiftKey: true,
        metaKey: true,
      }),
    ).toBe("Control+Alt+Shift+Super+F12");
  });

  it("ignores a modifier pressed on its own", () => {
    for (const key of ["Control", "Shift", "Alt", "Meta"]) {
      expect(keyboardEventToShortcut({ ...base, key, code: key })).toBeNull();
    }
  });

  it("rejects keys the backend cannot register", () => {
    expect(keyboardEventToShortcut({ ...base, key: "Tab", code: "Tab" })).toBeNull();
    expect(keyboardEventToShortcut({ ...base, ctrlKey: true, key: "?", code: "Slash" })).toBeNull();
  });

  it("renders a saved shortcut in a readable form", () => {
    expect(displayHotkey("Control+Shift+F12")).toBe("Ctrl + Shift + F12");
    expect(displayHotkey("F8")).toBe("F8");
  });
});
