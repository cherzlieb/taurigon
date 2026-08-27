/**
 * Datenbank-Store: verwaltet DBs pro Engine (MariaDB/Postgres).
 */
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type DbKind = "mariadb" | "postgres";
export type DatabaseInfo = { name: string };

/** Datenbanken je Engine. */
export const databases = writable<Record<DbKind, DatabaseInfo[]>>({
  mariadb: [],
  postgres: [],
});
/** Verfügbarkeit (läuft der Container?) je Engine. */
export const dbAvailable = writable<Record<DbKind, boolean>>({
  mariadb: false,
  postgres: false,
});
/** Ladezustand. */
export const dbLoading = writable(false);

/** Lädt Verfügbarkeit + DB-Listen für beide Engines. */
export async function loadDatabases() {
  dbLoading.set(true);
  try {
    for (const kind of ["mariadb", "postgres"] as DbKind[]) {
      const available = await invoke<boolean>("cmd_db_available", { kind });
      dbAvailable.update((a) => ({ ...a, [kind]: available }));

      if (available) {
        const list = await invoke<DatabaseInfo[]>("cmd_db_list", { kind });
        databases.update((d) => ({ ...d, [kind]: list }));
      } else {
        databases.update((d) => ({ ...d, [kind]: [] }));
      }
    }
  } catch (e) {
    console.error("DBs laden fehlgeschlagen:", e);
  } finally {
    dbLoading.set(false);
  }
}

/** Lädt nur beim ersten Mal, sonst Hintergrund-Refresh. */
export async function ensureDatabases() {
  const d = get(databases);
  if (d.mariadb.length === 0 && d.postgres.length === 0) {
    await loadDatabases();
  } else {
    loadDatabases();
  }
}

/** Legt eine Datenbank an. */
export async function createDatabase(kind: DbKind, name: string) {
  await invoke("cmd_db_create", { kind, name });
  await loadDatabases();
}

/** Löscht eine Datenbank (mit Rückfrage). */
export async function dropDatabase(kind: DbKind, name: string) {
  const ok = confirm(
    `Datenbank "${name}" wirklich löschen?\n\nDies ist unwiderruflich!`,
  );
  if (!ok) return;
  await invoke("cmd_db_drop", { kind, name });
  await loadDatabases();
}

/** Legt einen Benutzer an. */
export async function createDbUser(
  kind: DbKind,
  database: string,
  user: string,
  password: string,
) {
  await invoke("cmd_db_create_user", { kind, database, user, password });
}
