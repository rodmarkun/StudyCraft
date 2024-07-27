<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import Popover from './Popover.svelte';
  import { Cog } from 'lucide-svelte';

  export let name: string;
  const dispatch = createEventDispatcher();
 
  let showConfirmDialog = false;
  let showOptionsPopover = false;
  let showRenameInput = false;
  let newName = '';

  function handleDeleteClick() {
    showConfirmDialog = true;
    showOptionsPopover = false;
  }

  function handleConfirmDelete() {
    dispatch('deleteCollection');
    showConfirmDialog = false;
  }

  function handleCancelDelete() {
    showConfirmDialog = false;
  }

  function handleRenameCollection() {
    showRenameInput = true;
    showOptionsPopover = false;
    newName = name;
  }

  function confirmRename() {
    if (newName && newName !== name) {
      dispatch('renameCollection', newName);
    }
    showRenameInput = false;
  }

  function toggleOptionsPopover() {
    showOptionsPopover = !showOptionsPopover;
  }
</script>

<div class="flex justify-between items-center p-4 bg-primary-dark dark:bg-gray-700 text-text-dark dark:text-text-dark">
  {#if showRenameInput}
    <input
      bind:value={newName}
      on:keydown={(e) => e.key === 'Enter' && confirmRename()}
      on:blur={confirmRename}
      class="text-2xl font-semibold truncate flex-grow bg-transparent border-b border-gray-300 dark:border-gray-600 focus:outline-none focus:border-blue-500"
    />
  {:else}
    <h2 class="text-2xl font-semibold truncate flex-grow">{name}</h2>
  {/if}
  <Popover bind:open={showOptionsPopover} width="w-48">
    <button
      slot="trigger"
      on:click={toggleOptionsPopover}
      class="p-1 ml-2 text-gray-200 hover:text-white focus:outline-none"
      title="Collection Options"
    >
      <Cog size={20} />
    </button>
    <div class="py-2">
      <button
        on:click={handleRenameCollection}
        class="block w-full text-left px-4 py-2 hover:bg-gray-100 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200"
      >
        Rename Collection
      </button>
      <button
        on:click={handleDeleteClick}
        class="block w-full text-left px-4 py-2 text-red-500 hover:bg-gray-100 dark:hover:bg-gray-600"
      >
        Remove Collection
      </button>
    </div>
  </Popover>
</div>

{#if showConfirmDialog}
  <ConfirmDialog
    title="Confirm Deletion"
    message="Are you sure you want to delete the collection {name}?"
    on:confirm={handleConfirmDelete}
    on:cancel={handleCancelDelete}
  />
{/if}