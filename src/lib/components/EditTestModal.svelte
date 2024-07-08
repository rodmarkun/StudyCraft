<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Test, TestData } from '../stores/collections';
  import Modal from './Modal.svelte';

  export let test: Test | null = null;
  export let isOpen: boolean;

  const dispatch = createEventDispatcher();

  let editedTest: Test = test ? { ...test, questions: [...test.questions] } : { id: '', name: '', questions: [] };
  let newQuestion: TestData = { id: '', question: '', options: ['', '', '', ''], correctOptionIndex: 0 };

  $: {
    if (test) {
      editedTest = { ...test, questions: [...test.questions] };
    }
  }

  function addQuestion() {
    editedTest.questions = [...editedTest.questions, { ...newQuestion, id: Date.now().toString() }];
    newQuestion = { id: '', question: '', options: ['', '', '', ''], correctOptionIndex: 0 };
  }

  function removeQuestion(index: number) {
    editedTest.questions = editedTest.questions.filter((_, i) => i !== index);
  }

  function saveTest() {
    dispatch('update', editedTest);
    closeModal();
  }

  function closeModal() {
    dispatch('close');
  }
</script>

<Modal {isOpen} title={`Edit Test: ${editedTest.name}`} on:close={closeModal}>
  <div class="p-4">
    <input
      type="text"
      bind:value={editedTest.name}
      placeholder="Test Name"
      class="w-full p-2 mb-4 border rounded text-black"
    />
    <h3 class="text-xl font-bold mb-2">Questions</h3>
    {#each editedTest.questions as question, index}
      <div class="mb-4 p-2 bg-gray-100 rounded">
        <p class="font-bold text-black">{question.question}</p>
        <button
          class="mt-2 px-2 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
          on:click={() => removeQuestion(index)}
        >
          Remove
        </button>
      </div>
    {/each}
    <h3 class="text-xl font-bold mb-2">Add New Question</h3>
    <input
      type="text"
      bind:value={newQuestion.question}
      placeholder="Question"
      class="w-full p-2 mb-2 border rounded text-black"
    />
    {#each newQuestion.options as option, index}
      <div class="flex mb-2">
        <input
          type="text"
          bind:value={newQuestion.options[index]}
          placeholder={`Option ${index + 1}`}
          class="flex-grow p-2 border rounded-l text-black"
        />
        <button
          class="px-4 py-2 bg-blue-500 text-white rounded-r"
          class:bg-green-500={newQuestion.correctOptionIndex === index}
          on:click={() => newQuestion.correctOptionIndex = index}
        >
          {newQuestion.correctOptionIndex === index ? 'Correct' : 'Set Correct'}
        </button>
      </div>
    {/each}
    <button
      class="w-full mt-2 px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
      on:click={addQuestion}
    >
      Add Question
    </button>
    <div class="mt-4 flex justify-end space-x-2">
      <button
        class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
        on:click={closeModal}
      >
        Cancel
      </button>
      <button
        class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        on:click={saveTest}
      >
        Save Test
      </button>
    </div>
  </div>
</Modal>