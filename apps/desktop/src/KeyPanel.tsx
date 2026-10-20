import type { Action, KeyConfig_Serialize as KeyConfig } from "./bindings";
import { ACTION_LABELS, blankAction } from "./actions";
import { ActionFields } from "./ActionFields";
import { ActionProblem } from "./ActionProblem";
import { Field } from "./Field";
import { ImageFields } from "./ImageFields";

// Used when a key has no config yet. The first edit writes it into the layout.
export const BLANK_KEY: KeyConfig = {
  label: "",
  bg: "#141414",
  fg: "#E6E6E6",
  image: null,
  action: { type: "none" },
};

interface Props {
  /** Undefined when the key has no config yet. */
  config: KeyConfig | undefined;
  /** The key's rendered preview, from the draft. */
  preview?: string;
  builtins: string[];
  pageCount: number;
  onChange: (config: KeyConfig) => void;
}

/** The fields of the selected key: label, colors, image and press action. The hold is in HoldPanel. */
export function KeyPanel({ config, preview, builtins, pageCount, onChange }: Props) {
  const value = config ?? BLANK_KEY;
  const set = (patch: Partial<KeyConfig>) => onChange({ ...value, ...patch });
  const setAction = (action: Action) => set({ action });

  return (
    <section className="panel">
      <div className="row">
        <Field label="Label">
          <input value={value.label} onChange={(e) => set({ label: e.target.value })} />
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
      <div className="row">
        <Field label="Background">
          <input type="color" value={value.bg.toLowerCase()} onChange={(e) => set({ bg: e.target.value.toUpperCase() })} />
        </Field>
        <Field label="Text color">
          <input type="color" value={value.fg.toLowerCase()} onChange={(e) => set({ fg: e.target.value.toUpperCase() })} />
        </Field>
      </div>
      <ImageFields image={value.image ?? null} preview={preview} onChange={(image) => set({ image })} />
      <ActionFields action={value.action} builtins={builtins} pageCount={pageCount} onChange={setAction} />
      <ActionProblem action={value.action} pageCount={pageCount} />
    </section>
  );
}
