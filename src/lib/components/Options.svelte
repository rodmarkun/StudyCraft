<!-- src/lib/components/Options.svelte -->
<script lang="ts">
  import { options } from '../stores/options';
  import type { Options, AIConfig, CustomPrompts } from '../stores/options';
  import { onMount } from 'svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import LLMInstructionsModal from './LLMInstructionsModal.svelte';
  import { X, HelpCircle } from 'lucide-svelte';

  import RunpodIconLight from '../../assets/RunpodIconLight.png';
  import RunpodIconDark from '../../assets/RunpodIconDark.png';
  import OpenAIIconLight from '../../assets/OpenAIIconLight.png';
  import OpenAIIconDark from '../../assets/OpenAIIconDark.png';
  import OllamaIconLight from '../../assets/OllamaIconLight.png';
  import OllamaIconDark from '../../assets/OllamaIconDark.png';

  let showConfirmDialog = false;
  let isDeleting = false;
  let showInstructionsModal = false;
  let currentInstructionsProvider = '';

  // Initialize with default values
  let localCustomPrompts: CustomPrompts = {
    flashcardQuestionPrompt: '',
    flashcardAnswerPrompt: '',
    explanationPrompt: ''
  };
  let localAIConfig: AIConfig = {
    provider: 'none',
    ollama: { model: '', port: 0 },
    runpod: { apiKey: '', serverlessApiId: '' },
    openai: { apiKey: '', model: '' }
  };

  onMount(() => {
    const unsub = options.subscribe(currentOptions => {
      localCustomPrompts = { ...currentOptions.customPrompts };
      localAIConfig = JSON.parse(JSON.stringify(currentOptions.aiConfig));
    });

    return unsub;
  });

  function showInstructions(provider: string) {
    currentInstructionsProvider = provider;
    showInstructionsModal = true;
  }

  function handleChange(key: string, value: any) {
    if (key === 'aiConfig.provider') {
      localAIConfig.provider = value;
      options.setAIOption('provider', value);
    } else if (key.startsWith('aiConfig.')) {
      const [_, provider, aiKey] = key.split('.');
      localAIConfig[provider][aiKey] = value;
      options.setAIProviderOption(provider as AIConfig['provider'], aiKey, value);
    } else {
      options.setOption(key as keyof Options, value);
    }
  }

  function saveOptions() {
    options.saveOptions({
      ...$options,
      aiConfig: localAIConfig,
      customPrompts: localCustomPrompts
    });
  }

  function resetOptions() {
    options.resetToDefaults();
    localCustomPrompts = { ...$options.customPrompts };
    localAIConfig = JSON.parse(JSON.stringify($options.aiConfig));
  }

  function handleDeleteAllData() {
    showConfirmDialog = true;
  }

  async function handleConfirmDelete() {
    isDeleting = true;
    try {
      const success = await window.electronAPI.deleteAllData();
      if (success) {
        alert('All data has been deleted. The application will close now.');
        window.electronAPI.exitApp();
      } else {
        alert('There was an error deleting the data. Please try again.');
      }
    } catch (error) {
      console.error('Error deleting all data:', error);
      alert('There was an error deleting the data. Please try again.');
    } finally {
      isDeleting = false;
      showConfirmDialog = false;
    }
  }

  function handleCustomPromptChange(key: keyof CustomPrompts, value: string) {
    localCustomPrompts[key] = value;
  }

  function handleAIConfigChange(provider: AIConfig['provider'], key: string, value: any) {
    localAIConfig[provider][key] = value;
  }

  function resetCustomPrompts() {
    localCustomPrompts = { ...options.getDefaultPrompts() };
  }

  $: isDarkMode = document.documentElement.classList.contains('dark');
</script>

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

<div class="space-y-8 p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md">
  <div>
    <h2 class="text-2xl font-bold mb-4 text-gray-800 dark:text-white">General Options</h2>
    <div class="space-y-4">
      <label class="flex items-center space-x-3 text-gray-700 dark:text-gray-300">
        <input
          type="checkbox"
          checked={$options.openMaterialsInDefaultApp}
          on:change={() => options.setOption('openMaterialsInDefaultApp', !$options.openMaterialsInDefaultApp)}
          class="form-checkbox h-5 w-5 text-blue-600"
        />
        <span>Open Study Materials in default applications</span>
      </label>
      <label class="flex items-center space-x-3 text-gray-700 dark:text-gray-300">
        <input
          type="checkbox"
          checked={$options.simplifiedMaterialView}
          on:change={() => options.setOption('simplifiedMaterialView', !$options.simplifiedMaterialView)}
          class="form-checkbox h-5 w-5 text-blue-600"
        />
        <span>Simplified Study Material View</span>
      </label>
    </div>
  </div>

  <div>
    <h2 class="text-2xl font-bold mb-4 text-gray-800 dark:text-white">LLM Configuration</h2>
    <div class="space-y-4">
      <div class="flex space-x-4">
        <button
          class="p-3 rounded-lg {localAIConfig.provider === 'none' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'none')}
        >
          <X size={24} />
        </button>
        <button
          class="p-3 rounded-lg {localAIConfig.provider === 'ollama' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'ollama')}
        >
          <img src={isDarkMode ? OllamaIconDark : OllamaIconLight} alt="Ollama" class="w-6 h-6" />
        </button>
        <button
          class="p-3 rounded-lg {localAIConfig.provider === 'runpod' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'runpod')}
        >
          <img src={isDarkMode ? RunpodIconDark : RunpodIconLight} alt="Runpod" class="w-6 h-6" />
        </button>
        <button
          class="p-3 rounded-lg {localAIConfig.provider === 'openai' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'openai')}
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
                on:change={() => handleAIConfigChange('ollama', 'model', localAIConfig.ollama.model)}
                class="custom-input"
              />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Port
              <input
                type="number"
                bind:value={localAIConfig.ollama.port}
                on:change={() => handleAIConfigChange('ollama', 'port', localAIConfig.ollama.port)}
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
                on:change={() => handleAIConfigChange('runpod', 'apiKey', localAIConfig.runpod.apiKey)}
                class="custom-input"
              />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Serverless API ID
              <input
                type="text"
                bind:value={localAIConfig.runpod.serverlessApiId}
                on:change={() => handleAIConfigChange('runpod', 'serverlessApiId', localAIConfig.runpod.serverlessApiId)}
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
                on:change={() => handleAIConfigChange('openai', 'apiKey', localAIConfig.openai.apiKey)}
                class="custom-input"
              />
            </label>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Model
              <input
                type="text"
                bind:value={localAIConfig.openai.model}
                on:change={() => handleAIConfigChange('openai', 'model', localAIConfig.openai.model)}
                class="custom-input"
              />
            </label>
          </div>
        {/if}
      {/if}
    </div>
  </div>

  <div>
    <h2 class="text-2xl font-bold mb-4 text-gray-800 dark:text-white">Custom Prompts</h2>
    <div class="space-y-4">
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Flashcard Question Prompt
          <textarea
            bind:value={localCustomPrompts.flashcardQuestionPrompt}
            on:change={() => handleCustomPromptChange('flashcardQuestionPrompt', localCustomPrompts.flashcardQuestionPrompt)}
            class="custom-textarea"
          ></textarea>
        </label>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Flashcard Answer Prompt
          <textarea
            bind:value={localCustomPrompts.flashcardAnswerPrompt}
            on:change={() => handleCustomPromptChange('flashcardAnswerPrompt', localCustomPrompts.flashcardAnswerPrompt)}
            class="custom-textarea"
          ></textarea>
        </label>
      </div>
      <div>
        <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
          Explanation Prompt
          <textarea
            bind:value={localCustomPrompts.explanationPrompt}
            on:change={() => handleCustomPromptChange('explanationPrompt', localCustomPrompts.explanationPrompt)}
            class="custom-textarea"
          ></textarea>
        </label>
      </div>
      <button
        on:click={resetCustomPrompts}
        class="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-opacity-50 transition-all duration-200 ease-in-out"
      >
        Restore Default Prompts
      </button>
    </div>
  </div>

  <div class="flex justify-between">
    <button
      on:click={saveOptions}
      class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
    >
      Save Changes
    </button>
    <button
      on:click={resetOptions}
      class="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-opacity-50"
    >
      Reset to Defaults
    </button>
  </div>

  <div class="pt-6 border-t border-gray-300 dark:border-gray-600">
    <h2 class="text-2xl font-bold mb-4 text-red-600 dark:text-red-400">Danger Zone</h2>
    <button
      on:click={handleDeleteAllData}
      class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-opacity-50 disabled:opacity-50"
      disabled={isDeleting}
    >
      {isDeleting ? 'Deleting...' : 'Delete All Data'}
    </button>
  </div>
</div>

{#if showConfirmDialog}
  <ConfirmDialog
    title="Confirm Deletion"
    message="Are you sure you want to delete all data? This action cannot be undone and will close the application."
    confirmText="Delete All Data"
    on:confirm={handleConfirmDelete}
    on:cancel={() => showConfirmDialog = false}
  />
{/if}

{#if showInstructionsModal}
  <LLMInstructionsModal
    bind:isOpen={showInstructionsModal}
    provider={currentInstructionsProvider}
  />
{/if}