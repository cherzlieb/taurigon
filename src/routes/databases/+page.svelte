<script lang="ts">
  import { onMount } from "svelte";
  import { Database, Plus, Trash2, RotateCw, UserPlus, X } from "@lucide/svelte";
  import Select from "$lib/components/Select.svelte";
  import {
    databases,
    dbAvailable,
    dbLoading,
    ensureDatabases,
    loadDatabases,
    createDatabase,
    dropDatabase,
    createDbUser,
    type DbKind,
  } from "$lib/stores/databases";

  const engines: { id: DbKind; name: string }[] = [
    { id: "mariadb", name: "MariaDB" },
    { id: "postgres", name: "PostgreSQL" },
  ];

  // Formular: neue Datenbank
  let newDbKind: DbKind = "mariadb";
  let newDbName = "";
  let creatingDb = false;
  let dbError: string | null = null;

  const engineOptions = engines.map((e) => ({ value: e.id, label: e.name }));

  async function handleCreateDb() {
      dbError = null;

      // Erst prüfen, ob der Dienst überhaupt läuft.
      if (!$dbAvailable[newDbKind]) {
        const engineName =
          engines.find((e) => e.id === newDbKind)?.name ?? newDbKind;
        dbError = `${engineName}-Dienst läuft nicht. Bitte zuerst unter „Dienste" starten.`;
        return;
      }

      if (!newDbName.trim()) {
        dbError = "Bitte einen Namen eingeben.";
        return;
      }
      creatingDb = true;
      try {
        await createDatabase(newDbKind, newDbName.trim());
        newDbName = "";
      } catch (e) {
        dbError = String(e);
      } finally {
        creatingDb = false;
      }
    }

  // User-Dialog
  let userDialog: { kind: DbKind; database: string } | null = null;
  let userName = "";
  let userPass = "";
  let creatingUser = false;
  let userError: string | null = null;
  let userSuccess = false;

  function openUserDialog(kind: DbKind, database: string) {
    userDialog = { kind, database };
    userName = "";
    userPass = "";
    userError = null;
    userSuccess = false;
  }

  async function handleCreateUser() {
    userError = null;
    if (!userName.trim() || !userPass) {
      userError = "Benutzername und Passwort erforderlich.";
      return;
    }
    creatingUser = true;
    try {
      await createDbUser(
        userDialog!.kind,
        userDialog!.database,
        userName.trim(),
        userPass,
      );
      userSuccess = true;
    } catch (e) {
      userError = String(e);
    } finally {
      creatingUser = false;
    }
  }

  $: if (newDbName.trim()) dbError = null;

  onMount(ensureDatabases);
</script>

<main class="mx-auto max-w-4xl px-8 py-8">
  <header class="mb-6 flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold tracking-tight">Datenbanken</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400">
        Datenbanken und Benutzer verwalten
      </p>
    </div>
    <button
      class="flex items-center gap-2 rounded-lg bg-gray-200 px-3 py-1.5
             text-xs font-medium transition hover:bg-gray-300
             disabled:opacity-50 dark:bg-gray-800 dark:hover:bg-gray-700"
      on:click={loadDatabases}
      disabled={$dbLoading}
    >
      <RotateCw size={14} class={$dbLoading ? "animate-spin" : ""} />
      Aktualisieren
    </button>
  </header>

  <!-- Neue Datenbank anlegen -->
  <section
    class="mb-8 rounded-xl border border-gray-200 bg-white p-5
           dark:border-gray-800 dark:bg-gray-900"
  >
    <h2 class="mb-4 font-semibold">Neue Datenbank</h2>
    <div class="flex flex-wrap items-end gap-3">
      <div>
        <label
          for="db-engine"
          class="mb-1 block text-xs font-medium text-gray-500"
        >
          Engine
        </label>
        <div class="w-40">
          <Select id="db-engine" options={engineOptions} bind:value={newDbKind} />
        </div>
      </div>
      <div class="flex-1 min-w-[200px]">
        <label
          for="db-name"
          class="mb-1 block text-xs font-medium text-gray-500"
        >
          Datenbankname
        </label>
        <input
          id="db-name"
          type="text"
          bind:value={newDbName}
          placeholder="meine_app"
          class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3
                 text-sm outline-none focus:border-accent-500
                 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-100"
          on:keydown={(e) => e.key === "Enter" && handleCreateDb()}
        />
      </div>
      <button
        class="flex h-10 items-center gap-2 rounded-lg bg-accent-600 px-4
               text-sm font-medium text-white transition hover:bg-accent-500
               disabled:opacity-50"
        on:click={handleCreateDb}
        disabled={creatingDb}
      >
        <Plus size={16} />
        {creatingDb ? "Erstelle …" : "Anlegen"}
      </button>
    </div>
    <div class="mt-2 min-h-[1.25rem] text-xs">
      {#if dbError}
        <span class="text-status-stopped">{dbError}</span>
      {/if}
    </div>
  </section>

  <!-- Datenbank-Listen je Engine -->
  {#each engines as engine (engine.id)}
    <section class="mb-6">
      <div class="mb-3 flex items-center gap-2">
        <Database size={18} class="text-gray-500" />
        <h2 class="font-semibold">{engine.name}</h2>
        {#if !$dbAvailable[engine.id]}
          <span class="text-xs text-status-stopped">
            (Dienst nicht gestartet)
          </span>
        {/if}
      </div>

      {#if !$dbAvailable[engine.id]}
        <div
          class="rounded-xl border border-dashed border-gray-300 p-4
                 text-sm text-gray-500 dark:border-gray-700"
        >
          Starte den {engine.name}-Dienst unter „Dienste", um Datenbanken zu
          verwalten.
        </div>
      {:else if $databases[engine.id].length === 0}
        <div
          class="rounded-xl border border-dashed border-gray-300 p-4
                 text-sm text-gray-500 dark:border-gray-700"
        >
          Noch keine Datenbanken.
        </div>
      {:else}
        <div class="space-y-2">
          {#each $databases[engine.id] as db (db.name)}
            <div
              class="flex items-center justify-between rounded-xl border
                     border-gray-200 bg-white p-3
                     dark:border-gray-800 dark:bg-gray-900"
            >
              <div class="flex items-center gap-2">
                <Database size={16} class="text-accent-600" />
                <span class="font-medium">{db.name}</span>
              </div>
              <div class="flex gap-2">
                <button
                  class="flex items-center gap-1.5 rounded-lg bg-gray-200 px-3
                         py-1.5 text-sm transition hover:bg-gray-300
                         dark:bg-gray-800 dark:hover:bg-gray-700"
                  on:click={() => openUserDialog(engine.id, db.name)}
                  title="Benutzer anlegen"
                >
                  <UserPlus size={14} /> Benutzer
                </button>
                <button
                  class="flex items-center justify-center rounded-lg
                         bg-status-stopped/10 px-3 py-1.5 text-status-stopped
                         transition hover:bg-status-stopped/20"
                  on:click={() => dropDatabase(engine.id, db.name)}
                  title="Datenbank löschen"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/each}
</main>

<!-- Benutzer-Dialog -->
{#if userDialog}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
    on:click={() => (userDialog = null)}
    on:keydown={(e) => e.key === "Escape" && (userDialog = null)}
    role="button"
    tabindex="-1"
  >
    <div
      class="w-full max-w-md rounded-xl border border-gray-200 bg-white p-5
             dark:border-gray-800 dark:bg-gray-900"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      on:click|stopPropagation
      on:keydown|stopPropagation
    >
      <div class="mb-4 flex items-center justify-between">
        <h3 class="font-semibold">Benutzer für „{userDialog.database}"</h3>
        <button
          class="rounded p-1 text-gray-400 hover:bg-gray-100
                 dark:hover:bg-gray-800"
          on:click={() => (userDialog = null)}
        >
          <X size={18} />
        </button>
      </div>

      {#if userSuccess}
        <div
          class="rounded-lg bg-status-running/10 p-4 text-sm text-status-running"
        >
          Benutzer „{userName}" wurde angelegt und hat alle Rechte auf
          „{userDialog.database}".
        </div>
        <button
          class="mt-4 w-full rounded-lg bg-accent-600 px-4 py-2 text-sm
                 font-medium text-white transition hover:bg-accent-500"
          on:click={() => (userDialog = null)}
        >
          Schließen
        </button>
      {:else}
        <div class="space-y-3">
          <div>
            <label
              for="user-name"
              class="mb-1 block text-xs font-medium text-gray-500"
            >
              Benutzername
            </label>
            <input
              id="user-name"
              type="text"
              bind:value={userName}
              placeholder="app_user"
              class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3
                     text-sm outline-none focus:border-accent-500
                     dark:border-gray-700 dark:bg-gray-800 dark:text-gray-100"
            />
          </div>
          <div>
            <label
              for="user-pass"
              class="mb-1 block text-xs font-medium text-gray-500"
            >
              Passwort
            </label>
            <input
              id="user-pass"
              type="text"
              bind:value={userPass}
              placeholder="geheim"
              class="h-10 w-full rounded-lg border border-gray-300 bg-white px-3
                     text-sm outline-none focus:border-accent-500
                     dark:border-gray-700 dark:bg-gray-800 dark:text-gray-100"
            />
          </div>
        </div>

        {#if userError}
          <p class="mt-2 text-sm text-status-stopped">{userError}</p>
        {/if}

        <button
          class="mt-4 flex w-full items-center justify-center gap-2 rounded-lg
                 bg-accent-600 px-4 py-2 text-sm font-medium text-white
                 transition hover:bg-accent-500 disabled:opacity-50"
          on:click={handleCreateUser}
          disabled={creatingUser}
        >
          <UserPlus size={16} />
          {creatingUser ? "Erstelle …" : "Benutzer anlegen"}
        </button>
      {/if}
    </div>
  </div>
{/if}
