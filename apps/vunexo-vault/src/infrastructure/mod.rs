//! Infrastructure layer: concrete port implementations (the only place that
//! touches the filesystem, the `age` crate, or spawns processes).

pub mod age_vault_store;
pub mod git_cli;
pub mod hook_installer;
pub mod pattern_scanner;
pub mod std_process_runner;
pub mod toml_config_store;
