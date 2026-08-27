//! # Projekt-Datenmodell
//!
//! Definiert die Datenstruktur eines Projekts, wie sie in der Datenbank
//! gespeichert und ans Frontend serialisiert wird.

use serde::{Deserialize, Serialize};

/// Ein Web-Projekt, das über Taurigon verwaltet wird.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Eindeutige ID (Primärschlüssel).
    pub id: i64,
    /// Projektname (klein, alphanumerisch + Bindestrich), z. B. "myapp".
    pub name: String,
    /// Vollständige Domain, z. B. "myapp.localhost".
    pub domain: String,
    /// Projekttyp: "php" oder "static".
    pub project_type: String,
    /// PHP-Version (nur bei PHP-Projekten), z. B. "8.3".
    pub php_version: Option<String>,
    /// Absoluter Pfad zum Projektverzeichnis.
    pub path: String,
    /// Erstellungszeitpunkt (SQLite-datetime-String).
    pub created_at: String,
}
