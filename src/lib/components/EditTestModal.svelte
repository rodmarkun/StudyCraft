<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Test, TestData, StudyMaterial } from '../stores/collections';
  import Modal from './Modal.svelte';
  import Popover from './Popover.svelte';
  import { generateTestQuestions } from '../services/llmService';
  import { loadStudyMaterialContent } from '../stores/collections';
  import { scrapeWebsite } from '../services/webScraperService';
  import { cleanPDFContent } from '../utils/fileUtils';

  export let test: Test | null = null;
  export let isOpen: boolean;
  export let collectionId: string | undefined;
  export let studyMaterials: StudyMaterial[];

  const dispatch = createEventDispatcher();

  let editedTest: Test = test ? { ...test, questions: [...test.questions] } : { id: '', name: '', questions: [] };
  let newQuestion: TestData = { id: '', question: '', options: ['', '', '', ''], correctOptionIndex: 0 };
  let errorMessage = '';
  let numberOfQuestionsToGenerate = 5;
  let selectedMaterialId = '';
  let isGenerating = false;
  let isGeneratePopoverOpen = false;

  $: {
    if (test) {
      editedTest = { ...test, questions: [...test.questions] };
    }
  }

  function toggleGeneratePopover() {
    isGeneratePopoverOpen = !isGeneratePopoverOpen;
  }

  async function handleGenerateQuestions() {
    await generateQuestionsFromMaterial();
    isGeneratePopoverOpen = false;
  }

  async function generateQuestionsFromMaterial() {
    if (!editedTest || !selectedMaterialId) return;

    const selectedMaterial = studyMaterials.find(m => m.id === selectedMaterialId);
    if (!selectedMaterial) {
      errorMessage = 'Selected study material not found.';
      return;
    }

    isGenerating = true;
    errorMessage = '';
    try {
      let content;
      if (selectedMaterial.type === 'markdown') {
        content = await loadStudyMaterialContent(selectedMaterial.filePath!);
      } else if (selectedMaterial.type === 'pdf') {
        content = await cleanPDFContent(selectedMaterial.filePath!);
      } else if (selectedMaterial.type === 'webpage') {
        content = await scrapeWebsite(selectedMaterial.url!);
      } else {
        throw new Error('Unsupported material type');
      }

      const generatedQuestions = await generateTestQuestions(content, numberOfQuestionsToGenerate);

      editedTest.questions = [
        ...editedTest.questions,
        ...generatedQuestions.map((q) => ({
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          question: q.question,
          options: q.options,
          correctOptionIndex: q.correctOptionIndex
        }))
      ];

      errorMessage = '';
      isGeneratePopoverOpen = false;
    } catch (error) {
      console.error('Error generating questions:', error);
      errorMessage = `Failed to generate questions: ${error.message}. Please try again or check your LLM configuration.`;
    } finally {
      isGenerating = false;
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

    {#if errorMessage}
      <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded relative mb-4" role="alert">
        <strong class="font-bold">Error!</strong>
        <span class="block sm:inline">{errorMessage}</span>
      </div>
    {/if}

    <h3 class="text-xl font-bold mb-2">Questions</h3>
    {#each editedTest.questions as question, index}
      <div class="mb-4 p-2 bg-gray-100 rounded">
        <p class="font-bold text-black">{question.question}</p>
        <ul class="list-disc pl-5 mt-2">
          {#each question.options as option, optionIndex}
            <li class="{optionIndex === question.correctOptionIndex ? 'text-green-600 font-bold' : 'text-black'}">{option}</li>
          {/each}
        </ul>
        <button
          class="mt-2 px-2 py-1 text-sm bg-red-500 text-white rounded hover:bg-red-600"
          on:click={() => removeQuestion(index)}
        >
          Remove
        </button>
      </div>
    {/each}

    <div class="mb-4">
      <Popover bind:open={isGeneratePopoverOpen} width="w-96" customClass="dark:bg-gray-700">
        <button
          slot="trigger"
          on:click={toggleGeneratePopover}
          class="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600"
        >
          Generate Questions
        </button>
        <div class="p-4 space-y-4">
          <h3 class="text-lg font-semibold mb-2 text-gray-800 dark:text-white">Generate Questions</h3>
          <p class="text-sm text-gray-600 dark:text-gray-300 mb-4">
            Select a study material and specify the number of questions you want to generate.
            The AI will create questions based on the content of the selected material.
          </p>
          <select
            bind:value={selectedMaterialId}
            class="w-full p-2 border rounded text-gray-800 dark:text-white dark:bg-gray-600"
          >
            <option value="">Select a study material</option>
            {#each studyMaterials as material}
              <option value={material.id}>{material.name}</option>
            {/each}
          </select>
          <div class="mt-2">
            <label for="numQuestions" class="block text-sm font-medium text-gray-700 dark:text-gray-300">
              Number of questions to generate:
            </label>
            <input
              id="numQuestions"
              type="number"
              bind:value={numberOfQuestionsToGenerate}
              min="1"
              max="20"
              class="mt-1 w-full p-2 border rounded text-gray-800 dark:text-white dark:bg-gray-600"
            />
          </div>
          <button
            on:click={handleGenerateQuestions}
            class="w-full px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600"
            disabled={isGenerating || !selectedMaterialId}
          >
            {isGenerating ? 'Generating...' : 'Generate Questions'}
          </button>
        </div>
      </Popover>
    </div>

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