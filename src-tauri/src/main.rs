//! # Taurigon – Anwendungs-Einstiegspunkt

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod engine;
mod projects;
mod services;
mod state;
mod system;
mod terminal;
mod web;

use state::AppState;
use terminal::TerminalManager;

/// Anwendungs-Einstiegspunkt.
fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info"),
    )
    .init();

    log::info!("Taurigon startet …");

    tauri::Builder::default()
        .manage(AppState::new())
        .manage(TerminalManager::default())
        .invoke_handler(tauri::generate_handler![
            commands::cmd_inspect_system,
            commands::cmd_ensure_network,
            commands::cmd_list_services,
            commands::cmd_service_status_all,
            commands::cmd_start_service,
            commands::cmd_stop_service,
            commands::cmd_restart_service,
            commands::cmd_remove_service,
            commands::cmd_list_projects,
            commands::cmd_create_project,
            commands::cmd_delete_project,
            commands::cmd_open_in_editor,
            commands::cmd_start_web,
            commands::cmd_stop_web,
            commands::cmd_web_status,
            commands::cmd_reload_web,
            commands::cmd_open_url,
            commands::cmd_proxy_port,
            commands::cmd_terminal_open,
            commands::cmd_terminal_write,
            commands::cmd_terminal_resize,
            commands::cmd_terminal_close,
        ])
        .run(tauri::generate_context!())
        .expect("Fataler Fehler beim Starten von Taurigon");
}
