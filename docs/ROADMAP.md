# 🗺️ Roadmap

Geplante Meilensteine und Features. Abgeschlossenes wandert ins
[Changelog](../CHANGELOG.md).

**Legende:** ✅ fertig · 🚧 in Arbeit · ⬜ geplant

---

## Step 0 – Fundament ✅

- ✅ Projekt-Scaffold (Tauri 2 + SvelteKit)
- ✅ Systemerkennung (Engine, Distro, SELinux, Rootless)
- ✅ Container-Engine-Abstraktion (Podman/Docker)
- ✅ Gemeinsames Container-Netzwerk

## Step 1 – MVP „Es läuft" 🚧

- ✅ Dienstverwaltung (MariaDB, PostgreSQL, Redis)
- ✅ Status-Monitoring mit farbigen Indikatoren
- ✅ Datenpersistenz über Volumes
- ✅ Projektverwaltung (anlegen, auflisten, löschen)
- ✅ Editor-Integration (Projekt im Editor öffnen)
- ✅ Nginx-Reverse-Proxy für `.localhost`-Domains
- ✅ PHP-FPM-Container (8.2 / 8.3 / 8.4, geteilt)
- ✅ „Im Browser öffnen" für Projekte
- ✅ Integriertes Terminal (Basis)

## Step 2 – Komfort ⬜

- ✅ Datenbank-Wizard (DB + User + Rechte)
- ⬜ SSL-Zertifikate via mkcert
- ⬜ Terminal-Politur
- ⬜ Erweiterte Einstellungen
- ⬜ Bessere Lösch-Dialoge

## Step 3 – Projekt-Templates ⬜

- ⬜ Laravel-Template
- ⬜ WordPress-Template
- ⬜ Benutzerdefinierte Templates
- ⬜ Image-/Service-Katalog (statt Systempakete)
- ⬜ Autostart via `systemctl --user` + linger

## Step 4 – Politur & Release ⬜

- ⬜ Onboarding-Wizard (fehlende Voraussetzungen erklären)
- ⬜ Tray-Integration
- ⬜ Fedora-Support final testen (SELinux/`:Z`)
- ⬜ CI/CD, Packaging (AppImage / Flatpak)
- ⬜ Open-Source-Vorbereitung (Lizenz, Contributing-Guide)

## Step 5 – Optional / Zukunft ⬜

- ⬜ Windows-Support (WSL2-Backend)
- ⬜ Weitere Dienste (MongoDB, Elasticsearch, …)
- ⬜ Projekt-Import (bestehende Verzeichnisse einbinden)

---

## Offene Design-Fragen

- **Pull-Progress:** Aktuell werden Images synchron gepullt.
  Später: Live-Progress via Tauri-Events.
- **Lösch-UX:** Aktuell `confirm()`-Dialoge. Später: sauberes Modal mit
  Checkbox „Daten ebenfalls löschen".
- **Netzwerk-Isolation:** Aktuell ein globales Netzwerk. Optional später
  Sub-Netzwerke pro Projekt.
- **Terminal-Kontext:** Läuft auf dem Host. Optional später „im Container
ausführen"-Modus für composer/php im Container-Kontext.
