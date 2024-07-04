<!-- src/lib/components/Flashcard.svelte -->
<script lang="ts">
    export let question: string;
    export let answer: string;
    let showAnswer = false;
  
    function toggleAnswer() {
      showAnswer = !showAnswer;
    }
  </script>
  
  <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-md p-4 mb-4 cursor-pointer" on:click={toggleAnswer}>
    <div class="font-semibold mb-2">{question}</div>
    {#if showAnswer}
      <div class="text-gray-600 dark:text-gray-400 mt-2 italic">{answer}</div>
    {/if}
  </div>