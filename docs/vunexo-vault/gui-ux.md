---
status: locked
round: 7 (GUI amendment)
---

# Vunexo Vault GUI — Screen Inventory & Navigation

Companion doc to `docs/vunexo-vault/ui-ux.md`'s sibling-app spirit, scoped to the optional desktop GUI amendment recorded in `.ai/product-vunexo-vault.md` (2026-09-06). The GUI is a second interface to the exact same shared `vunexo-vault` crate the CLI uses — every screen below is a thin visual wrapper around an existing `application::*` use case, never a new secret-handling code path.

## Navigation structure

Flat, six sections plus one gating screen, same shape as Expense Manager's sidebar (`App.tsx`): fixed left sidebar (project-initial badge + open project's folder name, nav buttons with a left accent bar on the active item, theme toggle pinned at the bottom), a centered content pane on the right.

```
┌───────────────┬─────────────────────────────┐
│  [P] my-app   │                             │
│               │                             │
│  Secrets      │      (active section)       │
│  Environments │                             │
│  Scan         │                             │
│  Hooks        │                             │
│  Settings     │                             │
│               │                             │
│  ☀/☾ toggle   │                             │
└───────────────┴─────────────────────────────┘
```

The sidebar and its sections only render once a project is open and unlocked (see below) — until then, the whole window is the Open Project / Unlock gate, matching Expense Manager's `BusinessSetup` full-screen-gate pattern.

## Screens

### 1. Open Project (gate)

Renders instead of the sidebar layout whenever no project is currently open. A centered card: "Open Folder" (via `tauri-plugin-dialog`'s folder picker) plus a "Recent Projects" list (paths persisted in the OS app-data dir — GUI-only preference state, not part of the portable `.vunexo/` folder). Selecting a folder that has no `.vunexo/` yet offers "Initialize a vault here" instead of failing.

### 2. Unlock (gate)

Renders instead of the sidebar layout whenever a project is open but not yet unlocked for this session. A centered card: environment selector (defaults to the project's `active_environment`) + a masked passphrase field + Unlock. A wrong passphrase shows the same generic "Incorrect passphrase" message the CLI gives — never a different, more specific error (same oracle-avoidance principle as `user-flows.md` §7). Successful unlock stores the passphrase only in Tauri-managed in-memory state for the session; nothing is written to disk.

### 3. Secrets (default section once unlocked)

A `.card`-wrapped table: key name column + a masked value column (dots, with a per-row reveal-eye toggle and a copy-to-clipboard button — copying doesn't require revealing first), plus row actions (Edit, Remove) as pill buttons. An "Add Secret" button opens a small form (key + value, value field masked with a show/hide toggle). Wraps `application::secrets::{set_secret,get_secret,list_secrets,remove_secret}` exactly — no new validation rules beyond what those use cases already enforce.

### 4. Environments

A simple list (same active-environment `*`/highlight treatment as the CLI's `env list`) with a "New Environment" button (name input, created empty — matching the CLI's own deferred-vault-file-creation behavior, see `application-architecture.md`) and a "Switch to" action per row. Wraps `application::environments::*`.

### 5. Scan

A "Run Scan" button (plus a "Staged only" toggle, enabled only when the open project is a git repository) and a results table: file, line, rule name, redacted preview — the exact same `ScanFinding` shape the CLI prints, never the full matched string. An empty-state "No findings — clean" mirrors the CLI's own colored summary line. Wraps `application::scan::{scan_path,scan_staged}`.

### 6. Hooks

A single toggle-shaped control: "Pre-commit protection: Installed / Not installed," with an Install/Uninstall button that wraps the same `HookInstaller` port the CLI's `hooks install`/`uninstall` use. If a foreign (non-Vunexo) hook is already present, shows the same refusal message as the CLI rather than silently doing nothing.

### 7. Settings / About

Read-only: vault format version (from `config.toml`), the open project's path, app version, license. One action: "Lock" — clears the in-memory passphrase and returns to the Unlock screen without closing the app or forgetting the open project.

## Cross-cutting rules

- **No screen ever displays a full secret value by default.** Reveal is always an explicit per-row action (Secrets screen); Scan results only ever show the shared `redact_preview` output.
- **No screen writes a `.env` file or any plaintext secret to disk.** The GUI has no equivalent of `secrets run` — running a command with injected secrets is deliberately CLI-only (a GUI "run a shell command" feature would be new scope beyond what's locked; not built here).
- **Every action's error message matches the CLI's wording** for the same failure (wrong passphrase, not found, hook already exists) — one shared `VaultError` type, one set of messages, surfaced identically in both interfaces.
- Light/dark theme via the same token system as Expense Manager (`useTheme.tsx`, CSS-variable-based `tailwind.config.js`), matching the polish level the user referenced from Billing's own most recent pass (rounded-card tables, pill row actions, sidebar badge + accent bar).

## Round 7 (GUI amendment) definition of done

- Every CLI command in the locked V1 surface except `init`'s masked-confirm prompt and `secrets run` has a corresponding GUI screen or control (init is reachable via the Open Project gate's "Initialize a vault here").
- No screen introduces a Tauri command that doesn't call directly into an existing `application::*` use case or `infrastructure::*` port — confirmed by code review during implementation, not just by this doc.
