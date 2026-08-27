// SSR MUSS aus sein: Tauri lädt statische Dateien, es gibt keinen Node-Server.
export const ssr = false;

// Prerendering aus – die App ist dynamisch (Client-only).
export const prerender = false;
