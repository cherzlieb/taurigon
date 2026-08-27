<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { theme } from "$lib/stores/theme";
  import { accent } from "$lib/stores/accent";
  import Sidebar from "$lib/components/Sidebar.svelte";

  onMount(() => {
    theme.init();
    accent.init();
  });
</script>

<div class="flex h-screen overflow-hidden">
  <Sidebar />
  <div class="flex-1 overflow-y-auto">
    <slot />
  </div>
</div>
