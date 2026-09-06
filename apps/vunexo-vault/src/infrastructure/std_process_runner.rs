//! `ProcessRunner` implementation using `std::process::Command`.

use std::process::ExitStatus;

use crate::domain::vault::Vault;

use crate::application::ports::ProcessRunner;
use crate::application::VaultError;

pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    fn run(
        &self,
        command: &str,
        args: &[String],
        extra_env: &Vault,
    ) -> Result<ExitStatus, VaultError> {
        let mut cmd = std::process::Command::new(command);
        cmd.args(args);
        // Inherits the parent's environment by default (std::process::Command
        // does not clear it unless `.env_clear()` is called), and we only add
        // to it here — never replacing anything already inherited.
        for (key, value) in extra_env.iter() {
            cmd.env(key.as_str(), value.expose());
        }
        let status = cmd
            .status()
            .map_err(|e| VaultError::Io(format!("failed to run `{command}`: {e}")))?;
        Ok(status)
    }
}
