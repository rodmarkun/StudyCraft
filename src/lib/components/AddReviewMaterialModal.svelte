<!-- src/lib/components/AddReviewMaterialModal.svelte -->
<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import Modal from './Modal.svelte';
  
    export let isOpen = false;
    export let collectionId: string;
  
    const dispatch = createEventDispatcher();
  
    let type: 'flashcard' | 'test' = 'flashcard';
    let question = '';
    let answer = '';
    let options: string[] = ['', '', '', ''];
    let correctOptionIndex = 0;
  
    function handleSubmit() {
      const id = Date.now().toString();
      let data;
  
      if (type === 'flashcard') {
        data = { question, answer };
      } else {
        data = { question, options, correctOptionIndex };
      }
  
      dispatch('materialAdded', { type, id, data });
      reset();
      close();
    }
  
    function reset() {
      type = 'flashcard';
      question = '';
      answer = '';
      options = ['', '', '', ''];
      correctOptionIndex = 0;
    }
  
    function close() {
      reset();
      dispatch('close');
    }
  </script>
  
  <Modal {isOpen} title="Add Review Material" on:close={close}>
    <div class="space-y-4">
      <div>
        <label for="type" class="block mb-2">Type:</label>
        <select id="type" bind:value={type} class="w-full p-2 border rounded">
          <option value="flashcard">Flashcard</option>
          <option value="test">Test Question</option>
        </select>
      </div>
  
      <div>
        <label for="question" class="block mb-2">Question:</label>
        <textarea id="question" bind:value={question} class="w-full p-2 border rounded" rows="3"></textarea>
      </div>
  
      {#if type === 'flashcard'}
        <div>
          <label for="answer" class="block mb-2">Answer:</label>
          <textarea id="answer" bind:value={answer} class="w-full p-2 border rounded" rows="3"></textarea>
        </div>
      {:else}
        <div>
          <label class="block mb-2">Options:</label>
          {#each options as option, i}
            <div class="flex items-center mb-2">
              <input 
                type="radio" 
                id="option{i}" 
                name="correctOption" 
                value={i} 
                bind:group={correctOptionIndex} 
                class="mr-2"
              >
              <input 
                type="text" 
                bind:value={options[i]} 
                placeholder="Option {i + 1}" 
                class="w-full p-2 border rounded"
              >
            </div>
          {/each}
        </div>
      {/if}
  
      <button 
        on:click={handleSubmit}
        class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-opacity-50"
      >
        Add Material
      </button>
    </div>
  </Modal>