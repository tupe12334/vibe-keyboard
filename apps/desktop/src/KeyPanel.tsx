import type { Action, KeyConfig_Serialize as KeyConfig } from "./bindings";

// Used when a key has no config yet. The first edit writes it into the layout.
const BLANK_KEY: KeyConfig = {
  label: "",
  bg: "#141414",
  fg: "#E6E6E6",
  image: null,
  action: { type: "none" },
};

// Typed against the generated `Action` type, so a new or renamed action fails to compile here.
const ACTION_LABELS: Record<Action["type"], string> = {
  none: "None",
  open_url: "Open URL",
  run_command: "Run command",
  open_app: "Open app",
  hotkey: "Hotkey",
  go_to_page: "Go to page",
  builtin: "Built-in",
  push_to_talk: "Push to talk",
};

function blankAction(type: Action["type"], firstBuiltin: string): Action {
  switch (type) {
    case "none":
      return { type };
    case "open_url":
      return { type, url: "" };
    case "run_command":
      return { type, command: "" };
    case "open_app":
      return { type, name: "" };
    case "hotkey":
      return { type, keys: "" };
    case "go_to_page":
      return { type, page: 0 };
    case "builtin":
      return { type, name: firstBuiltin };
    case "push_to_talk":
      return { type };
  }
}

interface Props {
  /** Undefined when the key has no config yet. */
  config: KeyConfig | undefined;
  keyNumber: number;
  builtins: string[];
  pageCount: number;
  onChange: (config: KeyConfig) => void;
  onClear: () => void;
}

export function KeyPanel({ config, keyNumber, builtins, pageCount, onChange, onClear }: Props) {
  const value = config ?? BLANK_KEY;
  const set = (patch: Partial<KeyConfig>) => onChange({ ...value, ...patch });
  const setAction = (action: Action) => set({ action });

  return (
    <section className="panel">
      <strong>Key {keyNumber}</strong>
      <div className="row">
        <Field label="Label">
          <input value={value.label} onChange={(e) => set({ label: e.target.value })} />
        </Field>
        <Field label="Image (relative path, optional)">
          <input value={value.image ?? ""} onChange={(e) => set({ image: e.target.value || null })} />
        </Field>
      </div>
      <div className="row three">
        <Field label="Background">
          <input type="color" value={value.bg.toLowerCase()} onChange={(e) => set({ bg: e.target.value.toUpperCase() })} />
        </Field>
        <Field label="Text color">
          <input type="color" value={value.fg.toLowerCase()} onChange={(e) => set({ fg: e.target.value.toUpperCase() })} />
        </Field>
        <Field label="Action">
          <select
            value={value.action.type}
            onChange={(e) => setAction(blankAction(e.target.value as Action["type"], builtins[0] ?? ""))}
          >
            {(Object.keys(ACTION_LABELS) as Action["type"][]).map((type) => (
              <option key={type} value={type}>
                {ACTION_LABELS[type]}
              </option>
            ))}
          </select>
        </Field>
      </div>
      <ActionFields action={value.action} builtins={builtins} pageCount={pageCount} onChange={setAction} />
      <div>
        <button className="danger" onClick={onClear}>
          Clear key
        </button>
      </div>
    </section>
  );
}

function ActionFields({
  action,
  builtins,
  pageCount,
  onChange,
}: {
  action: Action;
  builtins: string[];
  pageCount: number;
  onChange: (action: Action) => void;
}) {
  switch (action.type) {
    case "none":
      return null;
    case "open_url":
      return (
        <Field label="URL">
          <input value={action.url} onChange={(e) => onChange({ type: "open_url", url: e.target.value })} />
        </Field>
      );
    case "run_command":
      return (
        <Field label="Command (run with sh -c)">
          <input value={action.command} onChange={(e) => onChange({ type: "run_command", command: e.target.value })} />
        </Field>
      );
    case "open_app":
      return (
        <Field label="App name">
          <input value={action.name} onChange={(e) => onChange({ type: "open_app", name: e.target.value })} />
        </Field>
      );
    case "hotkey":
      return (
        <Field label="Shortcut (for example cmd+shift+4)">
          <input value={action.keys} onChange={(e) => onChange({ type: "hotkey", keys: e.target.value })} />
        </Field>
      );
    case "go_to_page":
      return (
        <Field label="Page number (0 is the first page)">
          <input
            type="number"
            min={0}
            max={Math.max(pageCount - 1, 0)}
            value={action.page}
            onChange={(e) => onChange({ type: "go_to_page", page: Number(e.target.value) })}
          />
        </Field>
      );
    case "builtin":
      return (
        <Field label="Built-in">
          <select value={action.name} onChange={(e) => onChange({ type: "builtin", name: e.target.value })}>
            {builtins.map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </Field>
      );
  }
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return <label>{label}{children}</label>;
}
