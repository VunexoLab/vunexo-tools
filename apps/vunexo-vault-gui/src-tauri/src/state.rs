//! GUI session state: which project folder is currently open, and (once
//! unlocked) the in-memory passphrase for this session only.
//!
//! Nothing here is written to disk — the whole point of this struct is that
//! it disappears the moment the process exits, same as the CLI never
//! persisting a passphrase between invocations. `Lock` (Settings screen)
//! clears just the `passphrase` field, keeping `project_path` so the app
//! returns to the Unlock gate rather than the Open Project gate.

use std::path::PathBuf;
use std::sync::Mutex;

use vunexo_vault::secrecy::SecretString;

pub struct Session {
    pub project_path: PathBuf,
    /// `None` until `unlock` succeeds; cleared again by `lock`.
    pub passphrase: Option<SecretString>,
}

#[derive(Default)]
pub struct AppState {
    pub session: Mutex<Option<Session>>,
}
