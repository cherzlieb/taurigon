<script lang="ts">
  import { page } from "$app/state";
  import { theme } from "$lib/stores/theme";
  import {
    LayoutDashboard,
    Server,
    FolderGit2,
    Database,
    Settings,
    Sun,
    Moon,
  } from "@lucide/svelte";

  const navItems = [
    { href: "/", label: "Dashboard", icon: LayoutDashboard },
    { href: "/services", label: "Dienste", icon: Server },
    { href: "/projects", label: "Projekte", icon: FolderGit2 },
    { href: "/databases", label: "Datenbanken", icon: Database },
    { href: "/settings", label: "Einstellungen", icon: Settings },
  ];

  function isActive(href: string, currentPath: string): boolean {
    if (href === "/") return currentPath === "/";
    return currentPath.startsWith(href);
  }
</script>

<aside
  class="flex h-screen w-60 flex-col border-r border-gray-200 bg-white
         dark:border-gray-800 dark:bg-gray-900"
>
  <!-- Logo / Titel -->
  <div class="flex items-center gap-2 px-5 py-5">
    <div
      class="flex h-9 w-9 items-center justify-center rounded-lg
             bg-accent-600 text-white font-bold"
    >
      T
    </div>
    <div>
      <p class="font-bold leading-tight">Taurigon</p>
      <p class="text-xs text-gray-500">Dev Manager</p>
    </div>
  </div>

  <!-- Navigation -->
  <nav class="flex-1 space-y-1 px-3 py-2">
    {#each navItems as item (item.href)}
      {@const active = isActive(item.href, page.url.pathname)}
      <a
        href={item.href}
        class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium
               transition"
        class:bg-accent-600={active}
        class:text-white={active}
        class:text-gray-700={!active}
        class:dark:text-gray-300={!active}
        class:hover:bg-gray-100={!active}
        class:dark:hover:bg-gray-800={!active}
      >
        <svelte:component this={item.icon} size={18} />
        {item.label}
      </a>
    {/each}
  </nav>

  <!-- Theme-Toggle -->
  <div class="border-t border-gray-200 p-3 dark:border-gray-800">
    <button
      class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm
             font-medium text-gray-700 transition hover:bg-gray-100
             dark:text-gray-300 dark:hover:bg-gray-800"
      on:click={() => theme.toggle()}
    >
      {#if $theme === "dark"}
        <Sun size={18} /> Light Mode
      {:else}
        <Moon size={18} /> Dark Mode
      {/if}
    </button>
  </div>
</aside>
