// Mirrors the Rust DTOs exactly, field-for-field, including their literal
// (snake_case) JSON field names — see `src-tauri/src/dto.rs` and
// `src-tauri/src/error.rs`. There is deliberately no `SecretValue`/passphrase
// type here: a revealed secret crosses the bridge as a plain `string` only
// in direct response to an explicit reveal/copy action (never by default),
// and a passphrase only ever travels *into* a command argument, never back
// out of one.

export interface RecentProject {
  path: string;
  last_opened: string;
}

export interface OpenProjectResult {
  path: string;
  has_vault: boolean;
}

export interface EnvironmentInfo {
  name: string;
  active: boolean;
  created_at: string;
}

export interface ScanFinding {
  path: string;
  line: number;
  rule_name: string;
  redacted_preview: string;
}

export interface AppInfo {
  format_version: number;
  project_path: string;
  app_version: string;
  license: string;
}

// --- Error shape (src-tauri/src/error.rs's `ErrorDto`) ---

export type ErrorKind =
  | "wrong_passphrase"
  | "not_found"
  | "already_exists"
  | "not_a_git_repository"
  | "general"
  | "session_state";

export interface ErrorPayload {
  kind: ErrorKind;
  message: string;
}

function isErrorPayload(err: unknown): err is ErrorPayload {
  return (
    typeof err === "object" &&
    err !== null &&
    "kind" in err &&
    "message" in err &&
    typeof (err as { message: unknown }).message === "string"
  );
}

/**
 * Every `VaultError`-derived message is already the CLI's own exact wording
 * (`ErrorDto::from<VaultError>` in the Rust layer never re-derives text) —
 * this just recovers that string, falling back to a generic message only for
 * a genuinely unrecognized error shape (e.g. a Tauri-level IPC failure that
 * never reached our command handler at all).
 */
export function errorMessage(err: unknown): string {
  if (isErrorPayload(err)) {
    return err.message;
  }
  return "Something went wrong.";
}

export function errorKind(err: unknown): ErrorKind | null {
  return isErrorPayload(err) ? err.kind : null;
}
