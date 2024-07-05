<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Test } from '../stores/collections';
  import Modal from './Modal.svelte';

  export let test: Test | null = null;
  export let isOpen: boolean;

  const dispatch = createEventDispatcher();

  let currentQuestionIndex = 0;
  let score = 0;
  let showResult = false;

  $: currentQuestion = test && test.questions.length > 0 ? test.questions[currentQuestionIndex] : null;
  $: totalQuestions = test ? test.questions.length : 0;

  function handleAnswer(selectedIndex: number) {
    if (currentQuestion && selectedIndex === currentQuestion.correctOptionIndex) {
      score++;
    }

    if (currentQuestionIndex < totalQuestions - 1) {
      currentQuestionIndex++;
    } else {
      showResult = true;
    }
  }

  function closeModal() {
    dispatch('close');
    currentQuestionIndex = 0;
    score = 0;
    showResult = false;
  }
</script>

<Modal {isOpen} title={test ? test.name : 'Study Test'} on:close={closeModal}>
  {#if test && test.questions.length > 0}
    {#if !showResult && currentQuestion}
      <div class="p-4">
        <h3 class="text-xl font-bold mb-4">Question {currentQuestionIndex + 1} of {totalQuestions}</h3>
        <p class="mb-4">{currentQuestion.question}</p>
        <div class="space-y-2">
          {#each currentQuestion.options as option, index}
            <button
              class="w-full p-2 text-left bg-gray-100 hover:bg-gray-200 rounded"
              on:click={() => handleAnswer(index)}
            >
              {option}
            </button>
          {/each}
        </div>
      </div>
    {:else if showResult}
      <div class="p-4">
        <h3 class="text-xl font-bold mb-4">Test Results</h3>
        <p>Your score: {score} out of {totalQuestions}</p>
        <button
          class="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          on:click={closeModal}
        >
          Close
        </button>
      </div>
    {/if}
  {:else}
    <div class="p-4">
      <p>No questions available for this test.</p>
      <button
        class="mt-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        on:click={closeModal}
      >
        Close
      </button>
    </div>
  {/if}
</Modal>