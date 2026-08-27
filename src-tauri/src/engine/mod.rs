//! # Container-Engine-Abstraktion
//!
//! Kapselt alle Container-Operationen (Netzwerk, Start/Stop, Status) hinter dem
//! [`ContainerEngine`]-Trait. Der Rest der Anwendung arbeitet **nur** gegen
//! diesen Trait und muss nicht wissen, ob darunter Podman oder Docker läuft.
//!
//! ## Warum ein Trait?
//!
//! Podman und Docker haben eine fast identische CLI, aber nicht 100 % gleich.
//! Der Trait definiert die Schnittstelle; die Implementierungen
//! ([`podman::PodmanEngine`], [`docker::DockerEngine`]) füllen die Details.
//!
//! Um Duplikation zu vermeiden, liegt die eigentliche Kommando-Logik in diesem
//! Modul als geteilte Hilfsfunktionen (`cli_*`), parametrisiert über den
//! Binary-Namen. Die beiden Engine-Structs delegieren nur dorthin.

pub mod docker;
pub mod podman;

use async_trait::async_trait;
use thiserror::Error;
use tokio::process::Command;

use crate::system::inspector::{ContainerEngineKind, SystemInfo};

/// Name des gemeinsamen Container-Netzwerks.
///
/// Alle von Taurigon verwalteten Container hängen in diesem Netzwerk und
/// können sich darüber per Container-Namen erreichen (z. B. `php → mariadb`).
pub const NETWORK_NAME: &str = "taurigon-net";

/// Label, mit dem wir *unsere* Container markieren.
///
/// So können wir beim Auflisten gezielt nur Taurigon-Container filtern und
/// fremde Container des Users unangetastet lassen.
pub const MANAGED_LABEL: &str = "taurigon=true";

// ============================================================================
//  Fehler-Typen
// ============================================================================

/// Fehler, die bei Engine-Operationen auftreten können.
#[derive(Debug, Error)]
pub enum EngineError {
    /// Die Engine-Binary (podman/docker) wurde nicht im `PATH` gefunden.
    #[error("Container-Engine '{0}' nicht gefunden (ist sie installiert?)")]
    BinaryNotFound(String),

    /// Es ist überhaupt keine Engine verfügbar.
    #[error("Keine Container-Engine verfügbar")]
    NoEngineAvailable,

    /// Ein Kommando lief, endete aber mit einem Fehler-Exit-Code.
    #[error("Kommando fehlgeschlagen: {command}\n{stderr}")]
    CommandFailed {
        /// Das ausgeführte Kommando (für Debugging).
        command: String,
        /// Die Fehlerausgabe (stderr) des Prozesses.
        stderr: String,
    },

    /// Ein I/O-Fehler beim Ausführen des Subprozesses.
    #[error("I/O-Fehler: {0}")]
    Io(String),
}

/// Bequemer Result-Alias für dieses Modul.
pub type EngineResult<T> = Result<T, EngineError>;

// ============================================================================
//  Spezifikations-Typen (Input für run_container)
// ============================================================================

/// Ein Port-Mapping: Host-Port → Container-Port.
#[derive(Debug, Clone)]
pub struct PortMapping {
    /// Port auf dem Host (im Bereich 8000–8999, siehe Konzept).
    pub host: u16,
    /// Port im Container (z. B. 80 für Nginx).
    pub container: u16,
}

/// Ein Volume-Mount: Host-Pfad → Container-Pfad.
#[derive(Debug, Clone)]
pub struct VolumeMount {
    /// Absoluter Pfad auf dem Host (oder Name eines benannten Volumes).
    pub source: String,
    /// Zielpfad im Container.
    pub target: String,
    /// Ob dieses Volume SELinux-Relabeling (`:Z`) erhalten soll, falls
    /// SELinux enforcing ist. Für DB-Datenverzeichnisse: `true`.
    pub selinux_relabel: bool,
    /// Ob das Volume von mehreren Containern geteilt wird.
    /// `true`  → `:z` (shared), `false` → `:Z` (private).
    pub shared: bool,
}

/// Neustart-Verhalten eines Containers.
#[derive(Debug, Clone, Copy)]
pub enum RestartPolicy {
    /// Nie automatisch neu starten.
    No,
    /// Immer neu starten (auch nach Reboot, wenn Engine läuft).
    Always,
    /// Neu starten, außer der User hat manuell gestoppt.
    UnlessStopped,
    /// Nur bei Fehler-Exit neu starten.
    OnFailure,
}

impl RestartPolicy {
    /// Wandelt die Policy in das CLI-Argument um.
    fn as_arg(&self) -> &'static str {
        match self {
            RestartPolicy::No => "no",
            RestartPolicy::Always => "always",
            RestartPolicy::UnlessStopped => "unless-stopped",
            RestartPolicy::OnFailure => "on-failure",
        }
    }
}

/// Vollständige Spezifikation eines zu startenden Containers.
///
/// Wird typischerweise über die Builder-Methoden zusammengesetzt:
/// ```no_run
/// # use taurigon::engine::{ContainerSpec, PortMapping};
/// let spec = ContainerSpec::new("taurigon-redis", "docker.io/library/redis:7")
///     .with_port(8379, 6379)
///     .with_restart(taurigon::engine::RestartPolicy::UnlessStopped);
/// ```
#[derive(Debug, Clone)]
pub struct ContainerSpec {
    /// Eindeutiger Container-Name (z. B. "taurigon-mariadb").
    pub name: String,
    /// Vollständiger Image-Name inkl. Registry (z. B. "docker.io/library/nginx:alpine").
    pub image: String,
    /// Port-Mappings.
    pub ports: Vec<PortMapping>,
    /// Volume-Mounts.
    pub volumes: Vec<VolumeMount>,
    /// Umgebungsvariablen als (Key, Value)-Paare.
    pub env: Vec<(String, String)>,
    /// Neustart-Verhalten.
    pub restart: RestartPolicy,
    /// Optionaler Kommando-Override (leer = Image-Default verwenden).
    pub command: Vec<String>,
    /// Netzwerk, in das der Container gehängt wird (Default: [`NETWORK_NAME`]).
    pub network: String,
}

impl ContainerSpec {
    /// Erstellt eine Spezifikation mit sinnvollen Defaults.
    ///
    /// # Arguments
    /// * `name`  - Eindeutiger Container-Name.
    /// * `image` - Vollständiger Image-Name.
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            ports: Vec::new(),
            volumes: Vec::new(),
            env: Vec::new(),
            restart: RestartPolicy::UnlessStopped,
            command: Vec::new(),
            network: NETWORK_NAME.to_string(),
        }
    }

    /// Fügt ein Port-Mapping hinzu (Builder-Stil).
    pub fn with_port(mut self, host: u16, container: u16) -> Self {
        self.ports.push(PortMapping { host, container });
        self
    }

    /// Fügt ein privates Volume-Mount hinzu (Builder-Stil).
    ///
    /// Bei aktivem SELinux erhält es `:Z` (nur für diesen Container).
    pub fn with_volume(
        mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        selinux_relabel: bool,
    ) -> Self {
        self.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            selinux_relabel,
            shared: false,
        });
        self
    }

    /// Fügt ein geteiltes Volume-Mount hinzu (Builder-Stil).
    ///
    /// Bei aktivem SELinux erhält es `:z` (mehrere Container dürfen zugreifen).
    /// Nötig für Verzeichnisse, die z. B. Proxy **und** PHP-FPM mounten.
    pub fn with_shared_volume(
        mut self,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        self.volumes.push(VolumeMount {
            source: source.into(),
            target: target.into(),
            selinux_relabel: true,
            shared: true,
        });
        self
    }

    /// Fügt eine Umgebungsvariable hinzu (Builder-Stil).
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Setzt die Restart-Policy (Builder-Stil).
    pub fn with_restart(mut self, policy: RestartPolicy) -> Self {
        self.restart = policy;
        self
    }

    /// Überschreibt das Start-Kommando (Builder-Stil).
    pub fn with_command(mut self, command: Vec<String>) -> Self {
        self.command = command;
        self
    }
}

// ============================================================================
//  Status-Typen (Output von container_status)
// ============================================================================

/// Der Laufzeitzustand eines Containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerState {
    /// Container läuft.
    Running,
    /// Container existiert, ist aber gestoppt.
    Stopped,
    /// Container existiert nicht.
    NotFound,
}

/// Statusinformation zu einem Container.
#[derive(Debug, Clone)]
pub struct ContainerStatus {
    /// Container-Name.
    pub name: String,
    /// Laufzeitzustand.
    pub state: ContainerState,
}

// ============================================================================
//  Der zentrale Trait
// ============================================================================

/// Einheitliche Schnittstelle für Container-Engines (Podman/Docker).
///
/// Alle Methoden sind `async`, weil sie Subprozesse aufrufen und die UI nicht
/// blockieren dürfen. Der Trait ist objekt-sicher (`Box<dyn ContainerEngine>`)
/// dank `async_trait`.
#[async_trait]
pub trait ContainerEngine: Send + Sync {
    /// Liefert, um welche Engine es sich handelt.
    fn kind(&self) -> ContainerEngineKind;

    /// Stellt sicher, dass das gemeinsame Netzwerk existiert (legt es an, falls nicht).
    async fn ensure_network(&self) -> EngineResult<()>;

    /// Startet einen neuen Container gemäß Spezifikation.
    ///
    /// # Returns
    /// Die Container-ID bei Erfolg.
    async fn run_container(&self, spec: &ContainerSpec) -> EngineResult<String>;

    /// Startet einen bereits existierenden, gestoppten Container.
    async fn start_container(&self, name: &str) -> EngineResult<()>;

    /// Stoppt einen laufenden Container.
    async fn stop_container(&self, name: &str) -> EngineResult<()>;

    /// Startet einen Container neu.
    async fn restart_container(&self, name: &str) -> EngineResult<()>;

    /// Entfernt einen Container.
    ///
    /// # Arguments
    /// * `force` - Wenn `true`, wird ein laufender Container zwangsweise entfernt.
    async fn remove_container(&self, name: &str, force: bool) -> EngineResult<()>;

    /// Fragt den Status eines Containers ab.
    async fn container_status(&self, name: &str) -> EngineResult<ContainerStatus>;

    /// Listet alle von Taurigon verwalteten Container.
    async fn list_containers(&self) -> EngineResult<Vec<ContainerStatus>>;

    /// Prüft, ob ein Image lokal vorhanden ist.
    async fn image_exists(&self, image: &str) -> EngineResult<bool>;

    /// Lädt ein Image aus der Registry.
    async fn pull_image(&self, image: &str) -> EngineResult<()>;

    /// Führt einen Befehl **in** einem laufenden Container aus.
    ///
    /// # Arguments
    /// * `name` - Container-Name.
    /// * `cmd`  - Kommando + Argumente (z. B. `["nginx", "-s", "reload"]`).
    async fn exec(&self, name: &str, cmd: &[&str]) -> EngineResult<String>;
}

// ============================================================================
//  Factory
// ============================================================================

/// Erzeugt die passende Engine-Implementierung anhand der Systeminfo.
///
/// # Arguments
/// * `info` - Ergebnis der Systeminspektion.
///
/// # Returns
/// Eine boxed Engine, oder [`EngineError::NoEngineAvailable`], wenn weder
/// Podman noch Docker gefunden wurde.
pub fn create_engine(info: &SystemInfo) -> EngineResult<Box<dyn ContainerEngine>> {
    match &info.engine {
        Some(detected) => match detected.kind {
            ContainerEngineKind::Podman => {
                Ok(Box::new(podman::PodmanEngine::new(info.selinux_enforcing)))
            }
            ContainerEngineKind::Docker => {
                Ok(Box::new(docker::DockerEngine::new(info.selinux_enforcing)))
            }
        },
        None => Err(EngineError::NoEngineAvailable),
    }
}

// ============================================================================
//  Geteilte CLI-Logik (von Podman- und Docker-Engine genutzt)
// ============================================================================
//
//  Diese Funktionen enthalten die eigentliche Arbeit. Sie sind über den
//  `binary`-Parameter engine-agnostisch. Die beiden Engine-Structs delegieren
//  nur hierher – so gibt es keine doppelte Logik.

/// Führt ein Engine-Kommando aus und liefert stdout zurück.
///
/// # Arguments
/// * `binary` - "podman" oder "docker".
/// * `args`   - Argumentliste.
///
/// # Errors
/// - [`EngineError::BinaryNotFound`] wenn das Binary fehlt.
/// - [`EngineError::CommandFailed`] bei Exit-Code ≠ 0.
/// - [`EngineError::Io`] bei sonstigen Prozessfehlern.
pub(crate) async fn run_cli(binary: &str, args: Vec<String>) -> EngineResult<String> {
    let output = Command::new(binary)
        .args(&args)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                EngineError::BinaryNotFound(binary.to_string())
            } else {
                EngineError::Io(e.to_string())
            }
        })?;

    if !output.status.success() {
        return Err(EngineError::CommandFailed {
            command: format!("{binary} {}", args.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Führt ein Kommando aus, ignoriert aber stdout und meldet nur Erfolg/Fehler.
pub(crate) async fn run_cli_unit(binary: &str, args: Vec<String>) -> EngineResult<()> {
    run_cli(binary, args).await.map(|_| ())
}

/// Stellt das gemeinsame Netzwerk sicher (geteilte Implementierung).
///
/// Prüft via `network inspect` (funktioniert bei Podman **und** Docker) und
/// legt das Netzwerk mit `network create` an, falls es fehlt.
pub(crate) async fn cli_ensure_network(binary: &str) -> EngineResult<()> {
    // Existiert das Netzwerk bereits? inspect liefert Exit 0 wenn ja.
    let exists = run_cli(
        binary,
        vec!["network".into(), "inspect".into(), NETWORK_NAME.into()],
    )
    .await
    .is_ok();

    if exists {
        return Ok(());
    }

    // Anlegen.
    run_cli_unit(
        binary,
        vec!["network".into(), "create".into(), NETWORK_NAME.into()],
    )
    .await
}

/// Baut die Argumentliste für `run -d ...` aus einer [`ContainerSpec`].
///
/// # Arguments
/// * `spec`              - Die Container-Spezifikation.
/// * `selinux_enforcing` - Wenn `true`, wird bei relabel-Volumes `:Z` angehängt.
pub(crate) fn build_run_args(spec: &ContainerSpec, selinux_enforcing: bool) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "run".into(),
        "-d".into(),
        "--name".into(),
        spec.name.clone(),
        "--label".into(),
        MANAGED_LABEL.into(),
        "--network".into(),
        spec.network.clone(),
        "--restart".into(),
        spec.restart.as_arg().into(),
    ];

    for p in &spec.ports {
        args.push("-p".into());
        args.push(format!("{}:{}", p.host, p.container));
    }

    for v in &spec.volumes {
        // SELinux-Relabel-Suffix: :z (shared) oder :Z (private).
        let suffix = if selinux_enforcing && v.selinux_relabel {
            if v.shared {
                ":z"
            } else {
                ":Z"
            }
        } else {
            ""
        };
        args.push("-v".into());
        args.push(format!("{}:{}{}", v.source, v.target, suffix));
    }

    for (k, val) in &spec.env {
        args.push("-e".into());
        args.push(format!("{k}={val}"));
    }

    // Image kommt nach den Optionen.
    args.push(spec.image.clone());

    // Optionaler Kommando-Override.
    for c in &spec.command {
        args.push(c.clone());
    }

    args
}

/// Startet einen Container (geteilte Implementierung).
pub(crate) async fn cli_run_container(
    binary: &str,
    spec: &ContainerSpec,
    selinux_enforcing: bool,
) -> EngineResult<String> {
    let args = build_run_args(spec, selinux_enforcing);
    let id = run_cli(binary, args).await?;
    Ok(id.trim().to_string())
}

/// Fragt den Container-Status via `inspect` ab (geteilte Implementierung).
///
/// Nutzt `{{.State.Status}}` – identisch bei Podman und Docker. Schlägt der
/// inspect fehl (Container existiert nicht), wird [`ContainerState::NotFound`]
/// zurückgegeben statt eines Fehlers.
pub(crate) async fn cli_container_status(
    binary: &str,
    name: &str,
) -> EngineResult<ContainerStatus> {
    let result = run_cli(
        binary,
        vec![
            "inspect".into(),
            "--format".into(),
            "{{.State.Status}}".into(),
            name.into(),
        ],
    )
    .await;

    let state = match result {
        Ok(out) => parse_state(out.trim()),
        // inspect scheitert → Container existiert nicht.
        Err(_) => ContainerState::NotFound,
    };

    Ok(ContainerStatus {
        name: name.to_string(),
        state,
    })
}

/// Übersetzt den rohen State-String der Engine in [`ContainerState`].
fn parse_state(raw: &str) -> ContainerState {
    match raw {
        "running" => ContainerState::Running,
        // created, exited, paused, stopped, dead ...
        _ => ContainerState::Stopped,
    }
}

/// Listet alle von Taurigon verwalteten Container (geteilte Implementierung).
///
/// Filtert über unser [`MANAGED_LABEL`] und nutzt ein Tab-getrenntes Format,
/// das bei Podman und Docker gleich funktioniert (umgeht JSON-Unterschiede).
pub(crate) async fn cli_list_containers(binary: &str) -> EngineResult<Vec<ContainerStatus>> {
    let out = run_cli(
        binary,
        vec![
            "ps".into(),
            "-a".into(),
            "--filter".into(),
            format!("label={MANAGED_LABEL}"),
            "--format".into(),
            "{{.Names}}\t{{.State}}".into(),
        ],
    )
    .await?;

    let mut result = Vec::new();
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: "name<TAB>state"
        let mut parts = line.splitn(2, '\t');
        let name = parts.next().unwrap_or("").to_string();
        let state_raw = parts.next().unwrap_or("");
        result.push(ContainerStatus {
            name,
            state: parse_state(state_raw),
        });
    }

    Ok(result)
}

/// Prüft, ob ein Image lokal existiert (geteilte Implementierung).
pub(crate) async fn cli_image_exists(binary: &str, image: &str) -> EngineResult<bool> {
    let ok = run_cli(
        binary,
        vec!["image".into(), "inspect".into(), image.into()],
    )
    .await
    .is_ok();
    Ok(ok)
}

/// Führt einen Befehl in einem laufenden Container aus (geteilte Impl).
pub(crate) async fn cli_exec(
    binary: &str,
    name: &str,
    cmd: &[&str],
) -> EngineResult<String> {
    let mut args = vec!["exec".to_string(), name.to_string()];
    for c in cmd {
        args.push((*c).to_string());
    }
    run_cli(binary, args).await
}
