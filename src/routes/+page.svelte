<script lang="ts">
  import { onMount } from "svelte";
  import {
    Container,
    ShieldCheck,
    ShieldAlert,
    HardDrive,
    CircleCheck,
    CircleX,
    RefreshCw,
  } from "@lucide/svelte";
  import {
    systemInfo,
    systemLoading,
    systemError,
    ensureSystemInfo,
    loadSystemInfo,
  } from "$lib/stores/system";

  // Beim ersten Besuch laden, bei weiteren Cache nutzen (+ Hintergrund-Refresh).
  onMount(ensureSystemInfo);
</script>

<main class="mx-auto max-w-4xl px-8 py-8">
  <header class="mb-8 flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Dashboard</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400">
        Systemübersicht deiner Entwicklungsumgebung
      </p>
    </div>
    <button
      class="flex items-center gap-2 rounded-lg bg-gray-200 px-3 py-2 text-sm
             font-medium transition hover:bg-gray-300
             dark:bg-gray-800 dark:hover:bg-gray-700"
      on:click={() => loadSystemInfo(true)}
      disabled={$systemLoading}
    >
      <RefreshCw size={16} class={$systemLoading ? "animate-spin" : ""} />
      Neu prüfen
    </button>
  </header>

  {#if $systemLoading && !$systemInfo}
    <div class="flex items-center gap-3 text-gray-500 dark:text-gray-400">
      <RefreshCw size={20} class="animate-spin" />
      System wird analysiert …
    </div>
  {:else if $systemError && !$systemInfo}
    <div
      class="rounded-lg border border-status-stopped/30 bg-status-stopped/10
             p-4 text-status-stopped"
    >
      <div class="flex items-center gap-2 font-semibold">
        <CircleX size={20} />
        Fehler bei der Systemanalyse
      </div>
      <p class="mt-1 text-sm">{$systemError}</p>
    </div>
  {:else if $systemInfo}
    <!-- Gesamtstatus-Banner -->
    <div
      class="mb-6 flex items-center gap-3 rounded-xl border p-4
             {$systemInfo.is_ready
        ? 'border-status-running/30 bg-status-running/10'
        : 'border-status-stopped/30 bg-status-stopped/10'}"
    >
      {#if $systemInfo.is_ready}
        <CircleCheck size={28} class="text-status-running" />
        <div>
          <p class="font-semibold text-status-running">System bereit</p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            Eine Container-Engine wurde gefunden. Du kannst loslegen.
          </p>
        </div>
      {:else}
        <CircleX size={28} class="text-status-stopped" />
        <div>
          <p class="font-semibold text-status-stopped">
            Keine Container-Engine gefunden
          </p>
          <p class="text-sm text-gray-600 dark:text-gray-400">
            Bitte installiere Podman (empfohlen) oder Docker.
          </p>
        </div>
      {/if}
    </div>

    <!-- Info-Karten -->
    <div class="grid gap-4 sm:grid-cols-2">
      <!-- Engine -->
      <div
        class="rounded-xl border border-gray-200 bg-white p-4
               dark:border-gray-800 dark:bg-gray-900"
      >
        <div
          class="mb-2 flex items-center gap-2 text-gray-500 dark:text-gray-400"
        >
          <Container size={18} />
          <span class="text-sm font-medium">Container-Engine</span>
        </div>
        {#if $systemInfo.engine_kind}
          <p class="text-lg font-semibold capitalize">
            {$systemInfo.engine_kind}
            <span class="text-sm font-normal text-gray-500 lowercase">
              v{$systemInfo.engine_version}
            </span>
          </p>
        {:else}
          <p class="text-lg font-semibold text-status-stopped">Nicht gefunden</p>
        {/if}
      </div>

      <!-- Sicherheitsmodus -->
      <div
        class="rounded-xl border border-gray-200 bg-white p-4
               dark:border-gray-800 dark:bg-gray-900"
      >
        <div
          class="mb-2 flex items-center gap-2 text-gray-500 dark:text-gray-400"
        >
          {#if $systemInfo.engine_mode === "rootless"}
            <ShieldCheck size={18} />
          {:else}
            <ShieldAlert size={18} />
          {/if}
          <span class="text-sm font-medium">Sicherheitsmodus</span>
        </div>
        {#if $systemInfo.engine_mode === "rootless"}
          <p class="text-lg font-semibold text-status-running">Rootless</p>
        {:else if $systemInfo.engine_mode === "root-equivalent"}
          <p class="text-lg font-semibold text-status-pending">
            Root-äquivalent
          </p>
        {:else}
          <p class="text-lg font-semibold text-gray-400">—</p>
        {/if}
      </div>

      <!-- Distribution -->
      <div
        class="rounded-xl border border-gray-200 bg-white p-4
               dark:border-gray-800 dark:bg-gray-900"
      >
        <div
          class="mb-2 flex items-center gap-2 text-gray-500 dark:text-gray-400"
        >
          <HardDrive size={18} />
          <span class="text-sm font-medium">Distribution</span>
        </div>
        <p class="text-lg font-semibold capitalize">{$systemInfo.distro}</p>
      </div>

      <!-- SELinux -->
      <div
        class="rounded-xl border border-gray-200 bg-white p-4
               dark:border-gray-800 dark:bg-gray-900"
      >
        <div
          class="mb-2 flex items-center gap-2 text-gray-500 dark:text-gray-400"
        >
          <ShieldCheck size={18} />
          <span class="text-sm font-medium">SELinux</span>
        </div>
        <p class="text-lg font-semibold">
          {$systemInfo.selinux_enforcing ? "Enforcing" : "Inaktiv / Permissive"}
        </p>
        {#if $systemInfo.selinux_enforcing}
          <p class="mt-1 text-xs text-gray-500">
            Volume-Mounts nutzen automatisch das <code>:Z</code>-Flag.
          </p>
        {/if}
      </div>
    </div>
  {/if}
</main>
