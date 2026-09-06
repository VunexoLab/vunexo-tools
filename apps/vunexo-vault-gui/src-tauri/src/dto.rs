//! Serializable shapes returned to the frontend. Every field here is a
//! plain, non-secret projection of a shared-crate domain type (or of GUI-only
//! preference state) — no domain/application/infrastructure type is ever
//! re-derived, only read from and mapped.

use serde::Serialize;
use vunexo_vault::domain::config::VaultConfig;
use vunexo_vault::domain::scan::ScanFinding;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct OpenProjectResultDto {
    pub path: String,
    pub has_vault: bool,
}

#[derive(Debug, Serialize)]
pub struct EnvironmentDto {
    pub name: String,
    pub active: bool,
    pub created_at: String,
}

/// Projects a loaded `VaultConfig` into the sorted list the Environments
/// screen renders — mirrors the CLI's `env list` ordering (deterministic,
/// via `VaultConfig::environment_names`'s own `BTreeMap` iteration).
pub fn environments_from_config(config: &VaultConfig) -> Vec<EnvironmentDto> {
    config
        .environment_names()
        .into_iter()
        .map(|name| EnvironmentDto {
            name: name.to_string(),
            active: name == config.active_environment,
            created_at: config
                .environments
                .get(name)
                .map(|meta| meta.created_at.clone())
                .unwrap_or_default(),
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct ScanFindingDto {
    pub path: String,
    pub line: usize,
    pub rule_name: String,
    pub redacted_preview: String,
}

impl From<&ScanFinding> for ScanFindingDto {
    fn from(finding: &ScanFinding) -> Self {
        Self {
            path: finding.path.display().to_string(),
            line: finding.line,
            rule_name: finding.rule_name.to_string(),
            redacted_preview: finding.redacted_preview.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AppInfoDto {
    pub format_version: u32,
    pub project_path: String,
    pub app_version: String,
    pub license: String,
}
