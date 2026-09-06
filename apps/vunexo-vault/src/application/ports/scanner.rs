//! `Scanner` port: pattern + entropy secret detection.

use std::path::PathBuf;

use crate::domain::scan::ScanFinding;

pub trait Scanner {
    /// Scan the given paths (files and/or directories) and return every
    /// finding. Never errors — an unreadable individual file is silently
    /// skipped (see `infrastructure::pattern_scanner`), since scanning is a
    /// best-effort heuristic safety net, not a guarantee
    /// (`product-vunexo-vault.md`'s hard boundary #2).
    fn scan(&self, paths: &[PathBuf]) -> Vec<ScanFinding>;
}
