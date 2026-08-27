/**
 * Theme-Store: verwaltet Dark-/Light-Mode.
 *
 * Persistiert die Wahl in localStorage und setzt die `dark`-Klasse auf
 * `<html>`, worauf Tailwind (darkMode: "class") reagiert.
 */
import { writable } from "svelte/store";
import { browser } from "$app/environment";

export type Theme = "light" | "dark";

/** Liest das gespeicherte Theme oder fällt auf "dark" zurück. */
function initialTheme(): Theme {
  if (!browser) return "dark";
  const stored = localStorage.getItem("theme");
  return stored === "light" || stored === "dark" ? stored : "dark";
}

/** Wendet das Theme auf das <html>-Element an. */
function applyTheme(theme: Theme) {
  if (!browser) return;
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

function createThemeStore() {
  const { subscribe, set, update } = writable<Theme>(initialTheme());

  return {
    subscribe,
    /** Setzt ein konkretes Theme. */
    set(theme: Theme) {
      if (browser) localStorage.setItem("theme", theme);
      applyTheme(theme);
      set(theme);
    },
    /** Wechselt zwischen Dark und Light. */
    toggle() {
      update((current) => {
        const next: Theme = current === "dark" ? "light" : "dark";
        if (browser) localStorage.setItem("theme", next);
        applyTheme(next);
        return next;
      });
    },
    /** Wendet das initial geladene Theme an (beim App-Start aufrufen). */
    init() {
      applyTheme(initialTheme());
    },
  };
}

export const theme = createThemeStore();
