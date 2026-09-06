---
status: locked
round: 5
---

# Vunexo Vault — CLI/Terminal UX

The CLI's equivalent of a screen inventory: exact command surface, flags, exit codes, output/error conventions.

## Global conventions

- Human-readable text output by default. No `--json` in V1 (not in the locked command surface; add later only via an explicit new round, not silently).
- Passphrase is **never accepted as a plain CLI argument** (would leak into shell history and `ps`/process-list output). Two ways to supply it:
  - Interactive masked prompt (default) — used for `init`, `secrets set/get/list/remove`, `secrets run`.
  - `--passphrase-file <path>` — reads the passphrase from a file the user manages (e.g. for a local script). The file's own permissions/protection are the user's responsibility; Vunexo Vault does not manage or check them beyond reading it.
- Colors/formatting degrade gracefully on non-TTY output (e.g. piped to a file) — no ANSI codes when `stdout` isn't a terminal.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success |
| `1` | General error (I/O failure, malformed vault, git errors, etc.) |
| `2` | Wrong passphrase |
| `3` | Secret/environment not found (`get`/`remove`/`use` on something that doesn't exist) |
| `4` | **Scan finding present** (`secrets scan`/`scan --staged`) — the specific code the pre-commit hook checks to decide whether to block a commit |
| Child's own code | `secrets run` exits with whatever the spawned command exited with, not a Vunexo-specific code |

## Command reference

### `vunexo init`

No flags in V1. Refuses (exit `1`) if `.vunexo/` already exists.

### `vunexo secrets set KEY[=VALUE]`

- `vunexo secrets set KEY` — prompts for the value (masked).
- `vunexo secrets set KEY=VALUE` — takes the value inline (accepted for scripting; still never logged or echoed back).
- Always prompts for the vault passphrase (unless `--passphrase-file` given).

### `vunexo secrets get KEY`

Prints the value to stdout with no extra decoration (so it's pipeable, e.g. `vunexo secrets get API_KEY | pbcopy`). Exit `3` if `KEY` doesn't exist in the active environment.

### `vunexo secrets list`

Prints one key name per line, sorted. Never prints values (see `user-flows.md` §3).

### `vunexo secrets remove KEY`

Exit `3` if `KEY` doesn't exist. Idempotent otherwise — removing an already-absent key is a `3`, not silently a `0`, so scripts notice a typo'd key name.

### `vunexo env list`

No passphrase prompt. Prints each environment name, marking the active one (e.g. a leading `*`).

### `vunexo env use NAME`

Creates `NAME` if it doesn't exist yet (see `user-flows.md` §2). No passphrase prompt.

### `vunexo secrets run -- COMMAND [ARGS...]`

Everything after `--` is passed through untouched to the child process. Prompts for the passphrase once, then runs. Exit code is the child's.

### `vunexo secrets scan [--staged] [PATH]`

- No args: scans the current directory, respecting `.gitignore`.
- `--staged`: scans only `git diff --cached` paths.
- Optional `PATH`: scan a specific file or directory instead of the current directory (mutually exclusive with `--staged`).
- Output: one line per finding — `<path>:<line>: [<rule-name>] <redacted-preview>`. Exit `4` if any finding, `0` otherwise.

### `vunexo hooks install` / `vunexo hooks uninstall`

No flags. See `user-flows.md` §6 for the "don't clobber an existing hook" behavior; violating it is a `1`, not a silent overwrite.

## Error message conventions

- Every error prints to stderr, prefixed `error:`, one line, plain language — no raw library error text (e.g. no raw `age` decryption error strings, which can vary across `age` versions and might hint at internal details).
- `WrongPassphrase` is always exactly the same message text regardless of *why* decryption failed internally (bad passphrase vs. corrupted file both currently look the same from the outside) — see `crypto-and-scanning-engine.md` for why this isn't loosened later without a security review.

## Round 5 definition of done

- Every command in the locked surface has its exact flags and exit codes pinned above.
- A distinct, stable exit code (`4`) exists specifically for "scan found something," since the git hook's entire job depends on that contract.
- The passphrase-input story (masked prompt vs. `--passphrase-file`) is fully specified and avoids shell-history/process-list exposure.
