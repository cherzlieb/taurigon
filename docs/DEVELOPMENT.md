# 🛠️ Entwicklung

Anleitung für Mitwirkende und zum lokalen Aufsetzen der Entwicklungsumgebung.

---

## Voraussetzungen

| Tool | Zweck |
|---|---|
| **Rust** (≥ 1.77) | Backend |
| **Bun** | Frontend-Paketmanager & Runtime |
| **Podman** oder **Docker** | Container-Engine (zur Laufzeit) |

### System-Dependencies (Tauri)

**Arch / CachyOS:**

```bash
sudo pacman -S base-devel webkit2gtk-4.1 libappindicator-gtk3 librsvg
```

**Fedora:**

```bash
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel
```

---

## Setup

```bash
git clone https://github.com/cherzlieb/taurigon.git
cd taurigon
bun install
bun run tauri dev
```

Beim ersten Start dauert der Rust-Build einige Minuten (alle Dependencies werden kompiliert). Nachfolgende Starts sind deutlich schneller.

---

## Nützliche Befehle

| Befehl | Zweck |
|---|---|
| `bun run tauri dev` | App im Dev-Modus (Hot-Reload) |
| `bun run tauri build` | Produktions-Build |
| `bun run check` | Svelte/TypeScript-Typprüfung |
| `cargo fmt` | Rust-Code formatieren (in `src-tauri/`) |
| `cargo clippy` | Rust-Linting (in `src-tauri/`) |
| `cargo doc --open` | Rust-Doku generieren & öffnen |

> 💡 Für ausführlichere Logs: `RUST_LOG=debug bun run tauri dev`

---

## Code-Konventionen

### Rust

- **Dokumentation:** Alle öffentlichen Funktionen/Typen mit `///`-Doc-Comments (Sektionen `# Arguments`, `# Returns`, `# Errors`).
- **Fehlerbehandlung:** Typisierte Fehler via `thiserror`; keine `unwrap()` in produktivem Code (Ausnahme: dokumentierte Invarianten).
- **Async:** Container-Operationen sind `async` (blockieren die UI nicht).

### Frontend

- **State:** Geteilter Zustand in `src/lib/stores/` (Svelte Stores).
- **Komponenten:** Wiederverwendbares in `src/lib/components/`.
- **Styling:** Ausschließlich TailwindCSS-Klassen.
- **Icons:** Lucide (keine Emojis in der UI).

---

## Architektur-Prinzipien

1. **`commands.rs` bleibt dünn** – nur Wrapper, Fachlogik in den Modulen.
2. **Engine-agnostisch** – neue Features arbeiten gegen das `ContainerEngine`-Trait, nie direkt gegen Podman/Docker.
3. **Rootless zuerst** – jede neue Funktion muss ohne Root auskommen (oder den Root-Bedarf klar dokumentieren).

Siehe [ARCHITECTURE.md](ARCHITECTURE.md) für Details.

---

## Neuen Tauri-Command hinzufügen

1. Fachlogik im passenden Modul implementieren (z. B. `services/mod.rs`).
2. Dünnen Wrapper in `commands.rs` anlegen (`#[tauri::command]`).
3. Command in `main.rs` im `invoke_handler!` registrieren.
4. Im Frontend via `invoke("cmd_name", { args })` aufrufen.

**Beispiel (Rust):**

```rust
// 1. Logik in services/mod.rs
impl ServiceManager<'_> {
    pub async fn beispiel(&self) -> EngineResult<String> {
        // ...
    }
}

// 2. Wrapper in commands.rs
#[tauri::command]
pub async fn cmd_beispiel(state: State<'_, AppState>) -> Result<String, String> {
    let info = state.system_info().await;
    let engine = create_engine(&info).map_err(|e| e.to_string())?;
    let manager = ServiceManager::new(engine.as_ref());
    manager.beispiel().await.map_err(|e| e.to_string())
}

// 3. Registrieren in main.rs
.invoke_handler(tauri::generate_handler![
    // ...
    commands::cmd_beispiel,
])
```

**Beispiel (Frontend):**

```typescript
import { invoke } from "@tauri-apps/api/core";
const result = await invoke<string>("cmd_beispiel");
```

---

## Neuen Dienst hinzufügen

Dienste sind im `ServiceKind`-Enum in `src-tauri/src/services/catalog.rs` definiert. Um einen neuen hinzuzufügen:

1. Variante zum `ServiceKind`-Enum ergänzen.
2. In `ALL`, `id()`, `from_id()`, `display_name()`, `container_name()`, `image()`, `host_port()`, `container_port()` und `build_spec()` behandeln.
3. Fertig – die UI listet den Dienst automatisch.

---

## Neue Einstellung hinzufügen

Frontend-Einstellungen leben in `src/lib/stores/settings.ts` und werden per `persistedString` automatisch in `localStorage` gespeichert:

```typescript
export const meineEinstellung = persistedString("meineEinstellung", "default");
```

In der Einstellungsseite (`src/routes/settings/+page.svelte`) eine neue `<section>` mit `bind:value={$meineEinstellung}` ergänzen.

---

## Debugging-Tipps

| Problem | Ansatz |
|---|---|
| Backend-Fehler | `RUST_LOG=debug` für ausführliche Logs |
| Container-Probleme | Direkt prüfen: `podman ps -a`, `podman logs <name>` |
| DB-Inhalt prüfen | `sqlite3 ~/.local/share/taurigon/taurigon.db` |
| Frontend-State | Browser-DevTools in der WebView (Rechtsklick → Inspect) |
| Netzwerk prüfen | `podman network ls`, `podman network inspect taurigon-net` |

---

## Aufräumen (Reset der Umgebung)

Falls die lokale Umgebung zurückgesetzt werden soll:

```bash
# Alle Taurigon-Container entfernen
podman rm -f $(podman ps -aq --filter "label=taurigon=true")

# Netzwerk entfernen
podman network rm taurigon-net

# App-Daten löschen (Achtung: löscht Projekte & DB-Daten!)
rm -rf ~/.local/share/taurigon
```
