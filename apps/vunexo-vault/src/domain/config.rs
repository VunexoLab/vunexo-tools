//! `VaultConfig` mirrors the unencrypted `.vunexo/config.toml` metadata file
//! described in `docs/vunexo-vault/storage-schema.md`: format version, the
//! environment list with creation timestamps, and which environment is
//! active. **No secret material is ever stored here.**

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::vault::Environment;

/// The only vault format version V1 understands. A mismatch between this and
/// a config file's `format_version` is a "run a newer/older vunexo" error,
/// never a silent best-effort read (per `storage-schema.md`'s migration
/// strategy).
pub const CURRENT_FORMAT_VERSION: u32 = 1;

/// Per-environment metadata stored in `config.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentMeta {
    pub created_at: String,
}

/// The unencrypted `.vunexo/config.toml` document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultConfig {
    pub format_version: u32,
    pub active_environment: String,
    pub environments: BTreeMap<String, EnvironmentMeta>,
}

impl VaultConfig {
    /// Build a freshly-initialized config seeded with one default
    /// environment, `development`, per `user-flows.md` §1.
    pub fn new_with_default_environment(default_environment: &Environment, now: &str) -> Self {
        let mut environments = BTreeMap::new();
        environments.insert(
            default_environment.as_str().to_string(),
            EnvironmentMeta {
                created_at: now.to_string(),
            },
        );
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            active_environment: default_environment.as_str().to_string(),
            environments,
        }
    }

    pub fn has_environment(&self, environment: &Environment) -> bool {
        self.environments.contains_key(environment.as_str())
    }

    pub fn add_environment(&mut self, environment: &Environment, created_at: String) {
        self.environments
            .entry(environment.as_str().to_string())
            .or_insert(EnvironmentMeta { created_at });
    }

    pub fn set_active(&mut self, environment: &Environment) {
        self.active_environment = environment.as_str().to_string();
    }

    /// Environment names in deterministic (sorted) order.
    pub fn environment_names(&self) -> Vec<&str> {
        self.environments.keys().map(String::as_str).collect()
    }
}

/// A minimal, dependency-free RFC 3339 UTC timestamp for `created_at` fields.
///
/// This is calendar arithmetic, not cryptography, so it does not touch the
/// crypto boundary locked in `crypto-and-scanning-engine.md`. Implemented by
/// hand (rather than pulling in `chrono`/`time`) to avoid adding a dependency
/// beyond the scaffold's `Cargo.toml` for a single "what's today's date"
/// need; see the days-to-civil-date algorithm this is based on (Howard
/// Hinnant's `civil_from_days`).
pub fn now_rfc3339() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let days = (secs / 86_400) as i64;
    let secs_of_day = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Days-since-epoch (1970-01-01) to a proleptic Gregorian (year, month, day).
/// See Howard Hinnant's `chrono`-independent `civil_from_days` algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_seeds_development_environment() {
        let env = Environment::new("development").unwrap();
        let config = VaultConfig::new_with_default_environment(&env, "2026-09-06T00:00:00Z");
        assert_eq!(config.format_version, CURRENT_FORMAT_VERSION);
        assert_eq!(config.active_environment, "development");
        assert!(config.has_environment(&env));
    }

    #[test]
    fn toml_round_trip() {
        let env = Environment::new("development").unwrap();
        let config = VaultConfig::new_with_default_environment(&env, "2026-09-06T00:00:00Z");
        let serialized = toml::to_string(&config).expect("serialize");
        let deserialized: VaultConfig = toml::from_str(&serialized).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn add_and_set_active_environment() {
        let dev = Environment::new("development").unwrap();
        let mut config = VaultConfig::new_with_default_environment(&dev, "2026-09-06T00:00:00Z");
        let staging = Environment::new("staging").unwrap();
        config.add_environment(&staging, "2026-09-07T00:00:00Z".to_string());
        config.set_active(&staging);
        assert!(config.has_environment(&staging));
        assert_eq!(config.active_environment, "staging");
        assert_eq!(config.environment_names(), vec!["development", "staging"]);
    }

    #[test]
    fn now_rfc3339_has_expected_shape() {
        let ts = now_rfc3339();
        // e.g. 2026-09-06T12:34:56Z
        assert_eq!(ts.len(), 20);
        assert_eq!(ts.as_bytes()[4], b'-');
        assert_eq!(ts.as_bytes()[7], b'-');
        assert_eq!(ts.as_bytes()[10], b'T');
        assert_eq!(ts.as_bytes()[19], b'Z');
    }

    #[test]
    fn civil_from_days_known_epoch() {
        // 1970-01-01 is day 0.
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2000-03-01 is a well-known reference date in this algorithm's tests.
        assert_eq!(civil_from_days(11_017), (2000, 3, 1));
    }
}
