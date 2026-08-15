/**
 * Estado de conexão do trainer (SPEC-013 / SPEC-016).
 *
 * Fonte única para "posso operar agora?". O status do backend já resolve
 * processo, build e capacidades; aqui ele vira o estado observável da UI.
 */

import { useCallback, useRef, useState } from "react";

import { useTrainerPolling } from "@/hooks/useTrainerPolling";
import { toTrainerError } from "@/lib/errors";
import { trainerApi } from "@/services/trainer-api";
import { INITIAL_STATE, type TrainerState, deriveState, errorState } from "@/state/trainer-state";

/** Intervalo do status: rápido o bastante para detectar o jogo abrindo. */
const STATUS_INTERVAL_MS = 1_000;

export function useTrainerConnection(intervalMs = STATUS_INTERVAL_MS): TrainerState {
  const [state, setState] = useState<TrainerState>(INITIAL_STATE);
  // O último conjunto conhecido de executáveis esperados sobrevive a uma falha,
  // para o estado "detached" continuar dizendo o que o trainer procura.
  const expected = useRef<string[]>([]);

  const poll = useCallback(async () => {
    try {
      const status = await trainerApi.status();
      expected.current = status.expectedExecutables;
      setState(deriveState(status));
      return true;
    } catch (error) {
      setState(errorState(toTrainerError(error), expected.current));
      return false;
    }
  }, []);

  useTrainerPolling(poll, { intervalMs });

  return state;
}
