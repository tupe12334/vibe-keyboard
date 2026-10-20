import type { Action } from "./bindings";

// Typed against the generated `Action` type, so a new or renamed action fails to compile here.
export const ACTION_LABELS: Record<Action["type"], string> = {
  none: "None",
  open_url: "Open URL",
  run_command: "Run command",
  open_app: "Open app",
  hotkey: "Hotkey",
  go_to_page: "Go to page",
  builtin: "Built-in",
  push_to_talk: "Push to talk",
};

export function blankAction(type: Action["type"], firstBuiltin: string): Action {
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
