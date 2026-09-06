//! Maps `vunexo_vault::application::VaultError` (and a couple of GUI-only
//! session-state preconditions the stateless CLI has no equivalent of) to a
//! serializable shape the frontend can render.
//!
//! Every `VaultError` variant's `message` here is exactly that error's own
//! `Display` output — never a new, GUI-authored string — so the same
//! passphrase/not-found/hook wording the CLI prints to stderr is what shows
//! up in the GUI too (`docs/vunexo-vault/gui-ux.md`'s "every action's error
//! message matches the CLI's wording" cross-cutting rule).

use serde::Serialize;
use vunexo_vault::application::VaultError;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    WrongPassphrase,
    NotFound,
    AlreadyExists,
    NotAGitRepository,
    General,
    /// A GUI session-state precondition wasn't met (no project open, or the
    /// vault isn't unlocked yet). Not a `VaultError` at all — the CLI is
    /// stateless per invocation and has no notion of "not unlocked yet".
    SessionState,
}

#[derive(Debug, Serialize)]
pub struct ErrorDto {
    pub kind: ErrorKind,
    pub message: String,
}

impl ErrorDto {
    pub fn session_state(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::SessionState,
            message: message.into(),
        }
    }

    pub fn general(message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::General,
            message: message.into(),
        }
    }
}

impl From<VaultError> for ErrorDto {
    fn from(err: VaultError) -> Self {
        let kind = match &err {
            VaultError::WrongPassphrase => ErrorKind::WrongPassphrase,
            VaultError::VaultNotFound(_) | VaultError::SecretNotFound(_) => ErrorKind::NotFound,
            VaultError::AlreadyInitialized | VaultError::EnvironmentAlreadyExists(_) => {
                ErrorKind::AlreadyExists
            }
            VaultError::NotAGitRepository => ErrorKind::NotAGitRepository,
            _ => ErrorKind::General,
        };
        // `VaultError`'s own `Display` impl is the single source of truth for
        // wording (see `application/mod.rs`) — never re-derived here.
        let message = err.to_string();
        Self { kind, message }
    }
}
