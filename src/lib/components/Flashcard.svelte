<!-- src/lib/components/Flashcard.svelte -->
<script lang="ts">
  export let question: string;
  export let answer: string;

  let showAnswer = false;

  function toggleAnswer() {
    showAnswer = !showAnswer;
  }
</script>

<div class="bg-white dark:bg-gray-800 shadow rounded-lg p-4">
  <h4 class="font-bold mb-2">Flashcard</h4>
  <p class="mb-4">{question}</p>
  <button 
    on:click={toggleAnswer}
    class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
  >
    {showAnswer ? 'Hide Answer' : 'Show Answer'}
  </button>
  {#if showAnswer}
    <p class="mt-4">{answer}</p>
  {/if}
</div>