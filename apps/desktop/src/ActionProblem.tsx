import { useEffect, useState } from "react";
import { type Action, commands } from "./bindings";

/** The reason an action cannot run, from the Rust rule. Shown in red, as the egui editor does. */
export function ActionProblem({ action, pageCount }: { action: Action; pageCount: number }) {
  const [problem, setProblem] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    void commands.actionProblem(action, pageCount).then((found) => {
      if (live) setProblem(found);
    });
    return () => {
      live = false;
    };
  }, [action, pageCount]);
  return problem ? <p className="error">{problem}</p> : null;
}
