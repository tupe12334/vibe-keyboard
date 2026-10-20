export const INSPECTOR_TABS = [
  { id: "key", label: "Key" },
  { id: "hold", label: "Hold" },
  { id: "prompts", label: "Prompts" },
  { id: "side", label: "Side" },
  { id: "display", label: "Display" },
] as const;

export type InspectorTab = (typeof INSPECTOR_TABS)[number]["id"];

/** Segmented control: one button per inspector panel. Only the active panel is shown. */
export function InspectorTabs({ active, onSelect }: { active: InspectorTab; onSelect: (tab: InspectorTab) => void }) {
  return (
    <div className="segmented" role="tablist">
      {INSPECTOR_TABS.map(({ id, label }) => (
        <button
          key={id}
          role="tab"
          aria-selected={id === active}
          className={id === active ? "active" : undefined}
          onClick={() => onSelect(id)}
        >
          {label}
        </button>
      ))}
    </div>
  );
}
