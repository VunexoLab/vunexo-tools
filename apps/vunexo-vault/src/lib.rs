//! Shared library target: the vault/crypto/scanning domain, application use
//! cases, and infrastructure adapters — the single implementation of
//! everything `docs/vunexo-vault/crypto-and-scanning-engine.md` locks.
//!
//! Both the `vunexo` CLI binary (`src/main.rs`) and the optional desktop
//! companion GUI (`apps/vunexo-vault-gui/src-tauri`, via a path dependency on
//! this crate) depend on these modules rather than each having their own
//! copy — there is exactly one implementation of the `age`-based
//! encrypt/decrypt path and the secret scanner, never two that could drift
//! apart. CLI-only concerns (arg parsing, passphrase-prompt reading) live in
//! `src/cli/` and `src/main.rs`, not here.

pub mod application;
pub mod domain;
pub mod infrastructure;
