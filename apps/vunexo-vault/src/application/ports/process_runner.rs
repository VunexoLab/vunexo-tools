//! `ProcessRunner` port: spawn a child process with extra environment
//! variables injected, per `secrets run`'s locked behavior
//! (`user-flows.md` §4) — the child's environment is the one designed
//! plaintext boundary in this whole system.

use std::process::ExitStatus;

use crate::domain::vault::Vault;

use super::super::VaultError;

pub trait ProcessRunner {
    /// Spawn `command` with `args`, inheriting the parent process's own
    /// environment plus every secret in `extra_env` (never replacing the
    /// inherited environment). Streams the child's stdio straight through
    /// and returns its exit status once it finishes.
    fn run(
        &self,
        command: &str,
        args: &[String],
        extra_env: &Vault,
    ) -> Result<ExitStatus, VaultError>;
}
