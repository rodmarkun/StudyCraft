<!-- src/lib/components/StudyDeckModal.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { FlashcardDeck, Flashcard } from '../stores/collections';
  import Modal from './Modal.svelte';
  import { Brain } from 'lucide-svelte';

  export let deck: FlashcardDeck | null;
  export let isOpen: boolean;

  const dispatch = createEventDispatcher();

  let currentIndex = 0;
  let showAnswer = false;
  let shuffledCards: Flashcard[] = [];
  let explanation = '';
  let loadingExplanation = false;

  $: if (deck) {
    shuffledCards = [...deck.flashcards];
    resetStudy();
  }

  function resetStudy() {
    currentIndex = 0;
    showAnswer = false;
    explanation = '';
  }

  function nextCard() {
    if (currentIndex < shuffledCards.length - 1) {
      currentIndex++;
      showAnswer = false;
      explanation = '';
    }
  }

  function previousCard() {
    if (currentIndex > 0) {
      currentIndex--;
      showAnswer = false;
      explanation = '';
    }
  }

  function toggleAnswer() {
    showAnswer = !showAnswer;
    if (!showAnswer) {
      explanation = '';
    }
  }

  function shuffleDeck() {
    shuffledCards = shuffledCards
      .map(value => ({ value, sort: Math.random() }))
      .sort((a, b) => a.sort - b.sort)
      .map(({ value }) => value);
    resetStudy();
  }

  async function askForExplanation() {
    const currentCard = shuffledCards[currentIndex];
    const prompt = `Please explain the following answer to the question "${currentCard.question}": ${currentCard.answer}`;
    
    loadingExplanation = true;
    try {
      const response = await fetch("http://localhost:11434/api/generate", {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          "model": "llama3",
          "prompt": prompt,
          "stream": false
        }),
      });

      if (!response.ok) {
        throw new Error('Network response was not ok');
      }

      const data = await response.json();
      explanation = data.response;
    } catch (error) {
      console.error('Error fetching explanation:', error);
      explanation = "Sorry, I couldn't fetch an explanation at this time.";
    } finally {
      loadingExplanation = false;
    }
  }

  function close() {
    dispatch('close');
  }
</script>

<Modal {isOpen} title={deck ? `Studying: ${deck.name}` : ''} on:close={close}>
  {#if deck && shuffledCards.length > 0}
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
          <div class="mt-4 flex justify-end">
            <button
              on:click={askForExplanation}
              class="flex items-center px-4 py-2 bg-purple-500 text-white rounded-full hover:bg-purple-600 transition-colors duration-200"
              disabled={loadingExplanation}
            >
              <Brain class="mr-2" size={20} />
              {loadingExplanation ? 'Thinking...' : 'Ask for Explanation'}
            </button>
          </div>
          {#if explanation}
            <div class="mt-4 p-4 bg-gray-100 dark:bg-gray-700 rounded-lg">
              <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-white">Explanation:</h4>
              <p class="text-gray-700 dark:text-gray-300">{explanation}</p>
            </div>
          {/if}
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
      <button
        on:click={shuffleDeck}
        class="px-6 py-2 bg-green-500 text-white rounded-full hover:bg-green-600 transition-colors duration-200"
      >
        Shuffle Deck
      </button>
    </div>
  {:else}
    <p class="text-gray-600 dark:text-gray-400">No flashcards in this deck.</p>
  {/if}
</Modal>