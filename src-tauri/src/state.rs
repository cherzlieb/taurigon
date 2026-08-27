//! # Anwendungs-State
//!
//! Zentraler, von Tauri verwalteter Zustand:
//! - Gecachte [`SystemInfo`] (async Mutex)
//! - SQLite-Verbindung (sync Mutex, da rusqlite synchron ist)

use std::sync::Mutex as StdMutex;

use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::system::inspector::{inspect_system, SystemInfo};

/// Der globale, von Tauri verwaltete Anwendungszustand.
pub struct AppState {
    /// Zuletzt ermittelte Systeminfo (async Mutex, da über await gehalten).
    info: Mutex<SystemInfo>,
    /// SQLite-Verbindung (sync Mutex – rusqlite-Operationen sind synchron
    /// und werden nie über einen await-Punkt hinweg gehalten).
    db: StdMutex<Connection>,
}

impl AppState {
    /// Erstellt den State: initiale Systeminspektion + DB-Initialisierung.
    ///
    /// # Panics
    /// Bricht ab, wenn die Datenbank nicht initialisiert werden kann – ohne
    /// sie ist die App nicht sinnvoll nutzbar.
    pub fn new() -> Self {
        let conn = crate::db::open_and_migrate()
            .expect("Datenbank konnte nicht initialisiert werden");

        Self {
            info: Mutex::new(inspect_system()),
            db: StdMutex::new(conn),
        }
    }

    /// Liefert eine Kopie der aktuellen Systeminfo.
    pub async fn system_info(&self) -> SystemInfo {
        self.info.lock().await.clone()
    }

    /// Führt eine erneute Systeminspektion durch und speichert das Ergebnis.
    pub async fn refresh_system_info(&self) -> SystemInfo {
        let fresh = inspect_system();
        let mut guard = self.info.lock().await;
        *guard = fresh.clone();
        fresh
    }

    /// Liefert einen gesperrten Zugriff auf die DB-Verbindung.
    ///
    /// **Wichtig:** Den Guard nicht über `.await`-Punkte hinweg halten.
    pub fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.db.lock().expect("DB-Mutex ist vergiftet")
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
