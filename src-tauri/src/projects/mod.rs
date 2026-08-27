//! # Projekt-Verwaltung
//!
//! Der [`ProjectManager`] verwaltet Projekte: anlegen (inkl. Verzeichnis +
//! Start-Dateien), auflisten und löschen. Die Metadaten liegen in SQLite,
//! die Projektdateien unter `~/.local/share/taurigon/projects/<name>/`.

pub mod model;

use std::path::PathBuf;

use rusqlite::{params, Connection};

use model::Project;

/// Vorlage für ein leeres PHP-Projekt.
const PHP_INDEX: &str = r#"<?php
phpinfo();
"#;

/// Vorlage für ein statisches Projekt.
const STATIC_HTML: &str = r#"<!DOCTYPE html>
<html lang="de">
  <head>
    <meta charset="utf-8" />
    <title>Neues Taurigon-Projekt</title>
  </head>
  <body>
    <h1>Es funktioniert! 🎉</h1>
    <p>Dieses Projekt wird von Taurigon verwaltet.</p>
  </body>
</html>
"#;

/// Verwaltet Projekte über eine geliehene DB-Verbindung.
pub struct ProjectManager<'a> {
    /// Geliehene SQLite-Verbindung.
    conn: &'a Connection,
    /// Basisverzeichnis für Projektdateien.
    projects_root: PathBuf,
}

impl<'a> ProjectManager<'a> {
    /// Erstellt einen ProjectManager mit dem Standard-Projektverzeichnis.
    pub fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            projects_root: default_projects_root(),
        }
    }

    /// Listet alle Projekte, neueste zuerst.
    pub fn list(&self) -> Result<Vec<Project>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, domain, project_type, php_version, path, created_at
                 FROM projects ORDER BY created_at DESC",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |r| {
                Ok(Project {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    domain: r.get(2)?,
                    project_type: r.get(3)?,
                    php_version: r.get(4)?,
                    path: r.get(5)?,
                    created_at: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut out = Vec::new();
        for row in rows {
            out.push(row.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    /// Legt ein neues Projekt an: Verzeichnis + Start-Dateien + DB-Eintrag.
    ///
    /// # Arguments
    /// * `name`         - Gewünschter Projektname (wird normalisiert).
    /// * `project_type` - "php" oder "static".
    /// * `php_version`  - z. B. Some("8.3") bei PHP-Projekten.
    ///
    /// # Errors
    /// - Ungültiger Name
    /// - Verzeichnis existiert bereits
    /// - I/O- oder DB-Fehler (mit Rollback des Verzeichnisses)
    pub fn create(
        &self,
        name: &str,
        project_type: &str,
        php_version: Option<String>,
    ) -> Result<Project, String> {
        let name = name.trim().to_lowercase();
        validate_name(&name)?;

        let domain = format!("{name}.localhost");
        let path = self.projects_root.join(&name);

        if path.exists() {
            return Err(format!(
                "Verzeichnis existiert bereits: {}",
                path.display()
            ));
        }

        // Verzeichnis + Start-Dateien anlegen.
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Verzeichnis konnte nicht angelegt werden: {e}"))?;
        scaffold(&path, project_type)
            .map_err(|e| format!("Start-Dateien konnten nicht erstellt werden: {e}"))?;

        let path_str = path.to_string_lossy().to_string();

        // DB-Eintrag (bei Fehler Verzeichnis wieder entfernen).
        let insert = self.conn.execute(
            "INSERT INTO projects (name, domain, project_type, php_version, path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, domain, project_type, php_version, path_str],
        );

        if let Err(e) = insert {
            let _ = std::fs::remove_dir_all(&path);
            return Err(format!("DB-Eintrag fehlgeschlagen: {e}"));
        }

        let id = self.conn.last_insert_rowid();

        Ok(Project {
            id,
            name,
            domain,
            project_type: project_type.to_string(),
            php_version,
            path: path_str,
            created_at: String::new(),
        })
    }

    /// Löscht ein Projekt (DB-Eintrag + optional die Dateien).
    ///
    /// # Arguments
    /// * `id`           - Projekt-ID.
    /// * `delete_files` - Wenn `true`, wird auch das Verzeichnis gelöscht.
    pub fn delete(&self, id: i64, delete_files: bool) -> Result<(), String> {
        // Pfad vor dem Löschen merken.
        let path: Option<String> = self
            .conn
            .query_row("SELECT path FROM projects WHERE id = ?1", params![id], |r| {
                r.get(0)
            })
            .ok();

        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;

        if delete_files {
            if let Some(p) = path {
                let _ = std::fs::remove_dir_all(p);
            }
        }
        Ok(())
    }
}

/// Legt die Start-Dateien je nach Projekttyp an.
fn scaffold(path: &PathBuf, project_type: &str) -> std::io::Result<()> {
    match project_type {
        "static" => std::fs::write(path.join("index.html"), STATIC_HTML)?,
        // Default: PHP.
        _ => std::fs::write(path.join("index.php"), PHP_INDEX)?,
    }
    Ok(())
}

/// Validiert einen Projektnamen: nur `a-z`, `0-9`, `-`; nicht leer.
fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Der Projektname darf nicht leer sein.".into());
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid {
        return Err(
            "Nur Kleinbuchstaben, Ziffern und Bindestriche sind erlaubt.".into(),
        );
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err("Der Name darf nicht mit einem Bindestrich beginnen/enden.".into());
    }
    Ok(())
}

/// Liefert das Basisverzeichnis für Projektdateien.
fn default_projects_root() -> PathBuf {
    dirs::data_dir()
        .map(|d| d.join("taurigon").join("projects"))
        .unwrap_or_else(|| PathBuf::from("./taurigon-data/projects"))
}
