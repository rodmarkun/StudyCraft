<script lang="ts">
    import { options } from '../stores/options';
    import { onMount } from 'svelte';
  
    let apiKey = '';
    let model = 'gpt-3.5-turbo';
  
    onMount(() => {
      const unsubscribe = options.subscribe(value => {
        apiKey = value.aiConfig.openai.apiKey;
        model = value.aiConfig.openai.model;
      });
  
      return unsubscribe;
    });
  
    function handleChange(key: string, value: string) {
      options.setAIProviderOption('openai', key, value);
    }
  </script>
  
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
      OpenAI API Key
      <input
        type="password"
        bind:value={apiKey}
        on:input={() => handleChange('apiKey', apiKey)}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50 text-base"
      />
    </label>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
      Model
      <select
        bind:value={model}
        on:change={() => handleChange('model', model)}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
      >
        <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
        <option value="gpt-4">GPT-4 (if available)</option>
      </select>
    </label>
  </div>