import type { Action, EditorRules } from "./bindings";
import { ACTION_LABELS, blankAction } from "./actions";
import { ActionFields } from "./ActionFields";
import { ActionProblem } from "./ActionProblem";
import { Field } from "./Field";

interface Props {
  /** Null when the key has no hold action. */
  hold: Action | null;
  holdMs: number;
  /** The bounds of the hold time, from the Rust rules. */
  rules: EditorRules;
  builtins: string[];
  pageCount: number;
  onChange: (hold: Action | null) => void;
  onHoldMsChange: (ms: number) => void;
}

/** The hold action of a key, and how long the key must stay down before it runs. */
export function HoldFields({ hold, holdMs, rules, builtins, pageCount, onChange, onHoldMsChange }: Props) {
  const pick = (type: Action["type"]) => onChange(type === "none" ? null : blankAction(type, builtins[0] ?? ""));
  return (
    <section className="panel">
      <div className="row three">
        <Field label="Hold action">
          <select value={hold?.type ?? "none"} onChange={(e) => pick(e.target.value as Action["type"])}>
            {(Object.keys(ACTION_LABELS) as Action["type"][]).map((type) => (
              <option key={type} value={type}>
                {ACTION_LABELS[type]}
              </option>
            ))}
          </select>
        </Field>
        <Field label={`Hold time: ${holdMs} ms`}>
          <input
            type="range"
            min={rules.hold_ms_min}
            max={rules.hold_ms_max}
            step={10}
            value={holdMs}
            onChange={(e) => onHoldMsChange(Number(e.target.value))}
          />
        </Field>
      </div>
      {hold && <ActionFields action={hold} builtins={builtins} pageCount={pageCount} onChange={onChange} />}
      {hold && <ActionProblem action={hold} pageCount={pageCount} />}
    </section>
  );
}
