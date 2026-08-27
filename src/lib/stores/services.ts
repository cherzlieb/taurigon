/**
 * Dienste-Store.
 *
 * Hält Dienst-Metadaten und Laufzeitstatus modulweit vor. Der Zustand
 * überlebt Seitenwechsel; Aktionen (start/stop/...) aktualisieren gezielt
 * nur die betroffenen Status.
 */
import { writable, get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type ServiceInfo = {
  id: string;
  name: string;
  image: string;
  host_port: number;
};

export type ServiceStatus = {
  id: string;
  state: "running" | "stopped" | "not_found";
  host_port: number;
};

/** Statische Dienst-Metadaten (ändern sich nie zur Laufzeit). */
export const services = writable<ServiceInfo[]>([]);
/** Laufzeitstatus je Dienst-ID. */
export const statuses = writable<Record<string, ServiceStatus>>({});
/** Welche Dienste gerade eine Aktion ausführen. */
export const busy = writable<Record<string, boolean>>({});
/** Erstladung läuft? */
export const servicesLoading = writable(false);
/** Läuft gerade ein manueller Status-Refresh? (für Button-Spinner) */
export const servicesRefreshing = writable(false);

/** Aktualisiert nur die Status-Infos aller Dienste (leise). */
export async function refreshStatuses() {
  servicesRefreshing.set(true);
  try {
    const list = await invoke<ServiceStatus[]>("cmd_service_status_all");
    const map: Record<string, ServiceStatus> = {};
    for (const s of list) map[s.id] = s;
    statuses.set(map);
  } catch (e) {
    console.error("Status laden fehlgeschlagen:", e);
  } finally {
    servicesRefreshing.set(false);
  }
}

/** Lädt Metadaten + Status. Zeigt Ladezustand nur bei Erstladung. */
export async function loadServices() {
  const hasData = get(services).length > 0;
  if (!hasData) servicesLoading.set(true);

  try {
    if (!hasData) {
      services.set(await invoke<ServiceInfo[]>("cmd_list_services"));
    }
    await refreshStatuses();
  } catch (e) {
    console.error("Dienste laden fehlgeschlagen:", e);
  } finally {
    servicesLoading.set(false);
  }
}

/** Sorgt dafür, dass Daten vorhanden sind (Erstladung) + Hintergrund-Refresh. */
export async function ensureServices() {
  if (get(services).length === 0) {
    await loadServices();
  } else {
    // Metadaten sind da → nur Status leise aktualisieren.
    refreshStatuses();
  }
}

/** Setzt den Busy-Zustand eines Dienstes. */
function setBusy(id: string, value: boolean) {
  busy.update((b) => ({ ...b, [id]: value }));
}

/** Führt start/stop/restart aus und aktualisiert danach den Status. */
export async function serviceAction(
  id: string,
  action: "start" | "stop" | "restart",
) {
  setBusy(id, true);
  try {
    await invoke(`cmd_${action}_service`, { serviceId: id });
    await refreshStatuses();
  } catch (e) {
    console.error(`${action} fehlgeschlagen:`, e);
    alert(`Fehler: ${e}`);
  } finally {
    setBusy(id, false);
  }
}

/** Entfernt einen Dienst (mit Rückfragen). */
export async function removeService(id: string, name: string) {
  const proceed = confirm(
    `Dienst "${name}" entfernen?\n\n` +
      `OK = weiter zu den Lösch-Optionen\n` +
      `Abbrechen = nichts tun`,
  );
  if (!proceed) return;

  const alsoData = confirm(
    `Sollen auch die persistenten DATEN von "${name}" gelöscht werden?\n\n` +
      `OK = Daten ebenfalls löschen (unwiderruflich!)\n` +
      `Abbrechen = nur Container, Daten behalten`,
  );

  setBusy(id, true);
  try {
    await invoke("cmd_remove_service", { serviceId: id, deleteData: alsoData });
    await refreshStatuses();
  } catch (e) {
    console.error("Entfernen fehlgeschlagen:", e);
    alert(`Fehler: ${e}`);
  } finally {
    setBusy(id, false);
  }
}
