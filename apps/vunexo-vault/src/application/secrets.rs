//! `SetSecret`/`GetSecret`/`ListSecrets`/`RemoveSecret` use cases
//! (`user-flows.md` §3).

use secrecy::SecretString;

use crate::domain::vault::{Environment, SecretKey, SecretValue, Vault};

use super::ports::VaultStore;
use super::VaultError;

fn load_or_empty(
    vault_store: &dyn VaultStore,
    environment: &Environment,
    passphrase: &SecretString,
) -> Result<Vault, VaultError> {
    if vault_store.exists(environment) {
        vault_store.load(environment, passphrase)
    } else {
        Ok(Vault::empty(environment.clone()))
    }
}

fn require_existing_vault(
    vault_store: &dyn VaultStore,
    environment: &Environment,
    passphrase: &SecretString,
) -> Result<Vault, VaultError> {
    if !vault_store.exists(environment) {
        return Err(VaultError::VaultNotFound(environment.as_str().to_string()));
    }
    vault_store.load(environment, passphrase)
}

/// `vunexo secrets set KEY[=VALUE]` — insert or overwrite `key`.
pub fn set_secret(
    vault_store: &dyn VaultStore,
    environment: &Environment,
    passphrase: &SecretString,
    key: SecretKey,
    value: SecretValue,
) -> Result<(), VaultError> {
    let mut vault = load_or_empty(vault_store, environment, passphrase)?;
    vault.set(key, value);
    vault_store.save(&vault, passphrase)
}

/// `vunexo secrets get KEY`.
pub fn get_secret(
    vault_store: &dyn VaultStore,
    environment: &Environment,
    passphrase: &SecretString,
    key: &SecretKey,
) -> Result<SecretValue, VaultError> {
    let vault = require_existing_vault(vault_store, environment, passphrase)?;
    vault
        .get(key)
        .cloned()
        .ok_or_else(|| VaultError::SecretNotFound(key.as_str().to_string()))
}

/// `vunexo secrets list` — key names only, sorted, never values (see
/// `user-flows.md` §3).
pub fn list_secrets(
    vault_store: &dyn VaultStore,
    environment: &Environment,
    passphrase: &SecretString,
) -> Result<Vec<SecretKey>, VaultError> {
    let vault = require_existing_vault(vault_store, environment, passphrase)?;
    Ok(vault.keys().cloned().collect())
}

/// `vunexo secrets remove KEY`. Exit-code-3-worthy if `key` doesn't exist
/// (`cli-ux.md`): idempotent removal is deliberately *not* treated as
/// success, so a typo'd key name is noticed.
pub fn remove_secret(
    vault_store: &dyn VaultStore,
    environment: &Environment,
    passphrase: &SecretString,
    key: &SecretKey,
) -> Result<(), VaultError> {
    let mut vault = require_existing_vault(vault_store, environment, passphrase)?;
    if vault.remove(key).is_none() {
        return Err(VaultError::SecretNotFound(key.as_str().to_string()));
    }
    vault_store.save(&vault, passphrase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// An in-memory `VaultStore` test double — no real crypto, used only to
    /// exercise the use cases' control flow in isolation. Real `age`
    /// encryption is covered by the integration tests in `tests/`.
    #[derive(Default)]
    struct FakeVaultStore {
        files: RefCell<HashMap<String, Vault>>,
    }

    impl VaultStore for FakeVaultStore {
        fn load(
            &self,
            environment: &Environment,
            _passphrase: &SecretString,
        ) -> Result<Vault, VaultError> {
            self.files
                .borrow()
                .get(environment.as_str())
                .cloned()
                .ok_or_else(|| VaultError::VaultNotFound(environment.as_str().to_string()))
        }

        fn save(&self, vault: &Vault, _passphrase: &SecretString) -> Result<(), VaultError> {
            self.files
                .borrow_mut()
                .insert(vault.environment().as_str().to_string(), vault.clone());
            Ok(())
        }

        fn exists(&self, environment: &Environment) -> bool {
            self.files.borrow().contains_key(environment.as_str())
        }
    }

    fn env() -> Environment {
        Environment::new("development").unwrap()
    }

    fn passphrase() -> SecretString {
        SecretString::from("correct horse battery staple".to_string())
    }

    #[test]
    fn set_then_get_round_trips() {
        let store = FakeVaultStore::default();
        let key = SecretKey::new("API_KEY").unwrap();
        set_secret(
            &store,
            &env(),
            &passphrase(),
            key.clone(),
            SecretValue::new("s3cr3t"),
        )
        .unwrap();
        let value = get_secret(&store, &env(), &passphrase(), &key).unwrap();
        assert_eq!(value.expose(), "s3cr3t");
    }

    #[test]
    fn get_missing_key_is_not_found() {
        let store = FakeVaultStore::default();
        set_secret(
            &store,
            &env(),
            &passphrase(),
            SecretKey::new("OTHER").unwrap(),
            SecretValue::new("x"),
        )
        .unwrap();
        let err = get_secret(
            &store,
            &env(),
            &passphrase(),
            &SecretKey::new("MISSING").unwrap(),
        )
        .unwrap_err();
        assert!(matches!(err, VaultError::SecretNotFound(k) if k == "MISSING"));
    }

    #[test]
    fn get_against_nonexistent_vault_is_vault_not_found() {
        let store = FakeVaultStore::default();
        let err = get_secret(
            &store,
            &env(),
            &passphrase(),
            &SecretKey::new("KEY").unwrap(),
        )
        .unwrap_err();
        assert!(matches!(err, VaultError::VaultNotFound(e) if e == "development"));
    }

    #[test]
    fn list_returns_sorted_keys_only() {
        let store = FakeVaultStore::default();
        set_secret(
            &store,
            &env(),
            &passphrase(),
            SecretKey::new("Z").unwrap(),
            SecretValue::new("1"),
        )
        .unwrap();
        set_secret(
            &store,
            &env(),
            &passphrase(),
            SecretKey::new("A").unwrap(),
            SecretValue::new("2"),
        )
        .unwrap();
        let keys = list_secrets(&store, &env(), &passphrase()).unwrap();
        assert_eq!(
            keys.iter()
                .map(|k| k.as_str().to_string())
                .collect::<Vec<_>>(),
            vec!["A".to_string(), "Z".to_string()]
        );
    }

    #[test]
    fn remove_missing_key_is_not_found_not_silently_ok() {
        let store = FakeVaultStore::default();
        set_secret(
            &store,
            &env(),
            &passphrase(),
            SecretKey::new("KEEP").unwrap(),
            SecretValue::new("v"),
        )
        .unwrap();
        let err = remove_secret(
            &store,
            &env(),
            &passphrase(),
            &SecretKey::new("NOPE").unwrap(),
        )
        .unwrap_err();
        assert!(matches!(err, VaultError::SecretNotFound(k) if k == "NOPE"));
    }

    #[test]
    fn remove_existing_key_succeeds_and_persists() {
        let store = FakeVaultStore::default();
        let key = SecretKey::new("KEY").unwrap();
        set_secret(
            &store,
            &env(),
            &passphrase(),
            key.clone(),
            SecretValue::new("v"),
        )
        .unwrap();
        remove_secret(&store, &env(), &passphrase(), &key).unwrap();
        let err = get_secret(&store, &env(), &passphrase(), &key).unwrap_err();
        assert!(matches!(err, VaultError::SecretNotFound(_)));
    }
}
