//! `GitDiff` (staged files) and `HookInstaller` ports.

use std::path::PathBuf;

use super::super::VaultError;

pub trait GitDiff {
    /// Paths currently staged for commit (`git diff --cached --name-only`),
    /// relative to the repository root.
    fn staged_files(&self) -> Result<Vec<PathBuf>, VaultError>;
}

pub trait HookInstaller {
    /// Install the Vunexo Vault pre-commit block. Both `install` and
    /// `uninstall` operate only on a marker-delimited block inside
    /// `.git/hooks/pre-commit`, per `user-flows.md` §6 — never touching any
    /// pre-existing content outside that block.
    fn install(&self) -> Result<(), VaultError>;
    fn uninstall(&self) -> Result<(), VaultError>;
}
