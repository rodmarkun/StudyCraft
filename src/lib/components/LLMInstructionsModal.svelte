<!-- src/lib/components/LLMInstructionsModal.svelte -->
<script lang="ts">
  import Modal from './Modal.svelte';

  export let isOpen = false;
  export let provider = '';

  const instructions = {
    ollama: `
      <h3 class="text-xl font-bold mb-4">How to use Ollama</h3>
      <ol class="list-decimal list-inside space-y-2">
        <li>Visit <a href="https://ollama.com/" target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">https://ollama.com/</a> and follow the installation instructions for your operating system.</li>
        <li>Open a terminal or command prompt.</li>
        <li>Run a model using the following command: <code class="bg-gray-200 dark:bg-gray-700 px-2 py-1 rounded">ollama run llama2</code></li>
        <li>Wait for the model to download and start. This may take some time depending on your internet connection.</li>
        <li>Once it's running, come back to this application and select Ollama as your LLM provider.</li>
        <li>Enter your model name ("llama2" in this case) and "11434" as the port (unless you've changed the default port).</li>
      </ol>
      <p class="mt-4"><strong>Note:</strong> Using these models requires a computer with a good graphics card for optimal performance. Without a powerful GPU, response times may be longer.</p>
    `,
    runpod: `
      <h3 class="text-xl font-bold mb-4">How to use Runpod</h3>
      <ol class="list-decimal list-inside space-y-2">
        <li>Visit <a href="https://www.runpod.io/" target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">https://www.runpod.io/</a> and create an account.</li>
        <li>Navigate to the API section and generate an API key.</li>
        <li>Create a new serverless deployment and note down the Serverless API ID.</li>
        <li>In the application, select Runpod as your LLM provider.</li>
        <li>Enter your API key and Serverless API ID in the respective fields.</li>
      </ol>
      <p class="mt-4"><strong>Note:</strong> Runpod is a cloud service, so you'll be charged based on your usage. Make sure to review their pricing before proceeding.</p>
    `,
    openai: `
      <h3 class="text-xl font-bold mb-4">How to use OpenAI</h3>
      <ol class="list-decimal list-inside space-y-2">
        <li>Visit <a href="https://platform.openai.com/" target="_blank" rel="noopener noreferrer" class="text-blue-500 hover:underline">https://platform.openai.com/</a> and create an account.</li>
        <li>Navigate to the API section and generate an API key.</li>
        <li>In the application, select OpenAI as your LLM provider.</li>
        <li>Enter your API key in the respective field.</li>
        <li>Choose the model you want to use (e.g., "text-davinci-003").</li>
      </ol>
      <p class="mt-4"><strong>Note:</strong> OpenAI is a paid service. Make sure to review their pricing before proceeding.</p>
    `,
  };

  function handleClose() {
    isOpen = false;
  }
</script>

<Modal {isOpen} title="LLM Setup Instructions" on:close={handleClose}>
  <div class="prose dark:prose-invert max-w-none">
    {@html instructions[provider] || 'No instructions available for this provider.'}
  </div>
</Modal>