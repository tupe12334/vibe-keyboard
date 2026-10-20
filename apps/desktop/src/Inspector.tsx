import type { ReactNode } from "react";
import type { DisplaySettings_Serialize as DisplaySettings, EditorRules } from "./bindings";
import { DisplayPanel } from "./DisplayPanel";
import { HoldPanel } from "./HoldPanel";
import { INSPECTOR_TABS, type InspectorTab, InspectorTabs } from "./InspectorTabs";
import { KeyPanel } from "./KeyPanel";
import { PromptsPanel } from "./PromptsPanel";
import { SidePanel } from "./SidePanel";
import type { KeyConfig, Page, Profile } from "./types";

interface Props {
  draft: Profile;
  page: number;
  /** The key picked on the deck, or null. */
  selected: number | null;
  rules: EditorRules;
  builtins: string[];
  previews: Record<number, string>;
  tab: InspectorTab;
  onTab: (tab: InspectorTab) => void;
  setKey: (key: number, config: KeyConfig | undefined) => void;
  setHoldMs: (ms: number) => void;
  setDraft: (next: Profile) => void;
  updatePage: (change: (p: Page) => Page) => void;
  setDisplay: (display: DisplaySettings) => void;
}

/**
 * Right column: a title, a segmented control, and one panel at a time. Hidden panels stay
 * mounted, so their local state survives a tab switch.
 */
export function Inspector(props: Props) {
  const { draft, page, selected, rules, builtins, previews, tab, onTab, setKey, setHoldMs, setDraft, updatePage, setDisplay } = props;
  const current = draft.pages[page];
  const pageCount = draft.pages.length;
  const hint = <p className="status">Select a key on the deck to edit it.</p>;
  const keyEdit = (key: number) => (config: KeyConfig) => setKey(key, config);

  const panels: Record<InspectorTab, ReactNode> = {
    key:
      selected === null ? hint : (
        <KeyPanel
          key={selected}
          config={current.keys[selected]}
          preview={previews[selected]}
          builtins={builtins}
          pageCount={pageCount}
          onChange={keyEdit(selected)}
        />
      ),
    hold:
      selected === null ? hint : (
        <HoldPanel
          key={selected}
          config={current.keys[selected]}
          builtins={builtins}
          pageCount={pageCount}
          holdMs={draft.input?.hold_ms ?? rules.default_hold_ms}
          rules={rules}
          onHoldMsChange={setHoldMs}
          onChange={keyEdit(selected)}
        />
      ),
    prompts: <PromptsPanel profile={draft} limits={rules} onChange={setDraft} />,
    side: <SidePanel side={current.side ?? null} onChange={(side) => updatePage((p) => ({ ...p, side }))} />,
    display: <DisplayPanel display={draft.display} onChange={setDisplay} />,
  };

  return (
    <aside className="inspector">
      <div className="inspector-head">
        <strong>{selected === null ? "No key selected" : `Key ${selected}`}</strong>
        {selected !== null && (
          <button className="danger" onClick={() => setKey(selected, undefined)}>
            Clear key
          </button>
        )}
      </div>
      <InspectorTabs active={tab} onSelect={onTab} />
      {INSPECTOR_TABS.map(({ id }) => (
        <div key={id} role="tabpanel" className="tabpanel" hidden={id !== tab}>
          {panels[id]}
        </div>
      ))}
    </aside>
  );
}
