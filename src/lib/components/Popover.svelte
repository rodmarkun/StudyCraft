<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let open = false;
  export let width = 'w-64';
  export let customClass = '';

  const dispatch = createEventDispatcher();

  function handleClickOutside(event: MouseEvent) {
    if (open && !event.target.closest('.popover-content') && !event.target.closest('.popover-trigger')) {
      dispatch('close');
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="relative inline-block">
  <div class="popover-trigger">
    <slot name="trigger" />
  </div>
  
  {#if open}
    <div class={`fixed z-50 mt-2 ${width} rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 popover-content ${customClass}`} style="max-height: 80vh; overflow-y: auto;">
      <div class="p-4">
        <slot />
      </div>
    </div>
  {/if}
</div>