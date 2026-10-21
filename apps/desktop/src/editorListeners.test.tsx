import { render } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { registerEditorListeners } from "./editorListeners";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve(undefined)),
}));

describe("editor listeners", () => {
  beforeAll(() => registerEditorListeners());
  beforeEach(() => vi.mocked(invoke).mockClear());

  it("sends one log_webview_error for a window error, with its message and place", () => {
    window.dispatchEvent(new ErrorEvent("error", { message: "boom", filename: "app.js", lineno: 3 }));
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("log_webview_error", {
      kind: "error",
      message: "boom at app.js:3",
    });
  });

  it("sends one log_webview_error for an unhandled rejection, with the reason", () => {
    const event = Object.assign(new Event("unhandledrejection"), { reason: new Error("nope") });
    window.dispatchEvent(event);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("log_webview_error", {
      kind: "unhandledrejection",
      message: expect.stringContaining("nope"),
    });
  });

  it("sends one log_webview_error for a CSP violation, with the directive and URL", () => {
    const event = Object.assign(new Event("securitypolicyviolation"), {
      violatedDirective: "script-src",
      blockedURI: "inline",
    });
    window.dispatchEvent(event);
    expect(invoke).toHaveBeenCalledTimes(1);
    expect(invoke).toHaveBeenCalledWith("log_webview_error", {
      kind: "csp",
      message: "script-src blocked inline",
    });
  });

  it("still sends one line per event after a React re-render", () => {
    const { rerender } = render(<p>editor</p>);
    rerender(<p>editor</p>);
    window.dispatchEvent(new ErrorEvent("error", { message: "again", filename: "a.js", lineno: 1 }));
    expect(invoke).toHaveBeenCalledTimes(1);
  });
});
