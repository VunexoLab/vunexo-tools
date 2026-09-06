//! `RunWithSecrets` use case — `vunexo secrets run -- <command>`
//! (`user-flows.md` §4).

use std::process::ExitStatus;

use secrecy::SecretString;

use crate::domain::vault::Environment;

use super::ports::{ProcessRunner, VaultStore};
use super::VaultError;

/// Decrypt the active environment's vault and spawn `command` with every
/// decrypted secret injected into its environment (on top of, never
/// replacing, the current process's own inherited environment).
pub fn run_with_secrets(
    vault_store: &dyn VaultStore,
    process_runner: &dyn ProcessRunner,
    environment: &Environment,
    passphrase: &SecretString,
    command: &str,
    args: &[String],
) -> Result<ExitStatus, VaultError> {
    if !vault_store.exists(environment) {
        return Err(VaultError::VaultNotFound(environment.as_str().to_string()));
    }
    let vault = vault_store.load(environment, passphrase)?;
    process_runner.run(command, args, &vault)
}
