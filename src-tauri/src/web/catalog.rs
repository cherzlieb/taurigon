//! # Web-Katalog
//!
//! Definitionen für die Web-Infrastruktur: Nginx-Reverse-Proxy und die
//! PHP-FPM-Container (pro Version, geteilt über alle Projekte).

/// Container-Name des Nginx-Reverse-Proxys.
pub const PROXY_NAME: &str = "taurigon-proxy";

/// Image des Reverse-Proxys.
pub const PROXY_IMAGE: &str = "docker.io/library/nginx:alpine";

/// Host-Port, auf dem der Proxy lauscht (HTTP).
pub const PROXY_HTTP_PORT: u16 = 8080;

/// Unterstützte PHP-Versionen (für PHP-FPM-Container).
pub const PHP_VERSIONS: [&str; 3] = ["8.2", "8.3", "8.4"];

/// Liefert den Container-Namen für eine PHP-Version.
///
/// # Beispiel
/// `"8.3"` → `"taurigon-php83"`
pub fn php_container_name(version: &str) -> String {
    format!("taurigon-php{}", version.replace('.', ""))
}

/// Liefert das PHP-FPM-Image für eine Version.
///
/// # Beispiel
/// `"8.3"` → `"docker.io/library/php:8.3-fpm-alpine"`
pub fn php_image(version: &str) -> String {
    format!("docker.io/library/php:{version}-fpm-alpine")
}
