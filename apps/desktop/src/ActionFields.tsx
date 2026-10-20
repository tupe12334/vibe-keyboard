import type { Action } from "./bindings";
import { Field } from "./Field";

/** The inputs for the chosen action type. Each type has its own fields. */
export function ActionFields({
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
