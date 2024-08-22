<script lang="ts">
  import { options } from '../stores/options';
  import type { AIConfig } from '../stores/options';
  import { X, HelpCircle } from 'lucide-svelte';
  import LLMInstructionsModal from './LLMInstructionsModal.svelte';

  import RunpodIconLight from '../../assets/RunpodIconLight.png';
  import RunpodIconDark from '../../assets/RunpodIconDark.png';
  import OpenAIIconLight from '../../assets/OpenAIIconLight.png';
  import OpenAIIconDark from '../../assets/OpenAIIconDark.png';
  import OllamaIconLight from '../../assets/OllamaIconLight.png';
  import OllamaIconDark from '../../assets/OllamaIconDark.png';

  let localAIConfig: AIConfig;
  $: localAIConfig = $options.aiConfig;

  let showInstructionsModal = false;
  let currentInstructionsProvider = '';

  function handleChange(key: string, value: any) {
    if (key === 'provider') {
      options.setAIOption('provider', value);
    } else {
      const [provider, aiKey] = key.split('.');
      options.setAIProviderOption(provider as AIConfig['provider'], aiKey, value);
    }
  }

  function showInstructions(provider: string) {
    currentInstructionsProvider = provider;
    showInstructionsModal = true;
  }

  $: isDarkMode = document.documentElement.classList.contains('dark');
</script>

<div class="options-section">
  <div class="space-y-4">
    <div class="flex space-x-4">
      <button
        class="p-3 rounded-lg {localAIConfig.provider === 'none' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
        on:click={() => handleChange('provider', 'none')}
      >
        <X size={24} />
      </button>
      <button
        class="p-3 rounded-lg {localAIConfig.provider === 'ollama' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
        on:click={() => handleChange('provider', 'ollama')}
      >
        <img src={isDarkMode ? OllamaIconDark : OllamaIconLight} alt="Ollama" class="w-6 h-6" />
      </button>
      <button
        class="p-3 rounded-lg {localAIConfig.provider === 'runpod' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
        on:click={() => handleChange('provider', 'runpod')}
      >
        <img src={isDarkMode ? RunpodIconDark : RunpodIconLight} alt="Runpod" class="w-6 h-6" />
      </button>
      <button
        class="p-3 rounded-lg {localAIConfig.provider === 'openai' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
        on:click={() => handleChange('provider', 'openai')}
      >
        <img src={isDarkMode ? OpenAIIconDark : OpenAIIconLight} alt="OpenAI" class="w-6 h-6" />
      </button>
    </div>
    {#if localAIConfig.provider && localAIConfig.provider !== 'none'}
      <button
        class="w-full mt-2 px-4 py-2 bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 flex items-center justify-center"
        on:click={() => showInstructions(localAIConfig.provider)}
      >
        <HelpCircle size={20} class="mr-2" />
        How to use {localAIConfig.provider}
      </button>
      {#if localAIConfig.provider === 'ollama'}
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Model
            <input
              type="text"
              bind:value={localAIConfig.ollama.model}
              on:input={(e) => handleChange('ollama.model', e.target.value)}
              class="custom-input"
            />
          </label>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Port
            <input
              type="number"
              bind:value={localAIConfig.ollama.port}
              on:input={(e) => handleChange('ollama.port', parseInt(e.target.value))}
              class="custom-input"
            />
          </label>
        </div>
      {:else if localAIConfig.provider === 'runpod'}
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            API Key
            <input
              type="password"
              bind:value={localAIConfig.runpod.apiKey}
              on:input={(e) => handleChange('runpod.apiKey', e.target.value)}
              class="custom-input"
            />
          </label>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Serverless API ID
            <input
              type="text"
              bind:value={localAIConfig.runpod.serverlessApiId}
              on:input={(e) => handleChange('runpod.serverlessApiId', e.target.value)}
              class="custom-input"
            />
          </label>
        </div>
      {:else if localAIConfig.provider === 'openai'}
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            API Key
            <input
              type="password"
              bind:value={localAIConfig.openai.apiKey}
              on:input={(e) => handleChange('openai.apiKey', e.target.value)}
              class="custom-input"
            />
          </label>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Model
            <input
              type="text"
              bind:value={localAIConfig.openai.model}
              on:input={(e) => handleChange('openai.model', e.target.value)}
              class="custom-input"
            />
          </label>
        </div>
      {/if}
    {/if}
  </div>
</div>

<LLMInstructionsModal
  bind:isOpen={showInstructionsModal}
  provider={currentInstructionsProvider}
/>