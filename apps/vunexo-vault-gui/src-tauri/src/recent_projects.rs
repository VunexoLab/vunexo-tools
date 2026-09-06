//! The one piece of GUI-only preference state: a list of recently-opened
//! project folder paths, persisted under Tauri's own app-data directory —
//! never inside a project's `.vunexo/` folder, and never containing any
//! secret material (just filesystem paths and timestamps).
//!
//! Reuses `vunexo_vault::domain::config::now_rfc3339` for the timestamp
//! rather than re-deriving a "what's today's date" helper — the shared crate
//! already has one, and per the architecture amendment this app is meant to
//! lean on the shared crate wherever it already does the job, not just for
//! crypto.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use vunexo_vault::domain::config::now_rfc3339;

const MAX_RECENT: usize = 8;
const FILE_NAME: &str = "recent_projects.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub last_opened: String,
}

fn store_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(FILE_NAME)
}

/// Reads the persisted list, newest first. A missing or unreadable file is
/// treated as an empty list — this is best-effort convenience state, not
/// anything worth failing project-open over.
pub fn load(app_data_dir: &Path) -> Vec<RecentProject> {
    let path = store_path(app_data_dir);
    let Ok(raw) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Records `project_path` as the most recently opened project: moves it to
/// the front if already present, otherwise inserts it, then caps the list at
/// `MAX_RECENT` entries.
pub fn record_opened(
    app_data_dir: &Path,
    project_path: &str,
) -> std::io::Result<Vec<RecentProject>> {
    let mut list = load(app_data_dir);
    list.retain(|p| p.path != project_path);
    list.insert(
        0,
        RecentProject {
            path: project_path.to_string(),
            last_opened: now_rfc3339(),
        },
    );
    list.truncate(MAX_RECENT);

    fs::create_dir_all(app_data_dir)?;
    let serialized = serde_json::to_string_pretty(&list)?;
    fs::write(store_path(app_data_dir), serialized)?;
    Ok(list)
}
