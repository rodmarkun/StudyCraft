<script lang="ts">
  import { collections } from '../stores/collections';
  import Collection from './Collection.svelte';
  import AddCollection from './AddCollection.svelte';

  $: sortedCollections = $collections.sort((a, b) => a.name.localeCompare(b.name));
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
    <Collection {...collection} />
  </div>
{/each}
</div>

{#if sortedCollections.length === 0}
<p class="text-center text-gray-500 dark:text-gray-400 mt-8">No collections yet. Add one to get started!</p>
{/if}