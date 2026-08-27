//! # Docker-Engine
//!
//! Implementierung des [`ContainerEngine`]-Traits für Docker.
//!
//! Docker ist der **Fallback**, falls Podman nicht verfügbar ist. Achtung:
//! Zugriff über die `docker`-Gruppe ist effektiv root-äquivalent – im
//! Onboarding weisen wir darauf hin und empfehlen Podman.
//!
//! Die Kommando-Logik ist mit Podman geteilt (siehe Elternmodul); lediglich
//! der Binary-Name unterscheidet sich.

use async_trait::async_trait;

use super::{
    cli_container_status, cli_ensure_network, cli_image_exists, cli_list_containers,
    cli_run_container, run_cli_unit, ContainerEngine, ContainerSpec, ContainerStatus,
    EngineResult,
};
use crate::system::inspector::ContainerEngineKind;

/// Container-Engine auf Basis der `docker`-CLI.
pub struct DockerEngine {
    /// Ob SELinux enforcing ist (steuert das `:Z`-Volume-Flag).
    selinux_enforcing: bool,
}

impl DockerEngine {
    /// Der Name des Binaries im `PATH`.
    const BINARY: &'static str = "docker";

    /// Erstellt eine neue Docker-Engine.
    ///
    /// # Arguments
    /// * `selinux_enforcing` - Aus der Systeminspektion; steuert `:Z`.
    pub fn new(selinux_enforcing: bool) -> Self {
        Self { selinux_enforcing }
    }
}

#[async_trait]
impl ContainerEngine for DockerEngine {
    fn kind(&self) -> ContainerEngineKind {
        ContainerEngineKind::Docker
    }

    async fn ensure_network(&self) -> EngineResult<()> {
        cli_ensure_network(Self::BINARY).await
    }

    async fn run_container(&self, spec: &ContainerSpec) -> EngineResult<String> {
        cli_run_container(Self::BINARY, spec, self.selinux_enforcing).await
    }

    async fn start_container(&self, name: &str) -> EngineResult<()> {
        run_cli_unit(Self::BINARY, vec!["start".into(), name.into()]).await
    }

    async fn stop_container(&self, name: &str) -> EngineResult<()> {
        run_cli_unit(Self::BINARY, vec!["stop".into(), name.into()]).await
    }

    async fn restart_container(&self, name: &str) -> EngineResult<()> {
        run_cli_unit(Self::BINARY, vec!["restart".into(), name.into()]).await
    }

    async fn remove_container(&self, name: &str, force: bool) -> EngineResult<()> {
        let mut args = vec!["rm".into()];
        if force {
            args.push("-f".into());
        }
        args.push(name.into());
        run_cli_unit(Self::BINARY, args).await
    }

    async fn container_status(&self, name: &str) -> EngineResult<ContainerStatus> {
        cli_container_status(Self::BINARY, name).await
    }

    async fn list_containers(&self) -> EngineResult<Vec<ContainerStatus>> {
        cli_list_containers(Self::BINARY).await
    }

    async fn image_exists(&self, image: &str) -> EngineResult<bool> {
        cli_image_exists(Self::BINARY, image).await
    }

    async fn pull_image(&self, image: &str) -> EngineResult<()> {
        run_cli_unit(Self::BINARY, vec!["pull".into(), image.into()]).await
    }
}
