import { commands } from "./bindings";

/** Sends a webview error to app.log. Never throws: an error handler must not raise another. */
function logToAppLog(kind: string, message: string): void {
  try {
    commands.logWebviewError(kind, message).catch(() => {});
  } catch {
    // The invoke bridge is gone. Nothing else to do.
  }
}

function describe(value: unknown): string {
  if (value instanceof Error) return value.stack ?? value.message;
  return String(value);
}

/** Forwards webview errors, unhandled rejections and CSP violations to app.log. Call once per page load. */
export function registerEditorListeners(): void {
  window.addEventListener("error", (event) => {
    logToAppLog("error", `${event.message} at ${event.filename}:${event.lineno}`);
  });

  window.addEventListener("unhandledrejection", (event) => {
    logToAppLog("unhandledrejection", describe(event.reason));
  });

  window.addEventListener("securitypolicyviolation", (event) => {
    logToAppLog("csp", `${event.violatedDirective} blocked ${event.blockedURI}`);
  });
}
