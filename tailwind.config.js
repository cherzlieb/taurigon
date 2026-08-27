/** @type {import('tailwindcss').Config} */
export default {
  // "class"-Strategie: Dark-Mode wird über <html class="dark"> gesteuert,
  // damit wir später einen Theme-Toggle bauen können.
  darkMode: "class",
  content: ["./src/**/*.{html,js,svelte,ts}"],
  theme: {
    extend: {
      colors: {
        // Akzentfarbe über CSS-Variablen (zur Laufzeit wechselbar).
        // <alpha-value> ermöglicht Opacity-Modifier wie accent/10.
        accent: {
          500: "rgb(var(--accent-500) / <alpha-value>)",
          600: "rgb(var(--accent-600) / <alpha-value>)",
          DEFAULT: "rgb(var(--accent-600) / <alpha-value>)",
        },
        // Semantische Status-Farben für Dienst-Indikatoren.
        status: {
          running: "#22c55e", // grün
          stopped: "#ef4444", // rot
          pending: "#f59e0b", // orange
          unknown: "#6b7280", // grau
        },
      },
    },
  },
  plugins: [],
};
