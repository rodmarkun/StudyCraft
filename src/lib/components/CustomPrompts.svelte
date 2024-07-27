<script lang="ts">
    import { options } from '../stores/options';
    import type { CustomPrompts } from '../stores/options';
  
    let localCustomPrompts: CustomPrompts;
    $: localCustomPrompts = $options.customPrompts;
  
    function handleCustomPromptChange(key: keyof CustomPrompts, value: string) {
      options.setCustomPrompt(key, value);
    }
  
    function resetCustomPrompts() {
      options.resetCustomPrompts();
    }
  </script>
  
  <div class="options-section">
    <h2 class="section-title">Custom Prompts</h2>
    <div class="space-y-8">
      <div>
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Flashcard Question Prompt</h3>
        <textarea
          bind:value={localCustomPrompts.flashcardQuestionPrompt}
          on:input={() => handleCustomPromptChange('flashcardQuestionPrompt', localCustomPrompts.flashcardQuestionPrompt)}
          class="w-full h-48 p-2 mt-2 text-sm text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-y"
        ></textarea>
      </div>
      <div>
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Flashcard Answer Prompt</h3>
        <textarea
          bind:value={localCustomPrompts.flashcardAnswerPrompt}
          on:input={() => handleCustomPromptChange('flashcardAnswerPrompt', localCustomPrompts.flashcardAnswerPrompt)}
          class="w-full h-48 p-2 mt-2 text-sm text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-y"
        ></textarea>
      </div>
      <div>
        <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">Explanation Prompt</h3>
        <textarea
          bind:value={localCustomPrompts.explanationPrompt}
          on:input={() => handleCustomPromptChange('explanationPrompt', localCustomPrompts.explanationPrompt)}
          class="w-full h-48 p-2 mt-2 text-sm text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-y"
        ></textarea>
      </div>
      <button
        on:click={resetCustomPrompts}
        class="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-opacity-50 transition-all duration-200 ease-in-out"
      >
        Restore Default Prompts
      </button>
    </div>
  </div>