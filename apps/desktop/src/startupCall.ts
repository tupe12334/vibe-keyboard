/** How long a startup backend call may run before the editor shows an error. */
export const STARTUP_TIMEOUT_MS = 5000;

/** Rejects when `call` has not settled within `ms`. */
export function withStartupTimeout<T>(call: Promise<T>, ms = STARTUP_TIMEOUT_MS): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  const timeout = new Promise<never>((_, reject) => {
    timer = setTimeout(() => reject(new Error(`did not respond within ${ms / 1000} s`)), ms);
  });
  return Promise.race([call, timeout]).finally(() => clearTimeout(timer));
}

/** The message the editor shows when a startup call fails. Names the call and the error text. */
export function startupFailure(name: string, cause: unknown): string {
  const text = cause instanceof Error ? cause.message : String(cause);
  return `${name} failed: ${text}`;
}
