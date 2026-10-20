import { useEffect, useState } from "react";
import { commands, type RunningApp } from "./bindings";
import { Field } from "./Field";

const APP_PREFIX = "app:";

/** The installed app whose icon a key shows. Type its bundle id, or pick a running app. */
export function AppIconFields({
  image,
  preview,
  onChange,
}: {
  image: string | null;
  preview?: string;
  onChange: (image: string | null) => void;
}) {
  const bundle = image?.startsWith(APP_PREFIX) ? image.slice(APP_PREFIX.length) : "";
  const [running, setRunning] = useState<RunningApp[]>([]);
  const [valid, setValid] = useState<boolean | null>(null);

  const refresh = () => {
    void commands.listRunningApps().then(setRunning);
  };

  useEffect(() => {
    refresh();
  }, []);

  // The bundle id rule comes from the core crate, through this command.
  useEffect(() => {
    if (!bundle) {
      setValid(null);
      return;
    }
    let current = true;
    void commands.checkBundleId(bundle).then((ok) => {
      if (current) setValid(ok);
    });
    return () => {
      current = false;
    };
  }, [bundle]);

  return (
    <div>
      <Field label="Bundle id">
        <input value={bundle} onChange={(e) => onChange(`${APP_PREFIX}${e.target.value}`)} />
      </Field>
      {valid === false && <div className="error">Not a bundle id.</div>}
      <Field label="Or pick a running app">
        <select
          value=""
          onChange={(e) => e.target.value && onChange(`${APP_PREFIX}${e.target.value}`)}
        >
          <option value="">Choose...</option>
          {running.map((app) => (
            <option key={app.bundle_id} value={app.bundle_id}>
              {app.name} ({app.bundle_id})
            </option>
          ))}
        </select>
      </Field>
      <button onClick={refresh}>Refresh</button>
      {preview && <img className="app-icon-preview" src={preview} alt="Key preview" />}
    </div>
  );
}
