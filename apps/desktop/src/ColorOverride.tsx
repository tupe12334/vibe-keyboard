import { Field } from "./Field";

/** A color that is either off (the built-in color is used) or on, with its own `#RRGGBB`. */
export function ColorOverride({
  label,
  value,
  fallback,
  offLabel,
  onChange,
}: {
  label: string;
  value: string | null | undefined;
  /** The color a switched-on override starts from. */
  fallback: string;
  /** What the off state says, such as "Default". */
  offLabel: string;
  onChange: (value: string | null) => void;
}) {
  const on = value != null;
  return (
    <Field label={label}>
      <span className="toolbar">
        <input type="checkbox" checked={on} onChange={(e) => onChange(e.target.checked ? fallback : null)} />
        <span>{on ? "Custom" : offLabel}</span>
        {value != null && <input type="color" value={value.toLowerCase()} onChange={(e) => onChange(e.target.value.toUpperCase())} />}
      </span>
    </Field>
  );
}
