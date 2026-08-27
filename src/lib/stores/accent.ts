/**
 * Akzentfarben-Store.
 *
 * Verwaltet die vom Nutzer gewählte Akzentfarbe. Setzt CSS-Variablen
 * (--accent-500/600) auf <html>, worauf die Tailwind-`accent`-Farbe reagiert.
 * Persistiert die Wahl in localStorage.
 */
import { writable } from "svelte/store";
import { browser } from "$app/environment";

/** Eine wählbare Akzentfarbe mit ihren RGB-Werten (space-separated). */
export type AccentColor = {
  id: string;
  name: string;
  /** RGB-Werte des 500er-Shades, z. B. "59 130 246". */
  shade500: string;
  /** RGB-Werte des 600er-Shades. */
  shade600: string;
  /** Hex des 600er-Shades – nur für die Vorschau-Swatches in der UI. */
  preview: string;
};

/** Alle verfügbaren Akzentfarben (Tailwind-Paletten). */
export const ACCENT_COLORS: AccentColor[] = [
  { id: "blue",   name: "Blau",    shade500: "59 130 246",  shade600: "37 99 235",  preview: "#2563eb" },
  { id: "indigo", name: "Indigo",  shade500: "99 102 241",  shade600: "79 70 229",  preview: "#4f46e5" },
  { id: "purple", name: "Violett", shade500: "168 85 247",  shade600: "147 51 234", preview: "#9333ea" },
  { id: "pink",   name: "Pink",    shade500: "236 72 153",  shade600: "219 39 119", preview: "#db2777" },
  { id: "red",    name: "Rot",     shade500: "239 68 68",   shade600: "220 38 38",  preview: "#dc2626" },
  { id: "orange", name: "Orange",  shade500: "249 115 22",  shade600: "234 88 12",  preview: "#ea580c" },
  { id: "amber",  name: "Amber",   shade500: "245 158 11",  shade600: "217 119 6",  preview: "#d97706" },
  { id: "green",  name: "Grün",    shade500: "34 197 94",   shade600: "22 163 74",  preview: "#16a34a" },
  { id: "teal",   name: "Teal",    shade500: "20 184 166",  shade600: "13 148 136", preview: "#0d9488" },
  { id: "cyan",   name: "Cyan",    shade500: "6 182 212",   shade600: "8 145 178",  preview: "#0891b2" },
];

const STORAGE_KEY = "accent";
const DEFAULT_ID = "blue";

/** Ermittelt die gespeicherte Farbe oder den Default. */
function initialAccent(): AccentColor {
  if (browser) {
    const stored = localStorage.getItem(STORAGE_KEY);
    const found = ACCENT_COLORS.find((c) => c.id === stored);
    if (found) return found;
  }
  return ACCENT_COLORS.find((c) => c.id === DEFAULT_ID)!;
}

/** Setzt die CSS-Variablen auf <html>. */
function applyAccent(color: AccentColor) {
  if (!browser) return;
  const root = document.documentElement;
  root.style.setProperty("--accent-500", color.shade500);
  root.style.setProperty("--accent-600", color.shade600);
}

function createAccentStore() {
  const { subscribe, set } = writable<AccentColor>(initialAccent());

  return {
    subscribe,
    /** Wählt eine Farbe per ID. */
    select(id: string) {
      const color = ACCENT_COLORS.find((c) => c.id === id);
      if (!color) return;
      if (browser) localStorage.setItem(STORAGE_KEY, id);
      applyAccent(color);
      set(color);
    },
    /** Wendet die initial geladene Farbe an (beim App-Start aufrufen). */
    init() {
      applyAccent(initialAccent());
    },
  };
}

export const accent = createAccentStore();
