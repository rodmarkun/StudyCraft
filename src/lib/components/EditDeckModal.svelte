<!-- src/lib/components/EditDeckModal.svelte -->
<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import type { FlashcardDeck, Flashcard } from '../stores/collections';
    import Modal from './Modal.svelte';
  
    export let deck: FlashcardDeck | null;
    export let isOpen: boolean;
  
    const dispatch = createEventDispatcher();
  
    let editedDeck: FlashcardDeck | null = null;
    let newQuestion = '';
    let newAnswer = '';
  
    $: if (deck) {
      editedDeck = JSON.parse(JSON.stringify(deck));
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
        const cardIndex = editedDeck.flashcards.findIndex(card => card.id === id);
        if (cardIndex !== -1) {
          editedDeck.flashcards[cardIndex] = {
            ...editedDeck.flashcards[cardIndex],
            [field]: value
          };
        }
      }
    }
  
    function save() {
      if (editedDeck) {
        dispatch('update', editedDeck);
      }
      close();
    }
  
    function close() {
      dispatch('close');
    }
  </script>
  
  <Modal {isOpen} title={deck ? `Editing: ${deck.name}` : ''} on:close={close}>
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
          <button
            on:click={addFlashcard}
            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
          >
            Add Flashcard
          </button>
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
        >
          Save Changes
        </button>
      </div>
    {:else}
      <p>No deck selected for editing.</p>
    {/if}
  </Modal>