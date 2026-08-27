//! # Dienst-Katalog
//!
//! Definiert die von Taurigon verwaltbaren Dienste als feste Spezifikationen:
//! Image, Ports, Volumes, Umgebungsvariablen.
//!
//! In diesem ersten Schritt enthalten wir die self-contained Datendienste
//! (MariaDB, PostgreSQL, Redis). Nginx-Proxy und PHP-FPM folgen mit dem
//! vHost-Modul, da sie generierte Konfigurationsdateien benötigen.

use std::path::PathBuf;

use serde::Serialize;

use crate::engine::{ContainerSpec, RestartPolicy};

/// Präfix für alle Container-Namen, damit sie eindeutig Taurigon zuzuordnen sind.
pub const CONTAINER_PREFIX: &str = "taurigon-";

/// Die verfügbaren Dienst-Typen.
///
/// `Serialize` + `Deserialize`-freundlich als String, damit das Frontend
/// Dienste per Kennung (z. B. "mariadb") ansprechen kann.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceKind {
    /// MariaDB (MySQL-kompatibel).
    MariaDB,
    /// PostgreSQL.
    Postgres,
    /// Redis (In-Memory-Store).
    Redis,
}

impl ServiceKind {
    /// Alle bekannten Dienste – nützlich zum Auflisten in der UI.
    pub const ALL: [ServiceKind; 3] = [
        ServiceKind::MariaDB,
        ServiceKind::Postgres,
        ServiceKind::Redis,
    ];

    /// Stabile String-Kennung (für Frontend ↔ Backend-Kommunikation).
    pub fn id(&self) -> &'static str {
        match self {
            ServiceKind::MariaDB => "mariadb",
            ServiceKind::Postgres => "postgres",
            ServiceKind::Redis => "redis",
        }
    }

    /// Parst eine String-Kennung zurück in einen [`ServiceKind`].
    ///
    /// # Returns
    /// `Some(kind)` bei bekannter Kennung, sonst `None`.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "mariadb" => Some(ServiceKind::MariaDB),
            "postgres" => Some(ServiceKind::Postgres),
            "redis" => Some(ServiceKind::Redis),
            _ => None,
        }
    }

    /// Menschlich lesbarer Anzeigename.
    pub fn display_name(&self) -> &'static str {
        match self {
            ServiceKind::MariaDB => "MariaDB",
            ServiceKind::Postgres => "PostgreSQL",
            ServiceKind::Redis => "Redis",
        }
    }

    /// Der vollständige Container-Name (mit Präfix).
    pub fn container_name(&self) -> String {
        format!("{CONTAINER_PREFIX}{}", self.id())
    }

    /// Das zu verwendende Container-Image.
    pub fn image(&self) -> &'static str {
        match self {
            ServiceKind::MariaDB => "docker.io/library/mariadb:11",
            ServiceKind::Postgres => "docker.io/library/postgres:16",
            ServiceKind::Redis => "docker.io/library/redis:7-alpine",
        }
    }

    /// Der Host-Port (im Bereich 8000–8999 gemäß Konzept).
    pub fn host_port(&self) -> u16 {
        match self {
            ServiceKind::MariaDB => 8306,
            ServiceKind::Postgres => 8432,
            ServiceKind::Redis => 8379,
        }
    }

    /// Der Port im Container (Standard-Port des jeweiligen Dienstes).
    pub fn container_port(&self) -> u16 {
        match self {
            ServiceKind::MariaDB => 3306,
            ServiceKind::Postgres => 5432,
            ServiceKind::Redis => 6379,
        }
    }

    /// Baut die vollständige [`ContainerSpec`] für diesen Dienst.
    ///
    /// # Arguments
    /// * `data_root` - Basisverzeichnis für persistente Daten
    ///   (typischerweise `~/.local/share/taurigon/volumes`).
    ///
    /// # Returns
    /// Eine startfertige Spezifikation inkl. Port, Volume und Standard-Env.
    pub fn build_spec(&self, data_root: &PathBuf) -> ContainerSpec {
        // Persistenz-Verzeichnis pro Dienst: <data_root>/<id>
        let host_volume = data_root.join(self.id());
        let host_volume_str = host_volume.to_string_lossy().to_string();

        let base = ContainerSpec::new(self.container_name(), self.image())
            .with_port(self.host_port(), self.container_port())
            .with_restart(RestartPolicy::UnlessStopped);

        match self {
            ServiceKind::MariaDB => base
                // Root-Passwort für lokale Entwicklung. (Bewusst simpel.)
                .with_env("MARIADB_ROOT_PASSWORD", "root")
                .with_volume(host_volume_str, "/var/lib/mysql", true),

            ServiceKind::Postgres => base
                .with_env("POSTGRES_PASSWORD", "postgres")
                .with_volume(host_volume_str, "/var/lib/postgresql/data", true),

            ServiceKind::Redis => base
                // Redis persistiert nach /data (AOF/RDB).
                .with_volume(host_volume_str, "/data", true),
        }
    }
}

/// Frontend-freundliche Beschreibung eines Dienstes (statische Metadaten).
///
/// Enthält **keinen** Laufzeitstatus – der kommt separat vom ServiceManager.
#[derive(Debug, Serialize)]
pub struct ServiceInfoDto {
    /// Stabile Kennung (z. B. "mariadb").
    pub id: String,
    /// Anzeigename (z. B. "MariaDB").
    pub name: String,
    /// Verwendetes Image.
    pub image: String,
    /// Host-Port.
    pub host_port: u16,
}

impl From<ServiceKind> for ServiceInfoDto {
    fn from(kind: ServiceKind) -> Self {
        ServiceInfoDto {
            id: kind.id().to_string(),
            name: kind.display_name().to_string(),
            image: kind.image().to_string(),
            host_port: kind.host_port(),
        }
    }
}
