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
  let isTestMode = false;
  let testResults: { correct: number, kinda: number, wrong: number } = { correct: 0, kinda: 0, wrong: 0 };
  let selectedAnswer: number | null = null;
  let showTestResults = false;

  $: if (deck) {
    shuffledCards = [...deck.flashcards];
    resetStudy();
  }

  $: if (selectedAnswer !== null) {
    shuffledCards = [...shuffledCards];
  }

  function resetStudy() {
    currentIndex = 0;
    showAnswer = false;
    isStudyStarted = false;
    isTestMode = false;
    testResults = { correct: 0, kinda: 0, wrong: 0 };
    selectedAnswer = null;
    showTestResults = false;
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
      selectedAnswer = null;
    } else if (isTestMode) {
      showTestResults = true;
      console.log('Test completed. Results:', testResults);
    }
  }

  function previousCard() {
    if (currentIndex > 0 && !isTestMode) {
      currentIndex--;
      showAnswer = false;
      selectedAnswer = null;
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

  function handleTestAnswer(result: 'correct' | 'kinda' | 'wrong') {
    testResults[result]++;
    nextCard();
  }

  function handleTestChoice(index: number) {
    selectedAnswer = index;
    const currentCard = shuffledCards[currentIndex];
    if (isTestFlashcard(currentCard)) {
      const correctIndex = JSON.parse(currentCard.answer).correctIndex;
      if (index === correctIndex) {
        testResults.correct++;
      } else {
        testResults.wrong++;
      }
    }
    shuffledCards = [...shuffledCards];
  }

  function isTestFlashcard(card: Flashcard): boolean {
    return card.isTestFlashcard || false;
  }

  function getButtonClass(index: number, currentCard: Flashcard) {
    if (!isTestFlashcard(currentCard)) return 'bg-gray-100';
    const correctIndex = JSON.parse(currentCard.answer).correctIndex;
    if (selectedAnswer === null) return 'bg-gray-100';
    if (index === correctIndex) return 'bg-green-200';
    if (index === selectedAnswer) return 'bg-red-200';
    return 'bg-gray-100';
  }

  function getCorrectAnswer(card: Flashcard): string {
    if (isTestFlashcard(card)) {
      const { options, correctIndex } = JSON.parse(card.answer);
      return options[correctIndex];
    }
    return card.answer;
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
        <label class="flex items-center space-x-2 text-gray-700 dark:text-gray-300">
          <input type="checkbox" bind:checked={isTestMode} class="form-checkbox h-5 w-5 text-blue-600">
          <span>Test mode</span>
        </label>
        <button
          on:click={startStudy}
          class="px-6 py-3 bg-green-500 text-white rounded-full hover:bg-green-600 transition-colors duration-200 text-lg font-semibold"
        >
          Start Reviewing
        </button>
      </div>
    {:else if showTestResults}
      <div class="flex flex-col items-center space-y-6 p-6">
        <h2 class="text-2xl font-bold text-gray-800 dark:text-white">Test Results</h2>
        <p class="text-lg text-gray-700 dark:text-gray-300">
          Correct: {testResults.correct}<br>
          Kinda: {testResults.kinda}<br>
          Wrong: {testResults.wrong}
        </p>
        <button
          on:click={resetStudy}
          class="px-6 py-3 bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors duration-200 text-lg font-semibold"
        >
          Start Over
        </button>
      </div>
    {:else}
      <div class="flex flex-col items-center space-y-4">
        <div class="w-full max-w-2xl bg-white dark:bg-gray-800 rounded-lg shadow-md p-6">
          <h3 class="text-xl font-bold mb-4 text-gray-800 dark:text-white">Question:</h3>
          <p class="text-lg mb-4 text-gray-700 dark:text-gray-300">
            {shuffledCards[currentIndex].question}
          </p>
          {#if isTestMode && isTestFlashcard(shuffledCards[currentIndex])}
            {#each JSON.parse(shuffledCards[currentIndex].answer).options as option, index}
              <button
                on:click={() => handleTestChoice(index)}
                class="w-full p-2 mb-2 text-left {getButtonClass(index, shuffledCards[currentIndex])} rounded"
                disabled={selectedAnswer !== null}
              >
                {option}
              </button>
            {/each}
            {#if selectedAnswer !== null}
              <button
                on:click={nextCard}
                class="mt-4 px-6 py-2 bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors duration-200"
              >
                Next
              </button>
            {/if}
          {:else if showAnswer}
            <h3 class="text-xl font-bold mb-4 text-gray-800 dark:text-white">Answer:</h3>
            <p class="text-lg text-gray-700 dark:text-gray-300">
              {getCorrectAnswer(shuffledCards[currentIndex])}
            </p>
            <LLMExplanation
              question={shuffledCards[currentIndex].question}
              answer={getCorrectAnswer(shuffledCards[currentIndex])}
            />
            {#if isTestMode}
              <div class="flex space-x-4 mt-4">
                <button
                  on:click={() => handleTestAnswer('wrong')}
                  class="px-6 py-2 bg-red-500 text-white rounded-full hover:bg-red-600 transition-colors duration-200"
                >
                  Wrong
                </button>
                <button
                  on:click={() => handleTestAnswer('kinda')}
                  class="px-6 py-2 bg-yellow-500 text-white rounded-full hover:bg-yellow-600 transition-colors duration-200"
                >
                  Kinda
                </button>
                <button
                  on:click={() => handleTestAnswer('correct')}
                  class="px-6 py-2 bg-green-500 text-white rounded-full hover:bg-green-600 transition-colors duration-200"
                >
                  Got it!
                </button>
              </div>
            {/if}
          {/if}
        </div>
        {#if !isTestMode || (isTestMode && !isTestFlashcard(shuffledCards[currentIndex]))}
          <button
            on:click={toggleAnswer}
            class="px-6 py-2 bg-blue-500 text-white rounded-full hover:bg-blue-600 transition-colors duration-200"
          >
            {showAnswer ? 'Hide Answer' : 'Show Answer'}
          </button>
        {/if}
        <div class="flex justify-between w-full max-w-2xl">
          {#if !isTestMode}
            <button
              on:click={previousCard}
              disabled={currentIndex === 0}
              class="px-4 py-2 bg-gray-300 text-gray-800 rounded-full hover:bg-gray-400 disabled:opacity-50 transition-colors duration-200"
            >
              Previous
            </button>
          {:else}
            <div></div>
          {/if}
          <span class="text-gray-600 dark:text-gray-400">
            {currentIndex + 1} / {shuffledCards.length}
          </span>
          {#if !isTestMode}
            <button
              on:click={nextCard}
              disabled={currentIndex === shuffledCards.length - 1}
              class="px-4 py-2 bg-gray-300 text-gray-800 rounded-full hover:bg-gray-400 disabled:opacity-50 transition-colors duration-200"
            >
              Next
            </button>
          {:else}
            <div></div>
          {/if}
        </div>
      </div>
    {/if}
  {:else}
    <p class="text-gray-600 dark:text-gray-400">No flashcards in this deck.</p>
  {/if}
</Modal>