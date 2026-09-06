// One typed wrapper per Tauri command — src-tauri/src/main.rs's
// `invoke_handler` list is the source of truth for the set below.
import { callCommand } from "./client";
import type { AppInfo, EnvironmentInfo, OpenProjectResult, RecentProject, ScanFinding } from "./types";

// --- Open Project gate ---

export function listRecentProjects() {
  return callCommand<RecentProject[]>("list_recent_projects");
}

export function openProject(path: string) {
  return callCommand<OpenProjectResult>("open_project", { path });
}

export function initVault(passphrase: string) {
  return callCommand<void>("init_vault", { passphrase });
}

// --- Unlock gate ---

export function unlock(environment: string, passphrase: string) {
  return callCommand<void>("unlock", { environment, passphrase });
}

export function lock() {
  return callCommand<void>("lock");
}

// --- Environments ---

export function listEnvironments() {
  return callCommand<EnvironmentInfo[]>("list_environments");
}

// Named `setActiveEnvironment` here (not `useEnvironment`) purely to avoid
// tripping ESLint's `react-hooks/rules-of-hooks` on a plain function that
// happens to start with "use" — the backend command it calls is still named
// `use_environment` (see `src-tauri/src/commands.rs`), matching the CLI's
// `env use NAME`.
export function setActiveEnvironment(name: string) {
  return callCommand<void>("use_environment", { name });
}

// --- Secrets ---

export function listSecrets() {
  return callCommand<string[]>("list_secrets");
}

export function getSecret(key: string) {
  return callCommand<string>("get_secret", { key });
}

export function setSecret(key: string, value: string) {
  return callCommand<void>("set_secret", { key, value });
}

export function removeSecret(key: string) {
  return callCommand<void>("remove_secret", { key });
}

// --- Scan ---

export function isGitRepository() {
  return callCommand<boolean>("is_git_repository");
}

export function runScan(staged: boolean, path?: string) {
  return callCommand<ScanFinding[]>("run_scan", { staged, path: path ?? null });
}

// --- Hooks ---

export function hookStatus() {
  return callCommand<boolean>("hook_status");
}

export function installHook() {
  return callCommand<void>("install_hook");
}

export function uninstallHook() {
  return callCommand<void>("uninstall_hook");
}

// --- Settings / About ---

export function getAppInfo() {
  return callCommand<AppInfo>("get_app_info");
}
