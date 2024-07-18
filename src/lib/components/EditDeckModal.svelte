<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { FlashcardDeck, Flashcard, StudyMaterial } from '../stores/collections';
  import Modal from './Modal.svelte';
  import Popover from './Popover.svelte';
  import { updateFlashcardDeck } from '../stores/collections';
  import { generateFlashcards } from '../services/llmService';
  import { loadStudyMaterialContent } from '../stores/collections';
  import { scrapeWebsite } from '../services/webScraperService';

  export let deck: FlashcardDeck | null;
  export let isOpen: boolean;
  export let collectionId: string | undefined;
  export let studyMaterials: StudyMaterial[];

  const dispatch = createEventDispatcher();

  let editedDeck: FlashcardDeck | null = null;
  let newQuestion = '';
  let newAnswer = '';
  let errorMessage = '';
  let numberOfCardsToGenerate = 5;
  let selectedMaterialId = '';
  let isGenerating = false;
  let isGeneratePopoverOpen = false;

  $: if (deck) {
    editedDeck = JSON.parse(JSON.stringify(deck));
  }

  $: if (collectionId === undefined) {
    console.error('CollectionId is undefined in EditDeckModal');
    errorMessage = 'Error: Collection ID is missing';
  } else {
    errorMessage = '';
  }

  function toggleGeneratePopover() {
    isGeneratePopoverOpen = !isGeneratePopoverOpen;
  }

  async function handleGenerateFlashcards() {
    await generateFlashcardsFromMaterial();
    isGeneratePopoverOpen = false;
  }

  function addFlashcard() {
    if (editedDeck && newQuestion.trim() && newAnswer.trim()) {
      editedDeck.flashcards = [
        ...editedDeck.flashcards,
        {
          id: Date.now().toString(),
          question: newQuestion.trim(),
          answer: newAnswer.trim()
        }
      ];
      newQuestion = '';
      newAnswer = '';
    }
  }

  function removeFlashcard(id: string) {
    if (editedDeck) {
      editedDeck.flashcards = editedDeck.flashcards.filter(card => card.id !== id);
    }
  }

  function updateFlashcard(id: string, field: 'question' | 'answer', value: string) {
    if (editedDeck) {
      editedDeck.flashcards = editedDeck.flashcards.map(card =>
        card.id === id ? { ...card, [field]: value } : card
      );
    }
  }

  async function save() {
    if (editedDeck && collectionId) {
      try {
        await updateFlashcardDeck(collectionId, editedDeck);
        dispatch('update', editedDeck);
        close();
      } catch (error) {
        console.error('Error updating flashcard deck:', error);
        errorMessage = 'Failed to save changes. Please try again.';
      }
    } else {
      errorMessage = 'Cannot save: Missing deck or collection ID';
    }
  }

  function close() {
    dispatch('close');
  }

  async function generateFlashcardsFromMaterial() {
    if (!editedDeck || !selectedMaterialId) return;

    const selectedMaterial = studyMaterials.find(m => m.id === selectedMaterialId);
    if (!selectedMaterial) {
      errorMessage = 'Selected study material not found.';
      return;
    }

    isGenerating = true;
    errorMessage = '';
    try {
      let content;
      if (selectedMaterial.type === 'markdown') {
        content = await loadStudyMaterialContent(selectedMaterial.filePath!);
      } else if (selectedMaterial.type === 'pdf') {
        content = await loadStudyMaterialContent(selectedMaterial.filePath!);
      } else if (selectedMaterial.type === 'webpage') {
        content = await scrapeWebsite(selectedMaterial.url!);
      } else {
        throw new Error('Unsupported material type');
      }

      const generatedCards = await generateFlashcards(content, numberOfCardsToGenerate, selectedMaterial.type);


      const existingQuestions = new Set(editedDeck.flashcards.map(card => card.question.toLowerCase()));

      const newUniqueCards = generatedCards.filter(card => !existingQuestions.has(card.question.toLowerCase()));

      editedDeck.flashcards = [
        ...editedDeck.flashcards,
        ...newUniqueCards.map((card) => ({
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          question: card.question,
          answer: card.answer
        }))
      ];

      errorMessage = '';
      isGeneratePopoverOpen = false;
    } catch (error) {
      console.error('Error generating flashcards:', error);
      errorMessage = `Failed to generate flashcards: ${error.message}. Please try again or check your LLM configuration.`;
    } finally {
      isGenerating = false;
    }
  }
</script>

<Modal {isOpen} title={deck ? `Editing: ${deck.name}` : ''} on:close={close}>
  {#if errorMessage}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
      <strong class="font-bold">Error!</strong>
      <span class="block sm:inline">{errorMessage}</span>
    </div>
  {/if}

  {#if editedDeck}
    <div class="space-y-4">
      <div>
        <h3 class="text-lg font-semibold mb-2">Add New Flashcard</h3>
        <input
          type="text"
          bind:value={newQuestion}
          placeholder="Question"
          class="w-full p-2 border rounded mb-2 text-gray-800"
        />
        <input
          type="text"
          bind:value={newAnswer}
          placeholder="Answer"
          class="w-full p-2 border rounded mb-2 text-gray-800"
        />
        <div class="flex space-x-2">
          <button
            on:click={addFlashcard}
            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
          >
            Add Flashcard
          </button>
          <Popover bind:open={isGeneratePopoverOpen}>
            <button
              slot="trigger"
              on:click={toggleGeneratePopover}
              class="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600"
            >
              Generate Flashcards
            </button>
            <div class="p-4 space-y-4">
              <select
                bind:value={selectedMaterialId}
                class="w-full p-2 border rounded text-gray-800"
              >
                <option value="">Select a study material</option>
                {#each studyMaterials as material}
                  <option value={material.id}>{material.name}</option>
                {/each}
              </select>
              <input
                type="number"
                bind:value={numberOfCardsToGenerate}
                min="1"
                max="20"
                class="w-full p-2 border rounded text-gray-800"
                placeholder="Number of flashcards"
              />
              <button
                on:click={handleGenerateFlashcards}
                class="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600"
                disabled={isGenerating || !selectedMaterialId}
              >
                {isGenerating ? 'Generating...' : 'Generate'}
              </button>
            </div>
          </Popover>
        </div>
      </div>
      <div>
        <h3 class="text-lg font-semibold mb-2">Existing Flashcards</h3>
        {#each editedDeck.flashcards as card (card.id)}
          <div class="border p-2 rounded mb-2">
            <input
              type="text"
              value={card.question}
              on:input={(e) => updateFlashcard(card.id, 'question', e.target.value)}
              class="w-full p-2 border rounded mb-2 text-gray-800"
            />
            <input
              type="text"
              value={card.answer}
              on:input={(e) => updateFlashcard(card.id, 'answer', e.target.value)}
              class="w-full p-2 border rounded mb-2 text-gray-800"
            />
            <button
              on:click={() => removeFlashcard(card.id)}
              class="px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
            >
              Remove
            </button>
          </div>
        {/each}
      </div>
      <button
        on:click={save}
        class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        disabled={!collectionId}
      >
        Save Changes
      </button>
    </div>
  {:else}
    <p>No deck selected for editing.</p>
  {/if}
</Modal>