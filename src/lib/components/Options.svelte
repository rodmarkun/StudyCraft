<script lang="ts">
  import { options } from '../stores/options';
  import { onDestroy } from 'svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import GeneralOptions from './GeneralOptions.svelte';
  import LLMConfiguration from './LLMConfiguration.svelte';
  import CustomPrompts from './CustomPrompts.svelte';

  let showConfirmDialog = false;
  let isDeleting = false;

  function handleDeleteAllData() {
    showConfirmDialog = true;
  }

  async function handleConfirmDelete() {
    isDeleting = true;
    try {
      const success = await window.electronAPI.deleteAllData();
      if (success) {
        alert('All data has been deleted. The application will close now.');
        window.electronAPI.exitApp();
      } else {
        alert('There was an error deleting the data. Please try again.');
      }
    } catch (error) {
      console.error('Error deleting all data:', error);
      alert('There was an error deleting the data. Please try again.');
    } finally {
      isDeleting = false;
      showConfirmDialog = false;
    }
  }

  // Automatically save options when component is destroyed (i.e., when exiting options menu)
  onDestroy(() => {
    options.saveOptions($options);
  });
</script>

<style lang="postcss">
  :global(.options-section) {
    @apply mb-8 p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md;
  }

  :global(.options-section:last-child) {
    @apply mb-0;
  }

  :global(.section-title) {
    @apply text-2xl font-bold mb-6 text-gray-800 dark:text-white;
  }

  :global(.custom-input) {
    @apply mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 transition-all duration-200 ease-in-out;
  }

  :global(.custom-textarea) {
    @apply custom-input resize-none p-3 h-40;
    scrollbar-width: thin;
    scrollbar-color: rgba(155, 155, 155, 0.5) transparent;
  }

  :global(.custom-textarea::-webkit-scrollbar) {
    width: 6px;
  }

  :global(.custom-textarea::-webkit-scrollbar-track) {
    background: transparent;
  }

  :global(.custom-textarea::-webkit-scrollbar-thumb) {
    background-color: rgba(155, 155, 155, 0.5);
    border-radius: 20px;
    border: transparent;
  }
</style>

<div class="space-y-8 max-w-4xl mx-auto"> <!-- Added max-w-4xl and mx-auto -->
  <GeneralOptions />
  <LLMConfiguration />
  <CustomPrompts />

  <div class="options-section">
    <h2 class="section-title text-red-600 dark:text-red-400">Danger Zone</h2>
    <button
      on:click={handleDeleteAllData}
      class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-opacity-50 disabled:opacity-50"
      disabled={isDeleting}
    >
      {isDeleting ? 'Deleting...' : 'Delete All Data'}
    </button>
  </div>
</div>

{#if showConfirmDialog}
  <ConfirmDialog
    title="Confirm Deletion"
    message="Are you sure you want to delete all data? This action cannot be undone and will close the application."
    confirmText="Delete All Data"
    on:confirm={handleConfirmDelete}
    on:cancel={() => showConfirmDialog = false}
  />
{/if}