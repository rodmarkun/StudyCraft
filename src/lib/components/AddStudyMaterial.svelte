<!-- src/lib/components/AddStudyMaterial.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { saveFile } from '../utils/fileUtils';

  export let collectionId: string;
  export let collectionName: string;

  const dispatch = createEventDispatcher();

  let materialType: 'markdown' | 'webpage' | 'pdf' = 'markdown';
  let convertToMarkdown = false;
  let url = '';

  async function handleSubmit() {
    if (materialType === 'markdown') {
      const input = document.getElementById('file-upload') as HTMLInputElement;
      const files = input.files;

      if (files && files[0]) {
        const file = files[0];
        try {
          const content = await file.text();
          const filePath = await saveFile(content, file.name, collectionName);
          
          dispatch('materialAdded', { 
            type: 'markdown',
            fileName: file.name,
            filePath
          });

          input.value = ''; // Reset the input
        } catch (error) {
          console.error('Error uploading file:', error);
          alert('There was an error uploading the file. Please try again.');
        }
      }
    } else if (materialType === 'webpage') {
      // Here we would implement the logic to save the webpage
      // For now, we'll just dispatch an event with the URL
      dispatch('materialAdded', {
        type: 'webpage',
        url,
        convertToMarkdown
      });
      url = '';
      convertToMarkdown = false;
    } else if (materialType === 'pdf') {
      const input = document.getElementById('pdf-upload') as HTMLInputElement;
      const files = input.files;

      if (files && files[0]) {
        // Here we would implement the logic to save the PDF
        // For now, we'll just dispatch an event with the file name
        dispatch('materialAdded', {
          type: 'pdf',
          fileName: files[0].name
        });
        input.value = '';
      }
    }
  }
</script>

<div class="mt-4">
  <h3 class="text-lg font-semibold mb-2">Add Study Material</h3>
  <div class="mb-4">
    <label class="block mb-2">Material Type:</label>
    <select bind:value={materialType} class="w-full p-2 border rounded">
      <option value="markdown">Markdown File</option>
      <option value="webpage">Webpage / Blog Post</option>
      <option value="pdf">PDF</option>
    </select>
  </div>

  {#if materialType === 'markdown'}
    <div class="mb-4">
      <label for="file-upload" class="block mb-2">Select Markdown File:</label>
      <input id="file-upload" type="file" accept=".md" class="w-full p-2 border rounded" />
    </div>
  {:else if materialType === 'webpage'}
    <div class="mb-4">
      <label for="webpage-url" class="block mb-2">Webpage URL:</label>
      <input id="webpage-url" type="url" bind:value={url} class="w-full p-2 border rounded" placeholder="https://example.com" />
    </div>
    <div class="mb-4">
      <label class="inline-flex items-center">
        <input type="checkbox" bind:checked={convertToMarkdown} class="form-checkbox" />
        <span class="ml-2">Convert to Markdown</span>
      </label>
    </div>
  {:else if materialType === 'pdf'}
    <div class="mb-4">
      <label for="pdf-upload" class="block mb-2">Select PDF File:</label>
      <input id="pdf-upload" type="file" accept=".pdf" class="w-full p-2 border rounded" />
    </div>
  {/if}

  <button
    on:click={handleSubmit}
    class="px-4 py-2 bg-primary-light dark:bg-primary-dark text-white rounded-md hover:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-primary-light dark:focus:ring-primary-dark"
  >
    Add Material
  </button>
</div>