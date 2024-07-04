<!-- src/lib/components/Collection.svelte -->
<script lang="ts">
  import { deleteCollection, removeStudyMaterial, addStudyMaterials, addFlashcardDeck, removeFlashcardDeck, addTestQuestion, removeTestQuestion } from '../stores/collections';
  import type { StudyMaterial, FlashcardDeck, TestData } from '../stores/collections';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import StudyDeckModal from './StudyDeckModal.svelte';
  import EditDeckModal from './EditDeckModal.svelte';
  import Test from './Test.svelte';
  import Modal from './Modal.svelte';
  import AddStudyMaterialModal from './AddStudyMaterialModal.svelte';
  import { readFile, deleteFile } from '../utils/fileUtils';
  import DeckIcon from '../../assets/deck-icon.svg';
  import TestIcon from '../../assets/test-icon.svg';

  export let id: string;
  export let name: string;
  export let studyMaterials: StudyMaterial[];
  export let reviewMaterials: Array<FlashcardDeck | TestData>;

  let viewingFile: { name: string, type: 'pdf' | 'markdown' | 'webpage', filePath?: string, content?: string, url?: string } | null = null;
  let isAddStudyMaterialModalOpen = false;
  let isAddReviewMaterialModalOpen = false;
  let showAddFeedback = false;
  let addedMaterialsCount = 0;
  let studyingDeck: FlashcardDeck | null = null;
  let editingDeck: FlashcardDeck | null = null;
  let isAddingReviewMaterial = false;
  let reviewMaterialType: 'deck' | 'test' = 'deck';
  let newDeckName = ''; // Add this line to store the new deck name

  function generateUniqueId(material: StudyMaterial): string {
    return `${material.type}-${material.name}-${Date.now()}`;
  }

  $: studyMaterialsWithIds = studyMaterials.map(material => ({
    ...material,
    id: material.id || generateUniqueId(material)
  }));

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
    const newMaterials = event.detail.map(material => ({
      ...material,
      id: generateUniqueId(material)
    }));
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

  function openStudyDeck(deck: FlashcardDeck) {
    studyingDeck = deck;
  }

  function closeStudyDeck() {
    studyingDeck = null;
  }

  function openEditDeck(deck: FlashcardDeck) {
    editingDeck = deck;
  }

  function closeEditDeck() {
    editingDeck = null;
  }

  function handleDeckUpdate(event: CustomEvent<FlashcardDeck>) {
    const updatedDeck = event.detail;
    reviewMaterials = reviewMaterials.map(m => 
      ('flashcards' in m && m.id === updatedDeck.id) ? updatedDeck : m
    );
  }

  function handleDelete() {
    if (confirm(`Are you sure you want to delete the collection "${name}"?`)) {
      deleteCollection(id);
    }
  }

  function openAddReviewMaterialModal() {
    isAddReviewMaterialModalOpen = true;
  }

  function closeAddReviewMaterialModal() {
    isAddReviewMaterialModalOpen = false;
  }

  function handleReviewMaterialAdded(event: CustomEvent) {
    const { type, id, data } = event.detail;
    if (type === 'flashcard') {
      const newDeck: FlashcardDeck = {
        id,
        name: 'New Flashcard Deck',
        flashcards: [{ question: data.question, answer: data.answer }]
      };
      addFlashcardDeck(id, newDeck);
      reviewMaterials = [...reviewMaterials, newDeck];
    } else if (type === 'test') {
      const newTest: TestData = {
        id,
        question: data.question,
        options: data.options,
        correctOptionIndex: data.correctOptionIndex
      };
      addTestQuestion(id, newTest);
      reviewMaterials = [...reviewMaterials, newTest];
    }
  }

  function handleRemoveReviewMaterial(material: FlashcardDeck | TestData) {
    if ('flashcards' in material) {
      if (confirm(`Are you sure you want to remove the flashcard deck "${material.name}"?`)) {
        removeFlashcardDeck(id, material.id);
        reviewMaterials = reviewMaterials.filter(m => m !== material);
      }
    } else {
      if (confirm("Are you sure you want to remove this test question?")) {
        removeTestQuestion(id, material.id);
        reviewMaterials = reviewMaterials.filter(m => m !== material);
      }
    }
  }

  function handleAddReviewMaterial() {
  if (reviewMaterialType === 'deck') {
    if (newDeckName.trim()) {
      const newDeck: FlashcardDeck = {
        id: Date.now().toString(),
        name: newDeckName.trim(),
        flashcards: []
      };
      addFlashcardDeck(id, newDeck);
      reviewMaterials = [...reviewMaterials, newDeck];
      newDeckName = '';
    } else {
      alert('Please provide a name for the flashcard deck.');
      return;
    }
  } else {
    // ... (test question logic remains the same)
  }
  isAddingReviewMaterial = false;
}

function canStudyDeck(material: FlashcardDeck | TestData): boolean {
    return 'flashcards' in material && material.flashcards.length > 0;
  }
</script>

<div class="bg-white dark:bg-gray-800 shadow-lg rounded-lg overflow-hidden">
  <div class="flex justify-between items-center p-4 bg-gray-600 dark:bg-gray-700 text-text-dark dark:text-text-dark">
    <h2 class="text-2xl font-semibold truncate flex-grow">{name}</h2>
    <button
      on:click={handleDelete}
      class="p-1 ml-2 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 focus:outline-none"
      title="Delete Collection"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </button>
  </div>
  
  <div class="p-4">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-xl font-semibold text-text-light dark:text-text-dark">Study Materials</h3>
      <button
        on:click={openAddStudyMaterialModal}
        class="p-1 text-blue-500 hover:text-blue-600 focus:outline-none"
        title="Add Study Material"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>
    
    {#if studyMaterialsWithIds.length > 0}
      <div class="space-y-2">
        {#each studyMaterialsWithIds as material (material.id)}
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
              class="p-1 text-red-500 hover:text-red-700 focus:outline-none"
              title="Remove"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
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
        on:click={() => isAddingReviewMaterial = true}
        class="p-1 text-blue-500 hover:text-blue-600 focus:outline-none"
        title="Add Review Material"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>

    {#if isAddingReviewMaterial}
      <div class="mb-4 flex flex-col space-y-2">
        <select
          bind:value={reviewMaterialType}
          class="p-2 border rounded text-text-light"
        >
          <option value="deck">Flashcard Deck</option>
          <option value="test">Test Question</option>
        </select>
        {#if reviewMaterialType === 'deck'}
          <input
            type="text"
            bind:value={newDeckName}
            placeholder="Enter deck name"
            class="p-2 border rounded text-text-light"
          />
        {/if}
        <div class="flex space-x-2">
          <button
            on:click={handleAddReviewMaterial}
            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 flex-grow"
          >
            Add
          </button>
          <button
            on:click={() => {
              isAddingReviewMaterial = false;
              newDeckName = '';
            }}
            class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 flex-grow"
          >
            Cancel
          </button>
        </div>
      </div>
    {/if}

    {#if reviewMaterials.length > 0}
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
            {:else}
              <img src={TestIcon} alt="Test Question" class="w-16 h-16 mb-3 text-green-500" />
              <span class="text-center text-sm font-medium text-gray-800 dark:text-white">Test Question</span>
            {/if}
          </div>
          <div class="absolute inset-0 bg-black bg-opacity-50 flex flex-col items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-200 rounded-lg">
            {#if 'flashcards' in material}
              <button
                on:click={() => openStudyDeck(material)}
                class="px-3 py-1 bg-blue-500 text-white rounded mb-2 hover:bg-blue-600 w-3/4 disabled:opacity-50 disabled:cursor-not-allowed"
                disabled={!canStudyDeck(material)}
              >
                Study
              </button>
              <button
                on:click={() => openEditDeck(material)}
                class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 w-3/4 mb-2"
              >
                Edit
              </button>
            {:else}
              <button
                on:click={() => { /* Implement test taking functionality */ }}
                class="px-3 py-1 bg-blue-500 text-white rounded mb-2 hover:bg-blue-600 w-3/4"
              >
                Take Test
              </button>
              <button
                on:click={() => { /* Implement test editing functionality */ }}
                class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600 w-3/4 mb-2"
              >
                Edit Test
              </button>
            {/if}
            <button
              on:click={() => handleRemoveReviewMaterial(material)}
              class="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600 w-3/4"
            >
              Remove
            </button>
          </div>
        </div>
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

<StudyDeckModal 
  deck={studyingDeck}
  isOpen={studyingDeck !== null}
  on:close={closeStudyDeck}
/>

<EditDeckModal
  deck={editingDeck}
  isOpen={editingDeck !== null}
  on:close={closeEditDeck}
  on:update={handleDeckUpdate}
/>