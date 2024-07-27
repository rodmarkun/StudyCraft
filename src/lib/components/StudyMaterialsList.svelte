<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { options } from '../stores/options';
  import type { StudyMaterial } from '../stores/collections';
  import { loadStudyMaterialContent } from '../stores/collections';
  import { deleteFile } from '../utils/fileUtils';
  import { Trash2, AlertCircle, FileText, Globe, FileType } from 'lucide-svelte';

  export let studyMaterials: StudyMaterial[];
  export let collectionId: string;

  const dispatch = createEventDispatcher();

  let showDeleteConfirm = false;
  let materialToDelete: StudyMaterial | null = null;

  function getIcon(type: 'pdf' | 'markdown' | 'webpage') {
    switch (type) {
      case 'pdf':
        return FileType;
      case 'markdown':
        return FileText;
      case 'webpage':
        return Globe;
      default:
        return FileText;
    }
  }

  function getMaterialName(material: StudyMaterial): string {
    if (material.type === 'webpage') {
      return material.name || material.url;
    }
    return material.name;
  }

  async function handleViewFile(material: StudyMaterial) {
    if (material.type === 'pdf' && material.filePath) {
      try {
        await window.electronAPI.openFile(material.filePath);
      } catch (error) {
        console.error('Error opening PDF:', error);
        alert('Failed to open PDF file. Please try again.');
      }
    } else if (material.type === 'markdown' && material.filePath) {
      try {
        const content = await loadStudyMaterialContent(material.filePath);
        dispatch('viewFile', { ...material, content });
      } catch (error) {
        console.error('Error reading file:', error);
        alert('Failed to load file content. Please try again.');
      }
    } else if (material.type === 'webpage' && material.url) {
      window.open(material.url, '_blank');
    } else {
      dispatch('viewFile', material);
    }
  }

  function confirmDelete(material: StudyMaterial) {
    materialToDelete = material;
    showDeleteConfirm = true;
  }

  async function handleRemoveFile() {
    if (!materialToDelete) return;

    try {
      if (materialToDelete.filePath) {
        await deleteFile(materialToDelete.filePath);
      }
      dispatch('removeStudyMaterial', { collectionId, materialName: materialToDelete.name });
    } catch (error) {
      console.error('Error deleting file:', error);
      alert(`There was an error deleting the file: ${error.message}`);
    } finally {
      showDeleteConfirm = false;
      materialToDelete = null;
    }
  }
</script>

<div class={$options.simplifiedMaterialView ? 'space-y-2' : 'grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-6'}>
  {#each studyMaterials as material (material.id)}
    {#if $options.simplifiedMaterialView}
      <div
        class="flex items-center justify-between p-2 bg-white dark:bg-gray-800 rounded shadow-sm cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors duration-150"
        on:click={() => handleViewFile(material)}
      >
        <div class="flex items-center">
          <svelte:component this={getIcon(material.type)} size={24} class="mr-2 text-gray-600 dark:text-gray-300" />
          <span class="text-gray-800 dark:text-gray-200">
            {getMaterialName(material)}
          </span>
        </div>
        <button
          on:click|stopPropagation={() => confirmDelete(material)}
          class="p-1 text-gray-500 hover:text-red-500 dark:text-gray-400 dark:hover:text-red-400 focus:outline-none transition-colors duration-150"
          title="Remove"
        >
          <Trash2 size={20} />
        </button>
      </div>
    {:else}
      <div class="relative group bg-gray-100 dark:bg-gray-700 rounded-lg shadow-md overflow-hidden transition-all duration-300 hover:shadow-lg hover:scale-105">
        <div
          class="aspect-square flex flex-col items-center justify-center cursor-pointer p-4"
          on:click={() => handleViewFile(material)}
        >
          <svelte:component this={getIcon(material.type)} size={64} class="text-gray-600 dark:text-gray-300 mb-4" />
          <h3 class="text-sm font-medium text-gray-800 dark:text-white text-center line-clamp-3">{getMaterialName(material)}</h3>
        </div>
        <button
          on:click|stopPropagation={() => confirmDelete(material)}
          class="absolute top-2 right-2 p-2 bg-white dark:bg-gray-600 text-gray-600 dark:text-gray-300 rounded-full opacity-0 group-hover:opacity-100 transition-all duration-300 hover:bg-gray-200 dark:hover:bg-gray-500 focus:outline-none focus:ring-2 focus:ring-gray-300 dark:focus:ring-gray-400"
          title="Remove"
        >
          <Trash2 size={16} />
        </button>
      </div>
    {/if}
  {/each}
</div>

{#if showDeleteConfirm}
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl max-w-sm w-full">
      <div class="flex items-center mb-4">
        <AlertCircle class="text-gray-600 dark:text-gray-300 mr-2" size={24} />
        <h3 class="text-lg font-semibold text-gray-900 dark:text-white">Confirm Deletion</h3>
      </div>
      <p class="text-gray-700 dark:text-gray-300 mb-4">
        Are you sure you want to delete "{materialToDelete?.name}"? This action cannot be undone.
      </p>
      <div class="flex justify-end space-x-4">
        <button
          on:click={() => showDeleteConfirm = false}
          class="px-4 py-2 bg-gray-200 text-gray-800 dark:bg-gray-700 dark:text-gray-200 rounded hover:bg-gray-300 dark:hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-gray-300 dark:focus:ring-gray-500"
        >
          Cancel
        </button>
        <button
          on:click={handleRemoveFile}
          class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-300 dark:focus:ring-red-500"
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}