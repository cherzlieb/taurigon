/**
 * App-Einstellungen (persistiert in localStorage).
 */
import { writable, type Writable } from "svelte/store";
import { browser } from "$app/environment";

/** Erstellt einen String-Store, der automatisch in localStorage persistiert. */
function persistedString(key: string, initial: string): Writable<string> {
  const start = browser ? localStorage.getItem(key) ?? initial : initial;
  const store = writable<string>(start);
  if (browser) {
    store.subscribe((value) => localStorage.setItem(key, value));
  }
  return store;
}

/**
 * Editor-Befehl zum Öffnen von Projekten.
 * Template mit optionalem `{path}`-Platzhalter. Default: VS Code.
 */
export const editorCommand = persistedString("editorCommand", "code {path}");
