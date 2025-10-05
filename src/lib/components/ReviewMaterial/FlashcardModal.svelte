<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { fade } from 'svelte/transition';
  import "../../styles/variables.css";

  // Components
  import Button from '../Shared/Button.svelte';

  // Props
  export let isOpen = false;

  // Callbacks
  export let onAddCard = (cardData: { front: string; back: string }) => {};
  export let onClose = () => {};

  // State
  let frontTextarea: HTMLTextAreaElement;
  let backTextarea: HTMLTextAreaElement;
  let modalContainer: HTMLDivElement;
  let front = '';
  let back = '';

  // Reactive statements
  $: canAdd = front.trim() !== '' && back.trim() !== '';

  $: if (isOpen && frontTextarea) {
    tick().then(() => {
      frontTextarea.focus();
    });
  }

  // Functions
  function handleKeydown(event: KeyboardEvent) {
    if (event.ctrlKey && event.key === 'Enter') {
      event.preventDefault();
      handleAddCard();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      handleClose();
    } else if (event.key === 'Tab') {
      const target = event.target as HTMLElement;
      if (target === frontTextarea && !event.shiftKey) {
        event.preventDefault();
        backTextarea.focus();
      } else if (target === backTextarea && event.shiftKey) {
        event.preventDefault();
        frontTextarea.focus();
      }
    }
  }

  function handleAddCard() {
    if (front.trim() === '' || back.trim() === '') {
      return;
    }
    
    onAddCard({
      front: front.trim(),
      back: back.trim()
    });
    
    front = '';
    back = '';
    frontTextarea.focus();
  }

  function handleClose() {
    front = '';
    back = '';
    onClose();
  }

  function handleOutsideClick(event: MouseEvent) {
    if (event.target === modalContainer) {
      handleClose();
    }
  }

  // Lifecycle
  onMount(() => {
    if (isOpen && frontTextarea) {
      frontTextarea.focus();
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div
    class="fm-overlay"
    transition:fade={{ duration: 200 }}
    bind:this={modalContainer}
    on:click={handleOutsideClick}
    on:keydown={(e) => e.key === 'Escape' && handleClose()}
    role="dialog"
    aria-modal="true"
    aria-labelledby="fm-title"
    tabindex="-1"
  >
    <div class="fm-modal" on:click|stopPropagation>
      <div class="fm-header">
        <h3 id="fm-title">Add Flashcard</h3>
        <button class="fm-close" on:click={handleClose} aria-label="Close modal">×</button>
      </div>
      <div class="fm-content">
        <div class="fm-field">
          <label for="fm-front">Front (Question)</label>
          <textarea
            id="fm-front"
            bind:this={frontTextarea}
            bind:value={front}
            placeholder="Enter the question or term..."
            rows="4"
            on:keydown={handleKeydown}
          ></textarea>
        </div>
        <div class="fm-field">
          <label for="fm-back">Back (Answer)</label>
          <textarea
            id="fm-back"
            bind:this={backTextarea}
            bind:value={back}
            placeholder="Enter the answer or definition..."
            rows="4"
            on:keydown={handleKeydown}
          ></textarea>
        </div>
      </div>
      <div class="fm-footer">
        <div class="fm-shortcuts">
          <span class="fm-shortcut">
            <kbd>Tab</kbd> to switch fields
          </span>
          <span class="fm-shortcut">
            <kbd>Ctrl</kbd> + <kbd>Enter</kbd> to add
          </span>
          <span class="fm-shortcut">
            <kbd>Esc</kbd> to close
          </span>
        </div>
       
        <div class="fm-actions">
          <Button variant="secondary" onClick={handleClose} text="Cancel"/>
          <Button
            variant="primary"
            onClick={handleAddCard}
            disabled={!canAdd}
            text="Add Card"
          />
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
.fm-overlay {
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

.fm-modal {
  background: var(--surface);
  border-radius: var(--border-radius);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.3);
  border: 1px solid var(--border);
  max-width: 600px;
  width: 100%;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.fm-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md);
  border-bottom: 1px solid var(--border);
  background: var(--background);
}

.fm-header h3 {
  margin: 0;
  font-family: var(--font-heading);
  font-weight: 600;
  font-size: 1.3rem;
  color: var(--text);
}

.fm-close {
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

.fm-close:hover {
  color: var(--error);
}

.fm-content {
  flex: 1;
  padding: var(--space-md);
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
  overflow-y: auto;
}

.fm-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.fm-field label {
  font-weight: 600;
  color: var(--text);
  font-size: 0.95rem;
}

.fm-field textarea {
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

.fm-field textarea:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 15%, transparent);
}

.fm-field textarea::placeholder {
  color: var(--text-secondary);
  font-style: italic;
}

.fm-footer {
  padding: var(--space-md);
  border-top: 1px solid var(--border);
  background: var(--background);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.fm-shortcuts {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-md);
  justify-content: center;
}

.fm-shortcut {
  color: var(--text-secondary);
  font-size: 0.85rem;
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.fm-shortcut kbd {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.8rem;
  font-family: var(--font-mono, monospace);
  color: var(--text);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

.fm-actions {
  display: flex;
  gap: var(--space-md);
  justify-content: flex-end;
}

@media (max-width: 768px) {
  .fm-overlay {
    padding: var(--space-sm);
  }

  .fm-modal {
    max-height: 95vh;
  }

  .fm-header {
    padding: var(--space-md);
  }

  .fm-content {
    padding: var(--space-md);
    gap: var(--space-md);
  }

  .fm-footer {
    padding: var(--space-md);
  }

  .fm-shortcuts {
    flex-direction: column;
    gap: var(--space-xs);
    text-align: center;
  }

  .fm-actions {
    flex-direction: column;
    gap: var(--space-sm);
  }

  .fm-field textarea {
    min-height: 80px;
  }
}

@media (max-width: 480px) {
  .fm-overlay {
    padding: var(--space-xs);
  }

  .fm-header {
    padding: var(--space-sm);
  }

  .fm-content {
    padding: var(--space-sm);
  }

  .fm-footer {
    padding: var(--space-sm);
  }

  .fm-shortcut {
    font-size: 0.8rem;
  }

  .fm-shortcut kbd {
    font-size: 0.75rem;
    padding: 1px 4px;
  }
}

/* ===== Focus and Accessibility ===== */
.fm-close:focus,
.fm-field textarea:focus {
  outline: 2px solid var(--accent);
  outline-offset: 2px;
}

/* ===== Dark Mode Support ===== */
@media (prefers-color-scheme: dark) {
  .fm-shortcut kbd {
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
  }
}

@media (prefers-reduced-motion: reduce) {
  .fm-close {
    transition: none;
  }

  .fm-field textarea {
    transition: none;
  }
}
</style>