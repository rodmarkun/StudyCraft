<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Brain } from 'lucide-svelte';
  import { getLLMExplanation } from '../services/llmService';
  import { options } from '../stores/options';

  export let question: string;
  export let answer: string;

  let explanation = '';
  let loadingExplanation = false;
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

  async function askForExplanation() {
    loadingExplanation = true;
    explanation = await getLLMExplanation(question, answer);
    loadingExplanation = false;
  }
</script>

<div class="mt-4 flex flex-col space-y-4">
  <div class="flex justify-end">
    <button
      on:click={askForExplanation}
      class="flex items-center px-4 py-2 bg-purple-500 text-white rounded-full transition-colors duration-200 {isLLMConfigured && !loadingExplanation ? 'hover:bg-purple-600' : 'opacity-50 cursor-not-allowed'}"
      disabled={!isLLMConfigured || loadingExplanation}
    >
      <Brain class="mr-2" size={20} />
      {#if !isLLMConfigured}
        LLM Not Configured
      {:else if loadingExplanation}
        Thinking...
      {:else}
        Ask for Explanation
      {/if}
    </button>
  </div>
  {#if explanation}
    <div class="p-4 bg-gray-100 dark:bg-gray-700 rounded-lg">
      <h4 class="text-lg font-semibold mb-2 text-gray-800 dark:text-white">Explanation:</h4>
      <p class="text-gray-700 dark:text-gray-300">{explanation}</p>
    </div>
  {/if}
</div>