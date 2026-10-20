import { Field } from "./Field";
import type { Profile } from "./types";
import { usePrompts } from "./usePrompts";

interface Props {
  profile: Profile;
  /** Length limits from the core crate, through `editorRules`. Null until they load. */
  limits: { prompt_label_max: number; prompt_text_max: number } | null;
  onChange: (next: Profile) => void;
}

/** Session prompts: label and text, reorder, delete, add. The rows scroll in their own box. Edits mark the layout as changed. */
export function PromptsPanel({ profile, limits, onChange }: Props) {
  const { prompts, update, add, move, remove } = usePrompts(profile, onChange);

  return (
    <section className="panel">
      <strong>Prompts</strong>
      {prompts.length === 0 && <div className="status">None set. Session actions show no prompt keys.</div>}
      <div className="scroll-list">
        {prompts.map((prompt, index) => (
          <div key={index}>
            <div className="row">
              <Field label="Label">
                <input
                  value={prompt.label}
                  maxLength={limits?.prompt_label_max}
                  onChange={(e) => update(index, { label: e.target.value })}
                />
              </Field>
              <Field label="Text">
                <input
                  value={prompt.text}
                  maxLength={limits?.prompt_text_max}
                  onChange={(e) => update(index, { text: e.target.value })}
                />
              </Field>
            </div>
            <div className="toolbar">
              <button disabled={index === 0} onClick={() => void move(index, false)}>
                Up
              </button>
              <button disabled={index === prompts.length - 1} onClick={() => void move(index, true)}>
                Down
              </button>
              <button className="danger" onClick={() => void remove(index)}>
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>
      <div>
        <button onClick={() => void add()}>Add prompt</button>
      </div>
    </section>
  );
}
