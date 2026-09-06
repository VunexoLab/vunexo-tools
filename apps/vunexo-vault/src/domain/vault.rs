//! Core vault domain types: [`SecretKey`], [`SecretValue`], [`Environment`], [`Vault`].
//!
//! Per `docs/vunexo-vault/crypto-and-scanning-engine.md` §3, decrypted secret
//! values are wrapped in a zeroize-on-drop container (`secrecy::SecretString`)
//! with redacted `Debug`/`Display` impls, so an accidental `{:?}`/`{}` never
//! prints a secret to the terminal, a log line, or a bug report. This is a
//! best-effort guarantee against *accidental* exposure, not a defense against
//! a privileged attacker reading process memory (see the crypto doc for the
//! precise, honest claim).

use std::collections::BTreeMap;
use std::fmt;

use secrecy::{ExposeSecret, SecretString};

/// The name of a secret (e.g. `API_KEY`). Deliberately permissive — any
/// non-empty string without embedded newlines is accepted (it never becomes
/// part of a file name, unlike [`Environment`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SecretKey(String);

/// Error constructing a [`SecretKey`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SecretKeyError {
    #[error("secret key must not be empty")]
    Empty,
    #[error("secret key must not contain newlines")]
    ContainsNewline,
}

impl SecretKey {
    pub fn new(raw: impl Into<String>) -> Result<Self, SecretKeyError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(SecretKeyError::Empty);
        }
        if raw.contains('\n') || raw.contains('\r') {
            return Err(SecretKeyError::ContainsNewline);
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A decrypted secret value. Wraps `secrecy::SecretString` (zeroize-on-drop)
/// and redacts both `Debug` and `Display` output, per
/// `crypto-and-scanning-engine.md` §3 and `application-architecture.md`'s
/// domain type description.
///
/// Deliberately has **no** `serde::Serialize`/`Deserialize` impl: the
/// plaintext TOML (de)serialization of a vault's contents is handled at the
/// infrastructure boundary (`infrastructure::age_vault_store`) using plain
/// `String`s that exist only transiently around the encrypt/decrypt calls, so
/// this domain type itself can never be accidentally serialized (e.g. logged,
/// or written out through an unrelated serde path).
#[derive(Clone)]
pub struct SecretValue(SecretString);

impl SecretValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecretString::from(value.into()))
    }

    /// Expose the underlying secret. Callers must not print, log, or persist
    /// the result outside of the single designed plaintext boundary
    /// (`secrets run`'s child-process environment) or the encrypt/decrypt
    /// path.
    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretValue([REDACTED])")
    }
}

impl fmt::Display for SecretValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

/// Name of an environment (e.g. `development`). Validated on construction:
/// non-empty, filesystem-safe characters only, since it becomes part of a
/// file name (`.vunexo/vault/<environment>.age`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Environment(String);

/// Error constructing an [`Environment`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EnvironmentError {
    #[error("environment name must not be empty")]
    Empty,
    #[error("environment name may only contain letters, digits, '-' and '_' (got `{0}`)")]
    InvalidCharacters(String),
}

impl Environment {
    pub fn new(raw: impl Into<String>) -> Result<Self, EnvironmentError> {
        let raw = raw.into();
        if raw.is_empty() {
            return Err(EnvironmentError::Empty);
        }
        if !raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(EnvironmentError::InvalidCharacters(raw));
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The decrypted contents of one environment's vault: a `BTreeMap` (not
/// `HashMap`) so `secrets list`'s output is deterministically ordered, per
/// `application-architecture.md`.
#[derive(Clone)]
pub struct Vault {
    environment: Environment,
    secrets: BTreeMap<SecretKey, SecretValue>,
}

impl Vault {
    pub fn empty(environment: Environment) -> Self {
        Self {
            environment,
            secrets: BTreeMap::new(),
        }
    }

    pub fn from_entries(
        environment: Environment,
        secrets: BTreeMap<SecretKey, SecretValue>,
    ) -> Self {
        Self {
            environment,
            secrets,
        }
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn get(&self, key: &SecretKey) -> Option<&SecretValue> {
        self.secrets.get(key)
    }

    pub fn set(&mut self, key: SecretKey, value: SecretValue) {
        self.secrets.insert(key, value);
    }

    pub fn remove(&mut self, key: &SecretKey) -> Option<SecretValue> {
        self.secrets.remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &SecretKey> {
        self.secrets.keys()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&SecretKey, &SecretValue)> {
        self.secrets.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_key_rejects_empty() {
        assert_eq!(SecretKey::new(""), Err(SecretKeyError::Empty));
    }

    #[test]
    fn secret_key_rejects_newline() {
        assert_eq!(
            SecretKey::new("API\nKEY"),
            Err(SecretKeyError::ContainsNewline)
        );
    }

    #[test]
    fn secret_key_accepts_normal_name() {
        assert!(SecretKey::new("API_KEY").is_ok());
    }

    #[test]
    fn environment_rejects_empty() {
        assert_eq!(Environment::new(""), Err(EnvironmentError::Empty));
    }

    #[test]
    fn environment_rejects_path_separators() {
        assert!(matches!(
            Environment::new("../etc"),
            Err(EnvironmentError::InvalidCharacters(_))
        ));
        assert!(matches!(
            Environment::new("dev/prod"),
            Err(EnvironmentError::InvalidCharacters(_))
        ));
    }

    #[test]
    fn environment_accepts_alnum_dash_underscore() {
        assert!(Environment::new("development").is_ok());
        assert!(Environment::new("staging-2").is_ok());
        assert!(Environment::new("prod_eu").is_ok());
    }

    #[test]
    fn secret_value_debug_and_display_are_redacted() {
        let v = SecretValue::new("super-secret-value");
        assert_eq!(format!("{v:?}"), "SecretValue([REDACTED])");
        assert_eq!(format!("{v}"), "[REDACTED]");
        assert_eq!(v.expose(), "super-secret-value");
    }

    #[test]
    fn vault_set_get_remove_round_trip() {
        let env = Environment::new("development").unwrap();
        let mut vault = Vault::empty(env);
        let key = SecretKey::new("API_KEY").unwrap();
        vault.set(key.clone(), SecretValue::new("abc123"));
        assert_eq!(
            vault.get(&key).map(|v| v.expose().to_string()),
            Some("abc123".to_string())
        );
        let removed = vault.remove(&key);
        assert!(removed.is_some());
        assert!(vault.get(&key).is_none());
    }

    #[test]
    fn vault_keys_are_sorted() {
        let env = Environment::new("development").unwrap();
        let mut vault = Vault::empty(env);
        vault.set(SecretKey::new("Z_KEY").unwrap(), SecretValue::new("1"));
        vault.set(SecretKey::new("A_KEY").unwrap(), SecretValue::new("2"));
        let keys: Vec<_> = vault.keys().map(|k| k.as_str().to_string()).collect();
        assert_eq!(keys, vec!["A_KEY".to_string(), "Z_KEY".to_string()]);
    }
}
