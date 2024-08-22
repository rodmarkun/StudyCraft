<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { Brain, PlusCircle, X } from 'lucide-svelte';
  import type { FlashcardDeck, Flashcard, StudyMaterial } from '../stores/collections';
  import Modal from './Modal.svelte';
  import Popover from './Popover.svelte';
  import { updateFlashcardDeck } from '../stores/collections';
  import { generateFlashcards, generateTestFlashcards } from '../services/llmService';
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

  let selectedFlashcardType: 'normal' | 'test' = 'normal';
  let newTestAnswers = ['', '', '', ''];
  let newCorrectAnswerIndex = 0;

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

  function toggleFlashcardType(type: 'normal' | 'test') {
    selectedFlashcardType = selectedFlashcardType === type ? null : type;
    resetNewFlashcardInputs();
  }

  function resetNewFlashcardInputs() {
    newQuestion = '';
    newAnswer = '';
    newTestAnswers = ['', '', '', ''];
    newCorrectAnswerIndex = 0;
  }

  function addFlashcard() {
    if (editedDeck && newQuestion.trim()) {
      if (selectedFlashcardType === 'test') {
        const validAnswers = newTestAnswers.filter(answer => answer.trim() !== '');
        if (validAnswers.length >= 2 && newCorrectAnswerIndex < validAnswers.length) {
          editedDeck.flashcards = [
            ...editedDeck.flashcards,
            {
              id: Date.now().toString(),
              question: newQuestion.trim(),
              answer: JSON.stringify({
                options: validAnswers,
                correctIndex: newCorrectAnswerIndex
              }),
              isTestFlashcard: true
            }
          ];
          resetNewFlashcardInputs();
        } else {
          errorMessage = 'Test flashcard must have at least 2 valid answers and a correct answer selected.';
        }
      } else if (selectedFlashcardType === 'normal' && newAnswer.trim()) {
        editedDeck.flashcards = [
          ...editedDeck.flashcards,
          {
            id: Date.now().toString(),
            question: newQuestion.trim(),
            answer: newAnswer.trim(),
            isTestFlashcard: false
          }
        ];
        resetNewFlashcardInputs();
      }
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
        return true;
      } catch (error) {
        console.error('Error updating flashcard deck:', error);
        errorMessage = 'Failed to save changes. Please try again.';
        return false;
      }
    } else {
      errorMessage = 'Cannot save: Missing deck or collection ID';
      return false;
    }
  }

  async function close() {
    errorMessage = '';
    const saveSuccessful = await save();
    if (saveSuccessful) {
      dispatch('close');
    } else {
      const confirmClose = confirm('Failed to save changes. Close anyway?');
      if (confirmClose) {
        dispatch('close');
      }
    }
  }

  function toggleGeneratePopover() {
    isGeneratePopoverOpen = !isGeneratePopoverOpen;
  }

  async function handleGenerateFlashcards() {
    await generateFlashcardsFromMaterial(selectedFlashcardType === 'test');
    isGeneratePopoverOpen = false;
  }

  async function generateFlashcardsFromMaterial(isTestFlashcard: boolean) {
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

      let generatedCards;
      if (isTestFlashcard) {
        generatedCards = await generateTestFlashcards(content, numberOfCardsToGenerate);
      } else {
        generatedCards = await generateFlashcards(content, numberOfCardsToGenerate);
      }

      const existingQuestions = new Set(editedDeck.flashcards.map(card => card.question.toLowerCase()));

      const newUniqueCards = generatedCards.filter(card => !existingQuestions.has(card.question.toLowerCase()));

      editedDeck.flashcards = [
        ...editedDeck.flashcards,
        ...newUniqueCards.map((card) => ({
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          question: card.question,
          answer: isTestFlashcard ? JSON.stringify(card.answer) : card.answer,
          isTestFlashcard
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
      <div class="bg-red-100 border border-red-400 text-red-700 dark:bg-red-900 dark:border-red-600 dark:text-red-100 px-4 py-3 rounded relative mb-4" role="alert">
        <strong class="font-bold">Error!</strong>
        <span class="block sm:inline">{errorMessage}</span>
      </div>
    {/if}

    {#if editedDeck}
      <div class="flex items-center mb-6">
        <h3 class="text-lg font-semibold dark:text-white mr-4">Add Flashcards</h3>
        <div class="flex space-x-2">
          <div class="relative group">
            <button
              class="p-2 rounded-md {selectedFlashcardType === 'normal' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'} transition-colors duration-200"
              on:click={() => toggleFlashcardType('normal')}
            >
              Normal
            </button>
            <div class="absolute top-full left-1/2 transform -translate-x-1/2 mt-2 w-48 bg-white dark:bg-gray-800 p-2 rounded shadow-lg text-sm text-gray-800 dark:text-gray-200 invisible group-hover:visible transition-opacity duration-200 opacity-0 group-hover:opacity-100 z-50">
              Standard flashcards with a question on one side and the answer on the other.
            </div>
          </div>
          <div class="relative group">
            <button
              class="p-2 rounded-md {selectedFlashcardType === 'test' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'} transition-colors duration-200"
              on:click={() => toggleFlashcardType('test')}
            >
              Test
            </button>
            <div class="absolute top-full left-1/2 transform -translate-x-1/2 mt-2 w-48 bg-white dark:bg-gray-800 p-2 rounded shadow-lg text-sm text-gray-800 dark:text-gray-200 invisible group-hover:visible transition-opacity duration-200 opacity-0 group-hover:opacity-100 z-50">
              Multiple-choice flashcards that function as standard cards during study, but present options in test mode for self-assessment.
            </div>
          </div>
        </div>
      </div>

      {#if selectedFlashcardType}
        <div class="bg-gray-100 dark:bg-gray-800 p-4 rounded-lg mb-4 transition-all duration-300">
          <input
            type="text"
            bind:value={newQuestion}
            placeholder="Question"
            class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white bg-white dark:bg-gray-700"
          />
          {#if selectedFlashcardType === 'test'}
            {#each newTestAnswers as answer, index}
              <div class="flex mb-2">
                <input
                  type="text"
                  bind:value={newTestAnswers[index]}
                  placeholder="Answer option {index + 1}"
                  class="flex-grow p-2 border rounded text-gray-800 dark:text-white bg-white dark:bg-gray-700"
                />
                <input
                  type="radio"
                  bind:group={newCorrectAnswerIndex}
                  value={index}
                  class="ml-2"
                />
              </div>
            {/each}
          {:else}
            <input
              type="text"
              bind:value={newAnswer}
              placeholder="Answer"
              class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white bg-white dark:bg-gray-700"
            />
          {/if}
          <div class="flex items-center space-x-2 mt-2">
            <button
              on:click={addFlashcard}
              class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 transition-colors duration-200 flex items-center"
            >
              <PlusCircle class="mr-2" size={18} />
              Add
            </button>
            <Popover bind:open={isGeneratePopoverOpen} width="w-80" customClass="dark:bg-gray-700">
              <button
                slot="trigger"
                on:click={toggleGeneratePopover}
                class="flex items-center px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 transition-colors duration-200 {isLLMConfigured && !isGenerating ? 'hover:bg-purple-600' : 'opacity-50 cursor-not-allowed'}"
                disabled={!isLLMConfigured || isGenerating}
              >
                <Brain class="mr-2" size={18} />
                {#if !isLLMConfigured}
                  LLM Not Configured
                {:else if isGenerating}
                  Generating...
                {:else}
                  Generate
                {/if}
              </button>
              <div class="p-3 space-y-3">
                <h3 class="text-base font-semibold text-gray-800 dark:text-white">Generate Flashcards</h3>
                <p class="text-xs text-gray-600 dark:text-gray-300">
                  Select a study material and specify the number of flashcards to generate.
                </p>
                <select
                  bind:value={selectedMaterialId}
                  class="w-full p-2 text-sm border rounded text-gray-800 dark:text-white dark:bg-gray-600"
                >
                  <option value="">Select a study material</option>
                  {#each studyMaterials as material}
                    <option value={material.id}>{material.name}</option>
                  {/each}
                </select>
                <div class="flex items-center space-x-2">
                  <label for="numCards" class="text-xs font-medium text-gray-700 dark:text-gray-300 whitespace-nowrap">
                    Number of cards:
                  </label>
                  <input
                    id="numCards"
                    type="number"
                    bind:value={numberOfCardsToGenerate}
                    min="1"
                    max="20"
                    class="w-full p-1 text-sm border rounded text-gray-800 dark:text-white dark:bg-gray-600"
                  />
                </div>
                <button
                  on:click={handleGenerateFlashcards}
                  class="w-full px-3 py-2 text-sm bg-indigo-500 text-white rounded hover:bg-indigo-600 flex items-center justify-center {isLLMConfigured && !isGenerating && selectedMaterialId ? 'hover:bg-indigo-600' : 'opacity-50 cursor-not-allowed'}"
                  disabled={!isLLMConfigured || isGenerating || !selectedMaterialId}
                >
                  <Brain class="mr-2" size={16} />
                  {isGenerating ? 'Generating...' : 'Generate Flashcards'}
                </button>
              </div>
            </Popover>          
          </div>
        </div>
      {/if}

      <div class="flex-grow overflow-y-auto mb-4">
        <h3 class="text-lg font-semibold mb-2 dark:text-white">Existing Flashcards</h3>
        {#each editedDeck.flashcards as card (card.id)}
          <div class="border dark:border-gray-600 p-4 rounded-lg mb-4 bg-white dark:bg-gray-800 shadow-sm">
            <div class="flex justify-between items-center mb-2">
              <span class="font-semibold text-gray-700 dark:text-gray-300">
                {card.isTestFlashcard ? 'Test Flashcard' : 'Normal Flashcard'}
              </span>
              <button
                on:click={() => removeFlashcard(card.id)}
                class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 transition-colors duration-200"
              >
                <X size={18} />
              </button>
            </div>
            <input
              type="text"
              value={card.question}
              on:input={(e) => updateFlashcard(card.id, 'question', e.target.value)}
              class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white bg-gray-50 dark:bg-gray-700"
            />
            {#if card.isTestFlashcard}
              {#each JSON.parse(card.answer).options as option, index}
                <div class="flex mb-2">
                  <input
                    type="text"
                    value={option}
                    on:input={(e) => {
                      const updatedAnswer = JSON.parse(card.answer);
                      updatedAnswer.options[index] = e.target.value;
                      updateFlashcard(card.id, 'answer', JSON.stringify(updatedAnswer));
                    }}
                    class="flex-grow p-2 border rounded text-gray-800 dark:text-white bg-gray-50 dark:bg-gray-700"
                  />
                  <input
                    type="radio"
                    checked={index === JSON.parse(card.answer).correctIndex}
                    on:change={() => {
                      const updatedAnswer = JSON.parse(card.answer);
                      updatedAnswer.correctIndex = index;
                      updateFlashcard(card.id, 'answer', JSON.stringify(updatedAnswer));
                    }}
                    class="ml-2"
                  />
                </div>
              {/each}
            {:else}
              <input
                type="text"
                value={card.answer}
                on:input={(e) => updateFlashcard(card.id, 'answer', e.target.value)}
                class="w-full p-2 border rounded mb-2 text-gray-800 dark:text-white bg-gray-50 dark:bg-gray-700"
              />
            {/if}
          </div>
        {/each}
      </div>
      <div class="flex-shrink-0">
        <button
          on:click={close}
          class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors duration-200"
          disabled={!collectionId}
        >
          Save Changes
        </button>
      </div>
    {:else}
      <p class="dark:text-white">No deck selected for editing.</p>
    {/if}
  </div>
</Modal>