<!-- src/lib/components/AddStudyMaterialModal.svelte -->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from './Modal.svelte';
  import { saveFile } from '../utils/fileUtils';

  export let isOpen = false;
  export let collectionId: string;
  export let name: string;

  const dispatch = createEventDispatcher();

  let materials = [{ type: 'markdown', file: null, url: '', convertToMarkdown: false, name: '' }];
  let errorMessage = '';
  let uploadedFiles = new Set();

  function addMaterial() {
    materials = [...materials, { type: 'markdown', file: null, url: '', convertToMarkdown: false, name: '' }];
  }

  function removeMaterial(index: number) {
    const removedMaterial = materials[index];
    if (removedMaterial.file) {
      uploadedFiles.delete(removedMaterial.file.name);
    }
    materials = materials.filter((_, i) => i !== index);
  }

  async function handleFileChange(event: Event, index: number) {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files[0]) {
      const file = input.files[0];
      if (uploadedFiles.has(file.name)) {
        errorMessage = `File "${file.name}" has already been added. Please choose a different file.`;
        input.value = ''; // Reset the input
        return;
      }
      materials[index].file = file;
      materials[index].name = file.name;
      uploadedFiles.add(file.name);
      materials = [...materials];
      errorMessage = '';
    }
  }

  async function handleSubmit() {
    try {
      errorMessage = '';
      const processedMaterials = await Promise.all(materials.map(async (m) => {
        if (m.type === 'markdown' && m.file) {
          const content = await m.file.text();
          const filePath = await saveFile(content, m.name, name);
          return { ...m, filePath, content };
        } else if (m.type === 'webpage' && m.url) {
          // Handle webpage saving here if needed
          if (!isValidUrl(m.url)) {
            throw new Error(`Invalid URL: ${m.url}`);
          }
          return m;
        } else if (m.type === 'pdf' && m.file) {
          // Handle PDF saving if needed
          return m;
        }
        throw new Error(`Invalid material type or missing required data for ${m.name || 'unnamed material'}`);
      }));

      const validMaterials = processedMaterials.filter(m => 
        (m.type === 'markdown' && m.filePath) || 
        (m.type === 'webpage' && m.url) || 
        (m.type === 'pdf' && m.file)
      );
      
      if (validMaterials.length === 0) {
        throw new Error('No valid materials to add');
      }

      dispatch('materialsAdded', validMaterials);
      close();
    } catch (error) {
      console.error('Error processing materials:', error);
      errorMessage = `Error: ${error.message}`;
    }
  }

  function close() {
    materials = [{ type: 'markdown', file: null, url: '', convertToMarkdown: false, name: '' }];
    errorMessage = '';
    uploadedFiles.clear();
    dispatch('close');
  }

  function isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }
</script>

<Modal {isOpen} title="Add Study Materials" on:close={close}>
<div class="space-y-4">
  {#if errorMessage}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative" role="alert">
      <strong class="font-bold">Error!</strong>
      <span class="block sm:inline">{errorMessage}</span>
    </div>
  {/if}

  {#each materials as material, index}
    <div class="border p-4 rounded">
      <select 
        bind:value={material.type} 
        class="w-full p-2 mb-2 border rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
      >
        <option value="markdown">Markdown File</option>
        <option value="webpage">Webpage / Blog Post</option>
        <option value="pdf">PDF</option>
      </select>

      {#if material.type === 'markdown' || material.type === 'pdf'}
        <input 
          type="file" 
          accept={material.type === 'markdown' ? '.md' : '.pdf'}
          on:change={(e) => handleFileChange(e, index)}
          class="w-full p-2 border rounded text-gray-900 dark:text-white"
        />
      {:else if material.type === 'webpage'}
        <input 
          type="url" 
          bind:value={material.url} 
          placeholder="https://example.com"
          class="w-full p-2 mb-2 border rounded text-gray-900 dark:text-white bg-white dark:bg-gray-700"
        />
        <label class="inline-flex items-center">
          <input type="checkbox" bind:checked={material.convertToMarkdown} class="form-checkbox" />
          <span class="ml-2 text-gray-900 dark:text-white">Convert to Markdown</span>
        </label>
      {/if}

      {#if index > 0}
        <button 
          on:click={() => removeMaterial(index)}
          class="mt-2 px-2 py-1 bg-red-500 text-white rounded hover:bg-red-600"
        >
          Remove
        </button>
      {/if}
    </div>
  {/each}

  <button 
    on:click={addMaterial}
    class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
  >
    Add Another Material
  </button>

  <button 
    on:click={handleSubmit}
    class="w-full px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
  >
    Upload Materials
  </button>
</div>
</Modal>