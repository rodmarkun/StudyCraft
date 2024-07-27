<script lang="ts">
    import { ChevronDown, ChevronUp } from 'lucide-svelte';
    import type { AIConfig } from '../stores/options';
  
    export let config: AIConfig[keyof AIConfig];
    export let provider: AIConfig['provider'];
    export let onConfigChange: (key: string, value: any) => void;
  
    let showAdvancedOptions = false;
  
    const defaultValues = {
      ollama: {
        url: 'http://localhost:11434/api/generate',
        chunkSize: 3000,
        payload: {}
      },
      runpod: {
        url: 'https://api.runpod.ai/v2/{serverlessApiId}/runsync',
        chunkSize: 3000,
        payload: {}
      },
      openai: {
        url: 'https://api.openai.com/v1/chat/completions',
        chunkSize: 3000,
        payload: {}
      }
    };
  
    function handleChange(key: string, value: any) {
      onConfigChange(key, value);
    }
  
    function restoreDefaults() {
      handleChange('url', defaultValues[provider].url);
      handleChange('chunkSize', defaultValues[provider].chunkSize);
      handleChange('payload', defaultValues[provider].payload);
    }
  </script>
  
  <div class="mt-4">
    <button
      on:click={() => showAdvancedOptions = !showAdvancedOptions}
      class="flex items-center justify-between w-full px-4 py-2 text-left text-gray-700 bg-gray-100 rounded-lg dark:bg-gray-700 dark:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-600"
    >
      <span>Advanced Options</span>
      {#if showAdvancedOptions}
        <ChevronUp size={20} />
      {:else}
        <ChevronDown size={20} />
      {/if}
    </button>
    {#if showAdvancedOptions}
      <div class="mt-2 space-y-2 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg">
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          API URL/Endpoint
          <input
            type="text"
            value={config.url}
            placeholder={defaultValues[provider].url}
            on:change={(e) => handleChange('url', e.target.value)}
            class="custom-input"
          />
        </label>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Chunk Size
          <input
            type="number"
            value={config.chunkSize}
            placeholder={defaultValues[provider].chunkSize.toString()}
            on:change={(e) => handleChange('chunkSize', parseInt(e.target.value))}
            class="custom-input"
          />
        </label>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Payload (JSON)
          <textarea
            value={JSON.stringify(config.payload, null, 2)}
            placeholder={JSON.stringify(defaultValues[provider].payload, null, 2)}
            on:change={(e) => handleChange('payload', JSON.parse(e.target.value))}
            class="custom-textarea"
          ></textarea>
        </label>
        <button
          on:click={restoreDefaults}
          class="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-opacity-50 transition-all duration-200 ease-in-out"
        >
          Restore Default Advanced Options
        </button>
      </div>
    {/if}
  </div>
  
  <style lang="postcss">
    .custom-input {
      @apply mt-1 block w-full rounded-md border-gray-300 dark:border-gray-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 transition-all duration-200 ease-in-out;
    }
  
    .custom-textarea {
      @apply custom-input resize-none p-3 h-40;
      scrollbar-width: thin;
      scrollbar-color: rgba(155, 155, 155, 0.5) transparent;
    }
  
    .custom-textarea::-webkit-scrollbar {
      width: 6px;
    }
  
    .custom-textarea::-webkit-scrollbar-track {
      background: transparent;
    }
  
    .custom-textarea::-webkit-scrollbar-thumb {
      background-color: rgba(155, 155, 155, 0.5);
      border-radius: 20px;
      border: transparent;
    }
  </style>