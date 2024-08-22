<script lang="ts">
  import { options } from '../stores/options';
  import { onDestroy } from 'svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import GeneralOptions from './GeneralOptions.svelte';
  import LLMConfiguration from './LLMConfiguration.svelte';
  import CustomPrompts from './CustomPrompts.svelte';
  import { ChevronDown, ChevronUp } from 'lucide-svelte';

  let showConfirmDialog = false;
  let isDeleting = false;
  let generalOptionsOpen = true;
  let llmConfigOpen = true;

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
  .options-section {
    @apply mb-6 bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden;
  }

  .options-section:last-child {
    @apply mb-0;
  }

  .section-title {
    @apply text-xl font-bold text-gray-800 dark:text-white;
  }

  .section-header {
    @apply p-4 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors duration-200;
  }

  .section-content {
    @apply p-6 border-t border-gray-200 dark:border-gray-700;
  }

  .danger-zone {
    @apply bg-red-50 dark:bg-red-900;
  }

  .danger-title {
    @apply text-red-600 dark:text-red-400;
  }
</style>

<div class="space-y-6 max-w-3xl mx-auto">
  <div class="options-section">
    <button
      class="section-header w-full flex justify-between items-center"
      on:click={() => generalOptionsOpen = !generalOptionsOpen}
    >
      <h2 class="section-title">General Options</h2>
      {#if generalOptionsOpen}
        <ChevronUp size={20} />
      {:else}
        <ChevronDown size={20} />
      {/if}
    </button>
    {#if generalOptionsOpen}
      <div class="section-content">
        <GeneralOptions />
      </div>
    {/if}
  </div>

  <div class="options-section">
    <button
      class="section-header w-full flex justify-between items-center"
      on:click={() => llmConfigOpen = !llmConfigOpen}
    >
      <h2 class="section-title">Local LLM Configuration</h2>
      {#if llmConfigOpen}
        <ChevronUp size={20} />
      {:else}
        <ChevronDown size={20} />
      {/if}
    </button>
    {#if llmConfigOpen}
      <div class="section-content">
        <LLMConfiguration />
        <div class="mt-6">
          <CustomPrompts />
        </div>
      </div>
    {/if}
  </div>

  <div class="options-section danger-zone">
    <div class="section-content">
      <h2 class="section-title danger-title mb-4">Danger Zone</h2>
      <button
        on:click={handleDeleteAllData}
        class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-opacity-50 disabled:opacity-50 transition-colors duration-200"
        disabled={isDeleting}
      >
        {isDeleting ? 'Deleting...' : 'Delete All Data'}
      </button>
    </div>
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