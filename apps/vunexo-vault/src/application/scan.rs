//! `ScanPath`/`ScanStaged` use cases (`user-flows.md` §5).

use std::path::PathBuf;

use crate::domain::scan::ScanFinding;

use super::ports::{GitDiff, Scanner};
use super::VaultError;

/// `vunexo secrets scan [PATH]` — scan a specific path, or the current
/// directory if `path` is `None`.
pub fn scan_path(scanner: &dyn Scanner, path: Option<PathBuf>) -> Vec<ScanFinding> {
    let target = path.unwrap_or_else(|| PathBuf::from("."));
    scanner.scan(&[target])
}

/// `vunexo secrets scan --staged` — scan only the currently staged
/// (`git diff --cached`) files.
pub fn scan_staged(
    git_diff: &dyn GitDiff,
    scanner: &dyn Scanner,
) -> Result<Vec<ScanFinding>, VaultError> {
    let staged = git_diff.staged_files()?;
    Ok(scanner.scan(&staged))
}
