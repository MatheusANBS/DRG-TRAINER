/**
 * Testes de comportamento da aplicação (SPEC-008).
 *
 * Consultam por papel, rótulo e estado — nunca por classe CSS — para que o
 * polimento visual das SPECs 014/017 não quebre a suíte. O foco está no que o
 * trainer não pode errar: ação bloqueada quando a build não é confiável,
 * confirmação antes de mutação permanente e nenhum disparo duplicado.
 */

import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "@/App";
import {
  ALL_CAPABILITIES,
  attachedStatus,
  createApiMock,
  detachedStatus,
  trainerError,
  unsupportedStatus,
} from "@/test/trainer-api-mock";
import { renderApp, screen, waitFor, within } from "@/test/render";

const api = vi.hoisted(() => ({ current: null as ReturnType<typeof createApiMock> | null }));

vi.mock("@/services/trainer-api", () => ({
  get trainerApi() {
    if (!api.current) throw new Error("mock da API não inicializado");
    return api.current;
  },
}));

function mockApi(overrides: Partial<Record<string, unknown>> = {}) {
  api.current = createApiMock(overrides);
  return api.current;
}

/** Espera o primeiro ciclo de status assentar. */
async function waitForStatus(label: string) {
  await screen.findByText(label, {}, { timeout: 3_000 });
}

beforeEach(() => {
  mockApi();
});

describe("operational status (SPEC-016)", () => {
  it("states attachment and build trust as text", async () => {
    mockApi();
    renderApp(<App />);

    await waitForStatus("GAME ATTACHED");
    expect(screen.getByText("BUILD VERIFIED")).toBeInTheDocument();
    expect(screen.getByText(/PID 26248/)).toBeInTheDocument();
    expect(screen.getByText(/profile fsd-8e22e371/)).toBeInTheDocument();
  });

  it("says the game is not running and what it is waiting for", async () => {
    mockApi({ status: vi.fn().mockResolvedValue(detachedStatus()) });
    renderApp(<App />);

    await waitForStatus("GAME NOT RUNNING");
    expect(screen.getByText(/Waiting for FSD-Win64-Shipping\.exe/)).toBeInTheDocument();
    expect(screen.getByText(/Start Deep Rock Galactic/)).toBeInTheDocument();
  });

  it("announces an unsupported build without relying on a tooltip", async () => {
    mockApi({ status: vi.fn().mockResolvedValue(unsupportedStatus()) });
    renderApp(<App />);

    await waitForStatus("BUILD UNSUPPORTED");
    expect(
      screen.getByText(/Memory-dependent actions disabled until a build profile matches/),
    ).toBeInTheDocument();
  });

  it("recovers from a status failure without crashing", async () => {
    mockApi({
      status: vi.fn().mockRejectedValue(trainerError("STATE_UNAVAILABLE", "mutex envenenado")),
    });
    renderApp(<App />);

    await waitForStatus("TRAINER ERROR");
    expect(screen.getByText("BUILD UNKNOWN")).toBeInTheDocument();
  });
});

describe("action gating", () => {
  it("disables every permanent action while the game is not running", async () => {
    mockApi({ status: vi.fn().mockResolvedValue(detachedStatus()) });
    renderApp(<App />);
    await waitForStatus("GAME NOT RUNNING");

    for (const name of [/Max all/, /Promote all/, /Unlock perks/]) {
      expect(screen.getByRole("button", { name })).toBeDisabled();
    }
  });

  it("disables actions on an unsupported build even though the process is attached", async () => {
    mockApi({ status: vi.fn().mockResolvedValue(unsupportedStatus()) });
    renderApp(<App />);
    await waitForStatus("BUILD UNSUPPORTED");

    expect(screen.getByRole("button", { name: /Max all/ })).toBeDisabled();
    expect(screen.getByRole("switch", { name: /Toggle Infinite Magazine/ })).toBeDisabled();
  });

  it("disables only the capabilities the profile has not verified", async () => {
    mockApi({
      status: vi.fn().mockResolvedValue(
        attachedStatus({ capabilities: { ...ALL_CAPABILITIES, promotion: false } }),
      ),
    });
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    expect(screen.getByRole("button", { name: /Promote all/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Max all/ })).toBeEnabled();
  });
});

describe("permanent mutations (SPEC-015)", () => {
  it("marks them with a PERMANENT badge before the click", async () => {
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    expect(screen.getAllByText("PERMANENT").length).toBeGreaterThan(0);
  });

  it("requires confirmation and only then calls the backend", async () => {
    const user = userEvent.setup();
    const mocked = mockApi();
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    await user.click(screen.getByRole("button", { name: /Max all/ }));

    const dialog = await screen.findByRole("alertdialog");
    expect(within(dialog).getByText(/Set every class to level 25\?/)).toBeInTheDocument();
    expect(mocked.maxClassLevel).not.toHaveBeenCalled();

    await user.click(within(dialog).getByRole("button", { name: /Create backup & apply/ }));
    await waitFor(() => expect(mocked.maxClassLevel).toHaveBeenCalledTimes(1));
  });

  it("does nothing when the confirmation is cancelled", async () => {
    const user = userEvent.setup();
    const mocked = mockApi();
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    await user.click(screen.getByRole("button", { name: /Unlock perks/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /Cancel/ }));

    expect(mocked.unlockAllPerks).not.toHaveBeenCalled();
  });

  it("reports the backup path after a successful mutation", async () => {
    const user = userEvent.setup();
    const mocked = mockApi();
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    await user.click(screen.getByRole("button", { name: /Unlock perks/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /Create backup & unlock/ }));

    await waitFor(() => expect(mocked.unlockAllPerks).toHaveBeenCalled());
    expect(await screen.findByText(/Perks acquired: 0 → 23/)).toBeInTheDocument();
    expect(screen.getByText(/DRGTrainerBackups/)).toBeInTheDocument();
  });

  it("never fires the same mutation twice from a double click", async () => {
    const user = userEvent.setup();
    let resolve: ((value: unknown) => void) | undefined;
    const pending = new Promise((done) => {
      resolve = done;
    });
    const mocked = mockApi({ unlockAllWeapons: vi.fn().mockReturnValue(pending) });

    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");
    await user.click(screen.getByRole("button", { name: /^Weapons$/ }));

    const unlockButtons = screen.getAllByRole("button", { name: /Unlock all/ });
    await user.click(unlockButtons[0]);
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /Create backup & unlock/ }));

    await waitFor(() => expect(mocked.unlockAllWeapons).toHaveBeenCalledTimes(1));

    // Segundo clique enquanto a primeira chamada ainda está pendente.
    const stillThere = screen.getAllByRole("button", { name: /Unlocking/ })[0];
    expect(stillThere).toBeDisabled();
    expect(mocked.unlockAllWeapons).toHaveBeenCalledTimes(1);

    // Liberar a promessa e aguardar o resultado mantém o teste dentro do
    // ciclo de vida do React (sem aviso de act()).
    resolve?.({
      pid: 26248,
      weaponCount: 26,
      unlockedBefore: 0,
      unlockedAfter: 26,
      ownedBefore: 0,
      ownedAfter: 26,
      backup: { path: "C:\\backup", files: 1, totalBackups: 1 },
      message: "ok",
    });
    expect(await screen.findByText(/26 weapons processed/)).toBeInTheDocument();
  });
});

describe("error handling (SPEC-019)", () => {
  it("shows a specific message for a known backend error code", async () => {
    const user = userEvent.setup();
    mockApi({
      unlockAllPerks: vi
        .fn()
        .mockRejectedValue(trainerError("WORLD_NOT_READY", "controller ausente", true)),
    });
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    await user.click(screen.getByRole("button", { name: /Unlock perks/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /Create backup & unlock/ }));

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(/Enter the Space Rig/);
    expect(alert).toHaveTextContent(/controller ausente/);
  });

  it("lets the user dismiss a notification", async () => {
    const user = userEvent.setup();
    mockApi({
      unlockAllPerks: vi.fn().mockRejectedValue(trainerError("BACKUP_FAILED", "sem saves")),
    });
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    await user.click(screen.getByRole("button", { name: /Unlock perks/ }));
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: /Create backup & unlock/ }));

    const alert = await screen.findByRole("alert");
    await user.click(within(alert).getByRole("button", { name: /Dismiss notification/ }));
    await waitFor(() => expect(screen.queryByRole("alert")).not.toBeInTheDocument());
  });
});

describe("runtime toggles", () => {
  it("turns Infinite Magazine on through the backend", async () => {
    const user = userEvent.setup();
    const mocked = mockApi();
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    await user.click(screen.getByRole("switch", { name: /Toggle Infinite Magazine/ }));
    await waitFor(() => expect(mocked.setInfiniteMagazine).toHaveBeenCalledWith(true));
  });

  it("reverts the switch when the backend refuses", async () => {
    const user = userEvent.setup();
    mockApi({
      setWeaponDamage: vi.fn().mockRejectedValue(trainerError("SIGNATURE_MISMATCH", "divergiu")),
    });
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    const toggle = screen.getByRole("switch", { name: /Toggle Weapon Damage/ });
    await user.click(toggle);

    expect(await screen.findByRole("alert")).toHaveTextContent(/does not match the verified build/);
    await waitFor(() => expect(toggle).not.toBeChecked());
  });
});

describe("navigation", () => {
  it("switches modules and reveals the detailed inventory editors", async () => {
    const user = userEvent.setup();
    renderApp(<App />);
    await waitForStatus("BUILD VERIFIED");

    // No modo "All" as seções ficam compactas.
    expect(screen.queryByLabelText("Amount for all resources")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /^Inventory$/ }));

    expect(await screen.findByLabelText("Amount for all resources")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Max all/ })).not.toBeInTheDocument();
  });
});
