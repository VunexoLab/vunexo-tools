# Vunexo Vault

The simplest open-source secret manager for local development.

No account, no subscription, no cloud dependency, no team/server features. Everything stays on your machine, encrypted at rest. Store secrets per environment, run your app with them injected as real process environment variables (never a plaintext `.env` file), and catch likely secrets before they ever reach a commit.

See [`.ai/product-vunexo-vault.md`](../../.ai/product-vunexo-vault.md) for the locked V1 scope — including what's deliberately out (no CI/CD integrations, no team/shared vaults, no cloud sync; a local developer tool, not a hosted platform).

## Status

V1 implemented and tested (54 backend tests, `cargo fmt`/`clippy` clean) and manually smoke-tested end to end. Not yet released — no installer has been built or published. See [`.ai/progress/CURRENT.md`](../../.ai/progress/CURRENT.md) for the exact as-built state.

An optional desktop companion, [Vunexo Vault GUI](../vunexo-vault-gui/), also exists — a second interface to this exact same crate for people who'd rather click than type. This CLI remains the primary interface either way.

## Features (V1 scope)

- `vunexo init` — create a local encrypted vault.
- `vunexo secrets set/get/list/remove` — manage secrets, scoped to a named environment.
- `vunexo env list/use` — create and switch between named environments (`development`, `staging`, `production`, ...).
- `vunexo secrets run -- <command>` — run a real command with the active environment's secrets injected into its process environment only, no `.env` file ever written.
- `vunexo secrets scan [--staged]` — scan a directory (or just the staged git diff) for likely secrets, using pattern + entropy heuristics.
- `vunexo hooks install/uninstall` — a git pre-commit hook that blocks a commit if the scanner finds something.
- Cross-platform: Windows, macOS, Linux.

## Running from source

Requires [Rust](https://rustup.rs/) (stable).

```bash
# from apps/vunexo-vault/
cargo run -- init
```

### Building

```bash
# from apps/vunexo-vault/
cargo build --release
```

Produces a single native binary at `target/release/vunexo` (`vunexo.exe` on Windows).

### Verification

```bash
# from apps/vunexo-vault/
cargo build && cargo test --quiet && cargo fmt --check && cargo clippy --all-targets --quiet
```

CI (`.github/workflows/ci.yml`) runs the same checks on every push/PR, across Ubuntu, Windows, and macOS.

## Architecture

Plain Rust binary crate — the first of its kind in this monorepo, layered `cli → application → domain → infrastructure`. Encryption via the [`age`](https://github.com/str4d/rage) crate's own passphrase mechanism — no custom KDF or cipher code. `domain`/`application`/`infrastructure` are also exposed as a `[lib]` target so [Vunexo Vault GUI](../vunexo-vault-gui/) can depend on the identical crypto/domain code rather than a second implementation. See [`docs/vunexo-vault/`](../../docs/vunexo-vault/) for the full locked design, especially [`crypto-and-scanning-engine.md`](../../docs/vunexo-vault/crypto-and-scanning-engine.md), the security-design gate that had to be locked before implementation began.

## License

MIT — see [`LICENSE`](../../LICENSE) and [`THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).
