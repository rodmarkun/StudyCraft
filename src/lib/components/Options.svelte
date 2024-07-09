<script lang="ts">
    import { options } from '../stores/options';
    import ConfirmDialog from './ConfirmDialog.svelte';
  
    let localOptions;
    options.subscribe(value => {
      localOptions = { ...value };
    });
  
    let showConfirmDialog = false;
  
    function handleChange(key: string, value: any) {
      if (key.startsWith('llmConfig.')) {
        const llmKey = key.split('.')[1];
        options.setLLMOption(llmKey, value);
      } else {
        options.setOption(key, value);
      }
    }
  
    function saveOptions() {
      options.saveOptions(localOptions);
    }
  
    function resetOptions() {
      options.resetToDefaults();
      options.loadOptions();
    }
  
    function handleDeleteAllData() {
      showConfirmDialog = true;
    }
  
    function handleConfirmDelete() {
      // Implement the logic to delete all data here
      console.log('Deleting all data...');
      showConfirmDialog = false;
    }
  
    function handleCancelDelete() {
      showConfirmDialog = false;
    }
  </script>
  
  <div class="space-y-6">
    <div class="space-y-4">
      <h3 class="text-lg font-medium">General Options</h3>
      <div>
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            bind:checked={localOptions.openMaterialsInDefaultApp}
            on:change={() => handleChange('openMaterialsInDefaultApp', localOptions.openMaterialsInDefaultApp)}
            class="form-checkbox"
          />
          <span>Open Study Materials in default applications</span>
        </label>
      </div>
      <div>
        <label class="flex items-center space-x-2">
          <input
            type="checkbox"
            bind:checked={localOptions.simplifiedMaterialView}
            on:change={() => handleChange('simplifiedMaterialView', localOptions.simplifiedMaterialView)}
            class="form-checkbox"
          />
          <span>Simplified Study Material View</span>
        </label>
      </div>
    </div>
  
    <div class="space-y-4">
      <h3 class="text-lg font-medium">LLM Configuration</h3>
      <div>
        <label class="block">
          <span>LLM Provider</span>
          <select
            bind:value={localOptions.llmConfig.provider}
            on:change={() => handleChange('llmConfig.provider', localOptions.llmConfig.provider)}
            class="form-select mt-1 block w-full text-gray-900 dark:text-white bg-white dark:bg-gray-700"
          >
            <option value="none">None</option>
            <option value="ollama">Local Ollama</option>
          </select>
        </label>
      </div>
    </div>
  
    <div class="flex justify-between">
      <button
        on:click={saveOptions}
        class="px-4 py-2 bg-primary-light dark:bg-primary-dark text-white rounded hover:bg-opacity-90"
      >
        Save Changes
      </button>
      <button
        on:click={resetOptions}
        class="px-4 py-2 bg-gray-300 dark:bg-gray-600 text-gray-800 dark:text-white rounded hover:bg-opacity-90"
      >
        Reset to Defaults
      </button>
    </div>
  
    <div class="space-y-4 pt-6 border-t border-gray-300 dark:border-gray-600">
      <h3 class="text-lg font-medium text-red-600 dark:text-red-400">Danger Zone</h3>
      <button
        on:click={handleDeleteAllData}
        class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none"
      >
        Delete All Data
      </button>
    </div>
  </div>
  
  {#if showConfirmDialog}
    <ConfirmDialog
      title="Confirm Deletion"
      message="Are you sure you want to delete all data? This action cannot be undone."
      confirmText="Delete All Data"
      on:confirm={handleConfirmDelete}
      on:cancel={handleCancelDelete}
    />
  {/if}