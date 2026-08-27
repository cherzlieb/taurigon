//! # Datenbank-Modul
//!
//! Kapselt den SQLite-Zugang für persistente Metadaten (aktuell: Projekte).
//! Die Datenbank liegt XDG-konform unter `~/.local/share/taurigon/taurigon.db`.
//!
//! Wir nutzen `rusqlite` mit gebündeltem SQLite (Feature "bundled") – dadurch
//! gibt es keine System-Abhängigkeit zu einer libsqlite.

use std::path::PathBuf;

use rusqlite::Connection;

/// Öffnet die Datenbank und führt ausstehende Migrationen aus.
///
/// Legt bei Bedarf das übergeordnete Verzeichnis an.
///
/// # Errors
/// Gibt einen Fehlerstring zurück, wenn Verzeichnis/DB nicht erstellt oder
/// die Migration nicht ausgeführt werden konnte.
pub fn open_and_migrate() -> Result<Connection, String> {
    let path = db_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("DB-Verzeichnis konnte nicht angelegt werden: {e}"))?;
    }

    let conn = Connection::open(&path)
        .map_err(|e| format!("Datenbank konnte nicht geöffnet werden: {e}"))?;

    migrate(&conn)?;
    Ok(conn)
}

/// Führt das Schema-Setup aus (idempotent via `IF NOT EXISTS`).
fn migrate(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS projects (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT NOT NULL UNIQUE,
            domain        TEXT NOT NULL,
            project_type  TEXT NOT NULL,
            php_version   TEXT,
            path          TEXT NOT NULL,
            created_at    TEXT NOT NULL DEFAULT (datetime('now'))
        );",
    )
    .map_err(|e| format!("Migration fehlgeschlagen: {e}"))
}

/// Liefert den Pfad zur Datenbankdatei.
fn db_path() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("taurigon").join("taurigon.db"))
        .unwrap_or_else(|| PathBuf::from("./taurigon-data/taurigon.db"))
}
