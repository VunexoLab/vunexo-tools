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

// Re-exported so a downstream consumer (the GUI, `apps/vunexo-vault-gui`) can
// name `secrecy::SecretString` — the type `application::ports::VaultStore`
// and friends already use in their public signatures — without adding
// `secrecy` as a second, independent direct dependency. This is a pure
// visibility change (one `pub use`), not a new crypto surface: it does not
// add, wrap, or alter any cryptographic behavior, it only lets the one
// existing type be named from outside this crate.
pub use secrecy;
