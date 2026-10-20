import type { DeckStatus } from "./useDeckEvents";

interface Props {
  name: string;
  onRename: (name: string) => void;
  deleteDisabled: boolean;
  onDelete: () => void;
  deck: DeckStatus;
  dirty: boolean;
  discardDisabled: boolean;
  onDiscard: () => void;
  onSave: () => void;
}

/** Page name, page delete, device status, and Discard and Save. */
export function Toolbar({ name, onRename, deleteDisabled, onDelete, deck, dirty, discardDisabled, onDiscard, onSave }: Props) {
  return (
    <div className="toolbar">
      <label>
        Page name
        <input value={name} onChange={(e) => onRename(e.target.value)} />
      </label>
      <button onClick={onDelete} disabled={deleteDisabled} className="danger">
        Delete page
      </button>
      <span className="spacer" />
      <span className="status">
        Device {deck.connected ? "connected" : "not connected"} · controller {deck.running ? "running" : "paused"}
      </span>
      <button onClick={onDiscard} disabled={discardDisabled}>
        Discard
      </button>
      <button className="primary" onClick={onSave} disabled={!dirty}>
        Save
      </button>
    </div>
  );
}
