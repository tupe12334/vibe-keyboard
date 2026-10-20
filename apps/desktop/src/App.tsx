import { useEffect, useState } from "react";
import { commands, type EditorRules } from "./bindings";
import { KeyGrid } from "./KeyGrid";
import { KeyPanel } from "./KeyPanel";
import { PageList } from "./PageList";
import { DisplayPanel } from "./DisplayPanel";
import { PromptsPanel } from "./PromptsPanel";
import { Toolbar } from "./Toolbar";
import type { Page } from "./types";
import { useDeckEvents } from "./useDeckEvents";
import { usePageActions } from "./usePageActions";
import { usePreviews } from "./usePreviews";
import { useProfile } from "./useProfile";

export function App() {
  const { draft, setDraft, page, setPage, dirty, error, setError, notice, load, onReload, save, updatePage, setKey, setHoldMs, setDisplay } =
    useProfile();
  const { deck, pressed } = useDeckEvents(onReload);
  const previews = usePreviews(draft, page);
  const [selected, setSelected] = useState<number | null>(null);
  const [rules, setRules] = useState<EditorRules | null>(null);
  const [builtins, setBuiltins] = useState<string[]>([]);

  useEffect(() => {
    void load();
    void commands.listBuiltins().then(setBuiltins);
    void commands.editorRules().then(setRules);
  }, [load]);

  if (!draft || !rules) {
    return <div className="main">{error ? <div className="error">{error}</div> : "Loading…"}</div>;
  }

  const current: Page = draft.pages[page];
  const navKeys = new Set(rules?.nav_keys ?? []);
  const { addPage, deletePage } = usePageActions({ draft, page, setDraft, setPage, setError, setSelected });

  return (
    <div className="shell">
      <PageList
        pages={draft.pages}
        page={page}
        onSelect={(i) => {
          setPage(i);
          setSelected(null);
        }}
        onAdd={addPage}
      />
      <main className="main">
        <Toolbar
          name={current.name}
          onRename={(name) => updatePage((p) => ({ ...p, name }))}
          deleteDisabled={!rules || draft.pages.length <= rules.min_pages}
          onDelete={deletePage}
          deck={deck}
          dirty={dirty}
          discardDisabled={!dirty && !notice}
          onDiscard={() => void load()}
          onSave={() => void save()}
        />
        {deck.error && <div className="error">{deck.error}</div>}
        {error && <div className="error">{error}</div>}
        {notice && <div className="error">{notice}</div>}

        <KeyGrid navKeys={navKeys} selected={selected} pressed={pressed} previews={previews} onSelect={setSelected} />

        <PromptsPanel profile={draft} limits={rules} onChange={setDraft} />
        <DisplayPanel display={draft.display} onChange={setDisplay} />

        {selected !== null && (
          <KeyPanel
            key={selected}
            keyNumber={selected}
            preview={previews[selected]}
            config={current.keys[selected]}
            builtins={builtins}
            pageCount={draft.pages.length}
            holdMs={draft.input?.hold_ms ?? rules.default_hold_ms}
            rules={rules}
            onHoldMsChange={setHoldMs}
            onChange={(config) => setKey(selected, config)}
            onClear={() => setKey(selected, undefined)}
          />
        )}
      </main>
    </div>
  );
}
