import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

// @ts-expect-error – process ist in Node-Kontext vorhanden.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [sveltekit()],

  // Tauri erwartet einen festen Dev-Port (siehe tauri.conf.json -> devUrl).
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421  }
      : undefined,
    watch: {
      // src-tauri wird von Cargo überwacht, nicht von Vite.
      ignored: ["**/src-tauri/**"],
    },
  },
});
