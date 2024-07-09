<!-- src/lib/components/ConfirmDialog.svelte -->
<script lang="ts">
    import { createEventDispatcher } from 'svelte';
  
    export let title: string;
    export let message: string;
    export let confirmText: string = 'Confirm';
    export let cancelText: string = 'Cancel';
  
    const dispatch = createEventDispatcher();
  
    function handleConfirm() {
      dispatch('confirm');
    }
  
    function handleCancel() {
      dispatch('cancel');
    }
  </script>
  
  <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
    <div class="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl">
      <h3 class="text-lg font-semibold mb-4">{title}</h3>
      <p class="mb-4">{message}</p>
      <div class="flex justify-end space-x-2">
        <button
          on:click={handleCancel}
          class="px-4 py-2 bg-gray-300 text-gray-800 rounded hover:bg-gray-400 focus:outline-none"
        >
          {cancelText}
        </button>
        <button
          on:click={handleConfirm}
          class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 focus:outline-none"
        >
          {confirmText}
        </button>
      </div>
    </div>
  </div>