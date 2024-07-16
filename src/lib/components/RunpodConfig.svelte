<script lang="ts">
    import { options } from '../stores/options';
    import { onMount } from 'svelte';
  
    let apiKey = '';
    let serverlessApiId = '';
  
    onMount(() => {
      const unsubscribe = options.subscribe(value => {
        apiKey = value.aiConfig.runpod.apiKey;
        serverlessApiId = value.aiConfig.runpod.serverlessApiId;
      });
  
      return unsubscribe;
    });
  
    function handleChange(key: string, value: string) {
      options.setAIProviderOption('runpod', key, value);
    }
  </script>
  
  <div class="space-y-2">
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
      Runpod API Key
      <input
        type="password"
        bind:value={apiKey}
        on:input={() => handleChange('apiKey', apiKey)}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50 text-base"
      />
    </label>
    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300">
      Serverless API ID
      <input
        type="text"
        bind:value={serverlessApiId}
        on:input={() => handleChange('serverlessApiId', serverlessApiId)}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
      />
    </label>
  </div>