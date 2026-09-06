//! `InitVault` use case — `vunexo init` (`user-flows.md` §1).

use secrecy::SecretString;

use crate::domain::config::{now_rfc3339, VaultConfig};
use crate::domain::vault::{Environment, Vault};

use super::ports::{ConfigStore, VaultStore};
use super::VaultError;

/// The one default environment every fresh vault is seeded with.
pub const DEFAULT_ENVIRONMENT: &str = "development";

/// Initialize a new vault: refuses if one already exists, otherwise creates
/// `config.toml` (seeded with the `development` environment) and an empty
/// encrypted vault file for it.
pub fn init_vault(
    config_store: &dyn ConfigStore,
    vault_store: &dyn VaultStore,
    passphrase: &SecretString,
) -> Result<(), VaultError> {
    if config_store.exists() {
        return Err(VaultError::AlreadyInitialized);
    }

    let default_environment = Environment::new(DEFAULT_ENVIRONMENT)
        .expect("DEFAULT_ENVIRONMENT is a valid environment name");
    let config = VaultConfig::new_with_default_environment(&default_environment, &now_rfc3339());
    config_store.save(&config)?;

    let empty_vault = Vault::empty(default_environment);
    vault_store.save(&empty_vault, passphrase)?;

    Ok(())
}
