import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Ermöglicht TypeScript, PostCSS etc. in .svelte-Dateien.
  preprocess: vitePreprocess(),

  kit: {
    // static-adapter: erzeugt eine reine SPA (keine Server-Runtime),
    // genau das, was Tauri als "frontendDist" einbindet.
    adapter: adapter({
      pages: "build",
      assets: "build",
      // fallback = SPA-Modus: alle Routen laufen über index.html.
      fallback: "index.html",
      precompress: false,
      strict: true,
    }),
  },
};

export default config;
