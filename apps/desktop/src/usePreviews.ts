import { useEffect, useState } from "react";
import { commands } from "./bindings";
import { KEYS } from "./deckLayout";
import type { Profile } from "./types";

/** Preview image for each key on `page`, from the draft. Debounced to skip keystrokes. */
export function usePreviews(draft: Profile | null, page: number) {
  const [previews, setPreviews] = useState<Record<number, string>>({});

  useEffect(() => {
    if (!draft) return;
    let cancelled = false;
    const timer = setTimeout(async () => {
      const entries = await Promise.all(
        KEYS.map(async (key) => {
          const result = await commands.keyPreview(draft, page, key);
          return [key, result.status === "ok" ? result.data : ""] as const;
        }),
      );
      if (!cancelled) setPreviews(Object.fromEntries(entries));
    }, 120);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [draft, page]);

  return previews;
}
