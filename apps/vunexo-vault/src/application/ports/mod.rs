//! Port traits (dependency-inversion boundaries between `application` and
//! `infrastructure`). No implementations live here.

pub mod config_store;
pub mod git;
pub mod process_runner;
pub mod scanner;
pub mod vault_store;

pub use config_store::ConfigStore;
pub use git::{GitDiff, HookInstaller};
pub use process_runner::ProcessRunner;
pub use scanner::Scanner;
pub use vault_store::VaultStore;
