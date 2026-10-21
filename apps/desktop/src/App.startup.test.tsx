import { act, cleanup, render, screen, fireEvent } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { commands } from "./bindings";
import { KEYS } from "./deckLayout";

// Every command is a mock fn; any event listener resolves to a no-op unlisten.
vi.mock("./bindings", () => {
  const fns = new Map<string, unknown>();
  const fnsProxy = () => new Proxy({}, { get: (_, k: string) => fns.get(k) ?? fns.set(k, vi.fn()).get(k) });
  return {
    commands: fnsProxy(),
    events: new Proxy({}, { get: () => ({ listen: vi.fn(async () => () => {}) }) }),
  };
});

const profile = { pages: [{ name: "Main", keys: {} }], input: { hold_ms: 500 }, display: {} } as never;
const never = () => new Promise<never>(() => {});

/** Lets pending promise callbacks run without advancing fake timers. */
async function flush() {
  await act(async () => {
    await Promise.resolve();
  });
}

describe("App startup load", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.mocked(commands.getProfile).mockResolvedValue({ status: "ok", data: profile });
    vi.mocked(commands.listBuiltins).mockResolvedValue([]);
    vi.mocked(commands.editorRules).mockResolvedValue({ nav_keys: [], min_pages: 1 } as never);
    vi.mocked(commands.deviceStatus).mockResolvedValue({ connected: false, running: false, error: null } as never);
    vi.mocked(commands.keyPreview).mockResolvedValue({ status: "ok", data: "" } as never);
    vi.mocked(commands.displayDefaults).mockResolvedValue({} as never);
    vi.mocked(commands.buildInfo).mockResolvedValue("test build");
  });

  afterEach(() => {
    cleanup();
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it("shows an error with a Retry button when getProfile hangs past 5 s", async () => {
    vi.mocked(commands.getProfile).mockImplementation(never);
    render(<App />);
    await flush();
    expect(screen.getByText("Loading…")).toBeTruthy();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(4999);
    });
    expect(screen.queryByText(/getProfile failed/)).toBeNull();

    await act(async () => {
      await vi.advanceTimersByTimeAsync(1);
    });
    expect(screen.getByText(/getProfile failed: did not respond within 5 s/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy();
  });

  it("shows the call name and error text at once when a call rejects", async () => {
    vi.mocked(commands.editorRules).mockRejectedValue(new Error("rules missing"));
    render(<App />);
    await flush();
    expect(screen.getByText("editorRules failed: rules missing")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy();
    expect(screen.queryByText("Loading…")).toBeNull();
  });

  it("runs the startup calls again when Retry is clicked", async () => {
    vi.mocked(commands.getProfile).mockImplementationOnce(never);
    render(<App />);
    await act(async () => {
      await vi.advanceTimersByTimeAsync(5000);
    });
    expect(screen.getByText(/getProfile failed/)).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    await flush();
    expect(screen.queryByText(/getProfile failed/)).toBeNull();
    expect(screen.getAllByTitle(/^Key \d+$/)).toHaveLength(KEYS.length);
  });

  it("shows the keys once every startup call resolves", async () => {
    render(<App />);
    await flush();
    expect(screen.queryByText("Loading…")).toBeNull();
    expect(screen.getAllByTitle(/^Key \d+$/)).toHaveLength(KEYS.length);
  });
});
