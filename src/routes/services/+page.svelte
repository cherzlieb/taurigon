<script lang="ts">
  import { onMount } from "svelte";
  import { Play, Square, RotateCw, Trash2 } from "lucide-svelte";
  import {
    services,
    statuses,
    busy,
    servicesLoading,
    servicesRefreshing,
    ensureServices,
    refreshStatuses,
    serviceAction,
    removeService,
  } from "$lib/stores/services";

  // Erstladung beim ersten Besuch, danach Cache + Hintergrund-Refresh.
  onMount(ensureServices);
</script>

<main class="mx-auto max-w-4xl px-8 py-8">
  <header class="mb-6 flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Dienste</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400">
        Datenbanken und Caching-Dienste verwalten
      </p>
    </div>
      <button
        class="flex items-center gap-2 rounded-lg bg-gray-200 px-3 py-1.5
              text-xs font-medium transition hover:bg-gray-300
              disabled:opacity-50 dark:bg-gray-800 dark:hover:bg-gray-700"
        on:click={refreshStatuses}
        disabled={$servicesRefreshing}
      >
        <RotateCw size={14} class={$servicesRefreshing ? "animate-spin" : ""} />
        {$servicesRefreshing ? "Aktualisiere …" : "Status aktualisieren"}
      </button>
  </header>

  {#if $servicesLoading && $services.length === 0}
    <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
      <RotateCw size={20} class="animate-spin" />
      Dienste werden geladen …
    </div>
  {:else}
    <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
      {#each $services as svc (svc.id)}
        {@const status = $statuses[svc.id]}
        {@const state = status?.state ?? "not_found"}
        {@const isRunning = state === "running"}
        {@const isBusy = $busy[svc.id] ?? false}

        <div
          class="rounded-xl border border-gray-200 bg-white p-4
                dark:border-gray-800 dark:bg-gray-900"
        >
          <!-- Kopf: Name + Status-Indikator -->
          <div class="mb-3 flex items-center justify-between">
            <div>
              <h3 class="font-semibold">{svc.name}</h3>
              <p class="text-xs text-gray-500">Port {svc.host_port}</p>
            </div>
            <span
              class="inline-flex items-center gap-1.5 text-xs font-medium"
              class:text-status-running={isRunning}
              class:text-status-stopped={state === "stopped"}
              class:text-status-unknown={state === "not_found"}
            >
              <span
                class="h-2.5 w-2.5 rounded-full"
                class:bg-status-running={isRunning}
                class:bg-status-stopped={state === "stopped"}
                class:bg-status-unknown={state === "not_found"}
              ></span>
              {#if isRunning}Läuft
              {:else if state === "stopped"}Gestoppt
              {:else}Nicht erstellt{/if}
            </span>
          </div>

          <!-- Aktions-Buttons -->
          <div class="flex gap-2">
            {#if isRunning}
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-lg
                      bg-status-stopped/10 px-3 py-2 text-sm font-medium
                      text-status-stopped transition hover:bg-status-stopped/20
                      disabled:opacity-50"
                on:click={() => serviceAction(svc.id, "stop")}
                disabled={isBusy}
              >
                <Square size={15} /> Stoppen
              </button>
              <button
                class="flex items-center justify-center rounded-lg
                      bg-gray-200 px-3 py-2 text-sm transition
                      hover:bg-gray-300 disabled:opacity-50
                      dark:bg-gray-800 dark:hover:bg-gray-700"
                on:click={() => serviceAction(svc.id, "restart")}
                disabled={isBusy}
                title="Neu starten"
              >
                <RotateCw size={15} class={isBusy ? "animate-spin" : ""} />
              </button>
            {:else}
              <button
                class="flex flex-1 items-center justify-center gap-1.5 rounded-lg
                      bg-status-running/10 px-3 py-2 text-sm font-medium
                      text-status-running transition hover:bg-status-running/20
                      disabled:opacity-50"
                on:click={() => serviceAction(svc.id, "start")}
                disabled={isBusy}
              >
                {#if isBusy}
                  <RotateCw size={15} class="animate-spin" /> Startet …
                {:else}
                  <Play size={15} /> Starten
                {/if}
              </button>

              {#if state === "stopped"}
                <button
                  class="flex items-center justify-center rounded-lg
                        bg-status-stopped/10 px-3 py-2 text-sm transition
                        hover:bg-status-stopped/20 text-status-stopped
                        disabled:opacity-50"
                  on:click={() => removeService(svc.id, svc.name)}
                  disabled={isBusy}
                  title="Container entfernen"
                >
                  <Trash2 size={15} />
                </button>
              {/if}
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>
