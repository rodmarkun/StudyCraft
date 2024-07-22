<!-- src/lib/components/AddCollection.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { addCollection, collections } from '../stores/collections';
  import { writable } from 'svelte/store';

  const newCollectionName = writable('');
  let inputElement: HTMLInputElement;

  onMount(() => {
    focusInput();
  });

  async function handleSubmit() {
    $newCollectionName = $newCollectionName.trim();
    if ($newCollectionName) {
      try {
        await addCollection($newCollectionName);
        $newCollectionName = '';
        focusInput();
      } catch (error) {
        console.error('Failed to add collection:', error);
        alert('Failed to add collection. Please try again.');
      }
    }
  }

  function focusInput() {
    if (inputElement) {
      inputElement.focus();
    }
  }

  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    $newCollectionName = target.value;
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="mb-4">
  <input
    type="text"
    value={$newCollectionName}
    on:input={handleInput}
    bind:this={inputElement}
    placeholder="New collection name"
    class="px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-l-md focus:outline-none focus:ring-2 focus:ring-primary-light dark:focus:ring-primary-dark bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
  />
  <button
    type="submit"
    class="px-4 py-2 bg-primary-dark dark:bg-primary-dark text-white rounded-r-md hover:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-primary-light dark:focus:ring-primary-dark"
  >
    Add Collection
  </button>
</form>