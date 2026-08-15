/**
 * Barra de status operacional (SPEC-016).
 *
 * Responde, em um olhar, às duas perguntas que importam antes de qualquer ação:
 * o trainer está anexado, e a build é confiável. As duas respostas são texto —
 * cor e ícone só reforçam. Detalhes secundários (PID, perfil, hash) ficam na
 * segunda linha; tooltip não carrega informação essencial.
 */

import { AlertTriangle, CircleDashed, CircleDot, ShieldCheck, ShieldX } from "lucide-react";

import { cn } from "@/lib/utils";
import { buildLabel, connectionLabel, type TrainerConnection } from "@/state/trainer-state";

type Tone = "ok" | "warning" | "idle";

function processTone(connection: TrainerConnection): Tone {
  switch (connection.kind) {
    case "attached":
      return "ok";
    case "unsupported":
    case "unverified":
      return "warning";
    case "error":
      return "warning";
    default:
      return "idle";
  }
}

function buildTone(connection: TrainerConnection): Tone {
  switch (connection.kind) {
    case "attached":
      return "ok";
    case "unsupported":
    case "unverified":
      return "warning";
    default:
      return "idle";
  }
}

/** Linha secundária: quem está anexado e sob qual perfil. */
function detailLine(connection: TrainerConnection): string {
  switch (connection.kind) {
    case "connecting":
      return "Reading trainer status…";
    case "detached":
      return `Waiting for ${connection.expected.join(" / ")}`;
    case "attached":
      return `${connection.executable} · PID ${connection.pid} · profile ${connection.profileId}`;
    case "unverified":
      return `${connection.executable} · PID ${connection.pid} · profile ${connection.profileId} not promoted`;
    case "unsupported":
      return connection.sha256
        ? `${connection.executable} · PID ${connection.pid} · SHA-256 ${connection.sha256.slice(0, 12)}…`
        : `${connection.executable} · PID ${connection.pid} · build not identified`;
    case "error":
      return connection.error.message;
  }
}

/** Aviso explícito quando as ações estão travadas. */
function warningLine(connection: TrainerConnection): string | null {
  switch (connection.kind) {
    case "unsupported":
      return "Memory-dependent actions disabled until a build profile matches.";
    case "unverified":
      return "Memory-dependent actions disabled until this profile is verified.";
    case "detached":
      return "Start Deep Rock Galactic and enter the Space Rig.";
    case "error":
      return "Trainer could not read its own status.";
    default:
      return null;
  }
}

function ProcessIcon({ tone }: { tone: Tone }) {
  if (tone === "ok") return <CircleDot className="size-3.5" aria-hidden />;
  return <CircleDashed className="size-3.5" aria-hidden />;
}

function BuildIcon({ tone }: { tone: Tone }) {
  if (tone === "ok") return <ShieldCheck className="size-3.5" aria-hidden />;
  if (tone === "warning") return <ShieldX className="size-3.5" aria-hidden />;
  return <ShieldX className="size-3.5 opacity-60" aria-hidden />;
}

export function TrainerStatusBar({ connection }: { connection: TrainerConnection }) {
  const warning = warningLine(connection);

  return (
    <div className="trainer-status" role="status" aria-live="polite">
      <div className="trainer-status-row">
        <span className={cn("status-chip", `status-chip-${processTone(connection)}`)}>
          <ProcessIcon tone={processTone(connection)} />
          {connectionLabel(connection)}
        </span>
        <span className={cn("status-chip", `status-chip-${buildTone(connection)}`)}>
          <BuildIcon tone={buildTone(connection)} />
          {buildLabel(connection)}
        </span>
        {/* O detalhe divide a linha com os chips: em 940x660 cada pixel de
            altura do cabeçalho sai do conteúdo. */}
        <p className="trainer-status-detail">{detailLine(connection)}</p>
      </div>
      {warning && (
        <p className="trainer-status-warning">
          <AlertTriangle className="size-3 shrink-0" aria-hidden />
          {warning}
        </p>
      )}
    </div>
  );
}
