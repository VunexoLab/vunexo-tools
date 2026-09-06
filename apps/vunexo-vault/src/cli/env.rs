//! `vunexo env list|use` (`docs/vunexo-vault/cli-ux.md`).

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum EnvCommand {
    /// List every environment, marking the active one.
    List,
    /// Switch to `NAME`, creating it first if it doesn't exist yet.
    Use {
        /// Environment name.
        name: String,
    },
}
