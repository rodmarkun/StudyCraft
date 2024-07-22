<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  export let open = false;
  export let width = 'w-64';
  export let customClass = '';
  const dispatch = createEventDispatcher();

  let triggerEl: HTMLElement;
  let popoverEl: HTMLElement;

  function handleClickOutside(event: MouseEvent) {
    if (open && popoverEl && !popoverEl.contains(event.target as Node) && !triggerEl.contains(event.target as Node)) {
      dispatch('close');
    }
  }

  function positionPopover() {
    if (open && triggerEl && popoverEl) {
      const rect = triggerEl.getBoundingClientRect();
      const scrollY = window.scrollY || window.pageYOffset;
      const scrollX = window.scrollX || window.pageXOffset;

      popoverEl.style.position = 'fixed';
      popoverEl.style.top = `${rect.bottom + scrollY}px`;
      popoverEl.style.left = `${rect.left + scrollX}px`;

      // Adjust if popover is off-screen
      const popoverRect = popoverEl.getBoundingClientRect();
      if (popoverRect.right > window.innerWidth) {
        popoverEl.style.left = `${window.innerWidth - popoverRect.width}px`;
      }
      if (popoverRect.bottom > window.innerHeight) {
        popoverEl.style.top = `${rect.top + scrollY - popoverRect.height}px`;
      }
    }
  }

  onMount(() => {
    window.addEventListener('scroll', positionPopover);
    return () => {
      window.removeEventListener('scroll', positionPopover);
    };
  });

  $: if (open) {
    setTimeout(positionPopover, 0);
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="relative inline-block" bind:this={triggerEl}>
  <div class="popover-trigger">
    <slot name="trigger" />
  </div>
 
  {#if open}
    <div
      bind:this={popoverEl}
      class={`fixed z-50 ${width} rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 popover-content ${customClass}`}
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