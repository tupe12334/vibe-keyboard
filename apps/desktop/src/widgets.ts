import type { Widget } from "./bindings";

export type WidgetKind = Widget["type"];

// Kinds in the order the add buttons show them. Labels are typed per kind, so a new kind fails to compile here.
export const WIDGET_KINDS: WidgetKind[] = ["cpu_percent", "cpu_bar", "temperature", "disk_free", "text"];

export const WIDGET_LABELS: Record<WidgetKind, string> = {
  cpu_percent: "CPU %",
  cpu_bar: "CPU bar",
  temperature: "Temperature",
  disk_free: "Disk free",
  text: "Text",
};

export function blankWidget(kind: WidgetKind): Widget {
  return kind === "text" ? { type: "text", value: "" } : ({ type: kind } as Widget);
}
