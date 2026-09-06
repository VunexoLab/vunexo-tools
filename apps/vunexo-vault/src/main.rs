//! Entry point: parse args, construct infrastructure adapters, dispatch into
//! the matching use case, and map errors to exit codes.
//!
//! See `docs/vunexo-vault/application-architecture.md` for the module
//! layout and `docs/vunexo-vault/cli-ux.md` for the exact exit-code
//! contract this file implements.

mod cli;

use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use clap::Parser;
use owo_colors::{OwoColorize, Stream::Stdout};
use secrecy::SecretString;

use vunexo_vault::application;
use vunexo_vault::application::ports::{ConfigStore, HookInstaller as HookInstallerPort};
use vunexo_vault::application::VaultError;
use vunexo_vault::domain::vault::{Environment, SecretKey, SecretValue};
use vunexo_vault::infrastructure;

use cli::env::EnvCommand;
use cli::hooks::HooksCommand;
use cli::secrets::SecretsCommand;
use cli::{Cli, Command};
use infrastructure::age_vault_store::AgeVaultStore;
use infrastructure::git_cli::GitCli;
use infrastructure::hook_installer::HookInstaller;
use infrastructure::pattern_scanner::PatternScanner;
use infrastructure::std_process_runner::StdProcessRunner;
use infrastructure::toml_config_store::TomlConfigStore;

/// What to do once `run` returns successfully. Kept distinct from a plain
/// exit code so `secrets run`'s child exit status can be passed through
/// with full fidelity (an `ExitStatus`'s underlying code can exceed `u8` on
/// some platforms), while every other command uses the small, locked
/// exit-code contract in `cli-ux.md`.
enum ExitAction {
    Code(u8),
    ChildStatus(ExitStatus),
}

fn main() {
    let cli = Cli::parse();
    match run(cli) {
        Ok(ExitAction::Code(code)) => std::process::exit(code.into()),
        Ok(ExitAction::ChildStatus(status)) => {
            // On Unix, a process killed by a signal has no exit code; 1 is a
            // reasonable fallback (not itself part of the locked exit-code
            // contract, which only covers Vunexo's own outcomes).
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(exit_code_for(&err).into());
        }
    }
}

/// Maps a `VaultError` to the exit code pinned in `cli-ux.md`.
fn exit_code_for(err: &VaultError) -> u8 {
    match err {
        VaultError::WrongPassphrase => 2,
        VaultError::VaultNotFound(_) | VaultError::SecretNotFound(_) => 3,
        _ => 1,
    }
}

fn read_passphrase(
    passphrase_file: &Option<PathBuf>,
    prompt: &str,
) -> Result<SecretString, VaultError> {
    match passphrase_file {
        Some(path) => {
            let content = std::fs::read_to_string(path).map_err(|e| {
                VaultError::Io(format!(
                    "failed to read passphrase file {}: {e}",
                    path.display()
                ))
            })?;
            let trimmed = content.trim_end_matches(['\n', '\r']);
            Ok(SecretString::from(trimmed.to_string()))
        }
        None => {
            let value = rpassword::prompt_password(prompt)?;
            Ok(SecretString::from(value))
        }
    }
}

/// Reads and confirms a brand-new passphrase (masked, re-entered), per
/// `user-flows.md` §1's `ssh-keygen`/`gpg --gen-key`-style UX. When
/// `--passphrase-file` is given there is nothing to confirm — the file is
/// the single source of truth the user already manages.
fn read_new_passphrase(passphrase_file: &Option<PathBuf>) -> Result<SecretString, VaultError> {
    if passphrase_file.is_some() {
        return read_passphrase(passphrase_file, "");
    }
    let first = rpassword::prompt_password("New passphrase: ")?;
    let second = rpassword::prompt_password("Confirm passphrase: ")?;
    if first != second {
        return Err(VaultError::PassphraseMismatch);
    }
    Ok(SecretString::from(first))
}

fn active_environment(config_store: &dyn ConfigStore) -> Result<Environment, VaultError> {
    let config = config_store.load()?;
    Environment::new(config.active_environment.clone()).map_err(|e| {
        VaultError::Malformed(format!(
            "config.toml has an invalid active_environment: {e}"
        ))
    })
}

fn run(cli: Cli) -> Result<ExitAction, VaultError> {
    let cwd = std::env::current_dir()?;
    let vunexo_dir = cwd.join(".vunexo");
    let config_store = TomlConfigStore::new(&vunexo_dir);
    let vault_store = AgeVaultStore::new(&vunexo_dir);

    match cli.command {
        Command::Init => run_init(&cli.passphrase_file, &config_store, &vault_store),
        Command::Env { command } => run_env(command, &config_store),
        Command::Hooks { command } => run_hooks(command, &cwd),
        Command::Secrets { command } => run_secrets(
            command,
            &cli.passphrase_file,
            &cwd,
            &config_store,
            &vault_store,
        ),
    }
}

fn run_init(
    passphrase_file: &Option<PathBuf>,
    config_store: &TomlConfigStore,
    vault_store: &AgeVaultStore,
) -> Result<ExitAction, VaultError> {
    let passphrase = read_new_passphrase(passphrase_file)?;
    application::init_vault::init_vault(config_store, vault_store, &passphrase)?;
    println!(
        "{} Initialized an empty vault in .vunexo/ (environment: development).",
        "✓".if_supports_color(Stdout, |t| t.green().to_string())
    );
    println!(
        "It's safe to commit .vunexo/ to version control (ciphertext-at-rest) — review it once, then consider running `vunexo hooks install`."
    );
    Ok(ExitAction::Code(0))
}

fn run_env(command: EnvCommand, config_store: &TomlConfigStore) -> Result<ExitAction, VaultError> {
    match command {
        EnvCommand::List => {
            let config = application::environments::list_environments(config_store)?;
            for name in config.environment_names() {
                if name == config.active_environment {
                    println!(
                        "* {}",
                        name.if_supports_color(Stdout, |t| t.green().bold().to_string())
                    );
                } else {
                    println!("  {name}");
                }
            }
            Ok(ExitAction::Code(0))
        }
        EnvCommand::Use { name } => {
            let environment =
                Environment::new(name).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
            application::environments::use_environment(config_store, environment)?;
            Ok(ExitAction::Code(0))
        }
    }
}

fn run_hooks(command: HooksCommand, cwd: &Path) -> Result<ExitAction, VaultError> {
    let installer = HookInstaller::new(cwd);
    match command {
        HooksCommand::Install => {
            installer.install()?;
            println!(
                "{} Installed the pre-commit hook (runs `vunexo secrets scan --staged`).",
                "✓".if_supports_color(Stdout, |t| t.green().to_string())
            );
            Ok(ExitAction::Code(0))
        }
        HooksCommand::Uninstall => {
            installer.uninstall()?;
            println!(
                "{} Removed the pre-commit hook.",
                "✓".if_supports_color(Stdout, |t| t.green().to_string())
            );
            Ok(ExitAction::Code(0))
        }
    }
}

fn run_secrets(
    command: SecretsCommand,
    passphrase_file: &Option<PathBuf>,
    cwd: &Path,
    config_store: &TomlConfigStore,
    vault_store: &AgeVaultStore,
) -> Result<ExitAction, VaultError> {
    match command {
        SecretsCommand::Set { key_value } => {
            let environment = active_environment(config_store)?;
            let (key_str, inline_value) = match key_value.split_once('=') {
                Some((k, v)) => (k.to_string(), Some(v.to_string())),
                None => (key_value, None),
            };
            let key =
                SecretKey::new(key_str).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
            let value = match inline_value {
                Some(v) => SecretValue::new(v),
                None => {
                    let prompted = rpassword::prompt_password(format!("Value for {key}: "))?;
                    SecretValue::new(prompted)
                }
            };
            let passphrase = read_passphrase(passphrase_file, "Vault passphrase: ")?;
            application::secrets::set_secret(vault_store, &environment, &passphrase, key, value)?;
            Ok(ExitAction::Code(0))
        }
        SecretsCommand::Get { key } => {
            let environment = active_environment(config_store)?;
            let key = SecretKey::new(key).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
            let passphrase = read_passphrase(passphrase_file, "Vault passphrase: ")?;
            let value =
                application::secrets::get_secret(vault_store, &environment, &passphrase, &key)?;
            println!("{}", value.expose());
            Ok(ExitAction::Code(0))
        }
        SecretsCommand::List => {
            let environment = active_environment(config_store)?;
            let passphrase = read_passphrase(passphrase_file, "Vault passphrase: ")?;
            let keys = application::secrets::list_secrets(vault_store, &environment, &passphrase)?;
            for key in keys {
                println!("{key}");
            }
            Ok(ExitAction::Code(0))
        }
        SecretsCommand::Remove { key } => {
            let environment = active_environment(config_store)?;
            let key = SecretKey::new(key).map_err(|e| VaultError::InvalidInput(e.to_string()))?;
            let passphrase = read_passphrase(passphrase_file, "Vault passphrase: ")?;
            application::secrets::remove_secret(vault_store, &environment, &passphrase, &key)?;
            Ok(ExitAction::Code(0))
        }
        SecretsCommand::Run { command } => {
            let (program, args) = command.split_first().ok_or_else(|| {
                VaultError::InvalidInput("no command given after `--`".to_string())
            })?;
            let environment = active_environment(config_store)?;
            let passphrase = read_passphrase(passphrase_file, "Vault passphrase: ")?;
            let process_runner = StdProcessRunner;
            let status = application::run_with_secrets::run_with_secrets(
                vault_store,
                &process_runner,
                &environment,
                &passphrase,
                program,
                args,
            )?;
            Ok(ExitAction::ChildStatus(status))
        }
        SecretsCommand::Scan { staged, path } => {
            let scanner = PatternScanner::new();
            let findings = if staged {
                let git_diff = GitCli::new(cwd);
                application::scan::scan_staged(&git_diff, &scanner)?
            } else {
                application::scan::scan_path(&scanner, path)
            };
            for finding in &findings {
                println!(
                    "{}:{}: [{}] {}",
                    finding.path.display(),
                    finding.line,
                    finding
                        .rule_name
                        .if_supports_color(Stdout, |t| t.yellow().to_string()),
                    finding
                        .redacted_preview
                        .if_supports_color(Stdout, |t| t.dimmed().to_string())
                );
            }
            if findings.is_empty() {
                println!(
                    "{}",
                    "no findings, clean".if_supports_color(Stdout, |t| t.green().to_string())
                );
            } else {
                let count = findings.len();
                let noun = if count == 1 { "finding" } else { "findings" };
                println!(
                    "{}",
                    format!("{count} {noun}").if_supports_color(Stdout, |t| t.yellow().to_string())
                );
            }
            Ok(ExitAction::Code(if findings.is_empty() { 0 } else { 4 }))
        }
    }
}
