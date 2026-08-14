import { type Dispatch, type SetStateAction, useEffect } from "react";

export function useAutoDismiss<T>(
  value: T | null,
  setValue: Dispatch<SetStateAction<T | null>>,
  delay = 5_000,
) {
  useEffect(() => {
    if (value === null) return;
    const timer = window.setTimeout(() => setValue(null), delay);
    return () => window.clearTimeout(timer);
  }, [value, setValue, delay]);
}
