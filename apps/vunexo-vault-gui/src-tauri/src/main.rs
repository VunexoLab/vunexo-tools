#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod dto;
mod error;
mod recent_projects;
mod state;

use state::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::list_recent_projects,
            commands::open_project,
            commands::init_vault,
            commands::unlock,
            commands::lock,
            commands::list_environments,
            commands::use_environment,
            commands::list_secrets,
            commands::get_secret,
            commands::set_secret,
            commands::remove_secret,
            commands::is_git_repository,
            commands::run_scan,
            commands::hook_status,
            commands::install_hook,
            commands::uninstall_hook,
            commands::get_app_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running vunexo-vault-gui");
}
