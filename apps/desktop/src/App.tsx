import { useEffect, useState } from "react";
import { commands, type EditorRules } from "./bindings";
import { KeyGrid } from "./KeyGrid";
import { KeyPanel } from "./KeyPanel";
import { PageList } from "./PageList";
import { Toolbar } from "./Toolbar";
import type { KeyConfig, Page } from "./types";
import { useDeckEvents } from "./useDeckEvents";
import { usePreviews } from "./usePreviews";
import { useProfile } from "./useProfile";

export function App() {
  const { draft, setDraft, page, setPage, dirty, error, setError, notice, load, onReload, save } = useProfile();
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

  if (!draft) {
    return <div className="main">{error ? <div className="error">{error}</div> : "Loading…"}</div>;
  }

  const current: Page = draft.pages[page];
  const navKeys = new Set(rules?.nav_keys ?? []);
  const updatePage = (change: (p: Page) => Page) =>
    setDraft((d) => d && { ...d, pages: d.pages.map((p, i) => (i === page ? change(p) : p)) });
  const setKey = (key: number, config: KeyConfig | undefined) =>
    updatePage((p) => {
      const keys = { ...p.keys };
      if (config) keys[key] = config;
      else delete keys[key];
      return { ...p, keys };
    });
  // Page rules come from the core crate, through these commands.
  const addPage = async () => {
    setDraft(await commands.addPage(draft));
    setPage(draft.pages.length);
  };
  const deletePage = async () => {
    const result = await commands.deletePage(draft, page);
    if (result.status === "error") {
      setError(result.error);
      return;
    }
    setDraft(result.data);
    setPage((p) => Math.max(0, p - 1));
    setSelected(null);
  };

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

        {selected !== null && (
          <KeyPanel
            key={selected}
            keyNumber={selected}
            config={current.keys[selected]}
            builtins={builtins}
            pageCount={draft.pages.length}
            onChange={(config) => setKey(selected, config)}
            onClear={() => setKey(selected, undefined)}
          />
        )}
      </main>
    </div>
  );
}
