//! # Terminal-Modul
//!
//! Der [`TerminalManager`] verwaltet interaktive Terminal-Sessions über ein
//! Pseudo-Terminal (PTY). Jede Session startet eine Shell im Projektverzeichnis.
//!
//! ## Datenfluss
//! - **Output:** Ein Reader-Thread liest die PTY-Ausgabe und sendet sie als
//!   Tauri-Event `terminal-output` ans Frontend.
//! - **Input:** Das Frontend sendet Tastatureingaben via Command an den
//!   PTY-Writer.
//! - **Resize:** Bei Größenänderung des Frontend-Terminals wird die PTY-Größe
//!   angepasst (damit z. B. `htop` korrekt rendert).

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Event-Payload für Terminal-Ausgabe.
///
/// `serde` serialisiert `session_id` als `sessionId` (camelCase fürs Frontend).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TerminalOutput {
    session_id: String,
    data: String,
}

/// Eine aktive Terminal-Session (PTY + Prozess).
struct Session {
    /// Master-Seite des PTY (für Resize).
    master: Box<dyn MasterPty + Send>,
    /// Schreib-Ende (für Tastatureingaben).
    writer: Box<dyn Write + Send>,
    /// Der Shell-Kindprozess (zum Beenden).
    child: Box<dyn Child + Send + Sync>,
}

/// Verwaltet alle offenen Terminal-Sessions.
///
/// Wird als Tauri-State registriert. Zugriffe sind über einen `Mutex`
/// synchronisiert; die Commands sind synchron (kein `await` unter Lock).
#[derive(Default)]
pub struct TerminalManager {
    sessions: Mutex<HashMap<String, Session>>,
}

impl TerminalManager {
    /// Öffnet eine neue Terminal-Session.
    ///
    /// Startet die angegebene Shell im Arbeitsverzeichnis `cwd` und einen
    /// Reader-Thread, der die Ausgabe als Events ans Frontend streamt.
    ///
    /// # Arguments
    /// * `app`        - AppHandle zum Senden von Events.
    /// * `session_id` - Eindeutige ID (vom Frontend generiert).
    /// * `cwd`        - Startverzeichnis (Projektpfad).
    /// * `shell`      - Auszuführende Shell (z. B. "/bin/bash").
    pub fn open(
        &self,
        app: AppHandle,
        session_id: String,
        cwd: &str,
        shell: &str,
    ) -> Result<(), String> {
        log::info!("Terminal öffnen: shell={shell}, cwd={cwd}, id={session_id}");
        let pty_system = native_pty_system();

        // PTY mit sinnvoller Startgröße öffnen.
        let pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                let msg = format!("PTY konnte nicht geöffnet werden: {e}");
                log::error!("{msg}");
                msg
            })?;

        // Shell im Projektverzeichnis starten.
        let mut cmd = CommandBuilder::new(shell);
        cmd.cwd(cwd);
        // TERM setzen, damit Farben/Steuerzeichen korrekt funktionieren.
        cmd.env("TERM", "xterm-256color");

        let child = pair.slave.spawn_command(cmd).map_err(|e| {
            let msg = format!("Shell konnte nicht gestartet werden: {e}");
            log::error!("{msg}");
            msg
        })?;

        log::info!("Shell gestartet für Session {session_id}");

        // Slave-Ende schließen: sorgt für EOF im Reader, wenn die Shell endet.
        drop(pair.slave);

        // Reader (geklont) für den Streaming-Thread, Writer für Eingaben.
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Reader konnte nicht erstellt werden: {e}"))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Writer konnte nicht erstellt werden: {e}"))?;

        // Reader-Thread: liest PTY-Ausgabe und sendet sie als Events.
        let sid = session_id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    // EOF – Shell beendet.
                    Ok(0) => break,
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = app.emit(
                            "terminal-output",
                            TerminalOutput {
                                session_id: sid.clone(),
                                data,
                            },
                        );
                    }
                    Err(_) => break,
                }
            }
            // Frontend informieren, dass die Session zu Ende ist.
            let _ = app.emit("terminal-closed", sid);
        });

        // Session speichern.
        let session = Session {
            master: pair.master,
            writer,
            child,
        };
        self.sessions
            .lock()
            .map_err(|_| "Session-Lock vergiftet")?
            .insert(session_id, session);

        Ok(())
    }

    /// Schreibt Eingabedaten in eine Session (Tastatureingaben).
    pub fn write(&self, session_id: &str, data: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Session-Lock vergiftet")?;
        if let Some(session) = sessions.get_mut(session_id) {
            session
                .writer
                .write_all(data.as_bytes())
                .map_err(|e| e.to_string())?;
            session.writer.flush().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Passt die PTY-Größe an (bei Terminal-Resize im Frontend).
    pub fn resize(&self, session_id: &str, rows: u16, cols: u16) -> Result<(), String> {
        let sessions = self.sessions.lock().map_err(|_| "Session-Lock vergiftet")?;
        if let Some(session) = sessions.get(session_id) {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Schließt eine Session und beendet den Shell-Prozess.
    pub fn close(&self, session_id: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "Session-Lock vergiftet")?;
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.child.kill();
        }
        Ok(())
    }
}
