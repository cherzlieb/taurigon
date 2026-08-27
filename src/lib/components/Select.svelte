<script lang="ts">
  import { ChevronDown, Check } from "lucide-svelte";
  import { createEventDispatcher } from "svelte";

  /** Eine auswählbare Option. */
  type Option = { value: string; label: string };

  /** Die verfügbaren Optionen. */
  export let options: Option[] = [];
  /** Der aktuell gewählte Wert (bindbar). */
  export let value: string = "";
  /** Optionale ID für Label-Verknüpfung. */
  export let id: string | undefined = undefined;

  const dispatch = createEventDispatcher<{ change: string }>();

  let open = false;
  let container: HTMLDivElement;

  /** Das Label zur aktuellen Auswahl. */
  $: selectedLabel =
    options.find((o) => o.value === value)?.label ?? "Auswählen …";

  /** Wählt eine Option und schließt das Dropdown. */
  function select(v: string) {
    value = v;
    open = false;
    dispatch("change", v);
  }

  /** Schließt bei Klick außerhalb. */
  function handleClickOutside(event: MouseEvent) {
    if (container && !container.contains(event.target as Node)) {
      open = false;
    }
  }

  /** Tastatur: Escape schließt. */
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") open = false;
  }
</script>

<svelte:window on:click={handleClickOutside} on:keydown={handleKeydown} />

<div class="relative" bind:this={container}>
  <!-- Trigger-Button -->
  <button
    {id}
    type="button"
    class="flex h-10 w-full items-center justify-between gap-2 rounded-lg
           border border-gray-300 bg-white px-3 text-sm text-gray-900
           outline-none transition focus:border-accent-500
           dark:border-gray-700 dark:bg-gray-800 dark:text-gray-100"
    class:border-accent-500={open}
    on:click={() => (open = !open)}
  >
    <span>{selectedLabel}</span>
    <ChevronDown
      size={16}
      class="text-gray-400 transition-transform {open ? 'rotate-180' : ''}"
    />
  </button>

  <!-- Dropdown-Liste -->
  {#if open}
    <ul
      class="absolute z-50 mt-1 w-full overflow-hidden rounded-lg border
             border-gray-200 bg-white py-1 shadow-lg
             dark:border-gray-700 dark:bg-gray-800"
      role="listbox"
    >
      {#each options as option (option.value)}
        {@const isSelected = option.value === value}
        <li role="option" aria-selected={isSelected}>
          <button
            type="button"
            class="flex w-full items-center justify-between px-3 py-2 text-left
                   text-sm transition hover:bg-gray-100 dark:hover:bg-gray-700"
            class:text-accent-600={isSelected}
            class:font-medium={isSelected}
            on:click={() => select(option.value)}
          >
            {option.label}
            {#if isSelected}
              <Check size={15} />
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>
