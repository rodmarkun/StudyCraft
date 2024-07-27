<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { Brain } from 'lucide-svelte';
  import type { FlashcardDeck, Flashcard, StudyMaterial } from '../stores/collections';
  import Modal from './Modal.svelte';
  import Popover from './Popover.svelte';
  import { updateFlashcardDeck } from '../stores/collections';
  import { generateFlashcards } from '../services/llmService';
  import { loadStudyMaterialContent } from '../stores/collections';
  import { scrapeWebsite } from '../services/webScraperService';
  import { cleanPDFContent } from '../utils/fileUtils';
  import { options } from '../stores/options';

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
  let isLLMConfigured = false;
  let unsubscribe: () => void;

  onMount(() => {
    unsubscribe = options.subscribe(value => {
      isLLMConfigured = value.aiConfig.provider !== 'none';
    });
  });

  onDestroy(() => {
    if (unsubscribe) unsubscribe();
  });

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
        content = await cleanPDFContent(selectedMaterial.filePath!);
      } else if (selectedMaterial.type === 'webpage') {
        content = await scrapeWebsite(selectedMaterial.url!);
      } else {
        throw new Error('Unsupported material type');
      }

      const generatedCards = await generateFlashcards(content, numberOfCardsToGenerate);

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
  <div class="flex flex-col h-full">
    {#if errorMessage}
      <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4" role="alert">
        <strong class="font-bold">Error!</strong>
        <span class="block sm:inline">{errorMessage}</span>
      </div>
    {/if}

    {#if editedDeck}
      <div class="flex-shrink-0 mb-4">
        <h3 class="text-lg font-semibold mb-2">Add New Flashcard</h3>
        <input
          type="text"
          bind:value={newQuestion}
          placeholder="Question"
          class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white dark:bg-gray-700"
        />
        <input
          type="text"
          bind:value={newAnswer}
          placeholder="Answer"
          class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white dark:bg-gray-700"
        />
        <div class="flex space-x-2">
          <button
            on:click={addFlashcard}
            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
          >
            Add Flashcard
          </button>
          <Popover bind:open={isGeneratePopoverOpen} width="w-96" customClass="dark:bg-gray-700">
            <button
              slot="trigger"
              on:click={toggleGeneratePopover}
              class="flex items-center px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 transition-colors duration-200 {isLLMConfigured && !isGenerating ? 'hover:bg-purple-600' : 'opacity-50 cursor-not-allowed'}"
              disabled={!isLLMConfigured || isGenerating}
            >
              <Brain class="mr-2" size={20} />
              {#if !isLLMConfigured}
                LLM Not Configured
              {:else if isGenerating}
                Generating...
              {:else}
                Generate Flashcards
              {/if}
            </button>
            <div class="p-4 space-y-4">
              <h3 class="text-lg font-semibold mb-2 text-gray-800 dark:text-white">Generate Flashcards</h3>
              <p class="text-sm text-gray-600 dark:text-gray-300 mb-4">
                Select a study material and specify the number of flashcards you want to generate.
                The AI will create flashcards based on the content of the selected material.
              </p>
              <select
                bind:value={selectedMaterialId}
                class="w-full p-2 border rounded text-gray-800 dark:text-white dark:bg-gray-600"
              >
                <option value="">Select a study material</option>
                {#each studyMaterials as material}
                  <option value={material.id}>{material.name}</option>
                {/each}
              </select>
              <div class="mt-2">
                <label for="numCards" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Number of flashcards to generate:
                </label>
                <input
                  id="numCards"
                  type="number"
                  bind:value={numberOfCardsToGenerate}
                  min="1"
                  max="20"
                  class="mt-1 w-full p-2 border rounded text-gray-800 dark:text-white dark:bg-gray-600"
                />
              </div>
              <button
                on:click={handleGenerateFlashcards}
                class="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 flex items-center justify-center {isLLMConfigured && !isGenerating && selectedMaterialId ? 'hover:bg-indigo-600' : 'opacity-50 cursor-not-allowed'}"
                disabled={!isLLMConfigured || isGenerating || !selectedMaterialId}
              >
                <Brain class="mr-2" size={20} />
                {isGenerating ? 'Generating...' : 'Generate Flashcards'}
              </button>
            </div>
          </Popover>
        </div>
      </div>
      <div class="flex-grow overflow-y-auto mb-4">
        <h3 class="text-lg font-semibold mb-2">Existing Flashcards</h3>
        {#each editedDeck.flashcards as card (card.id)}
          <div class="border p-2 rounded mb-2">
            <input
              type="text"
              value={card.question}
              on:input={(e) => updateFlashcard(card.id, 'question', e.target.value)}
              class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white dark:bg-gray-700"
            />
            <input
              type="text"
              value={card.answer}
              on:input={(e) => updateFlashcard(card.id, 'answer', e.target.value)}
              class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white dark:bg-gray-700"
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
      <div class="flex-shrink-0">
        <button
          on:click={save}
          class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          disabled={!collectionId}
        >
          Save Changes
        </button>
      </div>
    {:else}
      <p>No deck selected for editing.</p>
    {/if}
  </div>
</Modal>