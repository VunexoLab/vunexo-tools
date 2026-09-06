//! Tauri commands: thin wrappers around the shared `vunexo_vault` crate's
//! `application::*` use cases and `infrastructure::*` port implementations.
//!
//! Every command here follows the same shape as `apps/vunexo-vault/src/
//! main.rs`'s dispatch functions — construct the concrete infrastructure
//! adapters pointed at the open project's folder, call straight into an
//! existing use case, map the `Result` to a serializable shape. No command
//! implements any new secret-handling, TOML-parsing, or scanning logic; the
//! only genuinely new code paths are session-state bookkeeping (which
//! project is open, whether it's unlocked) and the GUI-only recent-projects
//! preference list, neither of which touches secret material.

use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use vunexo_vault::application;
use vunexo_vault::application::ports::{
    ConfigStore, GitDiff, HookInstaller as HookInstallerPort, VaultStore,
};
use vunexo_vault::application::VaultError;
use vunexo_vault::domain::vault::{Environment, SecretKey, SecretValue};
use vunexo_vault::infrastructure::age_vault_store::AgeVaultStore;
use vunexo_vault::infrastructure::git_cli::GitCli;
use vunexo_vault::infrastructure::hook_installer::HookInstaller;
use vunexo_vault::infrastructure::pattern_scanner::PatternScanner;
use vunexo_vault::infrastructure::toml_config_store::TomlConfigStore;
use vunexo_vault::secrecy::SecretString;

use crate::dto::{
    environments_from_config, AppInfoDto, EnvironmentDto, OpenProjectResultDto, ScanFindingDto,
};
use crate::error::ErrorDto;
use crate::recent_projects::{self, RecentProject};
use crate::state::{AppState, Session};

// --- adapter construction (mirrors main.rs's wiring, one project folder at a time) ---

fn config_store_for(project_path: &std::path::Path) -> TomlConfigStore {
    TomlConfigStore::new(project_path.join(".vunexo"))
}

fn vault_store_for(project_path: &std::path::Path) -> AgeVaultStore {
    AgeVaultStore::new(project_path.join(".vunexo"))
}

fn git_diff_for(project_path: &std::path::Path) -> GitCli {
    GitCli::new(project_path)
}

fn hook_installer_for(project_path: &std::path::Path) -> HookInstaller {
    HookInstaller::new(project_path)
}

/// Mirrors `main.rs`'s own private `active_environment` helper exactly: read
/// `config.toml` and validate its `active_environment` field as a real
/// `Environment`. Kept as a few lines of glue (not re-exported from the CLI
/// binary crate, which isn't a library) rather than duplicated logic — it
/// calls nothing but the same `ConfigStore::load`/`Environment::new` the CLI
/// itself calls.
fn active_environment(config_store: &dyn ConfigStore) -> Result<Environment, VaultError> {
    let config = config_store.load()?;
    Environment::new(config.active_environment.clone()).map_err(|e| {
        VaultError::Malformed(format!(
            "config.toml has an invalid active_environment: {e}"
        ))
    })
}

// --- session helpers ---

fn require_project(state: &State<'_, AppState>) -> Result<PathBuf, ErrorDto> {
    let guard = state.session.lock().expect("session mutex poisoned");
    guard
        .as_ref()
        .map(|s| s.project_path.clone())
        .ok_or_else(|| ErrorDto::session_state("no project is open"))
}

fn require_unlocked(state: &State<'_, AppState>) -> Result<(PathBuf, SecretString), ErrorDto> {
    let guard = state.session.lock().expect("session mutex poisoned");
    let session = guard
        .as_ref()
        .ok_or_else(|| ErrorDto::session_state("no project is open"))?;
    let passphrase = session
        .passphrase
        .clone()
        .ok_or_else(|| ErrorDto::session_state("vault is locked"))?;
    Ok((session.project_path.clone(), passphrase))
}

// --- Open Project gate ---

#[tauri::command]
pub fn list_recent_projects(app: AppHandle) -> Result<Vec<RecentProject>, ErrorDto> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| ErrorDto::general(format!("failed to resolve app data directory: {e}")))?;
    Ok(recent_projects::load(&data_dir))
}

#[tauri::command]
pub fn open_project(
    path: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<OpenProjectResultDto, ErrorDto> {
    let project_path = PathBuf::from(&path);
    let config_store = config_store_for(&project_path);
    let has_vault = config_store.exists();

    {
        let mut guard = state.session.lock().expect("session mutex poisoned");
        *guard = Some(Session {
            project_path: project_path.clone(),
            passphrase: None,
        });
    }

    // Best-effort: a failure to persist the recent-projects preference file
    // should never block opening the project itself.
    if let Ok(data_dir) = app.path().app_data_dir() {
        let _ = recent_projects::record_opened(&data_dir, &path);
    }

    Ok(OpenProjectResultDto { path, has_vault })
}

#[tauri::command]
pub fn init_vault(passphrase: String, state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let project_path = require_project(&state)?;
    let config_store = config_store_for(&project_path);
    let vault_store = vault_store_for(&project_path);
    let passphrase = SecretString::from(passphrase);

    application::init_vault::init_vault(&config_store, &vault_store, &passphrase)
        .map_err(ErrorDto::from)?;

    // We just chose this exact passphrase ourselves, so auto-unlock into the
    // new session rather than asking the user to immediately re-type what
    // they typed one screen ago. Still never touches disk — same in-memory,
    // this-session-only state `unlock` would have set.
    let mut guard = state.session.lock().expect("session mutex poisoned");
    if let Some(session) = guard.as_mut() {
        session.passphrase = Some(passphrase);
    }
    Ok(())
}

// --- Unlock gate ---

#[tauri::command]
pub fn unlock(
    environment: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<(), ErrorDto> {
    let project_path = require_project(&state)?;
    let config_store = config_store_for(&project_path);
    let vault_store = vault_store_for(&project_path);

    let env = Environment::new(environment).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
    let passphrase = SecretString::from(passphrase);

    // Verify the passphrase (when there's anything to verify it against)
    // *before* committing the environment switch below — a failed unlock
    // attempt must never leave `config.toml`'s `active_environment` changed.
    // Only environments that already have an `.age` file can actually be
    // verified against a passphrase (storage-schema.md: a brand-new
    // environment's vault file doesn't exist until its first `secrets set`).
    // Unlocking into a not-yet-materialized environment is accepted
    // provisionally, same as the CLI never being able to validate a
    // passphrase against a vault that doesn't exist yet.
    if vault_store.exists(&env) {
        vault_store
            .load(&env, &passphrase)
            .map_err(ErrorDto::from)?;
    }

    application::environments::use_environment(&config_store, env).map_err(ErrorDto::from)?;

    let mut guard = state.session.lock().expect("session mutex poisoned");
    if let Some(session) = guard.as_mut() {
        session.passphrase = Some(passphrase);
    }
    Ok(())
}

#[tauri::command]
pub fn lock(state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let mut guard = state.session.lock().expect("session mutex poisoned");
    let session = guard
        .as_mut()
        .ok_or_else(|| ErrorDto::session_state("no project is open"))?;
    session.passphrase = None;
    Ok(())
}

// --- Environments screen (also used by the Unlock gate's environment picker) ---

#[tauri::command]
pub fn list_environments(state: State<'_, AppState>) -> Result<Vec<EnvironmentDto>, ErrorDto> {
    let project_path = require_project(&state)?;
    let config_store = config_store_for(&project_path);
    let config =
        application::environments::list_environments(&config_store).map_err(ErrorDto::from)?;
    Ok(environments_from_config(&config))
}

/// Backs both the Environments screen's "New Environment" (a name not seen
/// before) and "Switch to" (an existing name) actions — `use_environment`
/// itself already treats both the same way (`application/environments.rs`).
#[tauri::command]
pub fn use_environment(name: String, state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let project_path = require_project(&state)?;
    let config_store = config_store_for(&project_path);
    let environment =
        Environment::new(name).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
    application::environments::use_environment(&config_store, environment)
        .map_err(ErrorDto::from)?;
    Ok(())
}

// --- Secrets screen ---

#[tauri::command]
pub fn list_secrets(state: State<'_, AppState>) -> Result<Vec<String>, ErrorDto> {
    let (project_path, passphrase) = require_unlocked(&state)?;
    let config_store = config_store_for(&project_path);
    let vault_store = vault_store_for(&project_path);
    let environment = active_environment(&config_store).map_err(ErrorDto::from)?;

    match application::secrets::list_secrets(&vault_store, &environment, &passphrase) {
        Ok(keys) => Ok(keys.into_iter().map(|k| k.as_str().to_string()).collect()),
        // A brand-new environment has no `.age` file yet until its first
        // `secrets set` (see `application::environments`'s deferred-creation
        // note) — from the user's perspective that's indistinguishable from
        // "zero secrets," so the Secrets screen renders it as an empty list
        // rather than surfacing a raw "no vault for environment `x`" error
        // for a state that isn't actually a mistake.
        Err(VaultError::VaultNotFound(_)) => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub fn get_secret(key: String, state: State<'_, AppState>) -> Result<String, ErrorDto> {
    let (project_path, passphrase) = require_unlocked(&state)?;
    let config_store = config_store_for(&project_path);
    let vault_store = vault_store_for(&project_path);
    let environment = active_environment(&config_store).map_err(ErrorDto::from)?;
    let key = SecretKey::new(key).map_err(|e| VaultError::InvalidInput(e.to_string()))?;

    let value = application::secrets::get_secret(&vault_store, &environment, &passphrase, &key)
        .map_err(ErrorDto::from)?;
    Ok(value.expose().to_string())
}

#[tauri::command]
pub fn set_secret(key: String, value: String, state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let (project_path, passphrase) = require_unlocked(&state)?;
    let config_store = config_store_for(&project_path);
    let vault_store = vault_store_for(&project_path);
    let environment = active_environment(&config_store).map_err(ErrorDto::from)?;
    let key = SecretKey::new(key).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
    let value = SecretValue::new(value);

    application::secrets::set_secret(&vault_store, &environment, &passphrase, key, value)
        .map_err(ErrorDto::from)?;
    Ok(())
}

#[tauri::command]
pub fn remove_secret(key: String, state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let (project_path, passphrase) = require_unlocked(&state)?;
    let config_store = config_store_for(&project_path);
    let vault_store = vault_store_for(&project_path);
    let environment = active_environment(&config_store).map_err(ErrorDto::from)?;
    let key = SecretKey::new(key).map_err(|e| VaultError::InvalidInput(e.to_string()))?;

    application::secrets::remove_secret(&vault_store, &environment, &passphrase, &key)
        .map_err(ErrorDto::from)?;
    Ok(())
}

// --- Scan screen ---

/// Whether the open project is a git repository, so the Scan screen can
/// enable/disable its "Staged only" toggle before the user ever presses "Run
/// Scan." Thinly wraps the existing `GitDiff::staged_files` port call
/// (reusing whatever it does to detect a repo) rather than re-deriving git
/// detection: any `Ok` means it is one, `NotAGitRepository` means it isn't,
/// anything else is a real error worth surfacing.
#[tauri::command]
pub fn is_git_repository(state: State<'_, AppState>) -> Result<bool, ErrorDto> {
    let project_path = require_project(&state)?;
    let git_diff = git_diff_for(&project_path);
    match git_diff.staged_files() {
        Ok(_) => Ok(true),
        Err(VaultError::NotAGitRepository) => Ok(false),
        Err(e) => Err(e.into()),
    }
}

#[tauri::command]
pub fn run_scan(
    staged: bool,
    path: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<ScanFindingDto>, ErrorDto> {
    let project_path = require_project(&state)?;
    let scanner = PatternScanner::new();

    let findings = if staged {
        let git_diff = git_diff_for(&project_path);
        application::scan::scan_staged(&git_diff, &scanner).map_err(ErrorDto::from)?
    } else {
        let target = path
            .map(PathBuf::from)
            .unwrap_or_else(|| project_path.clone());
        application::scan::scan_path(&scanner, Some(target))
    };

    Ok(findings.iter().map(ScanFindingDto::from).collect())
}

// --- Hooks screen ---

#[tauri::command]
pub fn hook_status(state: State<'_, AppState>) -> Result<bool, ErrorDto> {
    let project_path = require_project(&state)?;
    let installer = hook_installer_for(&project_path);
    installer.is_installed().map_err(Into::into)
}

#[tauri::command]
pub fn install_hook(state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let project_path = require_project(&state)?;
    let installer = hook_installer_for(&project_path);
    installer.install().map_err(Into::into)
}

#[tauri::command]
pub fn uninstall_hook(state: State<'_, AppState>) -> Result<(), ErrorDto> {
    let project_path = require_project(&state)?;
    let installer = hook_installer_for(&project_path);
    installer.uninstall().map_err(Into::into)
}

// --- Settings / About screen ---

#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> Result<AppInfoDto, ErrorDto> {
    let project_path = require_project(&state)?;
    let config_store = config_store_for(&project_path);
    let config = config_store.load().map_err(ErrorDto::from)?;
    Ok(AppInfoDto {
        format_version: config.format_version,
        project_path: project_path.display().to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        license: "MIT".to_string(),
    })
}
