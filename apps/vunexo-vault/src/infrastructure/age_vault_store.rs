//! `VaultStore` implementation backed by the `age` crate's native
//! passphrase (scrypt) recipient/identity mechanism.
//!
//! Per `docs/vunexo-vault/crypto-and-scanning-engine.md` §1, this is the
//! **only** crypto surface in the whole project: `age::scrypt::Recipient`
//! for encryption and `age::scrypt::Identity` for decryption, both of which
//! are `age`'s own, independently-specified and audited passphrase
//! mechanism (scrypt KDF, salt, and work factor chosen and embedded by
//! `age` itself). There is no Vunexo-authored KDF, cipher, salt, or nonce
//! handling anywhere in this file — deliberately: `set_work_factor`/
//! `set_max_work_factor` are never called, since choosing KDF parameters
//! ourselves would be exactly the kind of decision the locked design
//! reserves to `age`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use secrecy::SecretString;
use zeroize::Zeroize;

use crate::domain::vault::{Environment, SecretKey, SecretValue, Vault};

use crate::application::ports::VaultStore;
use crate::application::VaultError;

pub struct AgeVaultStore {
    vault_dir: PathBuf,
}

impl AgeVaultStore {
    /// `vunexo_dir` is the path to `.vunexo/`; vault files live under
    /// `<vunexo_dir>/vault/<environment>.age`.
    pub fn new(vunexo_dir: impl Into<PathBuf>) -> Self {
        Self {
            vault_dir: vunexo_dir.into().join("vault"),
        }
    }

    fn path_for(&self, environment: &Environment) -> PathBuf {
        self.vault_dir.join(format!("{}.age", environment.as_str()))
    }
}

impl VaultStore for AgeVaultStore {
    fn load(
        &self,
        environment: &Environment,
        passphrase: &SecretString,
    ) -> Result<Vault, VaultError> {
        let path = self.path_for(environment);
        let ciphertext = fs::read(&path).map_err(|e| {
            VaultError::Io(format!(
                "failed to read vault for environment `{}` at {}: {e}",
                environment.as_str(),
                path.display()
            ))
        })?;

        // `age`'s own native passphrase-based identity: reads the scrypt
        // salt/work-factor embedded in the file itself. No Vunexo-chosen KDF
        // parameters.
        let identity = age::scrypt::Identity::new(passphrase.clone());

        let mut plaintext_bytes =
            age::decrypt(&identity, &ciphertext).map_err(|_| VaultError::WrongPassphrase)?;

        // The plaintext payload is UTF-8 TOML; anything else means either a
        // wrong passphrase produced garbage or the file was corrupted after
        // encryption. Per `cli-ux.md`, both look identical from the outside.
        let plaintext =
            String::from_utf8(plaintext_bytes.clone()).map_err(|_| VaultError::WrongPassphrase);
        plaintext_bytes.zeroize();
        let mut plaintext = plaintext?;

        let map: BTreeMap<String, String> =
            toml::from_str(&plaintext).map_err(|_| VaultError::WrongPassphrase)?;
        plaintext.zeroize();

        let mut secrets = BTreeMap::new();
        for (raw_key, raw_value) in map {
            let key = SecretKey::new(raw_key).map_err(|e| {
                VaultError::Malformed(format!("vault contains an invalid secret key: {e}"))
            })?;
            secrets.insert(key, SecretValue::new(raw_value));
        }

        Ok(Vault::from_entries(environment.clone(), secrets))
    }

    fn save(&self, vault: &Vault, passphrase: &SecretString) -> Result<(), VaultError> {
        let mut map = BTreeMap::new();
        for (key, value) in vault.iter() {
            map.insert(key.as_str().to_string(), value.expose().to_string());
        }
        let mut plaintext =
            toml::to_string(&map).map_err(|e| VaultError::Malformed(e.to_string()))?;

        // `age`'s own native passphrase-based recipient: the resulting file
        // embeds a fresh, random scrypt salt and the work factor `age`
        // itself picked. No Vunexo-chosen KDF parameters.
        let recipient = age::scrypt::Recipient::new(passphrase.clone());
        let ciphertext = age::encrypt(&recipient, plaintext.as_bytes())
            .map_err(|_| VaultError::EncryptionFailed)?;
        plaintext.zeroize();

        fs::create_dir_all(&self.vault_dir)?;
        let path = self.path_for(vault.environment());
        fs::write(&path, ciphertext)?;
        Ok(())
    }

    fn exists(&self, environment: &Environment) -> bool {
        self.path_for(environment).is_file()
    }
}
