import { describe, expect, it } from "vitest";

import { UNKNOWN_ERROR, errorDetail, errorMessage, isWaiting, toTrainerError } from "@/lib/errors";

describe("toTrainerError", () => {
  it("keeps a typed backend error intact", () => {
    const error = toTrainerError({
      code: "BUILD_UNSUPPORTED",
      message: "SHA-256 desconhecida",
      recoverable: false,
    });
    expect(error.code).toBe("BUILD_UNSUPPORTED");
    expect(error.message).toBe("SHA-256 desconhecida");
    expect(error.recoverable).toBe(false);
  });

  it("falls back for an unknown code so the UI never renders undefined", () => {
    const error = toTrainerError({ code: "NAO_EXISTE", message: "?" });
    expect(error).toEqual(UNKNOWN_ERROR);
    expect(errorMessage(error)).toBeTruthy();
  });

  it("normalises IPC failures that arrive as plain strings", () => {
    const error = toTrainerError("invoke falhou");
    expect(error.code).toBe("TASK_FAILED");
    expect(error.message).toBe("invoke falhou");
  });

  it("normalises thrown Errors", () => {
    const error = toTrainerError(new Error("boom"));
    expect(error.code).toBe("TASK_FAILED");
    expect(error.message).toBe("boom");
  });

  it("normalises null and undefined without throwing", () => {
    expect(toTrainerError(null)).toEqual(UNKNOWN_ERROR);
    expect(toTrainerError(undefined)).toEqual(UNKNOWN_ERROR);
  });
});

describe("error presentation", () => {
  it("gives every known code a specific message", () => {
    const codes = [
      "PROCESS_NOT_FOUND",
      "BUILD_UNSUPPORTED",
      "SIGNATURE_MISMATCH",
      "BACKUP_FAILED",
      "WORLD_NOT_READY",
      "HOTKEY_REGISTRATION_FAILED",
    ] as const;

    const messages = codes.map((code) =>
      errorMessage({ code, message: "detalhe", recoverable: false }),
    );
    expect(new Set(messages).size).toBe(codes.length);
    for (const message of messages) expect(message.length).toBeGreaterThan(10);
  });

  it("hides the technical detail when it repeats the friendly message", () => {
    const friendly = errorMessage({
      code: "WORLD_NOT_READY",
      message: "",
      recoverable: true,
    });
    expect(
      errorDetail({ code: "WORLD_NOT_READY", message: friendly, recoverable: true }),
    ).toBeNull();
    expect(
      errorDetail({ code: "WORLD_NOT_READY", message: "0x1234 nulo", recoverable: true }),
    ).toBe("0x1234 nulo");
  });

  it("treats a missing game as waiting, not as a failure", () => {
    expect(isWaiting({ code: "PROCESS_NOT_FOUND", message: "", recoverable: true })).toBe(true);
    expect(isWaiting({ code: "MODULE_NOT_FOUND", message: "", recoverable: true })).toBe(true);
    expect(isWaiting({ code: "BUILD_UNSUPPORTED", message: "", recoverable: false })).toBe(false);
  });
});
