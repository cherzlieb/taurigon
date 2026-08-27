//! Build-Script für Tauri.
//!
//! Wird von Cargo **vor** dem eigentlichen Kompilieren ausgeführt und stößt
//! die Tauri-Codegenerierung an (liest u. a. `tauri.conf.json`).

fn main() {
    tauri_build::build();
}
