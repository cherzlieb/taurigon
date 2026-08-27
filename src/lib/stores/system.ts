/**
 * System-Info-Store.
 *
 * Hält die Systeminspektion modulweit vor, damit sie beim Navigieren nicht
 * verloren geht. Folgt dem "stale-while-revalidate"-Muster: gecachte Daten
 * werden sofort angezeigt, während im Hintergrund aktualisiert wird.
 */
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type SystemInfo = {
  engine_kind: string | null;
  engine_mode: string | null;
  engine_version: string | null;
  distro: string;
  selinux_enforcing: boolean;
  is_ready: boolean;
};

/** Die aktuellen Systeminfos (null = noch nie geladen). */
export const systemInfo = writable<SystemInfo | null>(null);
/** Läuft gerade ein Ladevorgang? */
export const systemLoading = writable(false);
/** Letzter Fehler, falls vorhanden. */
export const systemError = writable<string | null>(null);

/**
 * Lädt die Systeminfo (immer frisch vom Backend).
 *
 * Zeigt nur dann den Ladezustand, wenn noch keine Daten vorhanden sind
 * (Erstladung). Bei Refresh bleiben die alten Daten sichtbar.
 */
export async function loadSystemInfo(force = false) {
  const hasData = get(systemInfo) !== null;
  // Bei erzwungenem Refresh IMMER den Ladezustand zeigen (für Button-Spinner).
  if (!hasData || force) systemLoading.set(true);
  systemError.set(null);

  try {
    const info = await invoke<SystemInfo>("cmd_inspect_system");
    systemInfo.set(info);
  } catch (e) {
    systemError.set(String(e));
  } finally {
    systemLoading.set(false);
  }
}

/**
 * Sorgt dafür, dass Daten vorhanden sind (lädt nur beim ersten Mal).
 *
 * Ideal für onMount: Beim ersten Besuch wird geladen, bei weiteren Besuchen
 * passiert nichts (Cache bleibt) – optional Hintergrund-Refresh.
 */
export async function ensureSystemInfo() {
  if (get(systemInfo) === null) {
    await loadSystemInfo();
  } else {
    // Still im Hintergrund aktualisieren (kein Ladezustand).
    loadSystemInfo();
  }
}
