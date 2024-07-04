<!-- src/lib/components/File.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let fileName: string;
  export let fileType: 'pdf' | 'markdown';
  const dispatch = createEventDispatcher();
  function handleView() {
    dispatch('view', { fileName, fileType });
  }
  function handleRemove() {
    dispatch('remove', { fileName, fileType });
  }
</script>

<div class="flex items-center p-2 bg-gray-100 dark:bg-gray-700 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors duration-200 w-full">
  <button
    on:click={handleView}
    class="flex items-center flex-grow overflow-hidden"
  >
    <span class="mr-3 text-2xl flex-shrink-0">
      {#if fileType === 'pdf'}
        📄
      {:else}
        📝
      {/if}
    </span>
    <span class="text-gray-800 dark:text-gray-200 text-left truncate">{fileName}</span>
  </button>
  <button
    on:click={handleRemove}
    class="ml-2 p-1 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 focus:outline-none"
    title="Remove file"
  >
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
    </svg>
  </button>
</div>