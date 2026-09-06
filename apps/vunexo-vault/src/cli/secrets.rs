//! `vunexo secrets set|get|list|remove|run|scan` (`docs/vunexo-vault/cli-ux.md`).

use std::path::PathBuf;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum SecretsCommand {
    /// Set a secret. `KEY` prompts for the value (masked); `KEY=VALUE`
    /// takes it inline (for scripting — still never logged or echoed back).
    Set {
        /// `KEY` or `KEY=VALUE`.
        key_value: String,
    },
    /// Print a secret's value to stdout, undecorated (pipeable).
    Get { key: String },
    /// List key names only, sorted. Never prints values.
    List,
    /// Remove a secret. Exit `3` if it doesn't exist (not silently `0`).
    Remove { key: String },
    /// Run a command with the active environment's secrets injected into
    /// its process environment only — no `.env` file is ever written.
    ///
    /// Usage: `vunexo secrets run -- COMMAND [ARGS...]`
    Run {
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Scan for likely secrets (pattern + entropy heuristics).
    Scan {
        /// Scan only the currently staged (`git diff --cached`) files.
        #[arg(long)]
        staged: bool,
        /// Scan this specific file or directory instead of the current
        /// directory. Mutually exclusive with `--staged`.
        #[arg(conflicts_with = "staged")]
        path: Option<PathBuf>,
    },
}
