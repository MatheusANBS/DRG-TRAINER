/**
 * Feed de resultado das ações (SPEC-019).
 *
 * Cada entrada informa o que aconteceu e some sozinha. Estado persistente
 * importante — conexão, build, toggles ligados — vive no status bar e nas
 * próprias seções, nunca aqui: um toast que some não pode ser a única fonte
 * de uma informação que continua valendo.
 */

import { AlertTriangle, CheckCircle2, X } from "lucide-react";

import type { FeedEntry } from "@/hooks/useActionFeed";
import { cn } from "@/lib/utils";

export function TrainerNotifications({
  entries,
  onDismiss,
}: {
  entries: FeedEntry[];
  onDismiss: (id: number) => void;
}) {
  if (entries.length === 0) return null;

  return (
    <div className="trainer-feed" aria-live="polite">
      {entries.map((entry) => (
        <div
          key={entry.id}
          className={cn("trainer-note", `trainer-note-${entry.tone}`)}
          role={entry.tone === "error" ? "alert" : "status"}
        >
          {entry.tone === "error" ? (
            <AlertTriangle className="size-3.5 shrink-0" aria-hidden />
          ) : (
            <CheckCircle2 className="size-3.5 shrink-0" aria-hidden />
          )}
          <div className="trainer-note-body">
            <span>{entry.message}</span>
            {entry.detail && <small>{entry.detail}</small>}
          </div>
          <button
            type="button"
            className="trainer-note-dismiss"
            onClick={() => onDismiss(entry.id)}
            aria-label="Dismiss notification"
          >
            <X className="size-3" aria-hidden />
          </button>
        </div>
      ))}
    </div>
  );
}
