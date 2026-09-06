//! `GitDiff` implementation: shells out to `git diff --cached --name-only`.

use std::path::PathBuf;
use std::process::Command;

use crate::application::ports::GitDiff;
use crate::application::VaultError;

pub struct GitCli {
    /// Directory to run `git` in (typically the current working directory).
    working_dir: PathBuf,
}

impl GitCli {
    pub fn new(working_dir: impl Into<PathBuf>) -> Self {
        Self {
            working_dir: working_dir.into(),
        }
    }

    /// The repository's top-level working directory, so relative paths from
    /// `git diff --name-only` resolve correctly regardless of the caller's
    /// own current directory.
    fn repo_root(&self) -> Result<PathBuf, VaultError> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| VaultError::Io(format!("failed to run git: {e}")))?;
        if !output.status.success() {
            return Err(VaultError::NotAGitRepository);
        }
        let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(root))
    }
}

impl GitDiff for GitCli {
    fn staged_files(&self) -> Result<Vec<PathBuf>, VaultError> {
        let root = self.repo_root()?;
        let output = Command::new("git")
            .arg("diff")
            .arg("--cached")
            .arg("--name-only")
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| VaultError::Io(format!("failed to run git: {e}")))?;
        if !output.status.success() {
            return Err(VaultError::Io(
                "git diff --cached --name-only failed".to_string(),
            ));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| root.join(line))
            .collect())
    }
}
