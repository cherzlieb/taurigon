# 🗺️ Roadmap

Geplante Meilensteine und Features. Abgeschlossenes wandert ins
[Changelog](../CHANGELOG.md).

**Legende:** ✅ fertig · 🚧 in Arbeit · ⬜ geplant

---

## M0 – Fundament ✅

- ✅ Projekt-Scaffold (Tauri 2 + SvelteKit)
- ✅ Systemerkennung (Engine, Distro, SELinux, Rootless)
- ✅ Container-Engine-Abstraktion (Podman/Docker)
- ✅ Gemeinsames Container-Netzwerk

## M1 – MVP „Es läuft" 🚧

- ✅ Dienstverwaltung (MariaDB, PostgreSQL, Redis)
- ✅ Status-Monitoring mit farbigen Indikatoren
- ✅ Datenpersistenz über Volumes
- ✅ Projektverwaltung (anlegen, auflisten, löschen)
- ✅ Editor-Integration (Projekt im Editor öffnen)
- ✅ Nginx-Reverse-Proxy für `.localhost`-Domains
- ✅ PHP-FPM-Container (8.2 / 8.3 / 8.4, geteilt)
- ✅ „Im Browser öffnen" für Projekte
- ⬜ Integriertes Terminal (Basis)

## M2 – Komfort ⬜

- ⬜ vHost-Management (mehrere PHP-Versionen parallel)
- ⬜ SSL-Zertifikate via mkcert (One-Click)
- ⬜ Datenbank-Wizard (DB + Benutzer + Rechte)
- ⬜ Terminal-Ausbau (History, Quick-Commands, farbige Ausgabe)
- ⬜ Erweiterte Einstellungen (Ports, Datenverzeichnis, Domain-Suffix)

## M3 – Projekt-Templates ⬜

- ⬜ Laravel-Template
- ⬜ WordPress-Template
- ⬜ Benutzerdefinierte Templates
- ⬜ Image-/Service-Katalog (statt Systempakete)
- ⬜ Autostart via `systemctl --user` + linger

## M4 – Politur & Release ⬜

- ⬜ Onboarding-Wizard (fehlende Voraussetzungen erklären)
- ⬜ Tray-Integration
- ⬜ Fedora-Support final testen (SELinux/`:Z`)
- ⬜ CI/CD, Packaging (AppImage / Flatpak)
- ⬜ Open-Source-Vorbereitung (Lizenz, Contributing-Guide)

## M5 – Optional / Zukunft ⬜

- ⬜ Windows-Support (WSL2-Backend)
- ⬜ Weitere Dienste (MongoDB, Elasticsearch, …)
- ⬜ Projekt-Import (bestehende Verzeichnisse einbinden)

---

## Offene Design-Fragen

- **Pull-Progress:** Aktuell werden Images synchron gepullt (Ansatz A).
  Später: Live-Progress via Tauri-Events (Ansatz B).
- **Lösch-UX:** Aktuell `confirm()`-Dialoge. Später: sauberes Modal mit
  Checkbox „Daten ebenfalls löschen".
- **Netzwerk-Isolation:** Aktuell ein globales Netzwerk. Optional später
  Sub-Netzwerke pro Projekt.
