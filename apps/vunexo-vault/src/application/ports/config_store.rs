//! `ConfigStore` port: read/write the unencrypted `.vunexo/config.toml`.
//!
//! Beyond the two methods named in `application-architecture.md`
//! (`load`/`save`), this also exposes `exists()` — needed by `init` to
//! implement its locked "refuse if `.vunexo/` already exists" behavior
//! (`user-flows.md` §1) without prompting for a passphrase or attempting a
//! `load()` that would itself error on a missing file. This mirrors
//! `VaultStore::exists`, which is already part of the locked design.

use crate::domain::config::VaultConfig;

use super::super::VaultError;

pub trait ConfigStore {
    fn load(&self) -> Result<VaultConfig, VaultError>;
    fn save(&self, config: &VaultConfig) -> Result<(), VaultError>;
    /// Whether `.vunexo/config.toml` already exists.
    fn exists(&self) -> bool;
}
