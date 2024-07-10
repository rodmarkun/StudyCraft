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
          <li>Run a model using the following command: <code class="bg-gray-200 dark:bg-gray-700 px-2 py-1 rounded">ollama run llama3</code></li>
          <li>Wait for the model to download and start. This may take some time depending on your internet connection.</li>
          <li>Once it's running, come back to this application and select Ollama as your LLM provider.</li>
          <li>Enter your model name ("llama3" in this case) and "11434" as the port (unless you've changed the default port).</li>
        </ol>
        <p class="mt-4"><strong>Note:</strong> Using these models requires a computer with a good graphics card for optimal performance. Without a powerful GPU, response times may be longer.</p>
      `,
      // Add instructions for other providers here
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