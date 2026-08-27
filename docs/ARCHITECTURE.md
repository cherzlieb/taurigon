# 🏛️ Architektur

Dieses Dokument beschreibt den technischen Aufbau von Taurigon und die
zentralen Design-Entscheidungen.

---

## Überblick

┌─────────────────────────────────────────────┐
│  Frontend (SvelteKit + Tailwind + Lucide)   │
│  Dashboard · Dienste · Projekte · Settings  │
└───────────────────┬─────────────────────────┘
│ Tauri IPC (invoke/events)
┌───────────────────▼─────────────────────────┐
│           Rust Backend (Tauri)               │
│  ┌────────────────────────────────────────┐ │
│  │ ContainerEngine (trait)                │ │
│  │   ├─ PodmanEngine                      │ │
│  │   └─ DockerEngine                      │ │
│  ├────────────────────────────────────────┤ │
│  │ ServiceManager   (Dienste-Container)   │ │
│  │ ProjectManager   (Projekt-CRUD)        │ │
│  │ SystemInspector  (Umgebungserkennung)  │ │
│  └────────────────────────────────────────┘ │
│  State: SQLite + gecachte SystemInfo         │
└─────────────────────────────────────────────┘
---

## Zentrale Design-Entscheidungen

### 1. Container statt Systemdienste

**Entscheidung:** Alle Dienste laufen als rootless Container (Podman/Docker),
nicht als System-Dienste via `systemctl`.

**Begründung:** Das Kernziel ist der rootless-Betrieb. Systemdienste, das
Editieren von `/etc/hosts` und das Binden privilegierter Ports erfordern alle
Root-Rechte. Container lösen das elegant im User-Kontext.

### 2. Engine-Abstraktion via Trait

**Entscheidung:** Ein `ContainerEngine`-Trait kapselt alle Container-Operationen;
Podman und Docker sind austauschbare Implementierungen.

**Begründung:** Podman und Docker haben eine fast identische CLI. Der Trait
erlaubt es dem Rest der App, engine-unabhängig zu arbeiten. Die gemeinsame
CLI-Logik liegt zentral, die Implementierungen delegieren nur.

### 3. `.localhost` statt `.test`

**Entscheidung:** Projekt-Domains nutzen `*.localhost`.

**Begründung:** `*.localhost` wird von `systemd-resolved` automatisch auf
`127.0.0.1` aufgelöst – **ohne** Eintrag in `/etc/hosts` (der Root bräuchte).
`.test` würde dnsmasq/NSS-Konfiguration mit Root erfordern.

### 4. Reverse-Proxy-Pattern

**Entscheidung:** Ein zentraler Nginx-Container routet alle Projekt-Domains
über einen einzigen Port (8080/8443).

**Begründung:** Statt jedem Projekt einen eigenen Port zu geben, genügt ein
Eingang. Neue Projekte = neue vHost-Config + Reload, kein neuer Port.

### 5. Feature-Detection statt Distro-Detection

**Entscheidung:** Systemeigenschaften werden über tatsächliche Fähigkeiten
geprüft (z. B. SELinux via `/sys/fs/selinux/enforce`), nicht über die Distro.

**Begründung:** Robuster gegenüber untypischen Setups (z. B. SELinux auf Arch
nachgerüstet). Das `:Z`-Volume-Flag wird nur gesetzt, wenn SELinux wirklich
enforcing ist.

### 6. Geteilte PHP-FPM-Container

**Entscheidung:** Ein PHP-FPM-Container pro Version bedient alle Projekte
dieser Version.

**Begründung:** Effizienter als ein Container pro Projekt (weniger Ressourcen).
Entspricht dem Vorgehen von Laragon/Valet. Die Trennung erfolgt über den Proxy.

---

## Projektstruktur

taurigon/
├── src/                          # Frontend (SvelteKit)
│   ├── lib/
│   │   ├── components/           # Wiederverwendbare UI-Komponenten
│   │   └── stores/               # Zustandsverwaltung (Svelte Stores)
│   ├── routes/                   # Seiten (Dateibasiertes Routing)
│   ├── app.html
│   └── app.css
│
└── src-tauri/                    # Backend (Rust)
├── src/
│   ├── main.rs               # Einstiegspunkt, Command-Registrierung
│   ├── commands.rs           # Tauri-Commands (Frontend-Brücke)
│   ├── state.rs              # Globaler App-State (DB + SystemInfo)
│   ├── db/                   # SQLite-Zugang & Migrationen
│   ├── engine/               # Container-Engine-Abstraktion
│   │   ├── mod.rs            #   Trait + geteilte Logik + Factory
│   │   ├── podman.rs
│   │   └── docker.rs
│   ├── services/             # Dienst-Verwaltung
│   │   ├── mod.rs            #   ServiceManager
│   │   └── catalog.rs        #   Dienst-Definitionen
│   ├── projects/             # Projekt-Verwaltung
│   │   ├── mod.rs            #   ProjectManager
│   │   └── model.rs          #   Datenmodell
│   └── system/               # Umgebungserkennung
│       └── inspector.rs
└── Cargo.toml
---

## Datenablage (XDG-konform)

~/.local/share/taurigon/
├── taurigon.db           # SQLite: Projekt-Metadaten
├── projects/             # Projekt-Dateien
│   └── <name>/
└── volumes/              # Persistente Dienst-Daten
├── mariadb/
├── postgres/
└── redis/
Nutzereinstellungen (Theme, Akzentfarbe, Editor-Befehl) liegen im
`localStorage` des Frontends.

---

## Frontend-Konzepte

### Stale-While-Revalidate

Daten (Systeminfo, Dienste, Projekte) werden in modulweiten Svelte-Stores
gehalten. Beim Navigieren zwischen Seiten bleibt der Zustand erhalten:
gecachte Daten werden sofort angezeigt, während im Hintergrund aktualisiert
wird. Das verhindert Flackern und Neu-Laden bei jedem Seitenwechsel.

### Custom-Komponenten

Native HTML-Controls (z. B. `<select>`) rendern unter WebKitGTK nicht
theme-konform. Deshalb existieren eigene Komponenten (`Select.svelte`) mit
voller Styling-Kontrolle.

---

## Rechte-Konzept

Taurigon ist auf minimale Rechte ausgelegt:

| Operation | Root nötig? | Lösung |
|---|---|---|
| Container starten/stoppen | ❌ | Rootless Podman/Docker |
| `.localhost`-Auflösung | ❌ | `systemd-resolved` |
| Proxy-Ports (8080/8443) | ❌ | High-Ports (> 1024) |
| SSL-Zertifikate | ❌ | mkcert (User-Trust-Store) |
| Datenpersistenz | ❌ | User-Volumes unter `~/.local/share` |

Der einzige potenzielle Root-Schritt ist die **einmalige Installation** der
Container-Engine und Tauri-System-Dependencies – das ist normale
Software-Installation, keine Laufzeit-Anforderung.
