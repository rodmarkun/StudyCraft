<!-- src/lib/components/Collections.svelte -->
<script lang="ts">
    import { collections } from '../stores/collections';
    import Collection from './Collection.svelte';
    import AddCollection from './AddCollection.svelte';
  
    $: sortedCollections = $collections.sort((a, b) => a.name.localeCompare(b.name));
  </script>
  
  <h1 class="text-3xl font-bold mb-8 text-center text-text-light dark:text-text-dark">Collections</h1>
  
  <AddCollection />
  
  <div class="grid grid-cols-1 md:grid-cols-2 gap-8 mt-8">
    {#each sortedCollections as collection (collection.id)}
      <Collection {...collection} />
    {/each}
  </div>
  
  {#if sortedCollections.length === 0}
    <p class="text-center text-gray-500 dark:text-gray-400 mt-8">No collections yet. Add one to get started!</p>
  {/if}