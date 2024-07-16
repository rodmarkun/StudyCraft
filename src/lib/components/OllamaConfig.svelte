<script lang="ts">
    import { options } from '../stores/options';
  
    function handleChange(key: string, value: any) {
      options.setAIProviderOption('ollama', key, value);
    }
  </script>
  
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
      Ollama Model
      <input
        type="text"
        bind:value={$options.aiConfig.ollama.model}
        on:change={() => handleChange('model', $options.aiConfig.ollama.model)}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50 text-base"
      />
    </label>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
      Ollama Port
      <input
        type="number"
        bind:value={$options.aiConfig.ollama.port}
        on:change={() => handleChange('port', $options.aiConfig.ollama.port)}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
      />
    </label>
  </div>