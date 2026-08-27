# 📄 CHANGELOG.md

Alle nennenswerten Änderungen an diesem Projekt werden hier dokumentiert.

Format orientiert an [Keep a Changelog](https://keepachangelog.com/de/1.1.0/),
Versionierung nach [SemVer](https://semver.org/lang/de/).

---

## [0.2.0] – 

### Hinzugefügt

- Nginx-Reverse-Proxy für `.localhost`-Domains (Port 8080)
- PHP-FPM-Container pro Version (8.2/8.3/8.4, geteilt)
- vHost-Generator (PHP & statisch)
- Projekte im Browser öffnen
- Live-Reload bei Projekt-Änderungen (kein Neustart nötig)
- Default-vHost (404 für unbekannte Domains)
- Integriertes Terminal mit Projektkontext

---

## [0.1.0] – Fundament & MVP-Kern

### Hinzugefügt

**Systemerkennung**
- Automatische Erkennung der Container-Engine (Podman/Docker) inkl. Version
- Rootless-/Root-Modus-Erkennung
- Distributions- und SELinux-Erkennung (Feature-Detection)

**Container-Engine**
- Engine-Abstraktion (`ContainerEngine`-Trait) für Podman & Docker
- Gemeinsames Container-Netzwerk (`taurigon-net`)
- Automatisches `:Z`-Flag bei aktivem SELinux

**Dienstverwaltung**
- MariaDB, PostgreSQL, Redis: starten, stoppen, neustarten, entfernen
- Live-Status mit farbigen Indikatoren und Port-Anzeige
- Datenpersistenz über benannte Volume-Verzeichnisse
- Optionales Löschen inkl. persistenter Daten

**Projektverwaltung**
- PHP- und statische Projekte anlegen (mit Verzeichnis-Scaffolding)
- Automatische `.localhost`-Domain-Zuweisung
- Projektliste mit Metadaten
- Projekt im konfigurierten Editor öffnen
- Löschen (optional inkl. Dateien)
- SQLite-Persistenz der Projekt-Metadaten

**Benutzeroberfläche**
- Sidebar-Navigation (Dashboard, Dienste, Projekte, Datenbanken, Einstellungen)
- Dark-/Light-Theme mit Persistenz
- Wählbare Akzentfarbe (10 Farben)
- Konfigurierbarer Editor-Befehl
- Stale-While-Revalidate für flüssige Navigation
- Wiederverwendbare Custom-Select-Komponente (Theme-konform)

### Technisches
- Tauri 2 + SvelteKit + TailwindCSS
- Zentraler App-State (SQLite + gecachte SystemInfo)
- Async Container-Operationen (tokio)
