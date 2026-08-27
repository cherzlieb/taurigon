//! # Tauri-Commands
//!
//! Die Brücke zwischen Frontend (JavaScript/TypeScript) und Backend (Rust).
//! Enthält nur dünne Wrapper; die Fachlogik liegt in den Modulen.

use tauri::State;

use crate::engine::{create_engine, NETWORK_NAME};
use crate::projects::model::Project;
use crate::projects::ProjectManager;
use crate::services::catalog::{ServiceInfoDto, ServiceKind};
use crate::services::{ServiceManager, ServiceStatusDto};
use crate::state::AppState;
use crate::system::inspector::SystemInfoDto;

// ============================================================================
//  System
// ============================================================================

/// Führt eine (erneute) Systeminspektion durch und liefert das Ergebnis.
#[tauri::command]
pub async fn cmd_inspect_system(state: State<'_, AppState>) -> Result<SystemInfoDto, String> {
    log::debug!("cmd_inspect_system aufgerufen");
    let info = state.refresh_system_info().await;
    Ok(info.into())
}

/// Stellt das gemeinsame Container-Netzwerk sicher.
#[tauri::command]
pub async fn cmd_ensure_network(state: State<'_, AppState>) -> Result<String, String> {
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    engine.ensure_network().await.map_err(|e| e.to_string())?;
    Ok(format!(
        "Netzwerk '{}' ist bereit (Engine: {})",
        NETWORK_NAME,
        engine.kind()
    ))
}

// ============================================================================
//  Dienste
// ============================================================================

/// Liefert die statische Liste aller verfügbaren Dienste (Metadaten).
#[tauri::command]
pub fn cmd_list_services() -> Vec<ServiceInfoDto> {
    ServiceKind::ALL.into_iter().map(Into::into).collect()
}

/// Liefert den Laufzeitstatus aller Dienste.
#[tauri::command]
pub async fn cmd_service_status_all(
    state: State<'_, AppState>,
) -> Result<Vec<ServiceStatusDto>, String> {
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    let manager = ServiceManager::new(engine.as_ref());
    manager.status_all().await.map_err(|e| e.to_string())
}

/// Startet einen Dienst.
#[tauri::command]
pub async fn cmd_start_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<(), String> {
    let kind = ServiceKind::from_id(&service_id)
        .ok_or_else(|| format!("Unbekannter Dienst: {service_id}"))?;
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    let manager = ServiceManager::new(engine.as_ref());
    manager.start(kind).await.map_err(|e| e.to_string())
}

/// Stoppt einen Dienst.
#[tauri::command]
pub async fn cmd_stop_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<(), String> {
    let kind = ServiceKind::from_id(&service_id)
        .ok_or_else(|| format!("Unbekannter Dienst: {service_id}"))?;
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    let manager = ServiceManager::new(engine.as_ref());
    manager.stop(kind).await.map_err(|e| e.to_string())
}

/// Startet einen Dienst neu.
#[tauri::command]
pub async fn cmd_restart_service(
    state: State<'_, AppState>,
    service_id: String,
) -> Result<(), String> {
    let kind = ServiceKind::from_id(&service_id)
        .ok_or_else(|| format!("Unbekannter Dienst: {service_id}"))?;
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    let manager = ServiceManager::new(engine.as_ref());
    manager.restart(kind).await.map_err(|e| e.to_string())
}

/// Entfernt den Container eines Dienstes (optional mit Daten).
#[tauri::command]
pub async fn cmd_remove_service(
    state: State<'_, AppState>,
    service_id: String,
    delete_data: bool,
) -> Result<(), String> {
    let kind = ServiceKind::from_id(&service_id)
        .ok_or_else(|| format!("Unbekannter Dienst: {service_id}"))?;
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    let manager = ServiceManager::new(engine.as_ref());
    if delete_data {
        manager.remove_with_data(kind).await.map_err(|e| e.to_string())
    } else {
        manager.remove(kind).await.map_err(|e| e.to_string())
    }
}

// ============================================================================
//  Projekte
// ============================================================================

/// Listet alle Projekte.
#[tauri::command]
pub fn cmd_list_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db();
    let manager = ProjectManager::new(&conn);
    manager.list()
}

/// Legt ein neues Projekt an.
///
/// # Arguments
/// * `name`         - Projektname (wird normalisiert/validiert).
/// * `project_type` - "php" oder "static".
/// * `php_version`  - z. B. "8.3" (nur bei PHP).
#[tauri::command]
pub fn cmd_create_project(
    state: State<'_, AppState>,
    name: String,
    project_type: String,
    php_version: Option<String>,
) -> Result<Project, String> {
    let conn = state.db();
    let manager = ProjectManager::new(&conn);
    manager.create(&name, &project_type, php_version)
}

/// Löscht ein Projekt (optional inkl. Dateien).
#[tauri::command]
pub fn cmd_delete_project(
    state: State<'_, AppState>,
    id: i64,
    delete_files: bool,
) -> Result<(), String> {
    let conn = state.db();
    let manager = ProjectManager::new(&conn);
    manager.delete(id, delete_files)
}

// ============================================================================
//  Editor
// ============================================================================

/// Öffnet einen Pfad im konfigurierten Editor.
///
/// Der `editor_command` ist ein Template: enthält er `{path}`, wird dieser
/// ersetzt; andernfalls wird der Pfad angehängt.
///
/// # Beispiele
/// - `"code {path}"`        → `code /pfad/zum/projekt`
/// - `"code"`               → `code /pfad/zum/projekt`
/// - `"kitty nvim {path}"`  → Terminal-Editor in kitty
///
/// # Hinweis
/// GUI-Editoren (code, subl, zed …) starten direkt. Terminal-Editoren
/// (vim/nano) benötigen einen Terminal-Wrapper (z. B. `kitty nvim {path}`)
/// oder später das integrierte Terminal.
#[tauri::command]
pub fn cmd_open_in_editor(path: String, editor_command: String) -> Result<(), String> {
    let cmd = editor_command.trim();
    if cmd.is_empty() {
        return Err("Kein Editor konfiguriert (siehe Einstellungen).".into());
    }

    let full = if cmd.contains("{path}") {
        cmd.replace("{path}", &path)
    } else {
        format!("{cmd} {path}")
    };

    let mut parts = full.split_whitespace();
    let program = parts.next().ok_or("Ungültiger Editor-Befehl")?;
    let args: Vec<&str> = parts.collect();

    std::process::Command::new(program)
        .args(&args)
        .spawn()
        .map_err(|e| format!("Editor konnte nicht gestartet werden: {e}"))?;

    Ok(())
}
