//! # Web-Layer
//!
//! Der [`WebManager`] verwaltet die Web-Infrastruktur:
//! - Nginx-Reverse-Proxy (Port 8080)
//! - PHP-FPM-Container pro Version (geteilt)
//! - vHost-Konfigurationen pro Projekt
//!
//! ## Ablauf beim Start
//! 1. vHost-Configs für alle Projekte schreiben
//! 2. Benötigte PHP-FPM-Container sicherstellen
//! 3. Proxy sicherstellen (lädt Configs beim Start)
//! 4. Falls Proxy bereits lief: `nginx -s reload`

pub mod catalog;
pub mod vhost;

use std::collections::HashSet;
use std::path::PathBuf;

use crate::engine::{ContainerEngine, ContainerSpec, ContainerState, EngineResult, RestartPolicy};
use crate::projects::model::Project;
use catalog::{
    php_container_name, php_image, PHP_VERSIONS, PROXY_HTTP_PORT, PROXY_IMAGE, PROXY_NAME,
};

/// Default-Server, der unbekannte Domains sauber ablehnt (statt auf den
/// ersten Projekt-vHost zurückzufallen).
const DEFAULT_VHOST: &str = r#"server {
    listen 80 default_server;
    server_name _;
    return 404 "Taurigon: Keine Website fuer diese Domain.\n";
    default_type text/plain;
}
"#;

/// Verwaltet die Web-Infrastruktur über eine Container-Engine.
pub struct WebManager<'a> {
    engine: &'a dyn ContainerEngine,
    /// Host-Verzeichnis mit den Projektdateien.
    projects_root: PathBuf,
    /// Host-Verzeichnis für generierte Nginx-Configs.
    nginx_conf_dir: PathBuf,
}

impl<'a> WebManager<'a> {
    /// Erstellt einen WebManager mit Standard-Verzeichnissen.
    pub fn new(engine: &'a dyn ContainerEngine) -> Self {
        Self {
            engine,
            projects_root: default_projects_root(),
            nginx_conf_dir: default_nginx_conf_dir(),
        }
    }

    /// Startet die Web-Umgebung für die gegebenen Projekte.
    ///
    /// Idempotent: Bereits laufende Container werden nicht neu gestartet.
    pub async fn start(&self, projects: &[Project]) -> EngineResult<()> {
        self.engine.ensure_network().await?;

        // 1. Config-Verzeichnis anlegen + vHosts schreiben.
        self.ensure_dirs();
        self.write_all_vhosts(projects)?;

        // 2. Benötigte PHP-Versionen ermitteln und starten.
        let needed = needed_php_versions(projects);
        for version in &needed {
            self.ensure_php(version).await?;
        }

        // 3. Lief der Proxy schon? (entscheidet über reload).
        let was_running = matches!(
            self.engine.container_status(PROXY_NAME).await?.state,
            ContainerState::Running
        );

        // 4. Proxy sicherstellen.
        self.ensure_proxy().await?;

        // 5. Bei bereits laufendem Proxy: Configs neu laden.
        if was_running {
            // Best effort – schlägt reload fehl, ist es nicht kritisch.
            let _ = self.engine.exec(PROXY_NAME, &["nginx", "-s", "reload"]).await;
        }

        Ok(())
    }

    /// Stoppt die Web-Umgebung (Proxy + alle PHP-FPM-Container).
    pub async fn stop(&self) -> EngineResult<()> {
        // Proxy stoppen.
        if matches!(
            self.engine.container_status(PROXY_NAME).await?.state,
            ContainerState::Running
        ) {
            self.engine.stop_container(PROXY_NAME).await?;
        }

        // Alle PHP-FPM-Container stoppen.
        for version in PHP_VERSIONS {
            let name = php_container_name(version);
            if matches!(
                self.engine.container_status(&name).await?.state,
                ContainerState::Running
            ) {
                self.engine.stop_container(&name).await?;
            }
        }

        Ok(())
    }

    /// Prüft, ob der Proxy läuft.
    pub async fn is_running(&self) -> EngineResult<bool> {
        Ok(matches!(
            self.engine.container_status(PROXY_NAME).await?.state,
            ContainerState::Running
        ))
    }


    /// Lädt die vHost-Konfiguration neu (ohne Container-Neustart).
    ///
    /// Schreibt alle vHosts neu, stellt benötigte PHP-FPM-Container sicher
    /// und führt `nginx -s reload` aus – aber **nur**, wenn der Proxy läuft.
    /// Läuft er nicht, passiert nichts (die Configs sind aber geschrieben).
    ///
    /// # Arguments
    /// * `projects` - Aktuelle Projektliste.
    pub async fn reload(&self, projects: &[Project]) -> EngineResult<()> {
        self.ensure_dirs();
        self.write_all_vhosts(projects)?;

        // Nur reloaden, wenn der Proxy läuft.
        if !self.is_running().await? {
            return Ok(());
        }

        // Neue PHP-Versionen könnten dazugekommen sein.
        for version in needed_php_versions(projects) {
            self.ensure_php(&version).await?;
        }

        self.engine
            .exec(PROXY_NAME, &["nginx", "-s", "reload"])
            .await
            .map(|_| ())
    }

    /// Entfernt die vHost-Config eines Projekts und lädt neu.
    ///
    /// # Arguments
    /// * `name`     - Projektname (Dateiname der Config).
    /// * `projects` - Verbleibende Projekte (für den Reload).
    pub async fn remove_vhost(
        &self,
        name: &str,
        projects: &[Project],
    ) -> EngineResult<()> {
        let file = self.nginx_conf_dir.join(format!("{name}.conf"));
        if file.exists() {
            let _ = std::fs::remove_file(&file);
        }

        // Reload, falls Proxy läuft (best effort).
        if self.is_running().await? {
            let _ = self
                .engine
                .exec(PROXY_NAME, &["nginx", "-s", "reload"])
                .await;
        }

        // projects wird aktuell nicht weiter gebraucht, aber für spätere
        // Erweiterungen (z. B. PHP-Container aufräumen) vorgehalten.
        let _ = projects;
        Ok(())
    }


    // ---- interne Helfer ----------------------------------------------------

    /// Stellt sicher, dass ein Container läuft (start/run bei Bedarf).
    async fn ensure_running(&self, spec: &ContainerSpec) -> EngineResult<()> {
        let status = self.engine.container_status(&spec.name).await?;
        match status.state {
            ContainerState::Running => Ok(()),
            ContainerState::Stopped => self.engine.start_container(&spec.name).await,
            ContainerState::NotFound => {
                if !self.engine.image_exists(&spec.image).await? {
                    log::info!("Pulle Image {} …", spec.image);
                    self.engine.pull_image(&spec.image).await?;
                }
                self.engine.run_container(spec).await.map(|_| ())
            }
        }
    }

    /// Stellt den Nginx-Proxy sicher.
    async fn ensure_proxy(&self) -> EngineResult<()> {
        let conf = self.nginx_conf_dir.to_string_lossy().to_string();
        let www = self.projects_root.to_string_lossy().to_string();

        let spec = ContainerSpec::new(PROXY_NAME, PROXY_IMAGE)
            .with_port(PROXY_HTTP_PORT, 80)
            // conf.d: nur der Proxy nutzt es → privat (:Z).
            .with_volume(conf, "/etc/nginx/conf.d", true)
            // Projektdateien: Proxy + PHP-FPM → geteilt (:z).
            .with_shared_volume(www, "/var/www")
            .with_restart(RestartPolicy::UnlessStopped);

        self.ensure_running(&spec).await
    }

    /// Stellt einen PHP-FPM-Container für eine Version sicher.
    async fn ensure_php(&self, version: &str) -> EngineResult<()> {
        let www = self.projects_root.to_string_lossy().to_string();

        let spec = ContainerSpec::new(php_container_name(version), php_image(version))
            // PHP-FPM braucht die Projektdateien (geteilt mit Proxy).
            .with_shared_volume(www, "/var/www")
            // Dev-Modus: OPcache soll Dateiänderungen sofort erkennen,
            // damit kein veralteter Code ausgeliefert wird.
            .with_env("PHP_OPCACHE_VALIDATE_TIMESTAMPS", "1")
            .with_env("PHP_OPCACHE_REVALIDATE_FREQ", "0")
            .with_restart(RestartPolicy::UnlessStopped);

        self.ensure_running(&spec).await
    }

    /// Legt die benötigten Host-Verzeichnisse an.
    fn ensure_dirs(&self) {
        if let Err(e) = std::fs::create_dir_all(&self.nginx_conf_dir) {
            log::warn!("nginx-conf-Verzeichnis konnte nicht angelegt werden: {e}");
        }
        if let Err(e) = std::fs::create_dir_all(&self.projects_root) {
            log::warn!("Projektverzeichnis konnte nicht angelegt werden: {e}");
        }
    }

    // Schreibt die vHost-Configs für alle Projekte (überschreibt bestehende).
    fn write_all_vhosts(&self, projects: &[Project]) -> EngineResult<()> {
        // Default-Server zuerst: fängt alle unbekannten Domains ab.
        let default_file = self.nginx_conf_dir.join("00-default.conf");
        if let Err(e) = std::fs::write(&default_file, DEFAULT_VHOST) {
            log::warn!("Default-vHost konnte nicht geschrieben werden: {e}");
        }

        for project in projects {
            let config = vhost::generate(
                &project.name,
                &project.domain,
                &project.project_type,
                project.php_version.as_deref(),
            );
            let file = self.nginx_conf_dir.join(format!("{}.conf", project.name));
            if let Err(e) = std::fs::write(&file, config) {
                log::warn!("vHost für '{}' konnte nicht geschrieben werden: {e}", project.name);
            }
        }
        Ok(())
    }
}

/// Ermittelt die von PHP-Projekten benötigten PHP-Versionen (dedupliziert).
fn needed_php_versions(projects: &[Project]) -> Vec<String> {
    let mut set = HashSet::new();
    for p in projects {
        if p.project_type == "php" {
            if let Some(v) = &p.php_version {
                set.insert(v.clone());
            }
        }
    }
    set.into_iter().collect()
}

/// Basisverzeichnis der Projektdateien.
fn default_projects_root() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("taurigon").join("projects"))
        .unwrap_or_else(|| PathBuf::from("./taurigon-data/projects"))
}

/// Verzeichnis für generierte Nginx-Configs.
fn default_nginx_conf_dir() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("taurigon").join("nginx").join("conf.d"))
        .unwrap_or_else(|| PathBuf::from("./taurigon-data/nginx/conf.d"))
}
