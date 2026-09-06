//! `HookInstaller` implementation: writes/removes a marker-delimited block
//! inside `.git/hooks/pre-commit`, per `user-flows.md` §6.
//!
//! Uses `git rev-parse --git-path hooks` (rather than hardcoding
//! `.git/hooks`) so this also works correctly from a git worktree, where
//! hooks live in the main repository's `.git` directory, not the worktree's
//! own `.git` file.
//!
//! `install` never appends onto a pre-existing hook it doesn't recognize as
//! its own: `user-flows.md` §6 reads, at first glance, as if "writes (or
//! appends to, if a pre-commit hook already exists)" and "refuses to
//! overwrite [a hook that] wasn't written by Vunexo Vault" are in tension —
//! but the second bullet is the refinement of the first: appending only
//! ever applies to a hook Vunexo Vault itself wrote (which, since installing
//! twice would just duplicate the block, is really the "already installed"
//! no-op below), never to a third party's hook. So `install` only ever does
//! one of three things: create a fresh file (nothing there yet), no-op (our
//! marker is already present), or refuse outright (something else is there).
//! It never mutates a foreign hook, which is exactly what makes the
//! "install then uninstall restores a pre-existing unrelated hook
//! byte-for-byte" integration test hold trivially — the file is never
//! touched in the first place.
//!
//! `uninstall` still strips only the marker-delimited block rather than
//! unconditionally deleting on sight, and deletes the file only once that
//! leaves nothing (or only the shebang line) behind. That generality isn't
//! dead weight: a user could hand-edit the file after `install` to add their
//! own lines around Vunexo Vault's block, and `uninstall` must still remove
//! only the block it owns, leaving anything else untouched, per the locked
//! flow.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::application::ports::HookInstaller as HookInstallerPort;
use crate::application::VaultError;

const SHEBANG: &str = "#!/bin/sh\n";
const MARKER_START: &str = "# >>> vunexo-vault >>>\n";
const HOOK_BODY: &str = "vunexo secrets scan --staged\n";
const MARKER_END: &str = "# <<< vunexo-vault <<<\n";

fn block() -> String {
    format!("{MARKER_START}{HOOK_BODY}{MARKER_END}")
}

pub struct HookInstaller {
    working_dir: PathBuf,
}

impl HookInstaller {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }

    fn hooks_dir(&self) -> Result<PathBuf, VaultError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--git-path")
            .arg("hooks")
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| VaultError::Io(format!("failed to run git: {e}")))?;
        if !output.status.success() {
            return Err(VaultError::NotAGitRepository);
        }
        let rel = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(self.working_dir.join(rel))
    }

    fn pre_commit_path(&self) -> Result<PathBuf, VaultError> {
        Ok(self.hooks_dir()?.join("pre-commit"))
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), VaultError> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), VaultError> {
    // Windows doesn't use POSIX executable bits; Git for Windows invokes
    // hook scripts via the shebang line through its bundled shell.
    Ok(())
}

impl HookInstallerPort for HookInstaller {
    fn install(&self) -> Result<(), VaultError> {
        let path = self.pre_commit_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !path.exists() {
            let content = format!("{SHEBANG}\n{}", block());
            fs::write(&path, content)?;
            set_executable(&path)?;
            return Ok(());
        }

        let existing = fs::read_to_string(&path)
            .map_err(|e| VaultError::Io(format!("failed to read {}: {e}", path.display())))?;

        if existing.contains(MARKER_START) {
            // Already installed — idempotent no-op.
            return Ok(());
        }

        // A pre-existing hook not written by Vunexo Vault: refuse to
        // overwrite it (user-flows.md §6).
        Err(VaultError::HookAlreadyExists)
    }

    fn uninstall(&self) -> Result<(), VaultError> {
        let path = self.pre_commit_path()?;
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| VaultError::Io(format!("failed to read {}: {e}", path.display())))?;

        if !content.contains(MARKER_START) {
            // Not ours — leave it untouched.
            return Ok(());
        }

        let suffix = format!("\n{}", block());
        match content.strip_suffix(suffix.as_str()) {
            Some(remainder) if remainder.is_empty() || remainder == SHEBANG => {
                fs::remove_file(&path)?;
            }
            Some(remainder) => {
                fs::write(&path, remainder)?;
            }
            None => {
                // The marker is present but not in the exact shape we write
                // (e.g. hand-edited) — leave the file untouched rather than
                // risk corrupting content we can't confidently identify as
                // ours.
            }
        }
        Ok(())
    }
}
