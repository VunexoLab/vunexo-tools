//! Application layer: use cases and the ports they depend on.
//!
//! One `VaultError` enum covers every failure mode named in
//! `docs/vunexo-vault/application-architecture.md`. `main.rs` maps each
//! variant to a specific exit code (`docs/vunexo-vault/cli-ux.md`) and a
//! human-readable message.

pub mod environments;
pub mod init_vault;
pub mod ports;
pub mod run_with_secrets;
pub mod scan;
pub mod secrets;

use std::fmt;

/// Errors surfaced by the application layer.
///
/// `WrongPassphrase` is deliberately used for **every** vault decrypt
/// failure — whether the passphrase was actually wrong or the ciphertext was
/// corrupted — per `user-flows.md` §7 and `crypto-and-scanning-engine.md`:
/// the message must never hint at *which* part of the check failed, to avoid
/// leaking oracle information about the vault's contents. The underlying
/// `age` library error is intentionally not included in the message or the
/// `Debug`/`Display` output anywhere in this enum, matching `cli-ux.md`'s
/// "no raw library error text" rule.
#[derive(Debug)]
pub enum VaultError {
    /// `.vunexo/` already exists; `init` refuses to overwrite it.
    AlreadyInitialized,
    /// Decryption failed — wrong passphrase or a corrupted/tampered file.
    /// Always the same generic message (see the enum's doc comment).
    WrongPassphrase,
    /// Encryption failed. Distinct from `WrongPassphrase` because it can
    /// only happen while *writing* a vault, never while reading one, so it
    /// carries no oracle-information risk.
    EncryptionFailed,
    /// No vault file exists yet for this environment.
    VaultNotFound(String),
    /// Reserved for a future explicit "create environment" flow. No command
    /// in the locked V1 surface can actually trigger this: `env use` on a
    /// name that already exists simply switches to it (see
    /// `application::environments::use_environment`). Kept in the enum
    /// because `application-architecture.md` names it explicitly as part of
    /// the locked `VaultError` design.
    #[allow(dead_code)]
    EnvironmentAlreadyExists(String),
    /// The requested secret key doesn't exist in the active environment.
    SecretNotFound(String),
    /// The current directory is not (the root of) a git repository.
    NotAGitRepository,
    /// `.git/hooks/pre-commit` already exists and was not written by Vunexo
    /// Vault; `hooks install` refuses to clobber it.
    HookAlreadyExists,
    /// The two passphrase entries during `init`/interactive prompts did not
    /// match.
    PassphraseMismatch,
    /// An I/O error (reading/writing `.vunexo/`, the passphrase file, a
    /// scanned file, etc).
    Io(String),
    /// A `.vunexo/config.toml` or vault plaintext payload could not be
    /// parsed, or its `format_version` is unsupported.
    Malformed(String),
    /// A user-supplied environment name or secret key failed validation.
    InvalidInput(String),
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultError::AlreadyInitialized => {
                write!(f, "a vault already exists here (.vunexo/); remove it manually first if you want to start over")
            }
            VaultError::WrongPassphrase => write!(f, "incorrect passphrase"),
            VaultError::EncryptionFailed => write!(f, "failed to encrypt vault"),
            VaultError::VaultNotFound(env) => write!(f, "no vault for environment `{env}`"),
            VaultError::EnvironmentAlreadyExists(env) => {
                write!(f, "environment `{env}` already exists")
            }
            VaultError::SecretNotFound(key) => write!(f, "secret `{key}` not found"),
            VaultError::NotAGitRepository => write!(f, "not a git repository"),
            VaultError::HookAlreadyExists => write!(
                f,
                "a pre-commit hook already exists and was not installed by vunexo; add `vunexo secrets scan --staged` to it manually"
            ),
            VaultError::PassphraseMismatch => write!(f, "passphrases did not match"),
            VaultError::Io(msg) => write!(f, "{msg}"),
            VaultError::Malformed(msg) => write!(f, "{msg}"),
            VaultError::InvalidInput(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(err: std::io::Error) -> Self {
        VaultError::Io(err.to_string())
    }
}
