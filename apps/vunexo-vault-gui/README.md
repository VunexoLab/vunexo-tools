# Vunexo Vault GUI

The optional desktop companion to the [`vunexo`](../vunexo-vault/) CLI (Vunexo Vault) — a visual way to browse and manage the exact same encrypted local vault, for people who'd rather click than type.

**The CLI remains the primary interface.** This app is a second, thin interface to the identical shared crypto/domain code (`apps/vunexo-vault`'s `[lib]` target) — it introduces no new secret-handling logic, no new vault format, and no new capability the CLI doesn't already have. Automation and git hooks call the CLI, not this app. See [`.ai/product-vunexo-vault.md`](../../.ai/product-vunexo-vault.md)'s 2026-09-06 amendment for the exact terms of this exception to the original CLI-only design.

## Status

Built and verified this session: `cargo build/test/fmt/clippy` clean on the Rust side, `pnpm typecheck/lint/build` clean on the frontend, and manually clicked through end to end (Open Project → Unlock → Secrets list/reveal → Scan → Hooks) against a real vault created by the CLI, confirming both interfaces read and write the identical encrypted data. Not yet released — no installer has been built or published.

## Features

- **Open Project** — pick a folder (with a recent-projects list) to work with its `.vunexo/` vault.
- **Unlock** — choose an environment and enter the passphrase; held in memory for the session only, never written to disk.
- **Secrets** — list keys (masked by default, reveal/copy per row), add/edit/remove.
- **Environments** — list/create/switch.
- **Scan** — run the same pattern/entropy leak scanner as the CLI, optionally staged-only.
- **Hooks** — install/uninstall the git pre-commit hook.
- **Settings** — vault format version, project path, license, and a Lock action.

There is deliberately no "run a command with secrets injected" feature here — that stays CLI-only (`vunexo secrets run`), so there's exactly one place secrets ever reach a child process's environment.

## Running from source

Requires [Rust](https://rustup.rs/) (stable) and [pnpm](https://pnpm.io/).

```bash
# from the repo root
pnpm install

# from apps/vunexo-vault-gui/
pnpm tauri dev
```

### Building

```bash
# from apps/vunexo-vault-gui/
pnpm tauri build
```

### Verification

```bash
# Backend, from apps/vunexo-vault-gui/src-tauri/
cargo build && cargo test --quiet && cargo fmt --check && cargo clippy --all-targets --quiet

# Frontend, from apps/vunexo-vault-gui/
pnpm typecheck && pnpm lint && pnpm build
```

CI (`.github/workflows/ci.yml`) runs the same checks on every push/PR, across Ubuntu, Windows, and macOS.

## Architecture

Rust/Tauri backend (`src-tauri/`) with a path dependency on the `vunexo-vault` crate — every Tauri command is a thin wrapper around that crate's existing `application::*` use cases and `infrastructure::*` adapters (`AgeVaultStore`, `TomlConfigStore`, `PatternScanner`, `HookInstaller`). React/TypeScript/Tailwind frontend (`src/`), same token-based design system as Expense Manager. See [`docs/vunexo-vault/gui-ux.md`](../../docs/vunexo-vault/gui-ux.md) for the locked screen inventory.

## License

MIT — see [`LICENSE`](../../LICENSE) and [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).
