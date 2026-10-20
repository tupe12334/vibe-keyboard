import type { Widget } from "./bindings";
import { blankWidget, WIDGET_KINDS, WIDGET_LABELS } from "./widgets";

/** The widget list of the side screen: edit text, reorder, remove, and add one of each kind. The list scrolls in its own box. */
export function SideWidgets({ widgets, onChange }: { widgets: Widget[]; onChange: (widgets: Widget[]) => void }) {
  const move = (from: number, to: number) => {
    const next = [...widgets];
    [next[from], next[to]] = [next[to], next[from]];
    onChange(next);
  };
  const update = (index: number, widget: Widget) => onChange(widgets.map((w, i) => (i === index ? widget : w)));
  const remove = (index: number) => onChange(widgets.filter((_, i) => i !== index));

  return (
    <div className="widgets">
      <strong>Widgets</strong>
      <div className="scroll-list">
        {widgets.map((widget, i) => (
          <div className="toolbar" key={i}>
            <span>{WIDGET_LABELS[widget.type]}</span>
            {widget.type === "text" && (
              <input value={widget.value ?? ""} onChange={(e) => update(i, { type: "text", value: e.target.value })} />
            )}
            <button disabled={i === 0} onClick={() => move(i, i - 1)}>
              Up
            </button>
            <button disabled={i === widgets.length - 1} onClick={() => move(i, i + 1)}>
              Down
            </button>
            <button className="danger" onClick={() => remove(i)}>
              Remove
            </button>
          </div>
        ))}
      </div>
      <div className="toolbar">
        {WIDGET_KINDS.map((kind) => (
          <button key={kind} onClick={() => onChange([...widgets, blankWidget(kind)])}>
            + {WIDGET_LABELS[kind]}
          </button>
        ))}
      </div>
    </div>
  );
}
