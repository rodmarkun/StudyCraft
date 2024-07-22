<script lang="ts">
  import { collections } from '../stores/collections';
  import Collection from './Collection.svelte';
  import AddCollection from './AddCollection.svelte';
  $: sortedCollections = $collections.sort((a, b) => a.name.localeCompare(b.name));

  function handleRenameCollection(event: CustomEvent<{id: string, name: string}>) {
    const { id, name } = event.detail;
    collections.renameCollection(id, name);
  }

  function handleExportCollection(event: CustomEvent<{id: string}>) {
    const { id } = event.detail;
    const collection = $collections.find(c => c.id === id);
    if (collection) {
      const exportData = JSON.stringify(collection);
      navigator.clipboard.writeText(exportData)
        .then(() => alert("Collection data copied to clipboard!"))
        .catch(err => console.error('Failed to copy: ', err));
    }
  }

  function handleImportCollection() {
    navigator.clipboard.readText()
      .then(text => {
        try {
          const importedData = JSON.parse(text);
          collections.importCollection(importedData);
          alert("Collection imported successfully!");
        } catch (error) {
          alert("Invalid collection data. Please make sure you've copied the correct JSON.");
        }
      })
      .catch(err => console.error('Failed to read clipboard: ', err));
  }
</script>

<style>
.collections-container {
  column-count: 1;
  column-gap: 2rem;
}

@media (min-width: 768px) {
  .collections-container {
    column-count: 2;
  }
}

.collection-wrapper {
  break-inside: avoid;
  page-break-inside: avoid;
  margin-bottom: 2rem;
}
</style>

<h1 class="text-3xl font-bold mb-8 text-center text-text-light dark:text-text-dark">Collections</h1>
<AddCollection />
<div class="collections-container mt-8">
  {#each sortedCollections as collection (collection.id)}
    <div class="collection-wrapper">
      <Collection
        {...collection}
        on:renameCollection={(event) => handleRenameCollection({detail: {id: collection.id, name: event.detail}})}
        on:exportCollection={() => handleExportCollection({detail: {id: collection.id}})}
        on:importCollection={handleImportCollection}
      />
    </div>
  {/each}
</div>
{#if sortedCollections.length === 0}
  <p class="text-center text-gray-500 dark:text-gray-400 mt-8">No collections yet. Add one to get started!</p>
{/if}