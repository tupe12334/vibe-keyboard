import { useEffect, useRef } from "react";

/** True for Cmd+S. Other keys and modifiers do not count. */
export function isSaveShortcut(event: { metaKey: boolean; key: string }): boolean {
  return event.metaKey && event.key.toLowerCase() === "s";
}

/** Runs `save` on Cmd+S, and stops the browser's own save. */
export function useSaveShortcut(save: () => void) {
  // The listener is set up once, so it calls the latest `save` through this ref.
  const latest = useRef(save);
  latest.current = save;

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!isSaveShortcut(event)) return;
      event.preventDefault();
      latest.current();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
