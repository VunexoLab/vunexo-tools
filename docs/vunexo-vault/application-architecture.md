---
status: locked
round: 4
---

# Vunexo Vault — Application Architecture

Same `domain` / `application` / `infrastructure` layering as the sibling apps, adapted for a CLI with no IPC boundary (there's no frontend process to cross into — the CLI parser calls straight into `application`).

## Module layout

```
apps/vunexo-vault/src/
├── main.rs                  # entry point: parse args, dispatch, map errors to exit codes
├── cli/                     # clap command/arg definitions, one module per top-level command
│   ├── init.rs
│   ├── secrets.rs
│   ├── env.rs
│   └── hooks.rs
├── domain/
│   ├── vault.rs              # Vault, Environment, SecretKey/SecretValue newtypes
│   ├── config.rs             # VaultConfig (mirrors config.toml's shape)
│   └── scan.rs                # ScanFinding, ScanRule
├── application/
│   ├── init_vault.rs
│   ├── secrets.rs             # SetSecret/GetSecret/ListSecrets/RemoveSecret use cases
│   ├── environments.rs        # ListEnvironments/UseEnvironment use cases
│   ├── run_with_secrets.rs
│   ├── scan.rs                 # ScanPath/ScanStaged use cases
│   └── ports/                  # trait definitions only, no impl
│       ├── vault_store.rs      # VaultStore port
│       ├── config_store.rs     # ConfigStore port
│       ├── process_runner.rs   # ProcessRunner port
│       ├── git.rs              # GitDiff (staged files) + HookInstaller ports
│       └── scanner.rs          # Scanner port
└── infrastructure/
    ├── age_vault_store.rs      # VaultStore impl using the `age` crate
    ├── toml_config_store.rs    # ConfigStore impl (plain TOML read/write)
    ├── std_process_runner.rs   # ProcessRunner impl (std::process::Command)
    ├── git_cli.rs               # GitDiff impl (shells out to `git diff --cached --name-only`)
    ├── hook_installer.rs        # HookInstaller impl (writes .git/hooks/pre-commit)
    └── pattern_scanner.rs       # Scanner impl (regex + entropy rules from crypto-and-scanning-engine.md)
```

## Domain types

- `SecretKey(String)`, `SecretValue` — `SecretValue` wraps its inner `String` in a zeroize-on-drop container (the `secrecy` crate; see `crypto-and-scanning-engine.md` for why) and its `Debug`/`Display` impls redact the value, so an accidental `{:?}`/log line never leaks a secret to the terminal or a bug report.
- `Environment(String)` — validated on construction (non-empty, filesystem-safe characters only — it becomes part of a file name).
- `Vault` — an in-memory `BTreeMap<SecretKey, SecretValue>` for one environment, plus the environment name it belongs to. `BTreeMap` (not `HashMap`) so `secrets list`'s output is deterministically ordered.
- `VaultConfig` — mirrors `config.toml`: `format_version`, `active_environment`, the environment list with creation timestamps.
- `ScanFinding` — file path, line number, rule name, a redacted preview (first/last few characters of the match, rest masked) — never the full matched secret, so a scan's own output can't itself become a leak vector.

## Ports (traits, `application::ports`)

- `VaultStore`: `load(environment, passphrase) -> Result<Vault>`, `save(vault, passphrase) -> Result<()>`, `exists(environment) -> bool`. Passphrase is a parameter, not stored on the trait — no port implementation is allowed to cache it beyond the call.
- `ConfigStore`: `load() -> Result<VaultConfig>`, `save(&VaultConfig) -> Result<()>`.
- `ProcessRunner`: `run(command, args, extra_env) -> Result<ExitStatus>` — spawns, inherits the parent's environment plus `extra_env`, streams stdio through, returns the child's exit status.
- `GitDiff`: `staged_files() -> Result<Vec<PathBuf>>`.
- `HookInstaller`: `install() -> Result<()>`, `uninstall() -> Result<()>` — both operate only on a marker-delimited block inside `.git/hooks/pre-commit`, per the locked flow in `user-flows.md` §6.
- `Scanner`: `scan(paths: &[PathBuf]) -> Vec<ScanFinding>`.

## Use cases (`application/`)

Each use case is a plain function/struct taking its needed ports by reference — no shared "god object" holding every port, so a use case's dependencies are visible in its own signature (same discipline as the sibling apps' `application` layer). `InitVault`, `SetSecret`/`GetSecret`/`ListSecrets`/`RemoveSecret`, `ListEnvironments`/`UseEnvironment`, `RunWithSecrets`, `ScanPath`/`ScanStaged`, `InstallHook`/`UninstallHook` — one use case per command in the locked surface, no more.

## CLI command surface (`cli/`, via `clap`)

`clap`'s derive API maps directly onto the locked command tree from `product-vunexo-vault.md` (`vunexo init`, `vunexo secrets <set|get|list|remove>`, `vunexo env <list|use>`, `vunexo secrets run -- <command>`, `vunexo secrets scan [--staged]`, `vunexo hooks <install|uninstall>`). `main.rs` parses into a typed command enum, constructs the concrete infrastructure adapters, and dispatches into the matching use case. Exact flags/exit codes are pinned in `cli-ux.md`.

## Error handling

One `VaultError` enum in `application`, covering: `WrongPassphrase`, `VaultNotFound(Environment)`, `EnvironmentAlreadyExists`, `SecretNotFound(SecretKey)`, `NotAGitRepository`, `HookAlreadyExists`, `Io(std::io::Error)`, `Crypto(age error, wrapped, never surfaced raw)`. `main.rs` maps each variant to a specific exit code (`cli-ux.md`) and a human-readable message — the `WrongPassphrase` message is deliberately generic (see `user-flows.md` §7's oracle-avoidance note).

## Verification guide

- Unit tests: domain newtypes' validation (`Environment`, `SecretKey`), `VaultConfig` (de)serialization round-trip.
- Integration tests: each use case against a real temp-directory `.vunexo/` (real `age` encryption, no mocked crypto) — e.g. `init` then `set` then `get` returns the same value; wrong passphrase on `get` after a correct `set` fails cleanly; `run` actually injects env vars into a real spawned process (assert via a trivial child command like `printenv`); `scan` against fixture files matches the exact test vectors from `crypto-and-scanning-engine.md`; `hooks install` followed by `hooks uninstall` leaves `.git/hooks/pre-commit` exactly as it was before install (byte-for-byte, when there was a pre-existing unrelated hook).

## Round 4 definition of done

- Every use case in the locked command surface has a named module and a named port it depends on.
- No port leaks passphrase/secret material beyond the single call that needs it.
- The verification guide names a concrete test for every locked flow from `user-flows.md`.
