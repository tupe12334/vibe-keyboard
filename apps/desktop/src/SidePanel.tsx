import type { SideConfig_Serialize as SideConfig } from "./bindings";
import { Field } from "./Field";
import { ImageFields } from "./ImageFields";
import { SideWidgets } from "./SideWidgets";

// Used when a page has no side screen yet. The first edit writes it into the layout.
const BLANK_SIDE: SideConfig = { label: "", bg: "#141414", fg: "#E6E6E6", image: null, widgets: [] };

/** The side screen of the current page: add or remove it, then its label, colors, image and widgets. */
export function SidePanel({ side, onChange }: { side: SideConfig | null; onChange: (side: SideConfig | null) => void }) {
  if (!side) {
    return (
      <section className="panel">
        <strong>Side screen</strong>
        <div>
          <button onClick={() => onChange(BLANK_SIDE)}>Add side screen</button>
        </div>
      </section>
    );
  }
  const set = (patch: Partial<SideConfig>) => onChange({ ...side, ...patch });

  return (
    <section className="panel">
      <strong>Side screen</strong>
      <div className="row three">
        <Field label="Label">
          <input value={side.label} onChange={(e) => set({ label: e.target.value })} />
        </Field>
        <Field label="Background">
          <input type="color" value={side.bg.toLowerCase()} onChange={(e) => set({ bg: e.target.value.toUpperCase() })} />
        </Field>
        <Field label="Text color">
          <input type="color" value={side.fg.toLowerCase()} onChange={(e) => set({ fg: e.target.value.toUpperCase() })} />
        </Field>
      </div>
      <ImageFields image={side.image ?? null} onChange={(image) => set({ image })} />
      <SideWidgets widgets={side.widgets ?? []} onChange={(widgets) => set({ widgets })} />
      <div>
        <button className="danger" onClick={() => onChange(null)}>
          Remove side screen
        </button>
      </div>
    </section>
  );
}
