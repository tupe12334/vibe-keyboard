import { useCallback, useEffect, useRef, useState } from "react";
import { commands, events, type EditorRules, type KeyConfig_Serialize, type Page_Serialize, type Profile_Serialize } from "./bindings";

// The `_Serialize` types are what the backend returns: every field is present.
type Profile = Profile_Serialize;
type Page = Page_Serialize;
type KeyConfig = KeyConfig_Serialize;
import { KeyPanel } from "./KeyPanel";

/** Logical key numbers, laid out as the deck shows them: 1–5 top row. */
const ROWS = [
  [1, 2, 3, 4, 5],
  [6, 7, 8, 9, 10],
  [11, 12, 13, 14, 15],
];
const KEYS = ROWS.flat();
interface DeckStatus {
  connected: boolean;
  running: boolean;
  error: string | null;
}

export function App() {
  const [saved, setSaved] = useState<Profile | null>(null);
  const [draft, setDraft] = useState<Profile | null>(null);
  const [page, setPage] = useState(0);
  const [selected, setSelected] = useState<number | null>(null);
  const [previews, setPreviews] = useState<Record<number, string>>({});
  const [pressed, setPressed] = useState<number | null>(null);
  const [deck, setDeck] = useState<DeckStatus>({ connected: false, running: false, error: null });
  const [rules, setRules] = useState<EditorRules | null>(null);
  const [builtins, setBuiltins] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const dirty = draft !== null && saved !== null && JSON.stringify(draft) !== JSON.stringify(saved);
  // Event handlers read this ref, so they see the current value without re-subscribing.
  const dirtyRef = useRef(false);
  dirtyRef.current = dirty;

  const load = useCallback(async () => {
    const result = await commands.getProfile();
    if (result.status === "error") {
      setError(result.error);
      return;
    }
    setSaved(result.data);
    setDraft(result.data);
    setPage((p) => Math.min(p, result.data.pages.length - 1));
    setError(null);
    setNotice(null);
  }, []);

  useEffect(() => {
    void load();
    void commands.listBuiltins().then(setBuiltins);
    void commands.editorRules().then(setRules);
    const refreshDeck = () => void commands.deviceStatus().then(setDeck);
    refreshDeck();
    const subscriptions = Promise.all([
      events.deviceConnected.listen(refreshDeck),
      events.deviceDisconnected.listen(refreshDeck),
      events.keyPressed.listen(({ payload }) => {
        setPressed(payload.key);
        setTimeout(() => setPressed((p) => (p === payload.key ? null : p)), 300);
      }),
      events.profileReloaded.listen(() => {
        if (dirtyRef.current) {
          setNotice("The layout changed on disk. Discard your edits to load it.");
        } else {
          void load();
        }
      }),
    ]);
    return () => {
      void subscriptions.then((unlisten) => unlisten.forEach((stop) => stop()));
    };
  }, [load]);

  // Preview the draft, so unsaved edits show on the grid. Debounced to skip keystrokes.
  useEffect(() => {
    if (!draft) return;
    let cancelled = false;
    const timer = setTimeout(async () => {
      const entries = await Promise.all(
        KEYS.map(async (key) => {
          const result = await commands.keyPreview(draft, page, key);
          return [key, result.status === "ok" ? result.data : ""] as const;
        }),
      );
      if (!cancelled) setPreviews(Object.fromEntries(entries));
    }, 120);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  }, [draft, page]);

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
  const save = async () => {
    const result = await commands.saveProfile(draft);
    if (result.status === "error") {
      setError(result.error);
      return;
    }
    setSaved(draft);
    setError(null);
    setNotice(null);
  };

  return (
    <div className="shell">
      <aside className="pages">
        <strong>Pages</strong>
        {draft.pages.map((p, i) => (
          <button
            key={i}
            className={`page${i === page ? " active" : ""}`}
            onClick={() => {
              setPage(i);
              setSelected(null);
            }}
          >
            {p.name}
          </button>
        ))}
        <button onClick={addPage}>Add page</button>
      </aside>

      <main className="main">
        <div className="toolbar">
          <label>
            Page name
            <input value={current.name} onChange={(e) => updatePage((p) => ({ ...p, name: e.target.value }))} />
          </label>
          <button onClick={deletePage} disabled={draft.pages.length <= (rules?.min_pages ?? 1)} className="danger">
            Delete page
          </button>
          <span className="spacer" />
          <span className="status">
            Device {deck.connected ? "connected" : "not connected"} · controller {deck.running ? "running" : "paused"}
          </span>
          <button onClick={() => void load()} disabled={!dirty && !notice}>
            Discard
          </button>
          <button className="primary" onClick={() => void save()} disabled={!dirty}>
            Save
          </button>
        </div>
        {deck.error && <div className="error">{deck.error}</div>}
        {error && <div className="error">{error}</div>}
        {notice && <div className="error">{notice}</div>}

        <div className="grid">
          {KEYS.map((key) => {
            const nav = navKeys.has(key);
            const classes = ["key", selected === key && "selected", pressed === key && "pressed"]
              .filter(Boolean)
              .join(" ");
            return (
              <button
                key={key}
                className={classes}
                disabled={nav}
                title={nav ? "Navigation key" : `Key ${key}`}
                onClick={() => setSelected(key)}
              >
                {previews[key] && <img src={previews[key]} alt={`Key ${key}`} />}
                <span className="num">{key}</span>
              </button>
            );
          })}
        </div>

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
