/**
 * Controle de ação com semântica declarada (SPEC-015).
 *
 * O componente recebe a semântica em vez de deduzi-la do texto: é ela que
 * decide badge, tratamento visual e se há confirmação. Vermelho fica reservado
 * a erro; mutação permanente usa âmbar reforçado, para que "irreversível" não
 * se confunda com "destrutivo".
 */

import { Loader2, type LucideIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { AsyncAction } from "@/hooks/useAsyncAction";
import { cn } from "@/lib/utils";
import { ACTION_SEMANTICS, type ActionSemantics } from "@/types/actions";

/** Badge que antecede o controle quando a semântica acrescenta informação. */
export function SemanticsBadge({ semantics }: { semantics: ActionSemantics }) {
  const meta = ACTION_SEMANTICS[semantics];
  if (!meta.badge) return null;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className={cn("semantics-badge", `semantics-badge-${semantics}`)}>{meta.badge}</span>
      </TooltipTrigger>
      <TooltipContent side="top" className="trainer-tooltip max-w-64">
        {meta.impact}
      </TooltipContent>
    </Tooltip>
  );
}

type ActionControlProps = {
  action: AsyncAction;
  semantics: ActionSemantics;
  label: string;
  pendingLabel: string;
  icon: LucideIcon;
  onActivate: () => void;
};

export function ActionControl({
  action,
  semantics,
  label,
  pendingLabel,
  icon: Icon,
  onActivate,
}: ActionControlProps) {
  // `onActivate` abre o diálogo quando a semântica exige confirmação e executa
  // direto quando não exige — quem monta a seção liga um ou outro.
  const button = (
    <Button
      type="button"
      size="xs"
      variant={semantics === "persistent" ? "default" : "secondary"}
      className={cn("action-control", `action-control-${semantics}`)}
      disabled={action.disabled}
      aria-disabled={action.disabled}
      data-phase={action.phase}
      onClick={onActivate}
    >
      {action.pending ? <Loader2 className="animate-spin" /> : <Icon />}
      {action.pending ? pendingLabel : label}
    </Button>
  );

  // Um botão desabilitado precisa explicar por quê; sem isso, "não funciona"
  // vira suporte.
  if (!action.blockedReason) return button;

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span className="inline-flex">{button}</span>
      </TooltipTrigger>
      <TooltipContent side="top" className="trainer-tooltip max-w-64">
        {action.blockedReason}
      </TooltipContent>
    </Tooltip>
  );
}
