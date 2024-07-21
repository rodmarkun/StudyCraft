<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';

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

<div class="flex justify-between items-center p-4 bg-primary-dark dark:bg-gray-700 text-text-dark dark:text-text-dark">
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
  <ConfirmDialog
    title="Confirm Deletion"
    message="Are you sure you want to delete the collection {name}?"
    on:confirm={handleConfirmDelete}
    on:cancel={handleCancelDelete}
  />
{/if}