<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fade, slide } from 'svelte/transition';
  import { flip } from 'svelte/animate';
  import "../../styles/variables.css";
  import "../../styles/ReviewMaterial/flashcardDeckCreator.css";

  // Components
  import Button from '../Shared/Button.svelte';
  import FlashcardModal from './FlashcardModal.svelte';

  // Logic & Types
  import { FlashcardDeckEditorLogic, type ReviewMaterial, type DeckEditFormData } from '../../logic/ReviewMaterial/flashcardDeckEditor';
  import type { StudyMaterial } from '../../logic/ReviewMaterial/flashcardDeckCreator';

  // Props
  export let deck: ReviewMaterial;
  export let isOpen = false;

  // Callbacks
  export let onClose = () => {};
  export let onUpdated = (details: any) => {};

  // Logic instance
  const logic = new FlashcardDeckEditorLogic();

  // Form state
  let formData: DeckEditFormData = {
    deckId: deck.id,
    name: deck.name, 
    language: 'English',
    selectedMaterials: [],
    concept: ''};

  // State
  let currentStep = 2;
  let isModalOpen = false;
  let updateInterval: number | null = null;
  let cards = [];
  let isLoading = false;
  let isSaving = false;
  let error = '';
  let hasUnsavedChanges = false;

  // Reactive statements
  $: stepsComplete = {
    2: cards.length > 0 && cards.every(card => 
      card.front.trim() !== '' && card.back.trim() !== ''
    )
  };

  $: isValid = cards.length > 0 && cards.every(card => 
    card.front.trim() !== '' && card.back.trim() !== ''
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
    cards = logic.getCards;
    hasUnsavedChanges = logic.getHasUnsavedChanges;
  }

  // Functions
  function updateReactiveVars() {
    currentStep = logic.currentStep || 2; 
    cards = logic.getCards;
    isLoading = logic.getIsLoading;
    isSaving = logic.getIsSaving;
    error = logic.getError;
    hasUnsavedChanges = logic.getHasUnsavedChanges;
  }

  async function loadExistingCards() {
    try {
      await logic.loadExistingCards(deck.id);
      updateReactiveVars();
    } catch (err) {
      console.error('Failed to load existing cards:', err);
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

  function handleRemoveCard(index: number) {
    if (confirm('Are you sure you want to delete this card?')) {
      logic.removeCard(index);
      updateReactiveVars();
    }
  }

  function handleCardChange(index: number, side: 'front' | 'back', value: string) {
    logic.updateCard(index, side, value);
    updateReactiveVars();
  }

  function handleNameChange() {
    logic.updateDeckName(formData.name);
    updateReactiveVars();
  }

  function handleOpenModal() {
    isModalOpen = true;
  }

  function handleCloseModal() {
    isModalOpen = false;
  }

  function handleModalAddCard(cardData: { front: string; back: string }) {
    const { front, back } = cardData;
    logic.addCardWithContent(front, back);
    updateReactiveVars();
  }

  async function handleSaveChanges() {
    try {
      await logic.saveChanges(formData);
      
      onUpdated({
        deckId: deck.id,
        name: formData.name,
        cardsCount: cards.length
      });
      
      onClose();
    } catch (err) {
      updateReactiveVars();
    }
  }

  onMount(async () => {
    if (deck && deck.id) {
      logic.currentStep = 2;
      await loadExistingCards();
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
        cardsToGenerate: material.cardsToGenerate || 5
      }))
    };
  }
</script>

{#if isOpen}
<div class="creator-container" transition:fade={{ duration: 200 }}>
  <div class="creator-header">
    <h3>Edit Flashcard Deck: {deck.name}</h3>
    <button class="close-button" on:click={handleClose}>×</button>
  </div>
  
  <div class="creator-progress">
    <div class="progress-step" class:active={currentStep >= 2} class:complete={stepsComplete[2]}>
      <div class="step-number">1</div>
      <div class="step-label">Edit Cards</div>
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
  
  {#if isLoading}
    <div class="fdc-generation-status" transition:fade>
      <div class="loading-spinner"></div>
      <div class="fdc-status-text">
        <strong>Loading Flashcards...</strong>
        <p>Retrieving existing cards from the database...</p>
      </div>
    </div>
  {/if}
  
  <div class="creator-content">
    {#if currentStep === 2}
      <div class="fdc-step-content" transition:fade>
        <div class="fdc-deck-info-section">
          <h4>Deck Information</h4>
          <div class="fdc-form-grid">
            <div class="fdc-form-field">
              <label for="deck-name">Deck Name *</label>
              <input 
                id="deck-name"
                type="text"
                bind:value={formData.name}
                on:input={handleNameChange}
                placeholder="Enter deck name..." 
                disabled={isSaving}
                class="input"
                on:keydown={(e) => {
                  e.stopPropagation();
                }}
              />
            </div>
          </div>
        </div>

        <div class="fdc-cards-header">
          <h4>Edit Your Flashcards</h4>
          <div class="fdc-cards-header-actions">
            <button class="fdc-add-card-modal-button" on:click={handleOpenModal}>
              <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="12" y1="9" x2="12" y2="15"></line>
                <line x1="9" y1="12" x2="15" y2="12"></line>
              </svg>
              Add Card
            </button>
              <span class="fdc-cards-count">{cards.filter(card => !card.isDeleted).length} card{cards.filter(card => !card.isDeleted).length !== 1 ? 's' : ''}</span>          </div>
        </div>
        
        <div class="fdc-cards-list">
          {#each cards.filter(card => !card.isDeleted) as card, index (card.tempId)}
            <div 
              class="fdc-card-item" 
              class:fdc-new-card={card.isNew}
              animate:flip={{ duration: 300 }} 
              transition:slide|local
            >
              <div class="fdc-card-header">
                <span>
                  Card {index + 1}
                  {#if card.isNew}
                    <span class="fdc-new-badge">New</span>
                  {/if}
                </span>
                <button class="fdc-remove-card" on:click={() => handleRemoveCard(index)}>
                  Remove Card
                </button>
              </div>
              
              <div class="fdc-card-content">
                <div class="fdc-card-side">
                  <p>Front (Question)</p>
                  <textarea 
                    value={card.front}
                    on:input={(e) => handleCardChange(index, 'front', (e.target as HTMLTextAreaElement).value)}
                    placeholder="Enter the question or term..."
                  ></textarea>
                </div>
                <div class="fdc-card-side">
                  <p>Back (Answer)</p>
                  <textarea 
                    value={card.back}
                    on:input={(e) => handleCardChange(index, 'back', (e.target as HTMLTextAreaElement).value)}
                    placeholder="Enter the answer or definition..."
                  ></textarea>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else if currentStep === 3}
      <div class="fdc-step-content" transition:fade>
        <div class="fdc-review-section">
          <h4>Review Changes</h4>
          <div class="fdc-review-item">
            <span class="fdc-review-label">Deck Name</span>
            <span class="fdc-review-value">{formData.name}</span>
          </div>
          <div class="fdc-review-item">
            <span class="fdc-review-label">Total Cards</span>
            <span class="fdc-review-value">{cards.filter(card => !card.isDeleted).length} flashcard{cards.filter(card => !card.isDeleted).length !== 1 ? 's' : ''}</span>
          </div>
          
          {#if cards.some(card => card.isNew)}
            <div class="fdc-review-item">
              <span class="fdc-review-label">New Cards</span>
              <span class="fdc-review-value fdc-new-cards-count">
                {cards.filter(card => card.isNew).length} new card{cards.filter(card => card.isNew).length !== 1 ? 's' : ''} added
              </span>
            </div>
          {/if}
        </div>
        
        <div class="fdc-review-cards-preview">
          <h4>Cards Preview</h4>
          <div class="fdc-cards-preview-list">
            {#each cards.slice(0, 3) as card, index}
              <div class="fdc-preview-card" class:fdc-new-card={card.isNew}>
                <div class="fdc-preview-card-header">
                  Card {index + 1}
                  {#if card.isNew}
                    <span class="fdc-new-badge">New</span>
                  {/if}
                </div>
                <div class="fdc-preview-card-content">
                  <div class="fdc-preview-front">
                    <span class="fdc-preview-label">Front</span>
                    <div class="fdc-preview-content">
                      {card.front.length > 80 ? card.front.substring(0, 80) + '...' : card.front}
                    </div>
                  </div>
                  <div class="fdc-preview-back">
                    <span class="fdc-preview-label">Back</span>
                    <div class="fdc-preview-content">
                      {card.back.length > 80 ? card.back.substring(0, 80) + '...' : card.back}
                    </div>
                  </div>
                </div>
              </div>
            {/each}
            {#if cards.length > 3}
              <div class="fdc-more-cards">
                +{cards.length - 3} more card{cards.length - 3 !== 1 ? 's' : ''} not shown
              </div>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
  
  <div class="creator-actions">
    {#if currentStep > 2}
      <Button variant="secondary" onClick={handlePreviousStep} disabled={isSaving} text="Back"/>
    {:else}
      <Button variant="secondary" onClick={handleClose} disabled={isSaving} text={hasUnsavedChanges ? 'Cancel' : 'Close'}/>
    {/if}
    
    {#if currentStep < 3}
      <Button 
        variant="primary" 
        text="Next"
        onClick={handleNextStep} 
        disabled={!stepsComplete[2] || isSaving}
      />
    {:else}
      <Button
        variant="primary" 
        text="Save Changes"
        onClick={handleSaveChanges} 
        disabled={!isValid || isSaving || !hasUnsavedChanges}
      />
    {/if}
  </div>
</div>

<FlashcardModal 
  bind:isOpen={isModalOpen}
  onAddCard={handleModalAddCard}
  onClose={handleCloseModal}
/>
{/if}