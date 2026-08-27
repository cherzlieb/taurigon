<script lang="ts">
  import { Database } from "lucide-svelte";
</script>

<main class="mx-auto max-w-4xl px-8 py-8">
  <header class="mb-8">
    <h1 class="text-2xl font-bold tracking-tight">Datenbanken</h1>
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Datenbanken und Benutzer verwalten
    </p>
  </header>

  <div
    class="flex flex-col items-center justify-center rounded-xl border
           border-dashed border-gray-300 py-16 text-center
           dark:border-gray-700"
  >
    <Database size={40} class="mb-3 text-gray-400" />
    <p class="font-medium text-gray-600 dark:text-gray-300">
      Datenbank-Verwaltung
    </p>
    <p class="mt-1 text-sm text-gray-500">
      Hier kommt später der DB-Wizard (DB + Benutzer anlegen).
    </p>
  </div>
</main>
