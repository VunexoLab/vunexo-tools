---
status: locked
round: 2
---

# Vunexo Vault — User Flows

Source of truth for *how* a user moves through the tool. Every flow below maps to a command in `.ai/product-vunexo-vault.md`'s locked command surface.

## 1. First-run / init

1. User runs `vunexo init` in a project directory.
2. If `.vunexo/` already exists, the command refuses and tells the user to remove it manually first (never silently overwrites an existing vault).
3. Otherwise: prompts for a new passphrase (masked input, confirmed by re-entry, same UX shape as `ssh-keygen`/`gpg --gen-key`).
4. Creates `.vunexo/config.toml` (unencrypted metadata: format version, environment list, active environment) seeded with one default environment, `development`.
5. Creates the encrypted, empty vault file for `development`.
6. Prints a short success message plus a reminder to add `.vunexo/` to version control review carefully (it's safe to commit — ciphertext-at-rest — but the user should still know it's there) and a suggestion to run `vunexo hooks install`.

## 2. Environment management

- `vunexo env list` — reads `.vunexo/config.toml` only (no passphrase needed), prints all environments and marks the active one.
- `vunexo env use <name>` — if `<name>` doesn't exist yet, creates it (a new empty encrypted vault file for that environment, using the same passphrase as the vault as a whole — see Round 3, environments are not separately keyed). Updates `active_environment` in `config.toml`. No passphrase prompt on its own (doesn't touch secret contents).

## 3. Managing secrets

- `vunexo secrets set KEY` — prompts for the value (masked, not echoed), prompts for the vault passphrase, decrypts the active environment's vault, inserts/overwrites `KEY`, re-encrypts, writes back. A bare `KEY=VALUE` form is also accepted for scripting, with the same shell-history caveat called out in `cli-ux.md`.
- `vunexo secrets get KEY` — prompts for the passphrase, decrypts, prints the value for `KEY` (or a clear "not found" message and nonzero exit).
- `vunexo secrets list` — prompts for the passphrase, decrypts, prints all keys for the active environment. **Prints key names only, never values** — listing values is what `get` is for, and a default that dumps every plaintext secret to the terminal (and likely a scrollback buffer/log) is an unnecessary blast-radius increase.
- `vunexo secrets remove KEY` — prompts for the passphrase, decrypts, removes `KEY` if present, re-encrypts, writes back.

## 4. Running a command with injected secrets

1. `vunexo secrets run -- <command...>`.
2. Prompts for the passphrase, decrypts the active environment's vault.
3. Spawns `<command...>` as a child process, with every decrypted key/value added to its environment (on top of the current process's own inherited environment — never replacing it).
4. Streams the child's stdout/stderr straight through; exits with the child's own exit code.
5. At no point is a `.env` file written, and the decrypted values exist only in this process's memory and the child's environment block — never on disk.

## 5. Scanning for likely secrets

- `vunexo secrets scan` — walks the current directory (respecting `.gitignore`, so it doesn't waste time on `node_modules`/build artifacts) and reports any file/line that matches a scanning rule (see `crypto-and-scanning-engine.md`). No passphrase needed — scanning never touches the vault.
- `vunexo secrets scan --staged` — same scan, restricted to the currently staged (`git diff --cached`) changes. This is what the pre-commit hook actually calls.
- Both print one line per finding (file, line, rule name, a redacted preview of the match) and exit nonzero if any finding exists, zero otherwise.

## 6. Git hook install / uninstall

- `vunexo hooks install` — must be run inside a git repository. Writes (or appends to, if a pre-commit hook already exists — see below) `.git/hooks/pre-commit`, which invokes `vunexo secrets scan --staged` and aborts the commit (non-zero exit) if it finds something.
  - If `.git/hooks/pre-commit` already exists and wasn't written by Vunexo Vault, the command refuses to overwrite it and tells the user to add the invocation manually (never silently clobbers another tool's hook).
- `vunexo hooks uninstall` — removes only the Vunexo-Vault-authored block from `.git/hooks/pre-commit` (identified by a marker comment), leaving anything else in that file untouched; deletes the file entirely if Vunexo Vault's block was the only content.

## 7. Error / recovery paths

- Wrong passphrase on any decrypt: a clear "incorrect passphrase" error, never a raw crypto-library error string, and never a hint about *which* part of the check failed (avoids leaking oracle information about the vault's contents).
- Vault file missing or corrupted: a clear error naming the environment and file path; no automatic recovery/repair is attempted in V1.
- `secrets get`/`list`/`run` against an environment with no vault file yet: clear "no vault for environment `<name>`" error, distinct from a wrong passphrase.

## Round 2 definition of done

- Every command in the locked command surface has an explicit, unambiguous flow above.
- Every flow states exactly when a passphrase prompt happens and when it doesn't.
- Failure modes (wrong passphrase, missing vault, pre-existing hook file) are each named with their intended behavior, not left implicit.
