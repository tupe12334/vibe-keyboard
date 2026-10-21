import { useEffect, useState, type Dispatch, type SetStateAction } from "react";
import { commands, type EditorRules } from "./bindings";
import { startupFailure, withStartupTimeout } from "./startupCall";

/** Runs the startup calls. Returns the first failure and a retry that runs them again. */
export function useStartupCalls(
  load: () => Promise<void>,
  setBuiltins: Dispatch<SetStateAction<string[]>>,
  setRules: Dispatch<SetStateAction<EditorRules | null>>,
) {
  const [startupError, setStartupError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    setStartupError(null);
    void load();
    withStartupTimeout(commands.listBuiltins()).then(setBuiltins, (e) => setStartupError(startupFailure("listBuiltins", e)));
    withStartupTimeout(commands.editorRules()).then(setRules, (e) => setStartupError(startupFailure("editorRules", e)));
  }, [load, attempt, setBuiltins, setRules]);

  return { startupError, retry: () => setAttempt((n) => n + 1) };
}
