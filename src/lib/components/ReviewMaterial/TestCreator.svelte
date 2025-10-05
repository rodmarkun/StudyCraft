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
  import { TestCreatorLogic, type StudyMaterial, type TestFormData } from '../../logic/ReviewMaterial/testCreator';
  const logic = new TestCreatorLogic();

  // Stores
  import { llmStore } from '../../stores/llmStore';
  import { materialsStore } from '../../stores/materialsStore';

  // Icons
  import { Bot as BotIcon, File as FileIcon } from 'lucide-svelte';

  // Constants
  import { LANGUAGE_OPTIONS } from '../../logic/Constants/creatorConstants';

  // Props
  export let initialSelectedMaterials: StudyMaterial[] = [];

  // Callbacks
  export let onClose = () => {};
  export let onCreated = (test: any) => {};
  export let onShowMaterialsSelector = (details: any) => {};

  // State
  let formData: TestFormData = {
    name: '',
    language: 'English',
    tags: [], 
    selectedMaterials: [...initialSelectedMaterials],
    concept: ''
  };
  let isModalOpen = false;
  let updateInterval: number | null = null;
  let currentStep = logic.getCurrentStep;
  let questions = logic.getQuestions;
  let isCreating = logic.getIsCreating;
  let isGenerating = logic.getIsGenerating;
  let error = logic.getError;
  let generationStatus = logic.getGenerationStatus;
  let isFormDataInitialized = false;

  // Reactivity
  $: llmStatus = $llmStore;
  $: canUseAI = llmStatus.hasConfiguredProvider && !llmStatus.isLoading;
  $: totalQuestionsToGenerate = logic.getTotalQuestionsToGenerate(formData.selectedMaterials);

  $: currentStepComplete = (() => {
    if (currentStep === 1) {
      return formData.name.trim() !== '';
    } else if (currentStep === 2) {
      return questions.length > 0 && questions.every(question => 
        question.question.trim() !== '' && 
        question.answers.length >= 2 &&
        question.answers.some(answer => answer.is_correct) &&
        question.answers.every(answer => answer.answer_text.trim() !== '')
      );
    }
    return false;
  })();

  $: isValid = questions.length > 0 && questions.every(question => 
    question.question.trim() !== '' && 
    question.answers.length >= 2 &&
    question.answers.some(answer => answer.is_correct) &&
    question.answers.every(answer => answer.answer_text.trim() !== '')
  ) && formData.name.trim() !== '';

  $: stepsComplete = {
    1: formData.name.trim() !== '',
    2: questions.length > 0 && questions.every(question => 
      question.question.trim() !== '' && 
      question.answers.length >= 2 &&
      question.answers.some(answer => answer.is_correct) &&
      question.answers.every(answer => answer.answer_text.trim() !== '')
    ) 
  };

  $: if (isFormDataInitialized && formData) {
    const dataToStore = {
      name: formData.name,
      language: formData.language,
      concept: formData.concept
    };
    sessionStorage.setItem('test-creator-form', JSON.stringify(dataToStore));
  }

  // Update interval management
  $: if (isGenerating) {
    if (!updateInterval) {
      updateInterval = setInterval(() => {
        currentStep = logic.getCurrentStep;
        questions = logic.getQuestions;
        isCreating = logic.getIsCreating;
        isGenerating = logic.getIsGenerating;
        error = logic.getError;
        generationStatus = logic.getGenerationStatus;
      }, 100);
    }
  } else {
    if (updateInterval) {
      clearInterval(updateInterval);
      updateInterval = null;
    }
  }

  // Functions
  function handleClose() {
    sessionStorage.removeItem('test-creator-form');
    onClose();
  }

  async function handleNextStep() {
    console.log('handleNextStep called, currentStep:', currentStep);
    
    if (currentStep === 1 && formData.selectedMaterials.length > 0) {
      if (!canUseAI) {
        error = 'No AI provider configured. Skipping automatic generation - you can create questions manually in the next step.';
        logic.moveToNextStep(formData);
        currentStep = logic.getCurrentStep;
        return;
      }
      
      isGenerating = true;
      try {
        await logic.moveToNextStep(formData);
        currentStep = logic.getCurrentStep;
        questions = logic.getQuestions;
        isGenerating = logic.getIsGenerating;
        error = logic.getError;
        generationStatus = logic.getGenerationStatus;
      } catch (err) {
        console.error('Generation failed:', err);
        error = logic.getError;
        isGenerating = false;
      }
    } else {
      logic.moveToNextStep(formData);
      currentStep = logic.getCurrentStep;
    }
  }

  function handlePreviousStep() {
    logic.moveToPreviousStep();
    currentStep = logic.getCurrentStep;
  }

  // Question management functions
  function handleAddQuestion() {
    logic.addQuestion();
    questions = logic.getQuestions;
  }

  function handleRemoveQuestion(index: number) {
    logic.removeQuestion(index);
    questions = logic.getQuestions; 
  }

  function handleQuestionChange(index: number, value: string) {
    logic.updateQuestion(index, value);
    questions = logic.getQuestions;
  }

  function handleAnswerChange(questionIndex: number, answerIndex: number, field: 'answer_text' | 'is_correct', value: string | boolean) {
    logic.updateAnswer(questionIndex, answerIndex, field, value);
    questions = logic.getQuestions;
  }

  function handleAddAnswer(questionIndex: number) {
    logic.addAnswer(questionIndex);
    questions = logic.getQuestions;
  }

  function handleRemoveAnswer(questionIndex: number, answerIndex: number) {
    logic.removeAnswer(questionIndex, answerIndex);
    questions = logic.getQuestions;
  }

  // Modal functions
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

  // Material management functions
  function handleShowMaterialSelector() {
    onShowMaterialsSelector({ 
      fromTestCreator: true,
      currentSelectedMaterials: formData.selectedMaterials || []
    });
  }

  function handleRemoveMaterial(material: StudyMaterial) {
    formData = {
      ...formData,
      selectedMaterials: logic.removeMaterial(material, formData.selectedMaterials)
    };
  }

  function handleMaterialQuestionCountChange(materialId: string, count: number) {
    formData = {
      ...formData,
      selectedMaterials: logic.updateMaterialQuestionCount(materialId, count, formData.selectedMaterials)
    };
  }

  // Test creation function
  async function handleCreateTest() {
    try {
      const deckId = await materialsStore.addReviewMaterial({
        name: formData.name,
        type: 'test',
        tags: formData.tags
      });
      const newTest = await logic.createTest(formData, deckId);
      sessionStorage.removeItem('test-creator-form');
      onCreated(newTest);
    } catch (err) {
      error = logic.getError;
    }
  }

  onMount(() => {
    const stored = sessionStorage.getItem('test-creator-form');
    if (stored) {
      try {
        const parsed = JSON.parse(stored);
        formData = { 
          ...formData,
          name: parsed.name || formData.name,
          language: parsed.language || formData.language,
          concept: parsed.concept || formData.concept
        };
      } catch (e) {
        console.warn('Failed to restore form data:', e);
      }
    }
    
    if (initialSelectedMaterials && initialSelectedMaterials.length > 0) {
      handleMaterialsSelected(initialSelectedMaterials);
    }
    
    isFormDataInitialized = true;
  });

  onDestroy(() => {
    if (updateInterval) {
      clearInterval(updateInterval);
    }
  });

  export function handleMaterialsSelected(materials: StudyMaterial[]) {
    if (!materials || materials.length === 0) {
      console.warn('TestCreator: No materials provided or empty array');
      return;
    }
    
    formData = {
      ...formData,  
      selectedMaterials: materials.map(material => ({
        ...material,
        questionsToGenerate: material.questionsToGenerate || 5
      }))
    };
  }
</script>

<div class="creator-container" transition:fade={{ duration: 200 }}>
  <div class="creator-header">
    <h3>Create Test</h3>
    <button class="close-button" on:click={handleClose}>×</button>
  </div>
  
  <div class="creator-progress">
    <div class="progress-step" class:active={currentStep >= 1} class:complete={stepsComplete[1]}>
      <div class="step-number">1</div>
      <div class="step-label">Info & Materials</div>
    </div>
    <div class="progress-line" class:active={currentStep >= 2}></div>
    <div class="progress-step" class:active={currentStep >= 2} class:complete={stepsComplete[2]}>
      <div class="step-number">2</div>
      <div class="step-label">Add & Edit Questions</div>
    </div>
    <div class="progress-line" class:active={currentStep >= 3}></div>
    <div class="progress-step" class:active={currentStep >= 3}>
      <div class="step-number">3</div>
      <div class="step-label">Review Test</div>
    </div>
  </div>
  
  {#if error}
    <div class="error-message" transition:fade>
      {error}
    </div>
  {/if}
  
  {#if isGenerating}
    <div class="tc-generation-status" transition:fade>
      <div class="loading-spinner"></div>
      <div class="tc-status-text">
        <strong>Generating Questions...</strong>
        <p>{generationStatus || 'Processing your study materials...'}</p>
      </div>
    </div>
  {/if}
  
  <div class="creator-content">
    {#if currentStep === 1}
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
                placeholder="Enter test name..." 
                disabled={isGenerating}
                class="input"
                on:keydown={(e) => {
                  e.stopPropagation();
                }}
              />
            </div>
            
            <div class="tc-form-field">
              <label for="language">Language</label>
              <select id="language" bind:value={formData.language} class="tc-language-select" disabled={isGenerating}>
                {#each LANGUAGE_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
          </div>
        </div>
        
        <div class="tc-generation-section">
          <div class="tc-section-header">
            <h4>AI Generation (Optional)</h4>
            <p>Generate test questions automatically from your study materials</p>
            {#if !canUseAI}
              <p class="tc-ai-warning">⚠️ No AI provider configured. Configure one in Settings to enable automatic generation, or create questions manually in the next step.</p>
            {/if}
          </div>
          
          <div class="tc-select-materials-wrapper" title={!canUseAI ? 'Configure an AI provider in Settings to use automatic generation' : ''}>
            <Button 
              variant="secondary"
              text="Select Study Materials" 
              onClick={handleShowMaterialSelector}
              icon={BotIcon}
              disabled={isGenerating || !canUseAI}
            >
            </Button>
          </div>
          
          {#if formData.selectedMaterials.length > 0}
            <div class="tc-selected-materials-compact">
              <div class="tc-materials-header">
                <span class="tc-materials-count">{formData.selectedMaterials.length} materials selected</span>
                <span class="tc-total-questions">Total: {totalQuestionsToGenerate} questions</span>
              </div>
              
              <div class="tc-materials-grid">
                {#each formData.selectedMaterials as material}
                  <div class="tc-material-chip">
                    <div class="tc-material-info">
                      <FileIcon size={14}></FileIcon>
                      <span class="tc-material-name">
                        {material.name || material.display_name}
                      </span>
                    </div>
                    
                    <div class="tc-material-controls">
                      <input 
                        type="number" 
                        value={material.questionsToGenerate || 5}
                        on:input={(e) => handleMaterialQuestionCountChange(material.id, parseInt((e.target as HTMLInputElement).value))}
                        min="1" 
                        max="20" 
                        class="tc-question-count-input" 
                        title="Questions to generate"
                        disabled={isGenerating || !canUseAI}
                      />
                      <button 
                        aria-label="Remove Material"
                        class="tc-remove-material-btn" 
                        on:click={() => handleRemoveMaterial(material)}
                        title="Remove material"
                        disabled={isGenerating}
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <line x1="18" y1="6" x2="6" y2="18"></line>
                          <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>
              
              <div class="tc-concept-field">
                <label for="concept">Focus Concept *</label>
                <input 
                  id="concept"
                  type="text"
                  bind:value={formData.concept}
                  placeholder="e.g., machine learning algorithms, quantum physics, penguin biology..."
                  disabled={isGenerating || !canUseAI}
                  on:keydown={(e) => {
                    e.stopPropagation();
                  }}
                />
                <p class="tc-concept-help">
                  Specify the particular concept or topic you want the test questions to focus on from the selected materials.
                </p>
              </div>
            </div>
          {:else}
            <div class="tc-no-materials">
              <p>No materials selected. Select materials and specify a concept to enable AI generation, or create questions manually in the next step.</p>
            </div>
          {/if}
        </div>
      </div>
    {:else if currentStep === 2}
      <div class="tc-step-content" transition:fade>
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
        
        {#if questions.length === 0}
          <div class="tc-no-questions">
            <div class="tc-no-questions-content">
              <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
                <path d="m12 17l.01 0"></path>
              </svg>
              <h5>No questions yet</h5>
              <p>Click "Add Question" above to create your first question</p>
            </div>
          </div>
        {:else}
          <div class="tc-questions-list">
            {#each questions as question, questionIndex (questionIndex)}
              <div 
                class="tc-question-item" 
                animate:flip={{ duration: 300 }} 
                transition:slide|local
              >
                <div class="tc-question-header">
                  <span>Question {questionIndex + 1}</span>
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
                                on:change={() => {
                                  // Set this answer as correct and others as incorrect
                                  question.answers.forEach((a, i) => {
                                    handleAnswerChange(questionIndex, i, 'is_correct', i === answerIndex);
                                  });
                                }}
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
        {/if}
      </div>
    {:else if currentStep === 3}
      <div class="tc-step-content" transition:fade>
        <div class="tc-review-section">
          <h4>Test Information</h4>
          <div class="tc-review-item">
            <span class="tc-review-label">Name</span>
            <span class="tc-review-value">{formData.name}</span>
          </div>
          <div class="tc-review-item">
            <span class="tc-review-label">Language</span>
            <span class="tc-review-value">{formData.language}</span>
          </div>
          {#if formData.concept.trim()}
            <div class="tc-review-item">
              <span class="tc-review-label">Focus Concept</span>
              <span class="tc-review-value tc-concept-value">{formData.concept}</span>
            </div>
          {/if}
          <div class="tc-review-item">
            <span class="tc-review-label">Questions</span>
            <span class="tc-review-value">{questions.length} question{questions.length !== 1 ? 's' : ''}</span>
          </div>
          
          {#if formData.selectedMaterials.length > 0}
            <div class="tc-review-item">
              <span class="tc-review-label">Materials</span>
              <div class="tc-review-materials">
                {#each formData.selectedMaterials as material}
                  <span class="tc-material-item">{material.name || material.display_name}</span>
                {/each}
              </div>
            </div>
          {/if}
        </div>
        
        <div class="tc-review-questions-preview">
          <h4>Questions Preview</h4>
          <div class="tc-questions-preview-list">
            {#each questions.slice(0, 3) as question, index}
              <div class="tc-preview-question">
                <div class="tc-preview-question-header">
                  Question {index + 1}
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
    {#if currentStep > 1}
    <Button 
        variant="secondary" 
        text="Back"
        disabled={isCreating || isGenerating}
        onClick={handlePreviousStep}
    />
    {:else}
    <Button 
        variant="secondary" 
        text="Cancel"
        disabled={isCreating || isGenerating}
        onClick={handleClose}
    />
    {/if}
    
    {#if currentStep < 3}
      {@const canProceed = currentStep === 1 ? formData.name.trim() !== '' : (questions.length > 0 && questions.every(question => question.question.trim() !== '' && question.answers.length >= 2 && question.answers.some(answer => answer.is_correct) && question.answers.every(answer => answer.answer_text.trim() !== '')))}
        <Button
        variant="primary"
        text={isGenerating ? "Generating..." : "Next"}
        changed={isGenerating}
        disabled={!canProceed || isCreating || isGenerating}
        onClick={handleNextStep}
        />
    {:else}
        <Button
        variant="primary"
        text={isCreating ? "Creating..." : "Create Test"}
        changed={isCreating}
        disabled={!isValid || isCreating}
        onClick={handleCreateTest}
        />
    {/if}
  </div>
</div>

<TestQuestionModal 
  bind:isOpen={isModalOpen}
  onAddQuestion={handleModalAddQuestion}
  onClose={handleCloseModal}
/>