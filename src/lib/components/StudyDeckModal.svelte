<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { FlashcardDeck, Flashcard } from '../stores/collections';
  import Modal from './Modal.svelte';
  import LLMExplanation from './LLMExplanation.svelte';

  export let deck: FlashcardDeck | null;
  export let isOpen: boolean;

  const dispatch = createEventDispatcher();

  let currentIndex = 0;
  let showAnswer = false;
  let shuffledCards: Flashcard[] = [];
  let shouldShuffle = true;
  let isStudyStarted = false;

  $: if (deck) {
    shuffledCards = [...deck.flashcards];
    resetStudy();
  }

  function resetStudy() {
    currentIndex = 0;
    showAnswer = false;
    isStudyStarted = false;
  }

  function startStudy() {
    if (shouldShuffle) {
      shuffleDeck();
    }
    isStudyStarted = true;
  }

  function nextCard() {
    if (currentIndex < shuffledCards.length - 1) {
      currentIndex++;
      showAnswer = false;
    }
  }

  function previousCard() {
    if (currentIndex > 0) {
      currentIndex--;
      showAnswer = false;
    }
  }

  function toggleAnswer() {
    showAnswer = !showAnswer;
  }

  function shuffleDeck() {
    shuffledCards = shuffledCards
      .map(value => ({ value, sort: Math.random() }))
      .sort((a, b) => a.sort - b.sort)
      .map(({ value }) => value);
  }

  function close() {
    dispatch('close');
  }
</script>

<Modal {isOpen} title={deck ? `Studying: ${deck.name}` : ''} on:close={close}>
  {#if deck && shuffledCards.length > 0}
    {#if !isStudyStarted}
      <div class="flex flex-col items-center space-y-6 p-6">
        <h2 class="text-2xl font-bold text-gray-800 dark:text-white">Ready to study?</h2>
        <p class="text-lg text-gray-700 dark:text-gray-300">
          This deck contains {shuffledCards.length} flashcard{shuffledCards.length !== 1 ? 's' : ''}.
        </p>
        <label class="flex items-center space-x-2 text-gray-700 dark:text-gray-300">
          <input type="checkbox" bind:checked={shouldShuffle} class="form-checkbox h-5 w-5 text-blue-600">
          <span>Shuffle deck</span>
        </label>
        <button
          on:click={startStudy}
          class="px-6 py-3 bg-green-500 text-white rounded-full hover:bg-green-600 transition-colors duration-200 text-lg font-semibold"
        >
          Start Reviewing
        </button>
      </div>
    {:else}
      <div class="flex flex-col items-center space-y-4">
        <div class="w-full max-w-2xl bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
          <h3 class="text-xl font-bold mb-4 text-gray-800 dark:text-white">Question:</h3>
          <p class="text-lg mb-4 text-gray-700 dark:text-gray-300">
            {shuffledCards[currentIndex].question}
          </p>
          {#if showAnswer}
            <h3 class="text-xl font-bold mb-4 text-gray-800 dark:text-white">Answer:</h3>
            <p class="text-lg text-gray-700 dark:text-gray-300">
              {shuffledCards[currentIndex].answer}
            </p>
            <LLMExplanation
              question={shuffledCards[currentIndex].question}
              answer={shuffledCards[currentIndex].answer}
            />
          {/if}
        </div>
        <button
          on:click={toggleAnswer}
          class="px-6 py-2 bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors duration-200"
        >
          {showAnswer ? 'Hide Answer' : 'Show Answer'}
        </button>
        <div class="flex justify-between w-full max-w-2xl">
          <button
            on:click={previousCard}
            disabled={currentIndex === 0}
            class="px-4 py-2 bg-gray-300 text-gray-800 rounded-full hover:bg-gray-400 disabled:opacity-50 transition-colors duration-200"
          >
            Previous
          </button>
          <span class="text-gray-600 dark:text-gray-400">
            {currentIndex + 1} / {shuffledCards.length}
          </span>
          <button
            on:click={nextCard}
            disabled={currentIndex === shuffledCards.length - 1}
            class="px-4 py-2 bg-gray-300 text-gray-800 rounded-full hover:bg-gray-400 disabled:opacity-50 transition-colors duration-200"
          >
            Next
          </button>
        </div>
      </div>
    {/if}
  {:else}
    <p class="text-gray-600 dark:text-gray-400">No flashcards in this deck.</p>
  {/if}
</Modal>