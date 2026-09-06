//! `VaultStore` port: load/save one environment's encrypted vault.
//!
//! Passphrase is a parameter to each call, never stored on the
//! implementation — no `VaultStore` impl is allowed to cache it beyond the
//! single call that needs it (`application-architecture.md`).

use secrecy::SecretString;

use crate::domain::vault::{Environment, Vault};

use super::super::VaultError;

pub trait VaultStore {
    /// Decrypt and load the vault for `environment` using `passphrase`.
    fn load(
        &self,
        environment: &Environment,
        passphrase: &SecretString,
    ) -> Result<Vault, VaultError>;

    /// Encrypt and persist `vault` using `passphrase`, overwriting any
    /// existing file for its environment.
    fn save(&self, vault: &Vault, passphrase: &SecretString) -> Result<(), VaultError>;

    /// Whether a vault file already exists for `environment` (no passphrase
    /// needed — this is purely a filesystem check).
    fn exists(&self, environment: &Environment) -> bool;
}
