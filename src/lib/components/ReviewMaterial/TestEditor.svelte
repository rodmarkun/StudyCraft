<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fade, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import "../../styles/variables.css";
  import "../../styles/ReviewMaterial/testCreator.css";

  // Components
  import Button from '../Shared/Button.svelte';
  import TestQuestionModal from './TestQuestionModal.svelte';

  // Logic & Types
  import { TestEditorLogic, type ReviewMaterial, type TestEditFormData } from '../../logic/ReviewMaterial/testEditor';
  import type { StudyMaterial } from '../../logic/ReviewMaterial/testCreator';
  const logic = new TestEditorLogic();

  // Stores
  import { toastStore } from '../../stores/toastStore';

  // Props
  export let test: ReviewMaterial;
  export let isOpen = false;

  // Callbacks
  export let onClose = () => {};
  export let onUpdated = (details: any) => {};

  // State
  let formData: TestEditFormData = {
    testId: test.id,
    name: test.name, 
    language: 'English',
    selectedMaterials: [],
    concept: ''
  };
  let currentStep = 2;
  let isModalOpen = false;
  let updateInterval: number | null = null;
  let questions = [];
  let isLoading = false;
  let isSaving = false;
  let error = '';
  let hasUnsavedChanges = false;
  let isSubmitting = false; // Lock to prevent concurrent save operations

  // Reactivity
  $: stepsComplete = {
    2: questions.length > 0 && questions.every(question => 
      question.question.trim() !== '' && 
      question.answers.length >= 2 &&
      question.answers.some(answer => answer.is_correct) &&
      question.answers.every(answer => answer.answer_text.trim() !== '')
    )
  };

  $: isValid = questions.length > 0 && questions.every(question => 
    question.question.trim() !== '' && 
    question.answers.length >= 2 &&
    question.answers.some(answer => answer.is_correct) &&
    question.answers.every(answer => answer.answer_text.trim() !== '')
  ) && formData.name.trim() !== '';

  $: if (isLoading || isSaving) {
    if (!updateInterval) {
      updateInterval = setInterval(updateReactiveVars, 100);
    }
  } else {
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
  }

  $: {
    questions = logic.getQuestions;
    hasUnsavedChanges = logic.getHasUnsavedChanges;
  }

  // Functions
  function updateReactiveVars() {
    currentStep = logic.currentStep || 2; 
    questions = logic.getQuestions;
    isLoading = logic.getIsLoading;
    isSaving = logic.getIsSaving;
    error = logic.getError;
    hasUnsavedChanges = logic.getHasUnsavedChanges;
  }

  async function loadExistingQuestions() {
    try {
      await logic.loadExistingQuestions(test.id);
      updateReactiveVars();
    } catch (err) {
      console.error('Failed to load existing questions:', err);
      updateReactiveVars();
    }
  }

  function handleClose() {
    if (hasUnsavedChanges) {
      if (confirm('You have unsaved changes. Are you sure you want to close without saving?')) {
        onClose();
      }
    } else {
      onClose();
    }
  }

  async function handleNextStep() {
    currentStep = Math.min(currentStep + 1, 3);
    logic.currentStep = currentStep;
    updateReactiveVars();
  }

  function handlePreviousStep() {
    currentStep = Math.max(currentStep - 1, 2);
    logic.currentStep = currentStep;
    updateReactiveVars();
  }

  function handleAddQuestion() {
    logic.addQuestion();
    updateReactiveVars();
  }

  function handleRemoveQuestion(index: number) {
    if (confirm('Are you sure you want to delete this question?')) {
      logic.removeQuestion(index);
      updateReactiveVars();
    }
  }

  function handleQuestionChange(index: number, value: string) {
    logic.updateQuestion(index, value);
    updateReactiveVars();
  }

  function handleAnswerChange(questionIndex: number, answerIndex: number, field: 'answer_text' | 'is_correct', value: string | boolean) {
    logic.updateAnswer(questionIndex, answerIndex, field, value);
    updateReactiveVars();
  }

  function handleCorrectAnswerChange(questionIndex: number, answerIndex: number) {
    const question = questions[questionIndex];
    if (question && question.answers) {
      question.answers.forEach((answer, i) => {
        logic.updateAnswer(questionIndex, i, 'is_correct', i === answerIndex);
      });
      updateReactiveVars();
    }
  }

  function handleAddAnswer(questionIndex: number) {
    logic.addAnswer(questionIndex);
    updateReactiveVars();
  }

  function handleRemoveAnswer(questionIndex: number, answerIndex: number) {
    logic.removeAnswer(questionIndex, answerIndex);
    updateReactiveVars();
  }

  function handleNameChange() {
    logic.updateTestName(formData.name);
    updateReactiveVars();
  }

  function handleOpenModal() {
    isModalOpen = true;
  }

  function handleCloseModal() {
    isModalOpen = false;
  }

  function handleModalAddQuestion(data: { question: string; answers: Array<{ answer_text: string; is_correct: boolean; position: number }> }) {
    const { question, answers } = data;
    logic.addQuestionWithContent(question, answers);
    questions = logic.getQuestions;
  }

  async function handleSaveChanges() {
    // Prevent concurrent submissions
    if (isSubmitting || isSaving) return;

    // Validate form before submission
    if (!formData.name.trim()) {
      toastStore.warning('Please enter a test name');
      return;
    }

    const activeQuestions = questions.filter(q => !q.isDeleted);
    if (activeQuestions.length === 0) {
      toastStore.warning('Test must have at least one question');
      return;
    }

    const invalidQuestions = activeQuestions.filter(
      (q) => !q.question.trim() ||
             q.answers.length < 2 ||
             !q.answers.some(a => a.is_correct) ||
             q.answers.some(a => !a.answer_text.trim())
    );
    if (invalidQuestions.length > 0) {
      toastStore.warning('Please ensure all questions have text, at least 2 answers, one correct answer, and all answers have text');
      return;
    }

    isSubmitting = true;

    try {
      await logic.saveChanges(formData);

      toastStore.success('Test updated successfully');
      onUpdated({
        testId: test.id,
        name: formData.name,
        questionsCount: activeQuestions.length
      });

      onClose();
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      console.error('Failed to save test changes:', err);
      error = errorMessage;
      toastStore.error(`Failed to save changes: ${errorMessage}`);
      updateReactiveVars();
    } finally {
      isSubmitting = false;
    }
  }

  onMount(async () => {
    if (test && test.id) {
      logic.currentStep = 2;
      logic.setOriginalTestName(test.name);
      await loadExistingQuestions();
    }
    updateReactiveVars();
  });

  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
  });

  export function handleMaterialsSelected(materials: StudyMaterial[]) {
    formData = {
      ...formData,
      selectedMaterials: materials.map(material => ({
        ...material,
        questionsToGenerate: material.questionsToGenerate || 5
      }))
    };
  }
</script>

{#if isOpen}
<div class="creator-container" transition:fade={{ duration: 200 }}>
  <div class="creator-header">
    <h3>Edit Test: {test.name}</h3>
    <button class="close-button" on:click={handleClose}>×</button>
  </div>
  
  <div class="creator-progress">
    <div class="progress-step" class:active={currentStep >= 2} class:complete={stepsComplete[2]}>
      <div class="step-number">1</div>
      <div class="step-label">Edit Questions</div>
    </div>
    <div class="progress-line" class:active={currentStep >= 3}></div>
    <div class="progress-step" class:active={currentStep >= 3}>
      <div class="step-number">2</div>
      <div class="step-label">Review Changes</div>
    </div>
  </div>
  
  {#if error}
    <div class="error-message" transition:fade>
      {error}
    </div>
  {/if}
  
  {#if hasUnsavedChanges}
    <div class="tc-unsaved-changes" transition:fade>
      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="10"></circle>
        <line x1="12" y1="8" x2="12" y2="12"></line>
        <line x1="12" y1="16" x2="12.01" y2="16"></line>
      </svg>
      You have unsaved changes
    </div>
  {/if}
  
  {#if isLoading}
    <div class="tc-generation-status" transition:fade>
      <div class="loading-spinner"></div>
      <div class="tc-status-text">
        <strong>Loading Questions...</strong>
        <p>Retrieving existing questions from the database...</p>
      </div>
    </div>
  {/if}
  
  <div class="creator-content">
    {#if currentStep === 2}
      <div class="tc-step-content" transition:fade>
        <div class="tc-test-info-section">
          <h4>Test Information</h4>
          <div class="tc-form-grid">
            <div class="tc-form-field">
              <label for="test-name">Test Name *</label>
              <input 
                id="test-name"
                type="text"
                bind:value={formData.name}
                on:input={handleNameChange}
                placeholder="Enter test name..." 
                disabled={isSaving}
                class="input"
                on:keydown={(e) => {
                  e.stopPropagation();
                }}
              />
            </div>
          </div>
        </div>

        <div class="tc-questions-header">
          <h4>Edit Your Questions</h4>
          <div class="tc-questions-header-actions">
            <button class="tc-add-question-modal-button" on:click={handleOpenModal}>
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="12" y1="9" x2="12" y2="15"></line>
                <line x1="9" y1="12" x2="15" y2="12"></line>
              </svg>
              Add Question
            </button>
            <span class="tc-questions-count">{questions.length} question{questions.length !== 1 ? 's' : ''}</span>
          </div>
        </div>
        
        <div class="tc-questions-list">
          {#each questions.filter(question => !question.isDeleted) as question, questionIndex (question.tempId)}
            <div 
              class="tc-question-item" 
              class:tc-new-question={question.isNew}
              animate:flip={{ duration: 300 }} 
              transition:slide|local
            >
              <div class="tc-question-header">
                <span>
                  Question {questionIndex + 1}
                  {#if question.isNew}
                    <span class="tc-new-badge">New</span>
                  {/if}
                </span>
                <button class="tc-remove-question" on:click={() => handleRemoveQuestion(questionIndex)}>
                  Remove Question
                </button>
              </div>
              
              <div class="tc-question-content">
                <div class="tc-question-text">
                  <p>Question</p>
                  <textarea 
                    value={question.question}
                    on:input={(e) => handleQuestionChange(questionIndex, (e.target as HTMLTextAreaElement).value)}
                    placeholder="Enter the question..."
                  ></textarea>
                </div>
                
                <div class="tc-answers-section">
                  <div class="tc-answers-header">
                    <p>Answers</p>
                    <button 
                      class="tc-add-answer-btn"
                      on:click={() => handleAddAnswer(questionIndex)}
                      title="Add answer option"
                    >
                      <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <line x1="12" y1="5" x2="12" y2="19"></line>
                        <line x1="5" y1="12" x2="19" y2="12"></line>
                      </svg>
                      Add Answer
                    </button>
                  </div>
                  
                  <div class="tc-answers-list">
                    {#each question.answers as answer, answerIndex (answerIndex)}
                      <div class="tc-answer-item">
                        <div class="tc-answer-controls">
                          <label class="tc-correct-checkbox">
                            <input 
                              type="radio" 
                              name="correct-{questionIndex}"
                              checked={answer.is_correct}
                              on:change={() => handleCorrectAnswerChange(questionIndex, answerIndex)}
                            />
                            <span class="tc-correct-label">Correct</span>
                          </label>
                          
                          {#if question.answers.length > 2}
                            <button 
                              class="tc-remove-answer-btn"
                              on:click={() => handleRemoveAnswer(questionIndex, answerIndex)}
                              title="Remove answer"
                              aria-label="Remove answer"
                            >
                              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <line x1="18" y1="6" x2="6" y2="18"></line>
                                <line x1="6" y1="6" x2="18" y2="18"></line>
                              </svg>
                            </button>
                          {/if}
                        </div>
                        
                        <input 
                          type="text"
                          value={answer.answer_text}
                          on:input={(e) => handleAnswerChange(questionIndex, answerIndex, 'answer_text', (e.target as HTMLInputElement).value)}
                          placeholder="Enter answer option..."
                          class="tc-answer-input"
                        />
                      </div>
                    {/each}
                  </div>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else if currentStep === 3}
      <div class="tc-step-content" transition:fade>
        <div class="tc-review-section">
          <h4>Review Changes</h4>
          <div class="tc-review-item">
            <span class="tc-review-label">Test Name</span>
            <span class="tc-review-value">{formData.name}</span>
          </div>
          <div class="tc-review-item">
            <span class="tc-review-label">Total Questions</span>
            <span class="tc-review-value">{questions.length} question{questions.length !== 1 ? 's' : ''}</span>
          </div>
          
          {#if questions.some(question => question.isNew)}
            <div class="tc-review-item">
              <span class="tc-review-label">New Questions</span>
              <span class="tc-review-value tc-new-questions-count">
                {questions.filter(question => question.isNew).length} new question{questions.filter(question => question.isNew).length !== 1 ? 's' : ''} added
              </span>
            </div>
          {/if}
          
          {#if hasUnsavedChanges}
            <div class="tc-review-item">
              <span class="tc-review-label">Status</span>
              <span class="tc-review-value tc-has-changes">Has unsaved changes</span>
            </div>
          {/if}
        </div>
        
        <div class="tc-review-questions-preview">
          <h4>Questions Preview</h4>
          <div class="tc-questions-preview-list">
            {#each questions.slice(0, 3) as question, index}
              <div class="tc-preview-question" class:tc-new-question={question.isNew}>
                <div class="tc-preview-question-header">
                  Question {index + 1}
                  {#if question.isNew}
                    <span class="tc-new-badge">New</span>
                  {/if}
                </div>
                <div class="tc-preview-question-content">
                  <div class="tc-preview-question-text">
                    <span class="tc-preview-label">Question</span>
                    <div class="tc-preview-content">
                      {question.question.length > 80 ? question.question.substring(0, 80) + '...' : question.question}
                    </div>
                  </div>
                  <div class="tc-preview-answers">
                    <span class="tc-preview-label">Answers</span>
                    <div class="tc-preview-answers-list">
                      {#each question.answers as answer}
                        <div class="tc-preview-answer" class:tc-correct={answer.is_correct}>
                          {answer.answer_text.length > 40 ? answer.answer_text.substring(0, 40) + '...' : answer.answer_text}
                        </div>
                      {/each}
                    </div>
                  </div>
                </div>
              </div>
            {/each}
            {#if questions.length > 3}
              <div class="tc-more-questions">
                +{questions.length - 3} more question{questions.length - 3 !== 1 ? 's' : ''} not shown
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
  
  <div class="creator-actions">
    {#if currentStep > 2}
      <Button 
        variant="secondary" 
        text="Back"
        disabled={isSaving}
        onClick={handlePreviousStep}
      />
    {:else}
      <Button 
        variant="secondary" 
        text={hasUnsavedChanges ? 'Cancel' : 'Close'}
        disabled={isSaving}
        onClick={handleClose}
      />
    {/if}
    
    {#if currentStep < 3}
      <Button 
        variant="primary" 
        text="Next"
        disabled={!stepsComplete[2] || isSaving}
        onClick={handleNextStep}
      />
    {:else}
      <Button
        variant="primary"
        text={isSaving || isSubmitting ? 'Saving...' : 'Save Changes'}
        changed={isSaving || isSubmitting}
        disabled={!isValid || isSaving || isSubmitting || !hasUnsavedChanges}
        onClick={handleSaveChanges}
      />
    {/if}
  </div>
</div>

<TestQuestionModal 
  bind:isOpen={isModalOpen}
  onAddQuestion={handleModalAddQuestion}
  onClose={handleCloseModal}
/>
{/if}