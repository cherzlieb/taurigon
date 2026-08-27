//! # System Inspector
//!
//! Erkennt die Laufzeitumgebung, auf der LinuxDev Manager läuft:
//! - Verfügbare Container-Engine (Podman bevorzugt, Docker als Fallback)
//! - Rootless-Fähigkeit der Engine
//! - Aktive Linux-Distribution
//! - SELinux-Status (entscheidet über das `:Z`-Flag bei Volume-Mounts)
//!
//! Dieses Modul führt **keine** verändernden Aktionen aus – es liest nur den
//! Systemzustand. Alle anderen Module (ServiceManager, ProjectManager, ...)
//! nutzen den hier ermittelten [`SystemInfo`]-Zustand als Grundlage.
//!
//! ## Design-Prinzip: Feature-Detection statt Distro-Detection
//!
//! Wo immer möglich, prüfen wir *Fähigkeiten* (z. B. "ist SELinux enforcing?")
//! statt fest von der Distribution abzuleiten. Das macht den Code robust
//! gegenüber untypischen Setups (z. B. SELinux nachgerüstet auf Arch).

use serde::Serialize;
use std::fmt;
use std::path::Path;
use std::process::Command;

/// Welche Container-Engine wird verwendet.
///
/// Podman wird bevorzugt, weil es rootless standardmäßig sicherer ist.
/// Docker in der `docker`-Gruppe ist effektiv Root-Äquivalent und daher
/// nur ein Fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerEngineKind {
    /// Podman – bevorzugt, echtes rootless möglich.
    Podman,
    /// Docker – Fallback. Achtung: `docker`-Gruppe ≈ passwordless root.
    Docker,
}

impl fmt::Display for ContainerEngineKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerEngineKind::Podman => write!(f, "podman"),
            ContainerEngineKind::Docker => write!(f, "docker"),
        }
    }
}

/// Der Sicherheits-/Rechte-Modus, in dem die Engine läuft.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineMode {
    /// Läuft komplett ohne root (ideal – Podman rootless).
    Rootless,
    /// Läuft mit Root-Äquivalenz (z. B. User in `docker`-Gruppe).
    /// Funktioniert, ist aber sicherheitstechnisch schlechter.
    RootEquivalent,
}

/// Erkannte Container-Engine inklusive Modus und Version.
#[derive(Debug, Clone)]
pub struct DetectedEngine {
    /// Podman oder Docker.
    pub kind: ContainerEngineKind,
    /// Rootless oder Root-Äquivalent.
    pub mode: EngineMode,
    /// Rohe Versions-Zeichenkette (z. B. "5.2.3"), rein informativ.
    pub version: String,
}

/// Die erkannte Linux-Distribution (aus `/etc/os-release`).
///
/// Wir nutzen das primär für nutzerfreundliche Hinweise im Onboarding,
/// **nicht** für funktionale Entscheidungen (siehe Modul-Doku).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Distro {
    /// Arch Linux (optional mit konkretem Derivat-Namen, z. B. "cachyos")
    Arch(Option<String>),
    /// Fedora (optional mit konkretem Derivat-Namen)
    Fedora(Option<String>),
    /// Debian/Ubuntu (optional mit konkretem Derivat-Namen)
    Debian(Option<String>),
    /// Unbekannte oder nicht zugeordnete Distribution.
    Other(String),
}

impl fmt::Display for Distro {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Distro::Arch(specific) => format_base(f, "Arch Based", specific.as_deref(), "arch"),
            Distro::Fedora(specific) => format_base(f, "Fedora Based", specific.as_deref(), "fedora"),
            Distro::Debian(specific) => format_base(f, "Debian Based", specific.as_deref(), "debian"),
            Distro::Other(id) if id.is_empty() => write!(f, "Unknown"),
            Distro::Other(id) => write!(f, "{id}"),
        }
    }
}

// Hilfsfunktion zur Vermeidung von Dopplungen wie "Arch Based - arch"
fn format_base(
    f: &mut fmt::Formatter<'_>,
    base_label: &str,
    specific: Option<&str>,
    base_id: &str,
) -> fmt::Result {
    match specific {
        Some(name) if !name.eq_ignore_ascii_case(base_id) => {
            write!(f, "{base_label} - {name}")
        }
        _ => write!(f, "{base_label}"),
    }
}

/// Gesammelter Systemzustand – das Ergebnis der Inspektion.
///
/// Wird einmal beim App-Start (und optional bei manuellem Refresh) ermittelt
/// und dann an das Frontend gereicht bzw. von anderen Managern konsumiert.
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Erkannte Engine, oder `None`, wenn weder Podman noch Docker nutzbar ist.
    pub engine: Option<DetectedEngine>,
    /// Erkannte Distribution.
    pub distro: Distro,
    /// `true`, wenn SELinux **enforcing** ist → Volume-Mounts brauchen `:Z`.
    pub selinux_enforcing: bool,
}

impl SystemInfo {
    /// Liefert das anzuhängende Volume-Mount-Suffix abhängig von SELinux.
    ///
    /// # Returns
    /// - `":Z"`  wenn SELinux enforcing ist (privates Relabeling des Volumes)
    /// - `""`    sonst (Arch ohne SELinux, oder SELinux permissive/disabled)
    pub fn volume_flag(&self) -> &'static str {
        if self.selinux_enforcing {
            ":Z"
        } else {
            ""
        }
    }

    /// Prüft, ob das System einsatzbereit ist (mindestens eine Engine vorhanden).
    ///
    /// # Returns
    /// `true`, wenn eine nutzbare Container-Engine gefunden wurde.
    pub fn is_ready(&self) -> bool {
        self.engine.is_some()
    }
}

/// Führt die vollständige Systeminspektion durch.
///
/// Sammelt Engine-, Distro- und SELinux-Informationen in einem Rutsch.
/// Diese Funktion ist die zentrale Eingangs-API des Moduls und sollte
/// beim App-Start (im Onboarding) aufgerufen werden.
///
/// # Returns
/// Ein vollständig gefülltes [`SystemInfo`]. Schlägt nie hart fehl –
/// fehlende Komponenten werden als `None`/`false` bzw. `Distro::Other`
/// abgebildet, damit das Onboarding dem User gezielt helfen kann.
pub fn inspect_system() -> SystemInfo {
    SystemInfo {
        engine: detect_engine(),
        distro: detect_distro(),
        selinux_enforcing: detect_selinux_enforcing(),
    }
}

/// Erkennt die bevorzugte Container-Engine.
///
/// Prüft in dieser Reihenfolge:
/// 1. **Podman** – bevorzugt (rootless).
/// 2. **Docker** – Fallback.
///
/// # Returns
/// - `Some(DetectedEngine)` für die erste nutzbare Engine
/// - `None`, wenn keine Engine im `PATH` gefunden bzw. lauffähig ist
fn detect_engine() -> Option<DetectedEngine> {
    if let Some(version) = engine_version("podman") {
        return Some(DetectedEngine {
            kind: ContainerEngineKind::Podman,
            mode: detect_podman_mode(),
            version,
        });
    }

    if let Some(version) = engine_version("docker") {
        return Some(DetectedEngine {
            kind: ContainerEngineKind::Docker,
            mode: detect_docker_mode(),
            version,
        });
    }

    None
}

/// Ruft die Version einer Engine ab und prüft damit implizit ihre Lauffähigkeit.
///
/// # Arguments
/// * `binary` - Name des Executables im `PATH` (z. B. `"podman"`).
///
/// # Returns
/// - `Some(version)` wenn der Befehl erfolgreich (Exit-Code 0) lief
/// - `None` wenn das Binary fehlt oder der Aufruf fehlschlug
fn engine_version(binary: &str) -> Option<String> {
    let output = Command::new(binary)
        .args(["version", "--format", "{{.Client.Version}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

/// Bestimmt den Modus einer Podman-Installation.
///
/// Fragt Podman direkt (`podman info`), ob es rootless läuft.
///
/// # Returns
/// [`EngineMode::Rootless`] im Normalfall, sonst [`EngineMode::RootEquivalent`].
fn detect_podman_mode() -> EngineMode {
    let output = Command::new("podman")
        .args(["info", "--format", "{{.Host.Security.Rootless}}"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let value = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            if value == "true" {
                EngineMode::Rootless
            } else {
                EngineMode::RootEquivalent
            }
        }
        _ => EngineMode::Rootless,
    }
}

/// Bestimmt den Modus einer Docker-Installation.
///
/// Docker wird für den MVP pauschal als root-äquivalent behandelt (Zugriff
/// via `docker`-Gruppe ≈ passwordless root). Rootless-Docker ist selten.
///
/// # Returns
/// [`EngineMode::RootEquivalent`] – bewusst konservativ.
fn detect_docker_mode() -> EngineMode {
    EngineMode::RootEquivalent
}

/// Liest die Linux-Distribution aus `/etc/os-release`.
///
/// # Returns
/// Die erkannte [`Distro`]. Bei fehlender/unlesbarer Datei
/// [`Distro::Other`] mit leerer Kennung.
fn detect_distro() -> Distro {
    let content = match std::fs::read_to_string("/etc/os-release") {
        Ok(c) => c,
        Err(_) => return Distro::Other(String::new()),
    };

    let id = content
        .lines()
        .find_map(|line| line.strip_prefix("ID="))
        .map(|v| v.trim().trim_matches('"').to_lowercase())
        .unwrap_or_default();

    // 1. Erst direkt über die ID prüfen
    let distro = map_to_distro(&id, &id);
    if !matches!(distro, Distro::Other(_)) {
        return distro;
    }

    // 2. Falls unbekannt (z.B. "cachyos"), ID_LIKE durchgehen
    let id_like = content
        .lines()
        .find_map(|line| line.strip_prefix("ID_LIKE="))
        .map(|v| v.trim().trim_matches('"').to_lowercase())
        .unwrap_or_default();

    for like in id_like.split_whitespace() {
        let distro = map_to_distro(like, &id);
        if !matches!(distro, Distro::Other(_)) {
            return distro;
        }
    }

    Distro::Other(id)
}

fn map_to_distro(lookup_name: &str, actual_id: &str) -> Distro {
    let specific = if actual_id.is_empty() {
        None
    } else {
        Some(actual_id.to_string())
    };

    match lookup_name {
        "arch" => Distro::Arch(specific),
        "fedora" | "rhel" | "centos" => Distro::Fedora(specific),
        "debian" | "ubuntu" => Distro::Debian(specific),
        _ => Distro::Other(actual_id.to_string()),
    }
}

/// Prüft, ob SELinux im **enforcing**-Modus läuft.
///
/// Nutzt die Kernel-Schnittstelle `/sys/fs/selinux/enforce`.
///
/// # Returns
/// `true` nur dann, wenn SELinux aktiv **und** enforcing ist.
fn detect_selinux_enforcing() -> bool {
    let path = Path::new("/sys/fs/selinux/enforce");
    match std::fs::read_to_string(path) {
        Ok(content) => content.trim() == "1",
        Err(_) => false,
    }
}

// ============================================================================
//  DTO (Data Transfer Object) – Frontend-freundliche Repräsentation
// ============================================================================

/// Frontend-freundliche, serialisierbare Repräsentation von [`SystemInfo`].
///
/// Enums werden zu einfachen Strings verflacht, damit das JS/TS-Frontend
/// bequem damit arbeiten kann.
#[derive(Debug, Serialize)]
pub struct SystemInfoDto {
    /// "podman" | "docker" | null
    pub engine_kind: Option<String>,
    /// "rootless" | "root-equivalent" | null
    pub engine_mode: Option<String>,
    /// Versionsstring der Engine, oder null.
    pub engine_version: Option<String>,
    /// Distributionsname als String.
    pub distro: String,
    /// SELinux enforcing?
    pub selinux_enforcing: bool,
    /// Ist mindestens eine Engine nutzbar?
    pub is_ready: bool,
}

impl From<SystemInfo> for SystemInfoDto {
    /// Konvertiert die interne [`SystemInfo`] in das serialisierbare DTO.
    fn from(info: SystemInfo) -> Self {
        let (kind, mode, version) = match &info.engine {
            Some(e) => (
                Some(e.kind.to_string()),
                Some(match e.mode {
                    EngineMode::Rootless => "rootless".to_string(),
                    EngineMode::RootEquivalent => "root-equivalent".to_string(),
                }),
                Some(e.version.clone()),
            ),
            None => (None, None, None),
        };

        SystemInfoDto {
            engine_kind: kind,
            engine_mode: mode,
            engine_version: version,
            distro: info.distro.to_string(),
            selinux_enforcing: info.selinux_enforcing,
            is_ready: info.is_ready(),
        }
    }
}
