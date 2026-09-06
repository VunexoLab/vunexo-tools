//! `ConfigStore` implementation: plain (unencrypted) TOML read/write of
//! `.vunexo/config.toml`.

use std::fs;
use std::path::PathBuf;

use crate::domain::config::{VaultConfig, CURRENT_FORMAT_VERSION};

use crate::application::ports::ConfigStore;
use crate::application::VaultError;

pub struct TomlConfigStore {
    config_path: PathBuf,
}

impl TomlConfigStore {
    /// `vunexo_dir` is the path to `.vunexo/`; the config file lives at
    /// `<vunexo_dir>/config.toml`.
    pub fn new(vunexo_dir: impl Into<PathBuf>) -> Self {
        Self {
            config_path: vunexo_dir.into().join("config.toml"),
        }
    }
}

impl ConfigStore for TomlConfigStore {
    fn load(&self) -> Result<VaultConfig, VaultError> {
        let raw = fs::read_to_string(&self.config_path).map_err(|e| {
            VaultError::Io(format!(
                "failed to read {}: {e}",
                self.config_path.display()
            ))
        })?;
        let config: VaultConfig = toml::from_str(&raw).map_err(|e| {
            VaultError::Malformed(format!("malformed {}: {e}", self.config_path.display()))
        })?;
        if config.format_version != CURRENT_FORMAT_VERSION {
            return Err(VaultError::Malformed(format!(
                "{} was written by a vault format version ({}) this build of vunexo ({}) does not support",
                self.config_path.display(),
                config.format_version,
                CURRENT_FORMAT_VERSION
            )));
        }
        Ok(config)
    }

    fn save(&self, config: &VaultConfig) -> Result<(), VaultError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized =
            toml::to_string_pretty(config).map_err(|e| VaultError::Malformed(e.to_string()))?;
        fs::write(&self.config_path, serialized)?;
        Ok(())
    }

    fn exists(&self) -> bool {
        self.config_path.is_file()
    }
}
