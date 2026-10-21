import { useEffect, useState } from "react";
import { commands } from "./bindings";

/** Muted build line, so a screenshot of any editor bug names its build. Empty until loaded. */
export function BuildFooter() {
  const [line, setLine] = useState("");
  useEffect(() => {
    void commands.buildInfo().then(setLine);
  }, []);
  return <span className="build-info">{line}</span>;
}
