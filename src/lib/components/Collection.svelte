<!-- src/lib/components/Collection.svelte -->
<script lang="ts">
  import { deleteCollection, removeStudyMaterial, addStudyMaterials, addFlashcardDeck, removeFlashcardDeck, addTest, removeTest, updateTest } from '../stores/collections';
  import type { StudyMaterial, FlashcardDeck, Test, TestData } from '../stores/collections';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import StudyDeckModal from './StudyDeckModal.svelte';
  import EditDeckModal from './EditDeckModal.svelte';
  import StudyTestModal from './StudyTestModal.svelte';
  import EditTestModal from './EditTestModal.svelte';
  import Modal from './Modal.svelte';
  import AddStudyMaterialModal from './AddStudyMaterialModal.svelte';
  import CollectionHeader from './CollectionHeader.svelte';
  import StudyMaterialsList from './StudyMaterialsList.svelte';
  import ReviewMaterialsList from './ReviewMaterialsList.svelte';

  export let id: string;
  export let name: string;
  export let studyMaterials: StudyMaterial[];
  export let reviewMaterials: Array<FlashcardDeck | Test>;

  let viewingFile: StudyMaterial | null = null;
  let isAddStudyMaterialModalOpen = false;
  let showAddFeedback = false;
  let addedMaterialsCount = 0;
  let studyingDeck: FlashcardDeck | null = null;
  let editingDeck: FlashcardDeck | null = null;
  let studyingTest: Test | null = null;
  let editingTest: Test | null = null;
  let isAddingReviewMaterial = false;
  let reviewMaterialType: 'deck' | 'test' = 'deck';
  let newMaterialName = '';

  function handleDeleteCollection() {
    deleteCollection(id);
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

  function handleViewFile(event: CustomEvent<StudyMaterial>) {
    viewingFile = event.detail;
  }

  function handleRemoveStudyMaterial(event: CustomEvent<{collectionId: string, materialName: string}>) {
    const { collectionId, materialName } = event.detail;
    removeStudyMaterial(collectionId, materialName);
    studyMaterials = studyMaterials.filter(m => m.name !== materialName);
  }

  function handleStudyMaterial(event: CustomEvent<{material: FlashcardDeck | Test}>) {
    const { material } = event.detail;
    if ('flashcards' in material) {
      studyingDeck = material;
    } else if ('questions' in material) {
      studyingTest = material;
    }
  }

  function handleEditMaterial(event: CustomEvent<{material: FlashcardDeck | Test}>) {
    const { material } = event.detail;
    if ('flashcards' in material) {
      editingDeck = material;
    } else if ('questions' in material) {
      editingTest = material;
    }
  }

  function handleRemoveMaterial(event: CustomEvent<{collectionId: string, material: FlashcardDeck | Test}>) {
    const { collectionId, material } = event.detail;
    if ('flashcards' in material) {
      removeFlashcardDeck(collectionId, material.id);
    } else if ('questions' in material) {
      removeTest(collectionId, material.id);
    }
    reviewMaterials = reviewMaterials.filter(m => m.id !== material.id);
  }

  function closeFileView() {
    viewingFile = null;
  }

  function closeStudyDeck() {
    studyingDeck = null;
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

  function handleAddReviewMaterial() {
    if (newMaterialName.trim()) {
      if (reviewMaterialType === 'deck') {
        const newDeck: FlashcardDeck = {
          id: Date.now().toString(),
          name: newMaterialName.trim(),
          flashcards: []
        };
        addFlashcardDeck(id, newDeck);
        reviewMaterials = [...reviewMaterials, newDeck];
      } else if (reviewMaterialType === 'test') {
        const newTest: Test = {
          id: Date.now().toString(),
          name: newMaterialName.trim(),
          questions: []
        };
        addTest(id, newTest);
        reviewMaterials = [...reviewMaterials, newTest];
      }
      newMaterialName = '';
      isAddingReviewMaterial = false;
    } else {
      alert('Please provide a name for the new material.');
    }
  }

  function handleTestUpdate(event: CustomEvent<Test>) {
    const updatedTest = event.detail;
    updateTest(id, updatedTest);
    reviewMaterials = reviewMaterials.map(m => 
      ('questions' in m && m.id === updatedTest.id) ? updatedTest : m
    );
  }
  
</script>

<div class="bg-white dark:bg-gray-800 shadow-lg rounded-lg overflow-hidden">
  <CollectionHeader {name} on:deleteCollection={handleDeleteCollection} />
  
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
    
    <StudyMaterialsList 
      {studyMaterials} 
      collectionId={id} 
      on:viewFile={handleViewFile} 
      on:removeStudyMaterial={handleRemoveStudyMaterial}
    />

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
          <option value="test">Test</option>
        </select>
        <input
          type="text"
          bind:value={newMaterialName}
          placeholder="Enter name"
          class="p-2 border rounded text-text-light"
        />
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
              newMaterialName = '';
            }}
            class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600 flex-grow"
          >
            Cancel
          </button>
        </div>
      </div>
    {/if}

    <ReviewMaterialsList 
      {reviewMaterials} 
      collectionId={id} 
      on:studyMaterial={handleStudyMaterial}
      on:editMaterial={handleEditMaterial}
      on:removeMaterial={handleRemoveMaterial}
    />
  </div>
</div>

<Modal isOpen={viewingFile !== null} title={viewingFile?.name || ''} on:close={closeFileView}>
  {#if viewingFile}
    <div class="max-w-full overflow-x-auto">
      {#if viewingFile.type === 'markdown'}
        <div class="prose dark:prose-invert max-w-none">
          <MarkdownRenderer content={viewingFile.content || ''} />
        </div>
      {:else if viewingFile.type === 'pdf'}
        <p>PDF viewer not implemented yet.</p>
      {:else if viewingFile.type === 'webpage'}
        <p>This is a webpage. Please click on the link to open it in a new tab.</p>
        <a href={viewingFile.url} target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">
          Open webpage
        </a>
      {/if}
    </div>
  {/if}
</Modal>

<AddStudyMaterialModal 
  isOpen={isAddStudyMaterialModalOpen} 
  collectionId={id} 
  {name}
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
  collectionId={id} 
  on:close={closeEditDeck}
  on:update={handleDeckUpdate}
/>

<StudyTestModal 
  test={studyingTest}
  isOpen={studyingTest !== null}
  on:close={() => studyingTest = null}
/>

<EditTestModal
  test={editingTest}
  isOpen={editingTest !== null}
  on:close={() => editingTest = null}
  on:update={handleTestUpdate}
/>