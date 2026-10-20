import { useEffect, useState } from "react";
import {
  commands,
  type DisplayDefaults,
  type DisplaySettings_Serialize as DisplaySettings,
  type KeyInset,
  type SideInset,
} from "./bindings";
import { Field } from "./Field";

const KEY_EDGES = ["top", "bottom", "left", "right"] as const;
const SIDE_EDGES = ["top", "bottom"] as const;

// Whole pixels, never negative. The core rejects anything else when the layout is saved.
const px = (value: string) => Math.max(0, Math.floor(Number(value)) || 0);

/** Hidden bezel bands in px: key margins and side screen margins. Unset values use the built-in ones. */
export function DisplayPanel({
  display,
  onChange,
}: {
  display: DisplaySettings | undefined;
  onChange: (display: DisplaySettings) => void;
}) {
  const [defaults, setDefaults] = useState<DisplayDefaults | null>(null);
  useEffect(() => {
    void commands.displayDefaults().then(setDefaults);
  }, []);
  if (!defaults) return null;

  const key: KeyInset = { ...defaults.key_inset, ...display?.key_inset };
  const side: SideInset = { ...defaults.side_inset, ...display?.side_inset };
  const setKey = (edge: keyof KeyInset, value: string) =>
    onChange({ key_inset: { ...key, [edge]: px(value) }, side_inset: side });
  const setSide = (edge: keyof SideInset, value: string) =>
    onChange({ key_inset: key, side_inset: { ...side, [edge]: px(value) } });

  return (
    <section className="panel">
      <strong>Display (hidden bezel band, px)</strong>
      <div className="row">
        {KEY_EDGES.map((edge) => (
          <Field key={edge} label={`Key ${edge}`}>
            <input type="number" min={0} value={key[edge]} onChange={(e) => setKey(edge, e.target.value)} />
          </Field>
        ))}
      </div>
      <div className="row">
        {SIDE_EDGES.map((edge) => (
          <Field key={edge} label={`Side ${edge}`}>
            <input type="number" min={0} value={side[edge]} onChange={(e) => setSide(edge, e.target.value)} />
          </Field>
        ))}
      </div>
    </section>
  );
}
