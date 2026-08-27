/**
 * Projekt-Store: hält die Projektliste modulweit vor.
 */
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type Project = {
  id: number;
  name: string;
  domain: string;
  project_type: string;
  php_version: string | null;
  path: string;
  created_at: string;
};

export const projects = writable<Project[]>([]);
export const projectsLoading = writable(false);

/** Lädt die Projektliste (Ladezustand nur bei Erstladung). */
export async function loadProjects() {
  const hasData = get(projects).length > 0;
  if (!hasData) projectsLoading.set(true);
  try {
    projects.set(await invoke<Project[]>("cmd_list_projects"));
  } catch (e) {
    console.error("Projekte laden fehlgeschlagen:", e);
  } finally {
    projectsLoading.set(false);
  }
}


/** Lädt nur beim ersten Mal, danach Hintergrund-Refresh. */
export async function ensureProjects() {
  if (get(projects).length === 0) {
    await loadProjects();
  } else {
    loadProjects();
  }
}

/** Legt ein Projekt an, aktualisiert die Liste und lädt vHosts neu. */
export async function createProject(
  name: string,
  projectType: string,
  phpVersion: string | null,
) {
  await invoke("cmd_create_project", {
    name,
    projectType,
    phpVersion,
  });
  await loadProjects();
  // Läuft der Webserver, neue vHost-Config sofort aktiv machen.
  if (get(webRunning)) {
    await invoke("cmd_reload_web").catch((e) =>
      console.error("Reload nach Anlegen fehlgeschlagen:", e),
    );
  }
}

/** Löscht ein Projekt (mit Rückfrage zu den Dateien). */
export async function deleteProject(id: number, name: string) {
  const proceed = confirm(`Projekt "${name}" löschen?`);
  if (!proceed) return;

  const deleteFiles = confirm(
    `Sollen auch die DATEIEN von "${name}" gelöscht werden?\n\n` +
      `OK = Dateien ebenfalls löschen (unwiderruflich!)\n` +
      `Abbrechen = nur aus der Liste entfernen`,
  );

  await invoke("cmd_delete_project", { id, deleteFiles });
  await loadProjects();
}

/** Öffnet ein Projekt im konfigurierten Editor. */
export async function openInEditor(path: string, editorCommand: string) {
  try {
    await invoke("cmd_open_in_editor", { path, editorCommand });
  } catch (e) {
    alert(`Editor konnte nicht geöffnet werden:\n${e}`);
  }
}

/** Läuft der Web-Proxy? */
export const webRunning = writable(false);
/** Web-Aktion in Arbeit? */
export const webBusy = writable(false);

/** Fragt den Web-Status ab. */
export async function loadWebStatus() {
  try {
    webRunning.set(await invoke<boolean>("cmd_web_status"));
  } catch (e) {
    console.error("Web-Status fehlgeschlagen:", e);
  }
}

/** Startet die Web-Umgebung. */
export async function startWeb() {
  webBusy.set(true);
  try {
    await invoke("cmd_start_web");
    await loadWebStatus();
  } catch (e) {
    alert(`Web-Start fehlgeschlagen:\n${e}`);
  } finally {
    webBusy.set(false);
  }
}

/** Stoppt die Web-Umgebung. */
export async function stopWeb() {
  webBusy.set(true);
  try {
    await invoke("cmd_stop_web");
    await loadWebStatus();
  } catch (e) {
    alert(`Web-Stopp fehlgeschlagen:\n${e}`);
  } finally {
    webBusy.set(false);
  }
}

/** Öffnet ein Projekt im Browser (startet Web bei Bedarf). */
export async function openProject(domain: string) {
  try {
    // Sicherstellen, dass der Web-Stack läuft.
    if (!get(webRunning)) {
      await startWeb();
    }
    const port = await invoke<number>("cmd_proxy_port");
    await invoke("cmd_open_url", { url: `http://${domain}:${port}` });
  } catch (e) {
    alert(`Projekt konnte nicht geöffnet werden:\n${e}`);
  }
}
