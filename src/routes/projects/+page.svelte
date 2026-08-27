<script lang="ts">
  import { onMount } from "svelte";
  import { FolderGit2, Plus, Code2, Trash2, Globe, ChevronDown, ExternalLink, Power } from "lucide-svelte";
  import {
    projects,
    projectsLoading,
    ensureProjects,
    createProject,
    deleteProject,
    openInEditor,
    webRunning,
    webBusy,
    loadWebStatus,
    startWeb,
    stopWeb,
    openProject,
  } from "$lib/stores/projects";
  import { editorCommand } from "$lib/stores/settings";
  import Select from "$lib/components/Select.svelte";

  // Formular-State
  let name = "";
  let projectType: "php" | "static" = "php";
  let phpVersion = "8.3";
  let creating = false;
  let formError: string | null = null;

  const phpVersions = ["8.2", "8.3", "8.4"];

  // Fehler automatisch löschen, sobald der Nutzer etwas eintippt.
  $: if (name.trim()) formError = null;

  async function handleCreate() {
    formError = null;
    if (!name.trim()) {
      formError = "Bitte einen Projektnamen eingeben.";
      return;
    }
    creating = true;
    try {
      await createProject(
        name,
        projectType,
        projectType === "php" ? phpVersion : null,
      );
      name = ""; // Formular zurücksetzen
    } catch (e) {
      formError = String(e);
    } finally {
      creating = false;
    }
  }

  // Options für die Dropdowns (Label/Value-Paare).
  const typeOptions = [
    { value: "php", label: "PHP" },
    { value: "static", label: "Statisch (HTML)" },
  ];
  const phpVersionOptions = phpVersions.map((v) => ({ value: v, label: v }));

  onMount(async () => {
    await ensureProjects();
    await loadWebStatus();
  });
</script>

<main class="mx-auto max-w-4xl px-8 py-8">
  <header class="mb-6">
    <h1 class="text-2xl font-bold tracking-tight">Projekte</h1>
    <p class="text-sm text-gray-500 dark:text-gray-400">
      Web-Projekte mit .localhost-Domains verwalten
    </p>
  </header>

  <!-- Web-Server-Steuerung -->
  <section
    class="mb-6 flex items-center justify-between rounded-xl border
           border-gray-200 bg-white p-4 dark:border-gray-800 dark:bg-gray-900"
  >
    <div class="flex items-center gap-3">
      <span
        class="h-2.5 w-2.5 rounded-full"
        class:bg-status-running={$webRunning}
        class:bg-status-stopped={!$webRunning}
      ></span>
      <div>
        <p class="text-sm font-medium">
          Webserver {$webRunning ? "läuft" : "gestoppt"}
        </p>
        <p class="text-xs text-gray-500">Nginx-Proxy auf Port 8080</p>
      </div>
    </div>
    {#if $webRunning}
      <button
        class="flex items-center gap-2 rounded-lg bg-status-stopped/10 px-3 py-2
               text-sm font-medium text-status-stopped transition
               hover:bg-status-stopped/20 disabled:opacity-50"
        on:click={stopWeb}
        disabled={$webBusy}
      >
        <Power size={15} /> Stoppen
      </button>
    {:else}
      <button
        class="flex items-center gap-2 rounded-lg bg-status-running/10 px-3 py-2
               text-sm font-medium text-status-running transition
               hover:bg-status-running/20 disabled:opacity-50"
        on:click={startWeb}
        disabled={$webBusy}
      >
        <Power size={15} /> {$webBusy ? "Startet …" : "Starten"}
      </button>
    {/if}
  </section>

  <!-- Neues Projekt anlegen -->
  <section
    class="mb-8 rounded-xl border border-gray-200 bg-white p-5
           dark:border-gray-800 dark:bg-gray-900"
  >
    <h2 class="mb-4 font-semibold">Neues Projekt</h2>

    <div class="flex flex-wrap items-end gap-3">
      <!-- Name -->
      <div class="flex-1 min-w-[200px]">
        <label
          for="project-name"
          class="mb-1 block text-xs font-medium text-gray-500"
        >
          Projektname
        </label>
        <input
          id="project-name"
          type="text"
          bind:value={name}
          placeholder="myapp"
          class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3
                 text-sm outline-none transition focus:border-accent-500
                 dark:border-gray-700 dark:bg-gray-800"
          on:keydown={(e) => e.key === "Enter" && handleCreate()}
        />
      </div>

      <!-- Typ -->
      <div>
        <label
          for="project-type"
          class="mb-1 block text-xs font-medium text-gray-500"
        >
          Typ
        </label>
        <div class="w-44">
          <Select
            id="project-type"
            options={typeOptions}
            bind:value={projectType}
          />
        </div>
      </div>

      <!-- PHP-Version (nur bei PHP) -->
      {#if projectType === "php"}
        <div>
          <label
            for="php-version"
            class="mb-1 block text-xs font-medium text-gray-500"
          >
            PHP-Version
          </label>
          <div class="w-24">
            <Select
              id="php-version"
              options={phpVersionOptions}
              bind:value={phpVersion}
            />
          </div>
        </div>
      {/if}

      <!-- Anlegen -->
      <button
        class="flex h-10 items-center gap-2 rounded-lg bg-accent-600 px-4
               text-sm font-medium text-white transition hover:bg-accent-500
               disabled:opacity-50"
        on:click={handleCreate}
        disabled={creating}
      >
        <Plus size={16} />
        {creating ? "Erstelle …" : "Anlegen"}
      </button>
    </div>

    <!-- Domain-Vorschau + Fehler in eigener Zeile (kein Springen des Layouts) -->
    <div class="mt-2 min-h-[1.25rem] text-xs">
      {#if formError}
        <span class="text-status-stopped">{formError}</span>
      {:else if name.trim()}
        <span class="text-gray-500">
          → {name.trim().toLowerCase()}.localhost
        </span>
      {/if}
    </div>
  </section>

  <!-- Projektliste -->
  {#if $projectsLoading && $projects.length === 0}
    <p class="text-gray-500">Projekte werden geladen …</p>
  {:else if $projects.length === 0}
    <div
      class="flex flex-col items-center justify-center rounded-xl border
             border-dashed border-gray-300 py-16 text-center
             dark:border-gray-700"
    >
      <FolderGit2 size={40} class="mb-3 text-gray-400" />
      <p class="font-medium text-gray-600 dark:text-gray-300">
        Noch keine Projekte
      </p>
      <p class="mt-1 text-sm text-gray-500">
        Lege oben dein erstes Projekt an.
      </p>
    </div>
  {:else}
    <div class="space-y-3">
      {#each $projects as project (project.id)}
        <div
          class="flex items-center justify-between rounded-xl border
                 border-gray-200 bg-white p-4
                 dark:border-gray-800 dark:bg-gray-900"
        >
          <div class="flex items-center gap-3">
            <div
              class="flex h-10 w-10 items-center justify-center rounded-lg
                     bg-accent-600/10 text-accent-600"
            >
              <FolderGit2 size={20} />
            </div>
            <div>
              <p class="font-semibold">{project.name}</p>
              <div class="flex items-center gap-2 text-xs text-gray-500">
                <Globe size={12} />
                {project.domain}
                <span
                  class="rounded bg-gray-100 px-1.5 py-0.5 dark:bg-gray-800"
                >
                  {project.project_type === "php"
                    ? `PHP ${project.php_version}`
                    : "Statisch"}
                </span>
              </div>
            </div>
          </div>

          <div class="flex gap-2">
            <button
              class="flex items-center gap-1.5 rounded-lg bg-accent-600 px-3 py-2
                     text-sm font-medium text-white transition
                     hover:bg-accent-500"
              on:click={() => openProject(project.domain)}
              title="Im Browser öffnen"
            >
              <ExternalLink size={15} /> Öffnen
            </button>
            <button
              class="flex items-center gap-1.5 rounded-lg bg-gray-200 px-3 py-2
                     text-sm font-medium transition hover:bg-gray-300
                     dark:bg-gray-800 dark:hover:bg-gray-700"
              on:click={() => openInEditor(project.path, $editorCommand)}
              title="Im Editor öffnen"
            >
              <Code2 size={15} /> Editor
            </button>
            <button
              class="flex items-center justify-center rounded-lg
                     bg-status-stopped/10 px-3 py-2 text-status-stopped
                     transition hover:bg-status-stopped/20"
              on:click={() => deleteProject(project.id, project.name)}
              title="Projekt löschen"
            >
              <Trash2 size={15} />
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</main>
