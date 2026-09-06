//! `clap` command/arg definitions. Maps directly onto the locked command
//! tree from `.ai/product-vunexo-vault.md`; exact flags/exit codes are
//! pinned in `docs/vunexo-vault/cli-ux.md`. No dispatch logic lives here —
//! that's `main.rs`'s job.

pub mod env;
pub mod hooks;
pub mod init;
pub mod secrets;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use env::EnvCommand;
use hooks::HooksCommand;
use secrets::SecretsCommand;

#[derive(Debug, Parser)]
#[command(
    name = "vunexo",
    version,
    about = "The simplest open-source secret manager for local development."
)]
pub struct Cli {
    /// Read the vault passphrase from this file instead of an interactive
    /// masked prompt. The passphrase is never accepted as a plain CLI
    /// argument (it would leak into shell history and process listings).
    #[arg(long, global = true)]
    pub passphrase_file: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create the local encrypted vault.
    Init,
    /// Manage individual secret values.
    Secrets {
        #[command(subcommand)]
        command: SecretsCommand,
    },
    /// Manage named environments.
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    /// Manage the git pre-commit hook.
    Hooks {
        #[command(subcommand)]
        command: HooksCommand,
    },
}
