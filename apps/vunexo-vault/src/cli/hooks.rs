//! `vunexo hooks install|uninstall` (`docs/vunexo-vault/cli-ux.md`).

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum HooksCommand {
    /// Install the git pre-commit hook.
    Install,
    /// Remove the git pre-commit hook (only Vunexo Vault's own block).
    Uninstall,
}
