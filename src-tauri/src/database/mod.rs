//! # Datenbank-Verwaltung
//!
//! Der [`DatabaseManager`] verwaltet Datenbanken und Benutzer in den laufenden
//! MariaDB- und PostgreSQL-Containern. Die SQL-Befehle werden per
//! `engine.exec()` **im jeweiligen Container** ausgeführt – es wird kein
//! separater DB-Client auf dem Host benötigt.
//!
//! ## Voraussetzung
//! Der jeweilige Dienst-Container muss laufen. Andernfalls schlägt `exec` fehl.

use serde::Serialize;

use crate::engine::{ContainerEngine, EngineResult};

/// Container-Name des MariaDB-Dienstes.
const MARIADB_CONTAINER: &str = "taurigon-mariadb";
/// Container-Name des PostgreSQL-Dienstes.
const POSTGRES_CONTAINER: &str = "taurigon-postgres";

/// Root-Zugang MariaDB (siehe services/catalog.rs).
const MARIADB_ROOT_PW: &str = "root";
/// Superuser-Zugang PostgreSQL (siehe services/catalog.rs).
const POSTGRES_SUPERUSER: &str = "postgres";
const POSTGRES_PW: &str = "postgres";

/// Systemdatenbanken, die nicht in der UI angezeigt werden sollen.
const MARIADB_SYSTEM_DBS: [&str; 4] =
    ["information_schema", "mysql", "performance_schema", "sys"];
const POSTGRES_SYSTEM_DBS: [&str; 2] = ["postgres", "template1"];

/// Welche Datenbank-Engine gemeint ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    MariaDB,
    Postgres,
}

impl DbKind {
    /// Parst die String-Kennung ("mariadb" | "postgres").
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "mariadb" => Some(DbKind::MariaDB),
            "postgres" => Some(DbKind::Postgres),
            _ => None,
        }
    }

    /// Der zugehörige Container-Name.
    fn container(&self) -> &'static str {
        match self {
            DbKind::MariaDB => MARIADB_CONTAINER,
            DbKind::Postgres => POSTGRES_CONTAINER,
        }
    }
}

/// Eine Datenbank-Information für das Frontend.
#[derive(Debug, Serialize)]
pub struct DatabaseInfo {
    /// Name der Datenbank.
    pub name: String,
}

/// Verwaltet Datenbanken über eine Container-Engine.
pub struct DatabaseManager<'a> {
    engine: &'a dyn ContainerEngine,
}

impl<'a> DatabaseManager<'a> {
    /// Erstellt einen DatabaseManager.
    pub fn new(engine: &'a dyn ContainerEngine) -> Self {
        Self { engine }
    }

    /// Listet die Benutzer-Datenbanken (ohne Systemdatenbanken).
    pub async fn list(&self, kind: DbKind) -> EngineResult<Vec<DatabaseInfo>> {
        let output = match kind {
            DbKind::MariaDB => {
                self.mariadb_sql("SHOW DATABASES;").await?
            }
            DbKind::Postgres => {
                self.postgres_sql(
                    "SELECT datname FROM pg_database WHERE datistemplate = false;",
                )
                .await?
            }
        };

        let system: &[&str] = match kind {
            DbKind::MariaDB => &MARIADB_SYSTEM_DBS,
            DbKind::Postgres => &POSTGRES_SYSTEM_DBS,
        };

        let dbs = output
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .filter(|l| !system.contains(l))
            .map(|l| DatabaseInfo {
                name: l.to_string(),
            })
            .collect();

        Ok(dbs)
    }

    /// Legt eine neue Datenbank an.
    ///
    /// # Arguments
    /// * `kind` - MariaDB oder Postgres.
    /// * `name` - Datenbankname (validiert).
    pub async fn create_database(
        &self,
        kind: DbKind,
        name: &str,
    ) -> EngineResult<()> {
        validate_ident(name)?;
        match kind {
            DbKind::MariaDB => {
                let sql = format!("CREATE DATABASE `{name}`;");
                self.mariadb_sql(&sql).await.map(|_| ())
            }
            DbKind::Postgres => {
                let sql = format!("CREATE DATABASE \"{name}\";");
                self.postgres_sql(&sql).await.map(|_| ())
            }
        }
    }

    /// Löscht eine Datenbank.
    pub async fn drop_database(
        &self,
        kind: DbKind,
        name: &str,
    ) -> EngineResult<()> {
        validate_ident(name)?;
        match kind {
            DbKind::MariaDB => {
                let sql = format!("DROP DATABASE `{name}`;");
                self.mariadb_sql(&sql).await.map(|_| ())
            }
            DbKind::Postgres => {
                let sql = format!("DROP DATABASE \"{name}\";");
                self.postgres_sql(&sql).await.map(|_| ())
            }
        }
    }

    /// Legt einen Benutzer an und gewährt ihm alle Rechte auf eine Datenbank.
    ///
    /// # Arguments
    /// * `kind`     - MariaDB oder Postgres.
    /// * `database` - Ziel-Datenbank (muss existieren).
    /// * `user`     - Benutzername (validiert).
    /// * `password` - Passwort (wird escaped).
    pub async fn create_user(
        &self,
        kind: DbKind,
        database: &str,
        user: &str,
        password: &str,
    ) -> EngineResult<()> {
        validate_ident(database)?;
        validate_ident(user)?;

        match kind {
            DbKind::MariaDB => {
                // Passwort einfach single-quote-escapen.
                let pw = password.replace('\'', "''");
                let sql = format!(
                    "CREATE USER '{user}'@'%' IDENTIFIED BY '{pw}'; \
                     GRANT ALL PRIVILEGES ON `{database}`.* TO '{user}'@'%'; \
                     FLUSH PRIVILEGES;"
                );
                self.mariadb_sql(&sql).await.map(|_| ())
            }
            DbKind::Postgres => {
                let pw = password.replace('\'', "''");
                let sql = format!(
                    "CREATE USER \"{user}\" WITH PASSWORD '{pw}'; \
                     GRANT ALL PRIVILEGES ON DATABASE \"{database}\" TO \"{user}\";"
                );
                self.postgres_sql(&sql).await.map(|_| ())
            }
        }
    }

    /// Prüft, ob der DB-Container läuft (für UI-Hinweise).
    pub async fn is_available(&self, kind: DbKind) -> EngineResult<bool> {
        use crate::engine::ContainerState;
        Ok(matches!(
            self.engine.container_status(kind.container()).await?.state,
            ContainerState::Running
        ))
    }

    // ---- interne SQL-Helfer ------------------------------------------------

    /// Führt SQL im MariaDB-Container aus und liefert stdout.
    async fn mariadb_sql(&self, sql: &str) -> EngineResult<String> {
        self.engine
            .exec(
                MARIADB_CONTAINER,
                &[
                    "mariadb",
                    "-uroot",
                    &format!("-p{MARIADB_ROOT_PW}"),
                    "-N", // keine Spaltenköpfe
                    "-e",
                    sql,
                ],
            )
            .await
    }

    /// Führt SQL im Postgres-Container aus und liefert stdout.
    ///
    /// Nutzt `sh -c`, um `PGPASSWORD` zu setzen.
    async fn postgres_sql(&self, sql: &str) -> EngineResult<String> {
        // SQL für die Shell in doppelte Quotes; interne Doppelquotes escapen.
        let escaped = sql.replace('"', "\\\"");
        let cmd = format!(
            "PGPASSWORD={POSTGRES_PW} psql -U {POSTGRES_SUPERUSER} -tAc \"{escaped}\""
        );
        self.engine
            .exec(POSTGRES_CONTAINER, &["sh", "-c", &cmd])
            .await
    }
}

/// Validiert einen Bezeichner (DB-/Benutzername).
///
/// Erlaubt sind Buchstaben, Ziffern und Unterstriche; nicht leer; darf nicht
/// mit einer Ziffer beginnen. Das verhindert SQL-Injection über den Namen.
fn validate_ident(name: &str) -> EngineResult<()> {
    use crate::engine::EngineError;

    if name.is_empty() {
        return Err(EngineError::Io("Name darf nicht leer sein".into()));
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid {
        return Err(EngineError::Io(
            "Nur Buchstaben, Ziffern und Unterstriche erlaubt".into(),
        ));
    }
    if name.chars().next().unwrap().is_ascii_digit() {
        return Err(EngineError::Io(
            "Name darf nicht mit einer Ziffer beginnen".into(),
        ));
    }
    Ok(())
}
