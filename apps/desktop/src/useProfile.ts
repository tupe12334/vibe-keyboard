import { useCallback, useRef, useState } from "react";
import { commands } from "./bindings";
import type { Profile } from "./types";

/** The saved layout, the editor draft, the page shown, and the messages above the grid. */
export function useProfile() {
  const [saved, setSaved] = useState<Profile | null>(null);
  const [draft, setDraft] = useState<Profile | null>(null);
  const [page, setPage] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const dirty = draft !== null && saved !== null && JSON.stringify(draft) !== JSON.stringify(saved);
  // Event handlers read this ref, so they see the current value without re-subscribing.
  const dirtyRef = useRef(false);
  dirtyRef.current = dirty;

  const load = useCallback(async () => {
    const result = await commands.getProfile();
    if (result.status === "error") {
      setError(result.error);
      return;
    }
    setSaved(result.data);
    setDraft(result.data);
    setPage((p) => Math.min(p, result.data.pages.length - 1));
    setError(null);
    setNotice(null);
  }, []);

  // The layout changed on disk. Keep unsaved edits and ask the user instead of reloading.
  const onReload = useCallback(() => {
    if (dirtyRef.current) {
      setNotice("The layout changed on disk. Discard your edits to load it.");
    } else {
      void load();
    }
  }, [load]);

  const save = async () => {
    if (!draft) return;
    const result = await commands.saveProfile(draft);
    if (result.status === "error") {
      setError(result.error);
      return;
    }
    setSaved(draft);
    setError(null);
    setNotice(null);
  };

  return { draft, setDraft, page, setPage, dirty, error, setError, notice, load, onReload, save };
}
