<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Modal from './Modal.svelte';
  import { saveFile } from '../utils/fileUtils';
  import type { StudyMaterial } from '../stores/collections';
  import { getWebsiteTitle } from '../services/webScraperService';

  export let isOpen = false;
  export let collectionId: string;
  export let name: string;

  const dispatch = createEventDispatcher();

  let webpageUrl = '';
  let errorMessage = '';
  let uploadedFiles: File[] = [];
  let isDragging = false;
  let dragCounter = 0;

  function handleDragEnter(event: DragEvent) {
    event.preventDefault();
    dragCounter++;
    isDragging = true;
  }

  function handleDragLeave(event: DragEvent) {
    event.preventDefault();
    dragCounter--;
    if (dragCounter === 0) {
      isDragging = false;
    }
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    event.dataTransfer!.dropEffect = 'copy';
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    isDragging = false;
    dragCounter = 0;
    const files = Array.from(event.dataTransfer!.files);
    handleFiles(files);
  }

  function handleFileInput(event: Event) {
    const input = event.target as HTMLInputElement;
    if (input.files) {
      const files = Array.from(input.files);
      handleFiles(files);
    }
  }

  function handleFiles(files: File[]) {
    const newFiles = files.filter(file => !uploadedFiles.some(f => f.name === file.name));
    uploadedFiles = [...uploadedFiles, ...newFiles];
    errorMessage = '';
  }

  function removeFile(fileName: string) {
    uploadedFiles = uploadedFiles.filter(file => file.name !== fileName);
  }

  async function handleSubmit() {
    try {
      errorMessage = '';
      const processedMaterials: StudyMaterial[] = [];

      // Process webpage
      if (webpageUrl) {
        if (!isValidUrl(webpageUrl)) {
          throw new Error(`Invalid URL: ${webpageUrl}`);
        }
        const websiteTitle = await getWebsiteTitle(webpageUrl);
        processedMaterials.push({
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          type: 'webpage',
          url: webpageUrl,
          name: websiteTitle
        });
      }

      // Process files
      for (const file of uploadedFiles) {
        const id = Date.now().toString() + Math.random().toString(36).substr(2, 9);
        if (file.name.endsWith('.md')) {
          const content = await file.text();
          const filePath = await window.electronAPI.saveFile(content, file.name, name);
          processedMaterials.push({ id, type: 'markdown', filePath, name: file.name });
        } else if (file.name.endsWith('.pdf')) {
          const content = await file.arrayBuffer();
          const filePath = await window.electronAPI.saveFile(content, file.name, name);
          processedMaterials.push({ id, type: 'pdf', filePath, name: file.name });
        } else {
          console.warn(`Skipping unsupported file type: ${file.name}`);
        }
      }

      if (processedMaterials.length === 0) {
        throw new Error('No valid materials to add');
      }

      dispatch('materialsAdded', processedMaterials);
      close();
    } catch (error) {
      console.error('Error processing materials:', error);
      errorMessage = `Error: ${error.message}`;
    }
  }

  function close() {
    webpageUrl = '';
    uploadedFiles = [];
    errorMessage = '';
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

    <div class="space-y-2">
      <label for="webpageUrl" class="block text-sm font-medium text-gray-700 dark:text-gray-300">Add Webpage URL</label>
      <input
        type="url"
        id="webpageUrl"
        bind:value={webpageUrl}
        placeholder="https://example.com"
        class="w-full p-2 border rounded text-gray-900 dark:text-white bg-white dark:bg-gray-700"
      />
    </div>

    <div
      class="border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg p-6 text-center"
      on:dragenter={handleDragEnter}
      on:dragleave={handleDragLeave}
      on:dragover={handleDragOver}
      on:drop={handleDrop}
    >
      <input
        type="file"
        id="fileInput"
        multiple
        accept=".md,.pdf"
        on:change={handleFileInput}
        class="hidden"
      />
      <label
        for="fileInput"
        class="cursor-pointer text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300"
      >
        Click to upload
      </label>
      <span class="text-gray-600 dark:text-gray-400"> or drag and drop</span>
      <p class="text-xs text-gray-500 dark:text-gray-400 mt-2">PDF, MD files are allowed</p>
    </div>

    {#if uploadedFiles.length > 0}
      <div class="mt-4">
        <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">Uploaded Files:</h4>
        <ul class="space-y-2">
          {#each uploadedFiles as file}
            <li class="flex items-center justify-between bg-gray-100 dark:bg-gray-700 p-2 rounded">
              <span class="text-sm text-gray-800 dark:text-gray-200">{file.name}</span>
              <button
                on:click={() => removeFile(file.name)}
                class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300"
              >
                Remove
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <button 
      on:click={handleSubmit}
      class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
    >
      Add Materials
    </button>
  </div>
</Modal>