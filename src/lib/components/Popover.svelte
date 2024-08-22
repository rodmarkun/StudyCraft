<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';

  export let open = false;
  export let width = 'w-64';
  export let customClass = '';

  const dispatch = createEventDispatcher();

  let triggerEl: HTMLElement;
  let popoverEl: HTMLElement;

  function handleClickOutside(event: MouseEvent) {
    if (!open) return;
    
    if (popoverEl && !popoverEl.contains(event.target as Node) && !triggerEl.contains(event.target as Node)) {
      event.preventDefault();
      event.stopPropagation();
      open = false;
      dispatch('close');
    }
  }

  function positionPopover() {
    if (open && triggerEl && popoverEl) {
      const rect = triggerEl.getBoundingClientRect();
      popoverEl.style.position = 'fixed';
      popoverEl.style.top = `${rect.bottom}px`;
      popoverEl.style.left = `${rect.left}px`;
      // Adjust if popover is off-screen
      const popoverRect = popoverEl.getBoundingClientRect();
      if (popoverRect.right > window.innerWidth) {
        popoverEl.style.left = `${window.innerWidth - popoverRect.width}px`;
      }
      if (popoverRect.bottom > window.innerHeight) {
        popoverEl.style.top = `${rect.top - popoverRect.height}px`;
      }
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside, true);
    window.addEventListener('resize', positionPopover);
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside, true);
    window.removeEventListener('resize', positionPopover);
  });

  $: if (open) {
    setTimeout(positionPopover, 0);
  }
</script>

<div class="relative inline-block" bind:this={triggerEl}>
  <div class="popover-trigger">
    <slot name="trigger" />
  </div>
 
  {#if open}
    <div
      bind:this={popoverEl}
      class={`fixed z-[60] ${width} rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 popover-content ${customClass}`}
      style="max-height: 80vh; overflow-y: auto;"
    >
      <div class="p-4">
        <slot />
      </div>
    </div>
  {/if}
</div>

<style>
  .popover-content {
    scrollbar-width: thin;
    scrollbar-color: rgba(155, 155, 155, 0.5) transparent;
  }
  .popover-content::-webkit-scrollbar {
    width: 6px;
  }
  .popover-content::-webkit-scrollbar-track {
    background: transparent;
  }
  .popover-content::-webkit-scrollbar-thumb {
    background-color: rgba(155, 155, 155, 0.5);
    border-radius: 20px;
    border: transparent;
  }
</style>
