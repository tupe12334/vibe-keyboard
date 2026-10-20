import type { BarStyle, EditorRules } from "./bindings";
import { ColorOverride } from "./ColorOverride";
import { Field } from "./Field";

/** The CPU bar look of this side screen. A field left off keeps the built-in look. */
export function BarStyleFields({
  style,
  rules,
  textColor,
  onChange,
}: {
  style: BarStyle | null;
  rules: EditorRules;
  /** The side text color. The percent text uses it until a label color is set. */
  textColor: string;
  onChange: (style: BarStyle | null) => void;
}) {
  const current: BarStyle = style ?? {};
  const set = (patch: Partial<BarStyle>) => onChange({ ...current, ...patch });
  const thickness = current.thickness ?? rules.bar_thickness_default;

  return (
    <div className="bar-style">
      <strong>Bar style</strong>
      <ColorOverride
        label="Fill"
        value={current.fill}
        fallback={rules.bar_fill_default}
        offLabel="Green, amber, red"
        onChange={(fill) => set({ fill })}
      />
      <ColorOverride
        label="Track"
        value={current.track}
        fallback={rules.bar_track_default}
        offLabel="Default"
        onChange={(track) => set({ track })}
      />
      <ColorOverride
        label="Value color"
        value={current.label_color}
        fallback={textColor}
        offLabel="Text color"
        onChange={(label_color) => set({ label_color })}
      />
      <Field label={`Thickness ${thickness} px`}>
        <input
          type="range"
          min={rules.bar_thickness_min}
          max={rules.bar_thickness_max}
          value={thickness}
          onChange={(e) => set({ thickness: Number(e.target.value) })}
        />
      </Field>
      <Field label="Show value">
        <input type="checkbox" checked={current.show_value ?? true} onChange={(e) => set({ show_value: e.target.checked })} />
      </Field>
      <div>
        <button disabled={style === null} onClick={() => onChange(null)}>
          Reset bar style
        </button>
      </div>
    </div>
  );
}
