<!-- src/lib/components/Collection.svelte -->
<script lang="ts">
  import { deleteCollection, removeStudyMaterial, addStudyMaterials, addFlashcardDeck, removeFlashcardDeck, addTest, removeTest, updateTest } from '../stores/collections';
  import type { StudyMaterial, FlashcardDeck, Test, TestData } from '../stores/collections';
  import { collections } from '../stores/collections';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import StudyDeckModal from './StudyDeckModal.svelte';
  import EditDeckModal from './EditDeckModal.svelte';
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
  let isDragging = false;
  let dragCounter = 0;
  let isEditDeckModalOpen = false;

  function handleDeleteCollection() {
    deleteCollection(id);
  }

  function openAddStudyMaterialModal() {
    isAddStudyMaterialModalOpen = true;
  }

  function closeAddStudyMaterialModal() {
    isAddStudyMaterialModalOpen = false;
  }

  async function handleMaterialsAdded(event: CustomEvent<StudyMaterial[]>) {
    const newMaterials = event.detail;
    addedMaterialsCount = newMaterials.length;
    showAddFeedback = true;
    setTimeout(() => {
      showAddFeedback = false;
    }, 3000);
    
    studyMaterials = [...studyMaterials, ...newMaterials];
    await addStudyMaterials(id, newMaterials);
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
      isEditDeckModalOpen = true;
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
    isEditDeckModalOpen = false
  }

  function handleDeckUpdate(event: CustomEvent<FlashcardDeck>) {
    const updatedDeck = event.detail;
    reviewMaterials = reviewMaterials.map(m => 
      ('flashcards' in m && m.id === updatedDeck.id) ? updatedDeck : m
    );
  }

  function handleAddReviewMaterial() {
  if (newMaterialName.trim()) {
    const newDeck: FlashcardDeck = {
      id: Date.now().toString(),
      name: newMaterialName.trim(),
      flashcards: []
    };
    addFlashcardDeck(id, newDeck);
    reviewMaterials = [...reviewMaterials, newDeck];
    
    // Reset the state after adding the new deck
    isAddingReviewMaterial = false;
    newMaterialName = '';
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

  function handleDragEnter(event: DragEvent) {
    event.preventDefault();
    dragCounter++;
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    dragCounter--;
    if (dragCounter === 0) {
      isDragging = false;
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.dataTransfer!.dropEffect = 'copy';
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    dragCounter = 0;
    const files = Array.from(event.dataTransfer!.files);
    
    const newMaterials: StudyMaterial[] = await Promise.all(
      files.map(async (file) => {
        const id = Date.now().toString() + Math.random().toString(36).substr(2, 9);
        const fileType = file.name.endsWith('.md') ? 'markdown' : 
                         file.name.endsWith('.pdf') ? 'pdf' : 'unknown';
        
        if (fileType === 'unknown') {
          console.warn(`Unsupported file type: ${file.name}`);
          return null;
        }

        const content = fileType === 'markdown' ? await file.text() : await file.arrayBuffer();
        const filePath = await window.electronAPI.saveFile(content, file.name, name);

        return { 
          id, 
          type: fileType, 
          filePath, 
          name: file.name 
        };
      })
    );

    const validMaterials = newMaterials.filter((m): m is StudyMaterial => m !== null);
    if (validMaterials.length > 0) {
      await handleMaterialsAdded({ detail: validMaterials } as CustomEvent<StudyMaterial[]>);
    }
  }

  async function handleRenameCollection(event: CustomEvent<string>) {
  const newName = event.detail;
  try {
    await collections.renameCollection(id, newName);
    name = newName;
  } catch (error) {
    console.error('Failed to rename collection:', error);
    alert('Failed to rename collection. Please try again.');
  }
}

  function handleExportCollection() {
    const exportData = JSON.stringify({ id, name, studyMaterials, reviewMaterials });
    navigator.clipboard.writeText(exportData)
      .then(() => alert("Collection data copied to clipboard!"))
      .catch(err => console.error('Failed to copy: ', err));
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

<div class="bg-gray-50 dark:bg-gray-800 shadow-lg rounded-lg overflow-hidden">
  <CollectionHeader 
    {name} 
    on:deleteCollection={handleDeleteCollection}
    on:renameCollection={handleRenameCollection}
    on:exportCollection={handleExportCollection}
    on:importCollection={handleImportCollection}
  />
  
  <div class="p-4 relative bg-white dark:bg-gray-800"
       on:dragenter={handleDragEnter}
       on:dragleave={handleDragLeave}
       on:dragover={handleDragOver}
       on:drop={handleDrop}>
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-xl font-semibold text-gray-800 dark:text-text-dark">Study Materials</h3>
      <button
        on:click={openAddStudyMaterialModal}
        class="p-1 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 focus:outline-none"
        title="Add Study Material"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>
    
    {#if studyMaterials.length > 0}
      <StudyMaterialsList 
        {studyMaterials} 
        collectionId={id} 
        on:viewFile={handleViewFile} 
        on:removeStudyMaterial={handleRemoveStudyMaterial}
      />
    {:else}
      <div class="text-center py-8 text-gray-600 dark:text-gray-400 bg-gray-100 dark:bg-gray-700 rounded-lg">
        <p>No study materials yet.</p>
        <p class="mt-2">You can drop files here to add study materials!</p>
      </div>
    {/if}

    {#if showAddFeedback}
      <p class="mt-2 text-green-600 dark:text-green-400">Successfully added {addedMaterialsCount} material{addedMaterialsCount !== 1 ? 's' : ''}</p>
    {/if}

    {#if isDragging}
      <div class="absolute inset-0 bg-blue-50 bg-opacity-90 dark:bg-blue-900 dark:bg-opacity-90 flex items-center justify-center z-10 border-2 border-dashed border-blue-500">
        <p class="text-2xl font-semibold text-blue-700 dark:text-blue-300">Drop files here to add study materials</p>
      </div>
    {/if}
  </div>

  <div class="p-4 bg-gray-100 dark:bg-gray-700">
    <div class="flex justify-between items-center mb-4">
      <h3 class="text-xl font-semibold text-gray-800 dark:text-text-dark">Flashcard Decks</h3>
      <button
        on:click={() => isAddingReviewMaterial = true}
        class="p-1 text-blue-600 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 focus:outline-none"
        title="Add Review Material"
      >
        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>

    {#if isAddingReviewMaterial}
      <div class="mb-4 flex flex-col space-y-2">
        <input
          type="text"
          bind:value={newMaterialName}
          placeholder="Enter name"
          class="p-2 border rounded text-gray-800 dark:text-gray-200 bg-white dark:bg-gray-600"
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
            class="px-4 py-2 bg-gray-300 text-gray-800 dark:bg-gray-600 dark:text-gray-200 rounded hover:bg-gray-400 dark:hover:bg-gray-500 flex-grow"
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
  isOpen={isEditDeckModalOpen}
  collectionId={id} 
  {studyMaterials}
  on:close={closeEditDeck}
  on:update={handleDeckUpdate}
/>