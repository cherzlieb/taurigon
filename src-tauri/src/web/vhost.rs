//! # vHost-Generator
//!
//! Erzeugt Nginx-Server-Konfigurationen pro Projekt. Für PHP-Projekte wird
//! an den passenden PHP-FPM-Container weitergeleitet; statische Projekte
//! werden direkt ausgeliefert.
//!
//! Die Projekt-Dateien liegen im Container unter `/var/www/<name>` (das
//! Host-Projektverzeichnis wird dorthin gemountet).

use super::catalog::php_container_name;

/// Template für PHP-Projekte. Platzhalter werden per `replace` ersetzt,
/// da die Nginx-Syntax geschweifte Klammern nutzt (kein Rust-`format!`).
const TPL_PHP: &str = r#"server {
    listen 80;
    server_name __DOMAIN__;
    root /var/www/__NAME__;
    index index.php index.html;

    location / {
        try_files $uri $uri/ /index.php?$query_string;
    }

    location ~ \.php$ {
        fastcgi_pass __PHP__:9000;
        fastcgi_index index.php;
        include fastcgi_params;
        fastcgi_param SCRIPT_FILENAME $document_root$fastcgi_script_name;
    }
}
"#;

/// Template für statische Projekte.
const TPL_STATIC: &str = r#"server {
    listen 80;
    server_name __DOMAIN__;
    root /var/www/__NAME__;
    index index.html;

    location / {
        try_files $uri $uri/ =404;
    }
}
"#;

/// Generiert die Nginx-Config für ein Projekt.
///
/// # Arguments
/// * `name`         - Projektname (= Verzeichnis unter /var/www).
/// * `domain`       - Server-Name, z. B. "myapp.localhost".
/// * `project_type` - "php" oder "static".
/// * `php_version`  - z. B. Some("8.3") bei PHP-Projekten.
///
/// # Returns
/// Der fertige Config-Text.
pub fn generate(
    name: &str,
    domain: &str,
    project_type: &str,
    php_version: Option<&str>,
) -> String {
    if project_type == "php" {
        let version = php_version.unwrap_or("8.3");
        let php = php_container_name(version);
        TPL_PHP
            .replace("__DOMAIN__", domain)
            .replace("__NAME__", name)
            .replace("__PHP__", &php)
    } else {
        TPL_STATIC
            .replace("__DOMAIN__", domain)
            .replace("__NAME__", name)
    }
}
