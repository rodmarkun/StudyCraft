<!-- src/lib/components/Options.svelte -->
<script lang="ts">
  import { options } from '../stores/options';
  import type { Options, AIConfig, VectorDBConfig } from '../stores/options';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import LLMInstructionsModal from './LLMInstructionsModal.svelte';
  import OllamaConfig from './OllamaConfig.svelte';
  import RunpodConfig from './RunpodConfig.svelte';
  import OpenAIConfig from './OpenAIConfig.svelte';
  import { Brain, Database, X, HelpCircle } from 'lucide-svelte';

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

  function showInstructions(provider: string) {
    currentInstructionsProvider = provider;
    showInstructionsModal = true;
  }

  function handleChange(key: string, value: any) {
    if (key === 'aiConfig.provider') {
      options.setAIOption('provider', value);
    } else if (key.startsWith('aiConfig.')) {
      const [_, provider, aiKey] = key.split('.');
      options.setAIProviderOption(provider as AIConfig['provider'], aiKey, value);
    } else if (key.startsWith('vectorDBConfig.')) {
      const vectorDBKey = key.split('.')[1] as keyof VectorDBConfig;
      options.setVectorDBOption(vectorDBKey, value);
    } else {
      options.setOption(key as keyof Options, value);
    }
  }

  function saveOptions() {
    options.saveOptions($options);
  }

  function resetOptions() {
    options.resetToDefaults();
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

  $: isDarkMode = document.documentElement.classList.contains('dark');
</script>

<div class="space-y-8 p-6 bg-white dark:bg-gray-800 rounded-lg shadow-md">
  <div>
    <h2 class="text-2xl font-bold mb-4 text-gray-800 dark:text-white">General Options</h2>
    <div class="space-y-4">
      <label class="flex items-center space-x-3 text-gray-700 dark:text-gray-300">
        <input
          type="checkbox"
          bind:checked={$options.openMaterialsInDefaultApp}
          on:change={() => handleChange('openMaterialsInDefaultApp', $options.openMaterialsInDefaultApp)}
          class="form-checkbox h-5 w-5 text-blue-600"
        />
        <span>Open Study Materials in default applications</span>
      </label>
      <label class="flex items-center space-x-3 text-gray-700 dark:text-gray-300">
        <input
          type="checkbox"
          bind:checked={$options.simplifiedMaterialView}
          on:change={() => handleChange('simplifiedMaterialView', $options.simplifiedMaterialView)}
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
          class="p-3 rounded-lg {$options.aiConfig.provider === 'none' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'none')}
        >
          <X size={24} />
        </button>
        <button
          class="p-3 rounded-lg {$options.aiConfig.provider === 'ollama' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'ollama')}
        >
          <img src={isDarkMode ? OllamaIconDark : OllamaIconLight} alt="Ollama" class="w-6 h-6" />
        </button>
        <button
          class="p-3 rounded-lg {$options.aiConfig.provider === 'runpod' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'runpod')}
        >
          <img src={isDarkMode ? RunpodIconDark : RunpodIconLight} alt="Runpod" class="w-6 h-6" />
        </button>
        <button
          class="p-3 rounded-lg {$options.aiConfig.provider === 'openai' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('aiConfig.provider', 'openai')}
        >
          <img src={isDarkMode ? OpenAIIconDark : OpenAIIconLight} alt="OpenAI" class="w-6 h-6" />
        </button>
      </div>
      {#if $options.aiConfig.provider && $options.aiConfig.provider !== 'none'}
        <button
          class="w-full mt-2 px-4 py-2 bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-300 dark:hover:bg-gray-600 flex items-center justify-center"
          on:click={() => showInstructions($options.aiConfig.provider)}
        >
          <HelpCircle size={20} class="mr-2" />
          How to use {$options.aiConfig.provider}
        </button>
        {#if $options.aiConfig.provider === 'ollama'}
          <OllamaConfig />
        {:else if $options.aiConfig.provider === 'runpod'}
          <RunpodConfig />
        {:else if $options.aiConfig.provider === 'openai'}
          <OpenAIConfig />
        {/if}
      {/if}
    </div>
  </div>

  <div>
    <h2 class="text-2xl font-bold mb-4 text-gray-800 dark:text-white">Vector Database Configuration</h2>
    <div class="space-y-4">
      <div class="flex space-x-4">
        <button
          class="p-3 rounded-lg {$options.vectorDBConfig.provider === 'none' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('vectorDBConfig.provider', 'none')}
        >
          <X size={24} />
        </button>
        <button
          class="p-3 rounded-lg {$options.vectorDBConfig.provider === 'chroma' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('vectorDBConfig.provider', 'chroma')}
        >
          <Database size={24} />
        </button>
        <button
          class="p-3 rounded-lg {$options.vectorDBConfig.provider === 'pinecone' ? 'bg-blue-500 text-white' : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300'}"
          on:click={() => handleChange('vectorDBConfig.provider', 'pinecone')}
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 2L2 7l10 5 10-5-10-5z"/>
            <path d="M2 17l10 5 10-5"/>
            <path d="M2 12l10 5 10-5"/>
          </svg>
        </button>
      </div>
      {#if $options.vectorDBConfig.provider === 'chroma'}
        <p class="text-sm text-gray-600 dark:text-gray-400">Chroma configuration options will be added here.</p>
      {:else if $options.vectorDBConfig.provider === 'pinecone'}
        <div class="space-y-2">
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Pinecone API Key
            <input
              type="password"
              bind:value={$options.vectorDBConfig.pineconeApiKey}
              on:change={() => handleChange('vectorDBConfig.pineconeApiKey', $options.vectorDBConfig.pineconeApiKey)}
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
            />
          </label>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Pinecone Environment
            <input
              type="text"
              bind:value={$options.vectorDBConfig.pineconeEnvironment}
              on:change={() => handleChange('vectorDBConfig.pineconeEnvironment', $options.vectorDBConfig.pineconeEnvironment)}
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
            />
          </label>
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
            Pinecone Index
            <input
              type="text"
              bind:value={$options.vectorDBConfig.pineconeIndex}
              on:change={() => handleChange('vectorDBConfig.pineconeIndex', $options.vectorDBConfig.pineconeIndex)}
              class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
            />
          </label>
        </div>
      {/if}
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