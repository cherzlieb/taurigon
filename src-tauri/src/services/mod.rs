//! # Services-Modul
//!
//! Der [`ServiceManager`] verbindet den [`crate::engine`]-Layer mit dem
//! [`catalog`]: Er startet, stoppt und überwacht die vordefinierten Dienste
//! (MariaDB, PostgreSQL, Redis).
//!
//! ## Ablauf beim Starten (Ansatz A – synchron)
//! 1. Netzwerk sicherstellen
//! 2. Image lokal vorhanden? Falls nein → pullen
//! 3. Persistenz-Verzeichnis anlegen
//! 4. Container-Status prüfen:
//!    - läuft bereits → nichts tun
//!    - existiert, gestoppt → `start`
//!    - existiert nicht → `run` (neu erstellen)

pub mod catalog;

use std::path::PathBuf;

use serde::Serialize;

use crate::engine::{ContainerEngine, ContainerState, EngineResult};
use catalog::ServiceKind;

/// Laufzeitstatus eines Dienstes – wird ans Frontend geliefert.
#[derive(Debug, Serialize)]
pub struct ServiceStatusDto {
    /// Dienst-Kennung (z. B. "mariadb").
    pub id: String,
    /// "running" | "stopped" | "not_found"
    pub state: String,
    /// Host-Port des Dienstes.
    pub host_port: u16,
}

/// Verwaltet die vordefinierten Dienste über eine Container-Engine.
///
/// Hält eine Referenz auf die (boxed) Engine und das Datenwurzel-Verzeichnis
/// für Persistenz.
pub struct ServiceManager<'a> {
    /// Die zu verwendende Container-Engine (Podman/Docker).
    engine: &'a dyn ContainerEngine,
    /// Basisverzeichnis für persistente Daten.
    data_root: PathBuf,
}

impl<'a> ServiceManager<'a> {
    /// Erstellt einen ServiceManager.
    ///
    /// # Arguments
    /// * `engine` - Referenz auf die aktive Container-Engine.
    pub fn new(engine: &'a dyn ContainerEngine) -> Self {
        Self {
            engine,
            data_root: default_data_root(),
        }
    }

    /// Startet einen Dienst (Ansatz A: pullt Image bei Bedarf synchron).
    ///
    /// Idempotent: Ein bereits laufender Dienst führt zu keinem Fehler.
    ///
    /// # Arguments
    /// * `kind` - Welcher Dienst gestartet werden soll.
    ///
    /// # Errors
    /// Gibt Engine-Fehler weiter (z. B. Pull fehlgeschlagen, Port belegt).
    pub async fn start(&self, kind: ServiceKind) -> EngineResult<()> {
        // 1. Netzwerk sicherstellen.
        self.engine.ensure_network().await?;

        let name = kind.container_name();

        // 2. Aktuellen Status prüfen.
        let status = self.engine.container_status(&name).await?;

        match status.state {
            // Läuft schon → nichts zu tun.
            ContainerState::Running => Ok(()),

            // Existiert, ist gestoppt → einfach starten.
            ContainerState::Stopped => self.engine.start_container(&name).await,

            // Existiert nicht → neu erstellen.
            ContainerState::NotFound => {
                // 2a. Image vorhanden? Sonst pullen.
                let image = kind.image();
                if !self.engine.image_exists(image).await? {
                    log::info!("Pulle Image {image} …");
                    self.engine.pull_image(image).await?;
                }

                // 2b. Persistenz-Verzeichnis anlegen.
                let volume_dir = self.data_root.join(kind.id());
                if let Err(e) = std::fs::create_dir_all(&volume_dir) {
                    log::warn!("Konnte Volume-Verzeichnis nicht anlegen: {e}");
                }

                // 2c. Container erstellen und starten.
                let spec = kind.build_spec(&self.data_root);
                self.engine.run_container(&spec).await.map(|_id| ())
            }
        }
    }

    /// Stoppt einen laufenden Dienst.
    ///
    /// Idempotent: Ist der Dienst nicht vorhanden/schon gestoppt, wird kein
    /// harter Fehler erzeugt.
    pub async fn stop(&self, kind: ServiceKind) -> EngineResult<()> {
        let name = kind.container_name();
        let status = self.engine.container_status(&name).await?;

        match status.state {
            ContainerState::Running => self.engine.stop_container(&name).await,
            // Bereits gestoppt oder nicht vorhanden → nichts zu tun.
            _ => Ok(()),
        }
    }

    /// Startet einen Dienst neu.
    pub async fn restart(&self, kind: ServiceKind) -> EngineResult<()> {
        let name = kind.container_name();
        self.engine.restart_container(&name).await
    }

    /// Entfernt den Container eines Dienstes.
    ///
    /// Die persistenten Daten (Volume-Verzeichnis) bleiben erhalten, sodass ein
    /// erneuter Start den vorherigen Zustand wiederherstellt.
    ///
    /// # Arguments
    /// * `kind` - Welcher Dienst entfernt werden soll.
    ///
    /// Idempotent: Ein nicht existierender Container erzeugt keinen Fehler.
    pub async fn remove(&self, kind: ServiceKind) -> EngineResult<()> {
        let name = kind.container_name();
        let status = self.engine.container_status(&name).await?;

        // Nur entfernen, wenn er überhaupt existiert.
        if status.state == ContainerState::NotFound {
            return Ok(());
        }

        // force=true: entfernt auch einen laufenden Container in einem Schritt.
        self.engine.remove_container(&name, true).await
    }

    /// Entfernt Container **und** die persistenten Daten eines Dienstes.
    ///
    /// Achtung: Löscht das Volume-Verzeichnis unwiderruflich!
    ///
    /// # Arguments
    /// * `kind` - Welcher Dienst vollständig entfernt werden soll.
    pub async fn remove_with_data(&self, kind: ServiceKind) -> EngineResult<()> {
        // Erst Container weg.
        self.remove(kind).await?;

        // Dann Datenverzeichnis löschen.
        let volume_dir = self.data_root.join(kind.id());
        if volume_dir.exists() {
            if let Err(e) = std::fs::remove_dir_all(&volume_dir) {
                log::warn!("Konnte Datenverzeichnis nicht löschen: {e}");
            }
        }

        Ok(())
    }

    /// Ermittelt den Status eines einzelnen Dienstes.
    pub async fn status(&self, kind: ServiceKind) -> EngineResult<ServiceStatusDto> {
        let name = kind.container_name();
        let status = self.engine.container_status(&name).await?;
        Ok(ServiceStatusDto {
            id: kind.id().to_string(),
            state: state_to_str(status.state).to_string(),
            host_port: kind.host_port(),
        })
    }

    /// Ermittelt den Status **aller** bekannten Dienste.
    ///
    /// Praktisch für das Dashboard, das alle Dienst-Kacheln auf einmal füllt.
    pub async fn status_all(&self) -> EngineResult<Vec<ServiceStatusDto>> {
        let mut result = Vec::with_capacity(ServiceKind::ALL.len());
        for kind in ServiceKind::ALL {
            result.push(self.status(kind).await?);
        }
        Ok(result)
    }
}

/// Übersetzt [`ContainerState`] in die Frontend-Stringrepräsentation.
fn state_to_str(state: ContainerState) -> &'static str {
    match state {
        ContainerState::Running => "running",
        ContainerState::Stopped => "stopped",
        ContainerState::NotFound => "not_found",
    }
}

/// Liefert das Datenwurzel-Verzeichnis: `<data_dir>/taurigon/volumes`.
///
/// Fällt bei nicht ermittelbarem Home-Verzeichnis auf `./taurigon-data` zurück.
fn default_data_root() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("taurigon").join("volumes"))
        .unwrap_or_else(|| PathBuf::from("./taurigon-data/volumes"))
}
