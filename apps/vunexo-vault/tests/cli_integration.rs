//! End-to-end integration tests that exercise the actual compiled `vunexo`
//! binary against real temp-directory `.vunexo/` state, using real `age`
//! encryption throughout (no mocked crypto), per
//! `docs/vunexo-vault/application-architecture.md`'s verification guide.
//!
//! These spawn the real binary (via `CARGO_BIN_EXE_vunexo`) rather than
//! calling internal Rust functions directly: the locked contract this
//! project has to honor — exact exit codes, stdout/stderr shape, hook file
//! bytes — is a property of the shipped CLI artifact, not of any one
//! internal API, and neither sibling app in this monorepo carries a
//! separate `tests/`-with-library-crate setup, so this keeps the same
//! "no `src/lib.rs` just for testing" shape. Passphrases are always supplied
//! via `--passphrase-file` and secret values via the `KEY=VALUE` inline
//! form, since `rpassword`'s masked prompts read from the controlling TTY
//! directly (not stdin), which a non-interactive test process doesn't have.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn vunexo_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vunexo")
}

fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(vunexo_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to spawn vunexo")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

/// Writes a passphrase file and returns its path.
fn passphrase_file(dir: &Path, name: &str, passphrase: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, passphrase).unwrap();
    path
}

fn init(dir: &Path, passphrase_path: &Path) -> Output {
    run(
        dir,
        &[
            "--passphrase-file",
            passphrase_path.to_str().unwrap(),
            "init",
        ],
    )
}

// --- init -> set -> get round trip ---

#[test]
fn init_set_get_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "correct horse battery staple");

    let init_output = init(dir.path(), &pass);
    assert!(
        init_output.status.success(),
        "init failed: {}",
        stderr(&init_output)
    );
    assert!(dir.path().join(".vunexo/config.toml").is_file());
    assert!(dir.path().join(".vunexo/vault/development.age").is_file());

    let set_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "set",
            "API_KEY=sk_super_secret_value",
        ],
    );
    assert!(
        set_output.status.success(),
        "set failed: {}",
        stderr(&set_output)
    );

    let get_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "get",
            "API_KEY",
        ],
    );
    assert!(
        get_output.status.success(),
        "get failed: {}",
        stderr(&get_output)
    );
    assert_eq!(stdout(&get_output).trim_end(), "sk_super_secret_value");

    // The on-disk vault file must never contain the plaintext value.
    let ciphertext = fs::read(dir.path().join(".vunexo/vault/development.age")).unwrap();
    assert!(!ciphertext
        .windows(b"sk_super_secret_value".len())
        .any(|w| w == b"sk_super_secret_value"));
}

#[test]
fn init_refuses_when_vunexo_dir_already_exists() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "hunter2-hunter2-hunter2");
    assert!(init(dir.path(), &pass).status.success());

    let second = init(dir.path(), &pass);
    assert_eq!(second.status.code(), Some(1));
    assert!(stderr(&second).starts_with("error: "));
}

// --- wrong passphrase failure ---

#[test]
fn wrong_passphrase_on_get_is_exit_code_2_with_generic_message() {
    let dir = tempfile::tempdir().unwrap();
    let right_pass = passphrase_file(dir.path(), "right.txt", "the-real-passphrase-123");
    let wrong_pass = passphrase_file(dir.path(), "wrong.txt", "not-the-right-one-456");

    assert!(init(dir.path(), &right_pass).status.success());
    let set_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            right_pass.to_str().unwrap(),
            "secrets",
            "set",
            "TOKEN=abc123xyz",
        ],
    );
    assert!(set_output.status.success());

    let get_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            wrong_pass.to_str().unwrap(),
            "secrets",
            "get",
            "TOKEN",
        ],
    );
    assert_eq!(get_output.status.code(), Some(2));
    assert_eq!(
        stderr(&get_output).trim_end(),
        "error: incorrect passphrase"
    );
    assert!(stdout(&get_output).is_empty());
}

// --- secrets list / remove ---

#[test]
fn list_prints_sorted_key_names_only_never_values() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "list-test-passphrase");
    assert!(init(dir.path(), &pass).status.success());

    for kv in ["ZEBRA=zzz-secret", "ALPHA=aaa-secret"] {
        let out = run(
            dir.path(),
            &[
                "--passphrase-file",
                pass.to_str().unwrap(),
                "secrets",
                "set",
                kv,
            ],
        );
        assert!(out.status.success());
    }

    let list_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "list",
        ],
    );
    assert!(list_output.status.success());
    let list_text = stdout(&list_output);
    let lines: Vec<&str> = list_text.lines().collect();
    assert_eq!(lines, vec!["ALPHA", "ZEBRA"]);
    assert!(!stdout(&list_output).contains("secret"));
}

#[test]
fn remove_missing_key_is_exit_code_3_not_silently_ok() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "remove-test-passphrase");
    assert!(init(dir.path(), &pass).status.success());
    assert!(run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "set",
            "KEEP=v"
        ],
    )
    .status
    .success());

    let remove_typo = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "remove",
            "KEPT",
        ],
    );
    assert_eq!(remove_typo.status.code(), Some(3));

    let remove_real = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "remove",
            "KEEP",
        ],
    );
    assert!(remove_real.status.success());

    let get_after_remove = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "get",
            "KEEP",
        ],
    );
    assert_eq!(get_after_remove.status.code(), Some(3));
}

// --- env list / use, including the lazily-created vault file ---

#[test]
fn env_use_creates_environment_lazily_no_vault_file_until_first_set() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "env-test-passphrase");
    assert!(init(dir.path(), &pass).status.success());

    let list1 = run(dir.path(), &["env", "list"]);
    assert!(list1.status.success());
    assert_eq!(stdout(&list1).trim_end(), "* development");

    // `env use` on a brand-new name must not prompt for a passphrase (no
    // --passphrase-file given here) and must not create a vault file yet.
    let use_output = run(dir.path(), &["env", "use", "staging"]);
    assert!(use_output.status.success(), "{}", stderr(&use_output));
    assert!(!dir.path().join(".vunexo/vault/staging.age").exists());

    let list2 = run(dir.path(), &["env", "list"]);
    let list2_text = stdout(&list2);
    let mut lines: Vec<&str> = list2_text.lines().collect();
    lines.sort();
    assert_eq!(lines, vec!["  development", "* staging"]);

    // The first `secrets set` against the new active environment creates
    // its vault file lazily.
    let set_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "set",
            "K=v",
        ],
    );
    assert!(set_output.status.success());
    assert!(dir.path().join(".vunexo/vault/staging.age").is_file());

    // development's vault is untouched by staging's secret.
    let use_dev = run(dir.path(), &["env", "use", "development"]);
    assert!(use_dev.status.success());
    let get_in_dev = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "get",
            "K",
        ],
    );
    assert_eq!(get_in_dev.status.code(), Some(3));
}

// --- `secrets run` injects env vars into a real spawned child process ---

#[cfg(unix)]
#[test]
fn run_injects_secrets_into_real_child_process_env() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "run-test-passphrase");
    assert!(init(dir.path(), &pass).status.success());
    assert!(run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "set",
            "GREETING=hello-from-the-vault",
        ],
    )
    .status
    .success());

    let run_output = run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "run",
            "--",
            "sh",
            "-c",
            "printf '%s' \"$GREETING\"",
        ],
    );
    assert!(run_output.status.success(), "{}", stderr(&run_output));
    assert_eq!(stdout(&run_output), "hello-from-the-vault");
}

#[cfg(unix)]
#[test]
fn run_preserves_the_parent_environment_and_exits_with_the_childs_code() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "run-test-passphrase-2");
    assert!(init(dir.path(), &pass).status.success());

    let run_output = Command::new(vunexo_bin())
        .current_dir(dir.path())
        .env("PARENT_ONLY_VAR", "still-here")
        .args([
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "run",
            "--",
            "sh",
            "-c",
            "printf '%s' \"$PARENT_ONLY_VAR\"; exit 7",
        ])
        .output()
        .unwrap();
    assert_eq!(stdout(&run_output), "still-here");
    assert_eq!(run_output.status.code(), Some(7));
}

// --- git hooks install/uninstall ---

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn hooks_install_then_uninstall_on_fresh_repo_creates_then_removes_the_file() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "--quiet"]);

    let install_output = run(dir.path(), &["hooks", "install"]);
    assert!(
        install_output.status.success(),
        "{}",
        stderr(&install_output)
    );
    let hook_path = dir.path().join(".git/hooks/pre-commit");
    assert!(hook_path.is_file());
    let content = fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("# >>> vunexo-vault >>>"));
    assert!(content.contains("vunexo secrets scan --staged"));
    assert!(content.contains("# <<< vunexo-vault <<<"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&hook_path).unwrap().permissions().mode();
        assert!(mode & 0o111 != 0, "hook should be executable");
    }

    // Installing again is an idempotent no-op, not an error.
    let reinstall_output = run(dir.path(), &["hooks", "install"]);
    assert!(reinstall_output.status.success());

    let uninstall_output = run(dir.path(), &["hooks", "uninstall"]);
    assert!(uninstall_output.status.success());
    assert!(!hook_path.exists(), "hook file should be deleted entirely");
}

#[test]
fn hooks_install_refuses_a_foreign_hook_and_never_touches_it() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "--quiet"]);

    let hooks_dir = dir.path().join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    let hook_path = hooks_dir.join("pre-commit");
    let original = "#!/bin/bash\necho 'a completely unrelated pre-existing hook'\nexit 0\n";
    fs::write(&hook_path, original).unwrap();
    let original_bytes = fs::read(&hook_path).unwrap();

    let install_output = run(dir.path(), &["hooks", "install"]);
    assert_eq!(install_output.status.code(), Some(1));
    assert_eq!(
        fs::read(&hook_path).unwrap(),
        original_bytes,
        "a foreign hook must never be modified by install"
    );

    // Uninstall on a hook that isn't ours must also be a no-op, leaving the
    // file byte-for-byte identical to what it was before `install` ever ran.
    let uninstall_output = run(dir.path(), &["hooks", "uninstall"]);
    assert!(uninstall_output.status.success());
    assert_eq!(
        fs::read(&hook_path).unwrap(),
        original_bytes,
        "install-then-uninstall must restore a pre-existing unrelated hook byte-for-byte"
    );
}

#[test]
fn hooks_install_requires_a_git_repository() {
    let dir = tempfile::tempdir().unwrap();
    let output = run(dir.path(), &["hooks", "install"]);
    assert_eq!(output.status.code(), Some(1));
}

// --- secrets scan / scan --staged, including exit code 4 ---

#[test]
fn scan_exits_4_when_a_finding_is_present_and_0_otherwise() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("clean.env"), "APP_NAME=my-app\nPORT=3000\n").unwrap();
    let clean_scan = run(dir.path(), &["secrets", "scan"]);
    assert_eq!(clean_scan.status.code(), Some(0));
    // No per-finding lines; just the "clean" summary (terminal-output polish
    // added a summary line to every scan, colored when stdout is a TTY —
    // tests run with piped, non-TTY stdout, so it's always the plain form).
    assert_eq!(stdout(&clean_scan).trim(), "no findings, clean");

    fs::write(
        dir.path().join("leaky.env"),
        "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n",
    )
    .unwrap();
    let dirty_scan = run(dir.path(), &["secrets", "scan"]);
    assert_eq!(dirty_scan.status.code(), Some(4));
    let out = stdout(&dirty_scan);
    assert!(out.contains("leaky.env"));
    assert!(out.contains("aws_access_key_id"));
    // The finding must never contain the full secret value.
    assert!(!out.contains("AKIAIOSFODNN7EXAMPLE"));
}

#[test]
fn scan_staged_only_looks_at_staged_files() {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "--quiet"]);
    git(dir.path(), &["config", "user.email", "test@example.com"]);
    git(dir.path(), &["config", "user.name", "Test"]);

    fs::write(
        dir.path().join("unstaged_leak.env"),
        "GITHUB_TOKEN=ghp_16C7e42F292c6912E7710c838347Ae178B4a\n",
    )
    .unwrap();
    fs::write(dir.path().join("staged_clean.env"), "APP_NAME=my-app\n").unwrap();
    git(dir.path(), &["add", "staged_clean.env"]);

    let scan_output = run(dir.path(), &["secrets", "scan", "--staged"]);
    assert_eq!(
        scan_output.status.code(),
        Some(0),
        "only the clean file is staged: {}",
        stdout(&scan_output)
    );

    git(dir.path(), &["add", "unstaged_leak.env"]);
    let scan_output_2 = run(dir.path(), &["secrets", "scan", "--staged"]);
    assert_eq!(scan_output_2.status.code(), Some(4));
    assert!(stdout(&scan_output_2).contains("unstaged_leak.env"));
}

#[test]
fn scan_never_flags_the_vault_itself() {
    let dir = tempfile::tempdir().unwrap();
    let pass = passphrase_file(dir.path(), "pass.txt", "scan-vs-vault-passphrase");
    assert!(init(dir.path(), &pass).status.success());
    assert!(run(
        dir.path(),
        &[
            "--passphrase-file",
            pass.to_str().unwrap(),
            "secrets",
            "set",
            "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE",
        ],
    )
    .status
    .success());

    // The vault's own ciphertext is high-entropy by construction and lives
    // right next to config.toml; scanning the whole directory must not
    // flag it.
    let scan_output = run(dir.path(), &["secrets", "scan"]);
    assert_eq!(
        scan_output.status.code(),
        Some(0),
        "{}",
        stdout(&scan_output)
    );
}

// --- Round 6 scanner test vectors, exercised against real fixture files ---
// (Line-level unit tests already cover these in
// `src/infrastructure/pattern_scanner.rs`; this additionally exercises the
// full file-walking `scan` path named in `application-architecture.md`'s
// verification guide.)

#[test]
fn scan_fixture_file_with_only_true_positive_vectors_is_flagged() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = concat!(
        "AKIAIOSFODNN7EXAMPLE\n",
        "ghp_16C7e42F292c6912E7710c838347Ae178B4a\n",
        "sk_test_FAKEFAKEFAKEFAKE1234\n",
        "-----BEGIN RSA PRIVATE KEY-----\n",
        "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dQw4w9WgXcQ\n",
        "AWS_SECRET_ACCESS_KEY = \"wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY\"\n",
    );
    fs::write(dir.path().join("secrets_fixture.txt"), fixture).unwrap();

    let output = run(dir.path(), &["secrets", "scan"]);
    assert_eq!(output.status.code(), Some(4));
    let text = stdout(&output);
    for rule in [
        "aws_access_key_id",
        "github_token",
        "stripe_key",
        "private_key_pem",
        "jwt",
        "generic_assignment",
    ] {
        assert!(text.contains(rule), "expected rule `{rule}` in:\n{text}");
    }
}

#[test]
fn scan_fixture_file_with_only_true_negative_vectors_is_clean() {
    let dir = tempfile::tempdir().unwrap();
    let fixture = concat!(
        "API_KEY = \"your-api-key-here\"\n",
        "password = \"changeme\"\n",
        "This is just a normal sentence used only for testing purposes.\n",
        "request_id = \"550e8400-e29b-41d4-a716-446655440000\"\n",
        "logo = \"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB\"\n",
    );
    fs::write(dir.path().join("placeholders_fixture.txt"), fixture).unwrap();

    let output = run(dir.path(), &["secrets", "scan"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "unexpected findings:\n{}",
        stdout(&output)
    );
}
