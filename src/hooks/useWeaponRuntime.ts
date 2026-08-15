/**
 * Toggles de runtime das armas (SPEC-012 / SPEC-015).
 *
 * São ações `runtime`: reversíveis, sem confirmação e sem backup. O estado
 * otimista existe porque o worker do backend leva alguns milissegundos para
 * publicar o primeiro status — sem ele o switch pareceria travado.
 */

import { useCallback, useRef, useState } from "react";

import type { ActionFeed } from "@/hooks/useActionFeed";
import { toTrainerError } from "@/lib/errors";
import { trainerApi } from "@/services/trainer-api";
import type { TrainerState } from "@/state/trainer-state";
import { blockedReason } from "@/state/trainer-state";
import type { ClipStatus, DamageStatus } from "@/types/trainer";

type Options = {
  state: TrainerState;
  feed: ActionFeed;
  clip: ClipStatus | null;
  damage: DamageStatus | null;
  setClip: (clip: ClipStatus | null) => void;
  setDamage: (damage: DamageStatus | null) => void;
};

export function useWeaponRuntime({ state, feed, clip, damage, setClip, setDamage }: Options) {
  const [togglingClip, setTogglingClip] = useState(false);
  const [togglingDamage, setTogglingDamage] = useState(false);
  const clipInFlight = useRef(false);
  const damageInFlight = useRef(false);

  const clipBlocked = blockedReason(state, "infiniteMagazine");
  const damageBlocked = blockedReason(state, "weaponDamage");

  const toggleClip = useCallback(
    (enabled: boolean) => {
      if (clipInFlight.current || clipBlocked) return;
      clipInFlight.current = true;
      setTogglingClip(true);
      // Otimista: o switch responde já, e volta atrás se o backend recusar.
      setClip(clip ? { ...clip, enabled } : null);

      void (async () => {
        try {
          setClip(await trainerApi.setInfiniteMagazine(enabled));
        } catch (error) {
          setClip(clip ? { ...clip, enabled: !enabled } : null);
          feed.pushError(toTrainerError(error));
        } finally {
          clipInFlight.current = false;
          setTogglingClip(false);
        }
      })();
    },
    [clip, clipBlocked, feed, setClip],
  );

  const toggleDamage = useCallback(
    (enabled: boolean) => {
      if (damageInFlight.current || damageBlocked) return;
      damageInFlight.current = true;
      setTogglingDamage(true);
      setDamage(damage ? { ...damage, enabled } : null);

      void (async () => {
        try {
          setDamage(await trainerApi.setWeaponDamage(enabled));
        } catch (error) {
          setDamage(damage ? { ...damage, enabled: !enabled } : null);
          feed.pushError(toTrainerError(error));
        } finally {
          damageInFlight.current = false;
          setTogglingDamage(false);
        }
      })();
    },
    [damage, damageBlocked, feed, setDamage],
  );

  return {
    toggleClip,
    toggleDamage,
    togglingClip,
    togglingDamage,
    clipBlocked,
    damageBlocked,
    clipEnabled: clip?.enabled ?? false,
    damageEnabled: damage?.enabled ?? false,
  };
}

export type WeaponRuntime = ReturnType<typeof useWeaponRuntime>;
