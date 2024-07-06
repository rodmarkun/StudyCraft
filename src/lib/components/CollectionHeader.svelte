<!-- src/lib/components/CollectionHeader.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';

  export let name: string;

  const dispatch = createEventDispatcher();

  let showConfirmDialog = false;

  function handleDeleteClick() {
    showConfirmDialog = true;
  }

  function handleConfirmDelete() {
    dispatch('deleteCollection');
    showConfirmDialog = false;
  }

  function handleCancelDelete() {
    showConfirmDialog = false;
  }
</script>

<div class="flex justify-between items-center p-4 bg-gray-600 dark:bg-gray-700 text-text-dark dark:text-text-dark">
  <h2 class="text-2xl font-semibold truncate flex-grow">{name}</h2>
  <button
    on:click={handleDeleteClick}
    class="p-1 ml-2 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 focus:outline-none"
    title="Delete Collection"
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
    </svg>
  </button>
</div>

{#if showConfirmDialog}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl">
      <h3 class="text-lg font-semibold mb-4">Confirm Deletion</h3>
      <p class="mb-4">Are you sure you want to delete the collection "{name}"?</p>
      <div class="flex justify-end space-x-2">
        <button
          on:click={handleCancelDelete}
          class="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400 focus:outline-none"
        >
          Cancel
        </button>
        <button
          on:click={handleConfirmDelete}
          class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}