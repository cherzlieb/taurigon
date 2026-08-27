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

/** Legt ein Projekt an und aktualisiert die Liste. */
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
