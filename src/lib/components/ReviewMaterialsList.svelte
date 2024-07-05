<!-- src/lib/components/ReviewMaterialsList.svelte -->
<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import type { FlashcardDeck, TestData, Test } from '../stores/collections';
    import DeckIcon from '../../assets/deck-icon.svg';
    import TestIcon from '../../assets/test-icon.svg';
  
    export let reviewMaterials: Array<FlashcardDeck | Test>;
    export let collectionId: string;
  
    const dispatch = createEventDispatcher();
  
    function canStudyMaterial(material: FlashcardDeck | Test): boolean {
      if ('flashcards' in material) {
        return material.flashcards.length > 0;
      } else if ('questions' in material) {
        return material.questions.length > 0;
      }
      return false;
    }
  
    function handleStudy(material: FlashcardDeck | Test) {
      dispatch('studyMaterial', { material });
    }
  
    function handleEdit(material: FlashcardDeck | Test) {
      dispatch('editMaterial', { material });
    }
  
    function handleRemove(material: FlashcardDeck | Test) {
      dispatch('removeMaterial', { collectionId, material });
    }
  </script>
  
  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
    {#each reviewMaterials as material (material.id)}
      <div class="relative group">
        <div class="flex flex-col items-center justify-center bg-white dark:bg-gray-700 p-6 rounded-lg shadow-md hover:shadow-lg transition-shadow duration-200 aspect-square">
          {#if 'flashcards' in material}
            <img src={DeckIcon} alt="Flashcard Deck" class="w-16 h-16 mb-3 text-blue-500" />
            <span class="text-center text-sm font-medium text-gray-800 dark:text-white">{material.name}</span>
            <span class="text-center text-xs text-gray-500 dark:text-gray-400 mt-1">
              {material.flashcards.length} card{material.flashcards.length !== 1 ? 's' : ''}
            </span>
          {:else if 'questions' in material}
            <img src={TestIcon} alt="Test" class="w-16 h-16 mb-3 text-green-500" />
            <span class="text-center text-sm font-medium text-gray-800 dark:text-white">{material.name}</span>
            <span class="text-center text-xs text-gray-500 dark:text-gray-400 mt-1">
              {material.questions.length} question{material.questions.length !== 1 ? 's' : ''}
            </span>
          {/if}
        </div>
        <div class="absolute inset-0 bg-black bg-opacity-50 flex flex-col items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-200 rounded-lg">
          <button
            on:click={() => handleStudy(material)}
            class="px-3 py-1 bg-blue-500 text-white rounded mb-2 hover:bg-blue-600 w-3/4 disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={!canStudyMaterial(material)}
          >
            Study
          </button>
          <button
            on:click={() => handleEdit(material)}
            class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 w-3/4 mb-2"
          >
            Edit
          </button>
          <button
            on:click={() => handleRemove(material)}
            class="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600 w-3/4"
          >
            Delete
          </button>
        </div>
      </div>
    {/each}
  </div>