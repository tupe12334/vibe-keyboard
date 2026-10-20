import { useEffect, useRef, useState } from "react";
import { commands, events } from "./bindings";

/** Answers a quit request. Asks first when edits are unsaved. `save` resolves true on success. */
export function useQuitPrompt(dirty: boolean, save: () => Promise<boolean>) {
  const [asking, setAsking] = useState(false);
  // The listener is set up once, so it reads the latest values through this ref.
  const latest = useRef({ dirty, save });
  latest.current = { dirty, save };

  useEffect(() => {
    const subscription = events.quitRequested.listen(() => {
      // Tells the app this editor is alive, so a later quit asks again.
      void commands.acknowledgeQuit();
      if (latest.current.dirty) setAsking(true);
      else void commands.confirmQuit();
    });
    return () => void subscription.then((unlisten) => unlisten());
  }, []);

  return {
    asking,
    cancel: () => setAsking(false),
    discard: () => void commands.confirmQuit(),
    // A failed save keeps the app open. Its error shows in the editor.
    saveAndQuit: async () => {
      if (await latest.current.save()) await commands.confirmQuit();
      else setAsking(false);
    },
  };
}
