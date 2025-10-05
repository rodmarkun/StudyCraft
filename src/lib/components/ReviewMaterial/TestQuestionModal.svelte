<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { fade } from 'svelte/transition';
  import "../../styles/variables.css";

  // Components
  import Button from '../Shared/Button.svelte';

  // Props
  export let isOpen = false;
  export let onAddQuestion: (data: { question: string; answers: Array<{ answer_text: string; is_correct: boolean; position: number }> }) => void = () => {};
  export let onClose: () => void = () => {};

  // State
  let questionTextarea: HTMLTextAreaElement;
  let modalContainer: HTMLDivElement;
  let question = '';
  let answers = [
    { answer_text: '', is_correct: true, position: 0 },
    { answer_text: '', is_correct: false, position: 1 }
  ];

  // Reactivity
  $: if (isOpen && questionTextarea) {
    tick().then(() => {
      questionTextarea.focus();
    });
  }
  $: canAdd = question.trim() !== '' && 
             answers.length >= 2 && 
             answers.some(answer => answer.is_correct) &&
            answers.every(answer => answer.answer_text.trim() !== '');

  // Functions
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey && event.key === 'Enter') {
      event.preventDefault();
      handleAddQuestion();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleClose();
    }
  }

  function handleAddAnswer() {
    answers = [...answers, {
      answer_text: '',
      is_correct: false,
      position: answers.length
    }];
  }

  function handleRemoveAnswer(index: number) {
    if (answers.length > 2) {
      answers = answers.filter((_, i) => i !== index);
      answers = answers.map((answer, i) => ({ ...answer, position: i }));
      
      if (!answers.some(answer => answer.is_correct)) {
        answers[0].is_correct = true;
      }
    }
  }

  function handleAnswerChange(index: number, field: 'answer_text' | 'is_correct', value: string | boolean) {
    if (field === 'is_correct' && value) {
      answers = answers.map((answer, i) => ({
        ...answer,
        is_correct: i === index
      }));
    } else {
      answers = answers.map((answer, i) => 
        i === index ? { ...answer, [field]: value } : answer
      );
    }
  }

  function handleAddQuestion() {
    if (question.trim() === '' || 
        answers.length < 2 || 
        !answers.some(answer => answer.is_correct) ||
        answers.some(answer => answer.answer_text.trim() === '')) {
      return;
    }

    onAddQuestion({
      question: question.trim(),
      answers: answers.map(answer => ({
        ...answer,
        answer_text: answer.answer_text.trim()
      }))
    });

    question = '';
    answers = [
      { answer_text: '', is_correct: true, position: 0 },
      { answer_text: '', is_correct: false, position: 1 }
    ];
    questionTextarea.focus();
  }

  function handleClose() {
    question = '';
    answers = [
      { answer_text: '', is_correct: true, position: 0 },
      { answer_text: '', is_correct: false, position: 1 }
    ];
    onClose();
  }

  function handleOutsideClick(event: MouseEvent) {
    if (event.target === modalContainer) {
      handleClose();
    }
  }

  onMount(() => {
    if (isOpen && questionTextarea) {
      questionTextarea.focus();
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div 
    class="tqm-overlay" 
    transition:fade={{ duration: 200 }}
    bind:this={modalContainer}
    on:click={handleOutsideClick}
    role="dialog"
    aria-modal="true"
    aria-labelledby="tqm-title"
  >
    <div class="tqm-modal" on:click|stopPropagation>
      <div class="tqm-header">
        <h3 id="tqm-title">Add Question</h3>
        <button class="tqm-close" on:click={handleClose} aria-label="Close modal">×</button>
      </div>

      <div class="tqm-content">
        <div class="tqm-field">
          <label for="tqm-question">Question</label>
          <textarea
            id="tqm-question"
            bind:this={questionTextarea}
            bind:value={question}
            placeholder="Enter the question..."
            rows="3"
            on:keydown={handleKeydown}
          ></textarea>
        </div>

        <div class="tqm-answers-section">
          <div class="tqm-answers-header">
            <label>Answer Options</label>
            <button 
              class="tqm-add-answer-btn"
              on:click={handleAddAnswer}
              title="Add answer option"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
              Add Answer
            </button>
          </div>
          
          <div class="tqm-answers-list">
            {#each answers as answer, index (index)}
              <div class="tqm-answer-item">
                <div class="tqm-answer-controls">
                  <label class="tqm-correct-radio">
                    <input 
                      type="radio" 
                      name="correct-answer"
                      checked={answer.is_correct}
                      on:change={() => handleAnswerChange(index, 'is_correct', true)}
                    />
                    <span class="tqm-correct-label">Correct</span>
                  </label>
                  
                  {#if answers.length > 2}
                    <button 
                      class="tqm-remove-answer-btn"
                      on:click={() => handleRemoveAnswer(index)}
                      title="Remove answer"
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
                  on:input={(e) => handleAnswerChange(index, 'answer_text', (e.target as HTMLInputElement).value)}
                  placeholder="Enter answer option..."
                  class="tqm-answer-input"
                  on:keydown={handleKeydown}
                />
              </div>
            {/each}
          </div>
        </div>
      </div>

      <div class="tqm-footer">
        <div class="tqm-shortcuts">
          <span class="tqm-shortcut">
            <kbd>Ctrl</kbd> + <kbd>Enter</kbd> to add
          </span>
          <span class="tqm-shortcut">
            <kbd>Esc</kbd> to close
          </span>
        </div>
        
        <div class="tqm-actions">
          <Button variant="secondary" onClick={handleClose} text="Cancel"/>
          <Button 
            variant="primary" 
            onClick={handleAddQuestion}
            disabled={!canAdd}
            text="Add Question"
          />
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
.tqm-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  backdrop-filter: blur(4px);
  padding: var(--space-md);
}

.tqm-modal {
  background: var(--surface);
  border-radius: var(--border-radius);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border);
  max-width: 700px;
  width: 100%;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.tqm-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md);
  border-bottom: 1px solid var(--border);
  background: var(--background);
}

.tqm-header h3 {
  margin: 0;
  font-family: var(--font-heading);
  font-weight: 600;
  font-size: 1.3rem;
  color: var(--text);
}

.tqm-close {
  width: 36px;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  font-size: 1.5rem;
  transition: all var(--transition-speed) ease;
}

.tqm-close:hover {
  background: color-mix(in srgb, var(--error) 10%, transparent);
  color: var(--error);
}

.tqm-content {
  flex: 1;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  overflow-y: auto;
}

.tqm-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.tqm-field label {
  font-weight: 600;
  color: var(--text);
  font-size: 0.95rem;
}

.tqm-field textarea {
  width: 100%;
  padding: var(--space-sm) var(--space-md);
  border: 2px solid var(--border);
  border-radius: var(--border-radius);
  background: var(--background);
  color: var(--text);
  font-family: var(--font-body);
  font-size: 1rem;
  line-height: 1.5;
  resize: vertical;
  min-height: 100px;
  transition: all var(--transition-speed) ease;
  box-sizing: border-box;
}

.tqm-field textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 15%, transparent);
}

.tqm-field textarea::placeholder {
  color: var(--text-secondary);
  font-style: italic;
}

.tqm-answers-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.tqm-answers-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tqm-answers-header label {
  font-weight: 600;
  color: var(--text);
  font-size: 0.95rem;
}

.tqm-add-answer-btn {
  background: var(--background);
  border: 1px solid var(--border);
  color: var(--text-secondary);
  cursor: pointer;
  padding: var(--space-xs) var(--space-sm);
  border-radius: var(--border-radius);
  transition: all var(--transition-speed) ease;
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  font-size: 0.85rem;
  font-weight: 500;
}

.tqm-add-answer-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
  background: color-mix(in srgb, var(--accent) 5%, var(--background));
}

.tqm-answers-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.tqm-answer-item {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
  background: var(--background);
  border: 1px solid var(--border);
  border-radius: var(--border-radius);
  padding: var(--space-sm);
}

.tqm-answer-controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.tqm-correct-radio {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  cursor: pointer;
}

.tqm-correct-radio input[type="radio"] {
  margin: 0;
  width: 16px;
  height: 16px;
  accent-color: var(--accent);
}

.tqm-correct-label {
  font-size: 0.85rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.tqm-remove-answer-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all var(--transition-speed) ease;
}

.tqm-remove-answer-btn:hover {
  background: color-mix(in srgb, var(--error) 10%, transparent);
  color: var(--error);
}

.tqm-answer-input {
  width: 100%;
  padding: var(--space-xs) var(--space-sm);
  border: 1px solid var(--border);
  border-radius: var(--border-radius);
  background: var(--background);
  color: var(--text);
  font-family: var(--font-body);
  font-size: 0.9rem;
  transition: border-color var(--transition-speed) ease;
  box-sizing: border-box;
}

.tqm-answer-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 15%, transparent);
}

.tqm-answer-input::placeholder {
  color: var(--text-secondary);
  font-style: italic;
}

.tqm-footer {
  padding: var(--space-md);
  border-top: 1px solid var(--border);
  background: var(--background);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.tqm-shortcuts {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md);
  justify-content: center;
}

.tqm-shortcut {
  color: var(--text-secondary);
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.tqm-shortcut kbd {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.8rem;
  font-family: var(--font-mono, monospace);
  color: var(--text);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.tqm-actions {
  display: flex;
  gap: var(--space-md);
  justify-content: flex-end;
}

@media (max-width: 768px) {
  .tqm-overlay {
    padding: var(--space-sm);
  }

  .tqm-modal {
    max-height: 95vh;
    max-width: none;
    width: 95%;
  }

  .tqm-header {
    padding: var(--space-md);
  }

  .tqm-content {
    padding: var(--space-md);
    gap: var(--space-md);
  }

  .tqm-footer {
    padding: var(--space-md);
  }

  .tqm-shortcuts {
    flex-direction: column;
    gap: var(--space-xs);
    text-align: center;
  }

  .tqm-actions {
    flex-direction: column;
    gap: var(--space-sm);
  }

  .tqm-field textarea {
    min-height: 80px;
  }

  .tqm-answers-header {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-sm);
  }

  .tqm-add-answer-btn {
    align-self: flex-end;
  }

  .tqm-answer-controls {
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-xs);
  }

  .tqm-remove-answer-btn {
    align-self: flex-end;
  }
}

@media (max-width: 480px) {
  .tqm-overlay {
    padding: var(--space-xs);
  }

  .tqm-header {
    padding: var(--space-sm);
  }

  .tqm-content {
    padding: var(--space-sm);
  }

  .tqm-footer {
    padding: var(--space-sm);
  }

  .tqm-shortcut {
    font-size: 0.8rem;
  }

  .tqm-shortcut kbd {
    font-size: 0.75rem;
    padding: 1px 4px;
  }

  .tqm-answers-header {
    gap: var(--space-xs);
  }

  .tqm-add-answer-btn {
    font-size: 0.8rem;
    padding: 6px 8px;
  }
}

.tqm-close:focus,
.tqm-field textarea:focus,
.tqm-answer-input:focus,
.tqm-add-answer-btn:focus,
.tqm-remove-answer-btn:focus {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

.tqm-correct-radio:focus-within {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
  border-radius: var(--border-radius);
}

@media (prefers-color-scheme: dark) {
  .tqm-shortcut kbd {
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }
}

@media (prefers-reduced-motion: reduce) {
  .tqm-close,
  .tqm-field textarea,
  .tqm-answer-input,
  .tqm-add-answer-btn,
  .tqm-remove-answer-btn {
    transition: none;
  }

  .tqm-add-answer-btn:hover,
  .tqm-remove-answer-btn:hover {
    transform: none;
  }
}
</style>