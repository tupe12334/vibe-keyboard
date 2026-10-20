import { useCallback, useRef, useState } from "react";
import { commands, type DisplaySettings_Serialize as DisplaySettings } from "./bindings";
import type { KeyConfig, Page, Profile } from "./types";
import { useSaveShortcut } from "./useSaveShortcut";

/** The saved layout, the editor draft, the page shown, and the messages above the grid. */
export function useProfile() {
  const [saved, setSaved] = useState<Profile | null>(null);
  const [draft, setDraft] = useState<Profile | null>(null);
  const [page, setPage] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  // The file changed on disk since it loaded. The user picks: reload or overwrite.
  const [conflict, setConflict] = useState(false);

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
    setConflict(false);
  }, []);

  // The layout changed on disk. Keep unsaved edits and ask the user instead of reloading.
  const onReload = useCallback(() => {
    if (dirtyRef.current) {
      setNotice("The layout changed on disk. Discard your edits to load it.");
    } else {
      void load();
    }
  }, [load]);

  /** `force` overwrites the file on disk. Only the Overwrite choice passes it. Resolves true when saved. */
  const save = async (force = false): Promise<boolean> => {
    if (!draft) return false;
    const result = await commands.saveProfile(draft, force);
    if (result.status === "error") {
      if (result.error.kind === "conflict") setConflict(true);
      else setError(result.error.message);
      return false;
    }
    setSaved(draft);
    setError(null);
    setNotice(null);
    setConflict(false);
    return true;
  };

  useSaveShortcut(() => void save());

  // Edits the page on screen. Every other field of the draft stays as it was.
  const updatePage = (change: (p: Page) => Page) =>
    setDraft((d) => d && { ...d, pages: d.pages.map((p, i) => (i === page ? change(p) : p)) });
  const setKey = (key: number, config: KeyConfig | undefined) =>
    updatePage((p) => {
      const keys = { ...p.keys };
      if (config) keys[key] = config;
      else delete keys[key];
      return { ...p, keys };
    });
  const setHoldMs = (hold_ms: number) => setDraft((d) => d && { ...d, input: { ...d.input, hold_ms } });
  const setDisplay = (display: DisplaySettings) => setDraft((d) => d && { ...d, display });

  return {
    draft,
    setDraft,
    page,
    setPage,
    dirty,
    error,
    setError,
    notice,
    conflict,
    load,
    onReload,
    save,
    updatePage,
    setKey,
    setHoldMs,
    setDisplay,
  };
}
