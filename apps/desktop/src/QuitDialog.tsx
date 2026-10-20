interface Props {
  onSave: () => void;
  onDiscard: () => void;
  onCancel: () => void;
}

/** Shown when the app quits with unsaved layout edits. */
export function QuitDialog({ onSave, onDiscard, onCancel }: Props) {
  return (
    <div className="modal-backdrop">
      <div className="modal" role="alertdialog" aria-modal="true" aria-labelledby="quit-title">
        <h2 id="quit-title">Quit with unsaved edits?</h2>
        <p>Quitting now discards the layout changes you have not saved.</p>
        <div className="modal-actions">
          <button onClick={onCancel}>Cancel</button>
          <button className="danger" onClick={onDiscard}>
            Discard
          </button>
          <button className="primary" onClick={onSave} autoFocus>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
