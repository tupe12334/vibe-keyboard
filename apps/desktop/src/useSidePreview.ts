import { useEffect, useState } from "react";
import { commands } from "./bindings";
import type { Profile } from "./types";

/** Preview image of the side screen of `page`, from the draft. Debounced like the key previews. */
export function useSidePreview(draft: Profile | null, page: number): string {
  const [preview, setPreview] = useState("");

  useEffect(() => {
    if (!draft) return;
    let cancelled = false;
    const timer = setTimeout(async () => {
      const result = await commands.sidePreview(draft, page);
      if (!cancelled) setPreview(result.status === "ok" ? result.data : "");
    }, 120);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [draft, page]);

  return preview;
}
