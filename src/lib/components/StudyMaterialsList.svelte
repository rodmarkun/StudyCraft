<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { StudyMaterial } from '../stores/collections';
  import { loadStudyMaterialContent } from '../stores/collections';
  import { deleteFile } from '../utils/fileUtils';

  export let studyMaterials: StudyMaterial[];
  export let collectionId: string;

  const dispatch = createEventDispatcher();

  function getIcon(type: 'pdf' | 'markdown' | 'webpage') {
    switch (type) {
      case 'pdf':
        return '📄';
      case 'markdown':
        return '📝';
      case 'webpage':
        return '🌐';
      default:
        return '📎';
    }
  }

  async function handleViewFile(material: StudyMaterial) {
    if (material.type === 'markdown' && material.filePath) {
      try {
        console.log(material.filePath)
        const content = await loadStudyMaterialContent(material.filePath);
        dispatch('viewFile', { ...material, content });
      } catch (error) {
        console.error('Error reading file:', error);
        alert('Failed to load file content. Please try again.');
      }
    } else {
      dispatch('viewFile', material);
    }
  }

  async function handleRemoveFile(material: StudyMaterial) {
    if (confirm(`Are you sure you want to remove "${material.name}"?`)) {
      try {
        if (material.filePath) {
          await deleteFile(material.filePath);
        }
        dispatch('removeStudyMaterial', { collectionId, materialName: material.name });
      } catch (error) {
        console.error('Error deleting file:', error);
        alert(`There was an error deleting the file: ${error.message}`);
      }
    }
  }
</script>
  
  <div class="space-y-2">
    {#each studyMaterials as material (material.id)}
      <div 
        class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-700 rounded cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-600"
        on:click={() => handleViewFile(material)}
      >
        <div class="flex items-center">
          <span class="text-2xl mr-2">{getIcon(material.type)}</span>
          <span class="text-text-light dark:text-text-dark">
            {material.name}
          </span>
        </div>
        <button
          on:click|stopPropagation={() => handleRemoveFile(material)}
          class="p-1 text-red-500 hover:text-red-700 focus:outline-none"
          title="Remove"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        </button>
      </div>
    {/each}
  </div>