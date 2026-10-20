import { useEffect, useState } from "react";
import { commands, events } from "./bindings";

export interface DeckStatus {
  connected: boolean;
  running: boolean;
  error: string | null;
}

/** Device status and the last key press. `onReload` runs when the layout changes on disk. */
export function useDeckEvents(onReload: () => void) {
  const [deck, setDeck] = useState<DeckStatus>({ connected: false, running: false, error: null });
  const [pressed, setPressed] = useState<number | null>(null);

  useEffect(() => {
    const refreshDeck = () => void commands.deviceStatus().then(setDeck);
    refreshDeck();
    const subscriptions = Promise.all([
      events.deviceConnected.listen(refreshDeck),
      events.deviceDisconnected.listen(refreshDeck),
      events.keyPressed.listen(({ payload }) => {
        setPressed(payload.key);
        setTimeout(() => setPressed((p) => (p === payload.key ? null : p)), 300);
      }),
      events.profileReloaded.listen(onReload),
    ]);
    return () => {
      void subscriptions.then((unlisten) => unlisten.forEach((stop) => stop()));
    };
  }, [onReload]);

  return { deck, pressed };
}
