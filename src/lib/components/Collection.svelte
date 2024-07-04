<!-- src/lib/components/Collection.svelte -->
<script lang="ts">
  import { deleteCollection, removeStudyMaterial, addStudyMaterials, addFlashcardDeck, removeFlashcardDeck, addFlashcardToDeck, removeFlashcardFromDeck } from '../stores/collections';
  import type { StudyMaterial, FlashcardDeck, Flashcard, TestData } from '../stores/collections';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import FlashcardDeck from './FlashcardDeck.svelte';
  import Test from './Test.svelte';
  import Modal from './Modal.svelte';
  import AddStudyMaterialModal from './AddStudyMaterialModal.svelte';
  import { readFile, deleteFile } from '../utils/fileUtils';

  export let id: string;
  export let name: string;
  export let studyMaterials: StudyMaterial[];
  export let reviewMaterials: Array<FlashcardDeck | TestData>;

  let viewingFile: { name: string, type: 'pdf' | 'markdown' | 'webpage', filePath?: string, content?: string, url?: string } | null = null;
  let isAddStudyMaterialModalOpen = false;
  let isAddingDeck = false;
  let newDeckName = '';
  let showAddFeedback = false;
  let addedMaterialsCount = 0;

  function getIcon(type: 'pdf' | 'markdown' | 'webpage') {
    switch (type) {
      case 'pdf':
        return '📄';
      case 'markdown':
        return '📝';
      case 'webpage':
        return '🌐';
      default:
        return '📎';
    }
  }

  async function handleViewFile(material: StudyMaterial) {
    if (material.type === 'markdown' && material.filePath && !material.content) {
      try {
        material.content = await readFile(material.filePath);
      } catch (error) {
        console.error('Error reading file:', error);
        alert('Failed to load file content. Please try again.');
        return;
      }
    }
    viewingFile = { ...material };
  }

  async function handleRemoveFile(material: StudyMaterial) {
    if (confirm(`Are you sure you want to remove "${material.name}"?`)) {
      try {
        if (material.filePath) {
          await deleteFile(material.filePath);
        }
        removeStudyMaterial(id, material.name);
        studyMaterials = studyMaterials.filter(m => m !== material);
      } catch (error) {
        console.error('Error deleting file:', error);
        alert(`There was an error deleting the file: ${error.message}`);
      }
    }
  }

  function closeFileView() {
    viewingFile = null;
  }

  function openAddStudyMaterialModal() {
    isAddStudyMaterialModalOpen = true;
  }

  function closeAddStudyMaterialModal() {
    isAddStudyMaterialModalOpen = false;
  }

  function handleMaterialsAdded(event: CustomEvent<StudyMaterial[]>) {
    const newMaterials = event.detail;
    addedMaterialsCount = newMaterials.length;
    showAddFeedback = true;
    setTimeout(() => {
      showAddFeedback = false;
    }, 3000);
    
    studyMaterials = [...studyMaterials, ...newMaterials];
    addStudyMaterials(id, newMaterials);
  }

  function handleAddDeck() {
    if (newDeckName.trim()) {
      addFlashcardDeck(id, newDeckName.trim());
      newDeckName = '';
      isAddingDeck = false;
    }
  }

  function handleRemoveDeck(deckId: string) {
    if (confirm("Are you sure you want to remove this flashcard deck?")) {
      removeFlashcardDeck(id, deckId);
      reviewMaterials = reviewMaterials.filter(m => !('flashcards' in m) || m.id !== deckId);
    }
  }

  function handleAddCardToDeck(event: CustomEvent<{ deckId: string, card: Flashcard }>) {
    const { deckId, card } = event.detail;
    addFlashcardToDeck(id, deckId, card);
    reviewMaterials = reviewMaterials.map(m => {
      if ('flashcards' in m && m.id === deckId) {
        return { ...m, flashcards: [...m.flashcards, card] };
      }
      return m;
    });
  }

  function handleRemoveCardFromDeck(event: CustomEvent<{ deckId: string, cardId: string }>) {
    const { deckId, cardId } = event.detail;
    removeFlashcardFromDeck(id, deckId, cardId);
    reviewMaterials = reviewMaterials.map(m => {
      if ('flashcards' in m && m.id === deckId) {
        return { ...m, flashcards: m.flashcards.filter(card => card.id !== cardId) };
      }
      return m;
    });
  }

  function handleDelete() {
    if (confirm(`Are you sure you want to delete the collection "${name}"?`)) {
      deleteCollection(id);
    }
  }
</script>

<div class="bg-white dark:bg-gray-800 shadow-lg rounded-lg overflow-hidden">
  <div class="flex justify-between items-center p-4 bg-gray-600 dark:bg-gray-700 text-text-dark dark:text-text-dark">
    <h2 class="text-2xl font-semibold truncate flex-grow">{name}</h2>
    <button
      on:click={handleDelete}
      class="px-2 py-1 ml-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-opacity-50 flex-shrink-0"
    >
      Delete
    </button>
  </div>
  
  <div class="p-4">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-xl font-semibold text-text-light dark:text-text-dark">Study Materials</h3>
      <button
        on:click={openAddStudyMaterialModal}
        class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
      >
        Add Study Material
      </button>
    </div>
    {#if studyMaterials.length > 0}
      <div class="space-y-2">
        {#each studyMaterials as material (material.name)}
          <div 
            class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-700 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-600"
            on:click={() => handleViewFile(material)}
          >
            <div class="flex items-center">
              <span class="text-2xl mr-2">{getIcon(material.type)}</span>
              <span class="text-text-light dark:text-text-dark">
                {material.name}
              </span>
            </div>
            <button
              on:click|stopPropagation={() => handleRemoveFile(material)}
              class="text-red-500 hover:text-red-700 focus:outline-none"
              title="Remove"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
              </svg>
            </button>
          </div>
        {/each}
      </div>
    {:else}
      <p class="text-text-light dark:text-text-dark">No study materials added yet.</p>
    {/if}
    {#if showAddFeedback}
      <p class="mt-2 text-green-600 dark:text-green-400">Successfully added {addedMaterialsCount} material{addedMaterialsCount !== 1 ? 's' : ''}</p>
    {/if}
  </div>

  <div class="p-4 bg-gray-600 dark:bg-gray-600">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-xl font-semibold text-text-dark dark:text-text-dark">Review Materials</h3>
      <button
        on:click={() => isAddingDeck = true}
        class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
      >
        Add Flashcard Deck
      </button>
    </div>

    {#if isAddingDeck}
      <div class="mb-4 flex space-x-2">
        <input
          type="text"
          bind:value={newDeckName}
          placeholder="Deck Name"
          class="flex-grow p-2 border rounded text-text-light"
        />
        <button
          on:click={handleAddDeck}
          class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
        >
          Add Deck
        </button>
        <button
          on:click={() => isAddingDeck = false}
          class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
        >
          Cancel
        </button>
      </div>
    {/if}

    {#if reviewMaterials.length > 0}
      <div class="space-y-4">
        {#each reviewMaterials as material (material.id)}
          {#if 'flashcards' in material}
            <FlashcardDeck
              deck={material}
              on:addCard={handleAddCardToDeck}
              on:removeCard={handleRemoveCardFromDeck}
            />
            <button
              on:click={() => handleRemoveDeck(material.id)}
              class="mt-2 text-red-500 hover:text-red-700 focus:outline-none"
              title="Remove Deck"
            >
              Remove Deck
            </button>
          {:else}
            <Test {...material} />
          {/if}
        {/each}
      </div>
    {:else}
      <p class="text-text-dark dark:text-text-dark">No review materials added yet.</p>
    {/if}
  </div>
</div>

<Modal isOpen={viewingFile !== null} title={viewingFile?.name || ''} on:close={closeFileView}>
  {#if viewingFile}
    <div class="max-w-full overflow-x-auto">
      {#if viewingFile.type === 'markdown'}
        <div class="prose dark:prose-invert max-w-none">
          <MarkdownRenderer content={viewingFile.content || ''} />
        </div>
      {:else if viewingFile.type === 'webpage'}
        <p>Webpage content viewer not implemented yet.</p>
      {:else if viewingFile.type === 'pdf'}
        <p>PDF viewer not implemented yet.</p>
      {/if}
    </div>
  {/if}
</Modal>

<AddStudyMaterialModal 
  isOpen={isAddStudyMaterialModalOpen} 
  collectionId={id} 
  name={name}
  on:close={closeAddStudyMaterialModal}
  on:materialsAdded={handleMaterialsAdded}
/>