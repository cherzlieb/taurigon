<script lang="ts">
  import { Check, Sun, Moon, Code2 } from "lucide-svelte";
  import { theme } from "$lib/stores/theme";
  import { accent, ACCENT_COLORS } from "$lib/stores/accent";
  import { editorCommand } from "$lib/stores/settings";
</script>

<main class="mx-auto max-w-4xl px-8 py-8">
  <header class="mb-8">
    <h1 class="text-2xl font-bold tracking-tight">Einstellungen</h1>
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Passe das Erscheinungsbild von Taurigon an
    </p>
  </header>

  <!-- Erscheinungsbild (Theme) -->
  <section class="mb-8">
    <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">
      Erscheinungsbild
    </h2>
    <div
      class="rounded-xl border border-gray-200 bg-white p-4
             dark:border-gray-800 dark:bg-gray-900"
    >
      <div class="flex gap-3">
        <!-- Light -->
        <button
          class="flex flex-1 items-center justify-center gap-2 rounded-lg
                 border px-4 py-3 text-sm font-medium transition"
          class:border-accent-600={$theme === "light"}
          class:bg-accent-600={$theme === "light"}
          class:text-white={$theme === "light"}
          class:border-gray-200={$theme !== "light"}
          class:dark:border-gray-700={$theme !== "light"}
          class:hover:bg-gray-100={$theme !== "light"}
          class:dark:hover:bg-gray-800={$theme !== "light"}
          on:click={() => theme.set("light")}
        >
          <Sun size={18} /> Hell
        </button>

        <!-- Dark -->
        <button
          class="flex flex-1 items-center justify-center gap-2 rounded-lg
                 border px-4 py-3 text-sm font-medium transition"
          class:border-accent-600={$theme === "dark"}
          class:bg-accent-600={$theme === "dark"}
          class:text-white={$theme === "dark"}
          class:border-gray-200={$theme !== "dark"}
          class:dark:border-gray-700={$theme !== "dark"}
          class:hover:bg-gray-100={$theme !== "dark"}
          class:dark:hover:bg-gray-800={$theme !== "dark"}
          on:click={() => theme.set("dark")}
        >
          <Moon size={18} /> Dunkel
        </button>
      </div>
    </div>
  </section>

  <!-- Akzentfarbe -->
  <section>
    <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">
      Akzentfarbe
    </h2>
    <div
      class="rounded-xl border border-gray-200 bg-white p-5
             dark:border-gray-800 dark:bg-gray-900"
    >
      <div class="grid grid-cols-5 gap-4 sm:grid-cols-10">
        {#each ACCENT_COLORS as color (color.id)}
          {@const selected = $accent.id === color.id}
          <button
            class="group relative flex aspect-square items-center justify-center
                   rounded-full transition hover:scale-110"
            style="background-color: {color.preview}"
            on:click={() => accent.select(color.id)}
            title={color.name}
            aria-label={color.name}
          >
            {#if selected}
              <Check size={18} class="text-white" strokeWidth={3} />
            {/if}
          </button>
        {/each}
      </div>

      <p class="mt-4 text-sm text-gray-500">
        Ausgewählt: <span class="font-medium">{$accent.name}</span>
      </p>
    </div>
  </section>
  <!-- Editor -->
  <section class="mt-8">
    <h2 class="mb-3 text-sm font-semibold uppercase tracking-wide text-gray-500">
      Editor
    </h2>
    <div
      class="rounded-xl border border-gray-200 bg-white p-5
            dark:border-gray-800 dark:bg-gray-900"
    >
      <label class="mb-2 flex items-center gap-2 text-sm font-medium">
        <Code2 size={16} /> Editor-Befehl
      </label>
      <input
        type="text"
        bind:value={$editorCommand}
        placeholder="code {'{path}'}"
        class="w-full rounded-lg border border-gray-300 bg-white px-3 py-2
              text-sm outline-none focus:border-accent-500
              dark:border-gray-700 dark:bg-gray-800"
      />
      <p class="mt-2 text-xs text-gray-500">
        Befehl zum Öffnen eines Projekts. <code>{"{path}"}</code> wird durch den
        Projektpfad ersetzt (sonst angehängt).
        Beispiele: <code>code {"{path}"}</code>, <code>subl {"{path}"}</code>,
        <code>zed {"{path}"}</code>. Terminal-Editoren brauchen einen Wrapper,
        z. B. <code>kitty nvim {"{path}"}</code>.
      </p>
    </div>
  </section>
</main>
