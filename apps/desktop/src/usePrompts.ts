import { useRef } from "react";
import { commands, type Prompt } from "./bindings";
import type { Profile } from "./types";

/**
 * Edits to the session prompt list. Adding, moving and deleting rows run in the core crate;
 * label and text edits are plain values. Each edit calls `onChange` with the new layout.
 */
export function usePrompts(profile: Profile, onChange: (next: Profile) => void) {
  const prompts = profile.prompts ?? [];
  // A row command reads the layout it was given. A second one sent before the first returns
  // would overwrite it, so row commands run one at a time.
  const pending = useRef(false);

  const runRowCommand = async (call: () => Promise<Profile>) => {
    if (pending.current) return;
    pending.current = true;
    try {
      onChange(await call());
    } finally {
      pending.current = false;
    }
  };

  const update = (index: number, patch: Partial<Prompt>) =>
    onChange({ ...profile, prompts: prompts.map((p, i) => (i === index ? { ...p, ...patch } : p)) });

  const add = () => runRowCommand(() => commands.addPrompt(profile));

  const move = (index: number, down: boolean) => runRowCommand(() => commands.movePrompt(profile, index, down));

  const remove = (index: number) => runRowCommand(() => commands.deletePrompt(profile, index));

  return { prompts, update, add, move, remove };
}
