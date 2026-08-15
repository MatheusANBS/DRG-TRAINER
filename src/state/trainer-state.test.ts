import { describe, expect, it } from "vitest";

import {
  ALL_CAPABILITIES,
  NO_CAPABILITIES,
  attachedStatus,
  detachedStatus,
  trainerError,
  unsupportedStatus,
} from "@/test/trainer-api-mock";
import {
  blockedReason,
  buildLabel,
  connectionLabel,
  currentPid,
  deriveState,
  errorState,
  isOperable,
} from "@/state/trainer-state";

describe("deriveState", () => {
  it("treats a verified build as the only operable state", () => {
    const state = deriveState(attachedStatus());
    expect(state.connection.kind).toBe("attached");
    expect(isOperable(state)).toBe(true);
    expect(state.capabilities).toEqual(ALL_CAPABILITIES);
  });

  it("keeps every capability off when the game is not running", () => {
    const state = deriveState(detachedStatus());
    expect(state.connection).toEqual({
      kind: "detached",
      expected: ["FSD-Win64-Shipping.exe"],
    });
    expect(state.capabilities).toEqual(NO_CAPABILITIES);
    expect(isOperable(state)).toBe(false);
  });

  it("makes an unsupported build a first-class state instead of an error", () => {
    const state = deriveState(unsupportedStatus());
    expect(state.connection.kind).toBe("unsupported");
    expect(currentPid(state.connection)).toBe(26248);
    expect(isOperable(state)).toBe(false);
  });

  it("refuses capabilities from an unverified profile even if the backend sent them", () => {
    const status = attachedStatus({
      build: {
        kind: "unverified",
        profileId: "fsd-novo",
        displayName: "Build nova",
        sha256: "b".repeat(64),
        status: "draft",
      },
      capabilities: ALL_CAPABILITIES,
    });

    const state = deriveState(status);
    expect(state.connection.kind).toBe("unverified");
    expect(state.capabilities).toEqual(NO_CAPABILITIES);
  });

  it("treats an attached process with an unidentified build as not operable", () => {
    const state = deriveState(attachedStatus({ build: { kind: "unknown" } }));
    expect(isOperable(state)).toBe(false);
    expect(state.capabilities).toEqual(NO_CAPABILITIES);
  });
});

describe("errorState", () => {
  it("maps a waiting error to detached rather than to an error banner", () => {
    const state = errorState(trainerError("PROCESS_NOT_FOUND", "Aguardando", true), [
      "FSD-Win64-Shipping.exe",
    ]);
    expect(state.connection).toEqual({
      kind: "detached",
      expected: ["FSD-Win64-Shipping.exe"],
    });
  });

  it("surfaces a real failure as an error state", () => {
    const state = errorState(trainerError("STATE_UNAVAILABLE"), []);
    expect(state.connection.kind).toBe("error");
    expect(isOperable(state)).toBe(false);
  });
});

describe("blockedReason", () => {
  it("returns null only when attached and the capability is verified", () => {
    expect(blockedReason(deriveState(attachedStatus()), "credits")).toBeNull();
  });

  it("explains why an action is blocked in every non-operable state", () => {
    const cases = [
      deriveState(detachedStatus()),
      deriveState(unsupportedStatus()),
      errorState(trainerError("STATE_UNAVAILABLE"), []),
    ];
    for (const state of cases) {
      const reason = blockedReason(state, "credits");
      expect(reason).toBeTruthy();
      expect(reason).not.toBe("");
    }
  });

  it("blocks a capability the current profile has not verified", () => {
    const state = deriveState(
      attachedStatus({ capabilities: { ...ALL_CAPABILITIES, schematicUnlock: false } }),
    );
    expect(blockedReason(state, "weaponUnlock")).toBeNull();
    expect(blockedReason(state, "schematicUnlock")).toBe(
      "Not verified on the current build profile.",
    );
  });
});

describe("status labels", () => {
  it("describes process and build as text, not only colour", () => {
    const attached = deriveState(attachedStatus()).connection;
    expect(connectionLabel(attached)).toBe("GAME ATTACHED");
    expect(buildLabel(attached)).toBe("BUILD VERIFIED");

    const unsupported = deriveState(unsupportedStatus()).connection;
    expect(connectionLabel(unsupported)).toBe("GAME ATTACHED");
    expect(buildLabel(unsupported)).toBe("BUILD UNSUPPORTED");

    const detached = deriveState(detachedStatus()).connection;
    expect(connectionLabel(detached)).toBe("GAME NOT RUNNING");
  });
});
