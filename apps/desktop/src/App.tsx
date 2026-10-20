import { useEffect, useState } from "react";
import { commands, type EditorRules } from "./bindings";
import { Inspector } from "./Inspector";
import type { InspectorTab } from "./InspectorTabs";
import { KeyGrid } from "./KeyGrid";
import { PageList } from "./PageList";
import { QuitPrompt } from "./QuitPrompt";
import { Toolbar } from "./Toolbar";
import type { Page } from "./types";
import { useDeckEvents } from "./useDeckEvents";
import { usePageActions } from "./usePageActions";
import { usePreviews } from "./usePreviews";
import { useProfile } from "./useProfile";

export function App() {
  const { draft, setDraft, page, setPage, dirty, error, setError, notice, conflict, load, onReload, save, updatePage, setKey, setHoldMs, setDisplay } =
    useProfile();
  const { deck, pressed } = useDeckEvents(onReload);
  const previews = usePreviews(draft, page);
  const [selected, setSelected] = useState<number | null>(null);
  const [tab, setTab] = useState<InspectorTab>("key");
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
  // Picking a key shows its fields first.
  const pickKey = (key: number) => {
    setSelected(key);
    setTab("key");
  };

  return (
    <div className="shell">
      <div className="topbar">
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
      </div>
      <PageList
        pages={draft.pages}
        page={page}
        onSelect={(i) => {
          setPage(i);
          setSelected(null);
        }}
        onAdd={addPage}
      />
      <main className="center">
        {deck.error && <div className="error">{deck.error}</div>}
        {error && <div className="error">{error}</div>}
        {notice && <div className="error">{notice}</div>}
        {conflict && (
          <div className="error">
            profile.toml changed on disk. Reload to discard your edits, or overwrite the disk version.
            <button onClick={() => void load()}>Reload from disk</button>
            <button onClick={() => void save(true)}>Overwrite</button>
          </div>
        )}
        <KeyGrid navKeys={navKeys} selected={selected} pressed={pressed} previews={previews} onSelect={pickKey} />
      </main>
      <QuitPrompt dirty={dirty} save={save} />
      <Inspector
        draft={draft}
        page={page}
        selected={selected}
        rules={rules}
        builtins={builtins}
        previews={previews}
        tab={tab}
        onTab={setTab}
        setKey={setKey}
        setHoldMs={setHoldMs}
        setDraft={setDraft}
        updatePage={updatePage}
        setDisplay={setDisplay}
      />
    </div>
  );
}
