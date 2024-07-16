<!-- src/lib/components/Modal.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let isOpen = false;
  export let title = '';

  const dispatch = createEventDispatcher();

  function close() {
    dispatch('close');
  }
</script>

<style>
  .modal-content {
    scrollbar-width: thin;
    scrollbar-color: rgba(155, 155, 155, 0.5) transparent;
  }

  .modal-content::-webkit-scrollbar {
    width: 6px;
  }

  .modal-content::-webkit-scrollbar-track {
    background: transparent;
  }

  .modal-content::-webkit-scrollbar-thumb {
    background-color: rgba(155, 155, 155, 0.5);
    border-radius: 20px;
    border: transparent;
  }
</style>

{#if isOpen}
  <div class="fixed inset-0 bg-black bg-opacity-50 z-40 flex justify-center items-center p-4" on:click|self={close}>
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-4xl max-h-[90vh] flex flex-col">
      <div class="flex justify-between items-center p-4 border-b border-gray-200 dark:border-gray-700">
        <h2 class="text-xl font-semibold text-text-light dark:text-text-dark">{title}</h2>
        <button on:click={close} class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </div>
      <div class="p-4 overflow-auto flex-grow modal-content">
        <slot></slot>
      </div>
    </div>
  </div>
{/if}