<!-- src/lib/components/FlashcardDeck.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { FlashcardDeck, Flashcard } from '../stores/collections';

  export let deck: FlashcardDeck;

  const dispatch = createEventDispatcher();

  let isAddingCard = false;
  let newQuestion = '';
  let newAnswer = '';

  function addFlashcard() {
    if (newQuestion.trim() && newAnswer.trim()) {
      const newCard: Flashcard = {
        id: Date.now().toString(),
        question: newQuestion.trim(),
        answer: newAnswer.trim()
      };
      dispatch('addCard', { deckId: deck.id, card: newCard });
      newQuestion = '';
      newAnswer = '';
      isAddingCard = false;
    }
  }

  function removeFlashcard(cardId: string) {
    dispatch('removeCard', { deckId: deck.id, cardId });
  }

  function startAddingCard() {
    isAddingCard = true;
  }

  function cancelAddingCard() {
    isAddingCard = false;
    newQuestion = '';
    newAnswer = '';
  }
</script>

<div class="bg-white dark:bg-gray-800 shadow-lg rounded-lg p-4">
  <h3 class="text-xl font-bold mb-4">{deck.name}</h3>
  <p class="mb-4">Total cards: {deck.flashcards.length}</p>
  
    {#if isAddingCard}
      <div class="mb-4 space-y-2">
        <input
          type="text"
          bind:value={newQuestion}
          placeholder="Question"
          class="w-full p-2 border rounded text-text-light"
        />
        <input
          type="text"
          bind:value={newAnswer}
          placeholder="Answer"
          class="w-full p-2 border rounded text-text-light"
        />
        <div class="flex justify-end space-x-2">
          <button
            on:click={cancelAddingCard}
            class="px-3 py-1 bg-gray-300 text-gray-800 rounded hover:bg-gray-400"
          >
            Cancel
          </button>
          <button
            on:click={addFlashcard}
            class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Add Card
          </button>
        </div>
      </div>
    {:else}
      <button
        on:click={startAddingCard}
        class="mb-4 px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
      >
        Add New Card
      </button>
    {/if}
  
    <div class="space-y-2">
      {#each deck.flashcards as card}
        <div class="flex justify-between items-center p-2 bg-gray-100 dark:bg-gray-700 rounded">
          <span class="truncate">{card.question}</span>
          <button
            on:click={() => removeFlashcard(card.id)}
            class="px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
          >
            Remove
          </button>
        </div>
      {/each}
    </div>
  </div>