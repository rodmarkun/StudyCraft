<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { flip } from "svelte/animate";
  import "../../styles/variables.css";
  import "../../styles/ReviewMaterial/flashcardDeckCreator.css";

  // Components
  import Button from "../Shared/Button.svelte";
  import FlashcardModal from "./FlashcardModal.svelte";

  // Logic & Types
  import {
    FlashcardDeckCreatorLogic,
    type StudyMaterial,
    type DeckFormData,
  } from "../../logic/ReviewMaterial/flashcardDeckCreator";

  // Stores
  import { llmStore } from "../../stores/llmStore";
  import { materialsStore } from "../../stores/materialsStore";

  // Icons
  import { Bot as BotIcon, Plus as PlusIcon, File as FileIcon } from "lucide-svelte";

  // Constants
  import { LANGUAGE_OPTIONS } from "../../logic/Constants/creatorConstants";

  // Props
  export let initialSelectedMaterials: StudyMaterial[] = [];

  // Callbacks
  export let onClose = () => {};
  export let onCreated = (deck: any) => {};
  export let onShowMaterialsSelector = (details: any) => {};

  // Logic instance
  const logic = new FlashcardDeckCreatorLogic();

  // Form state
  let formData: DeckFormData = {
    name: "",
    language: "English",
    tags: [],
    selectedMaterials: [...initialSelectedMaterials],
    concept: "",
  };

  // State
  let isModalOpen = false;
  let updateInterval: number | null = null;
  let currentStep = logic.getCurrentStep;
  let cards = logic.getCards;
  let isCreating = logic.getIsCreating;
  let isGenerating = logic.getIsGenerating;
  let error = logic.getError;
  let generationStatus = logic.getGenerationStatus;
  let isFormDataInitialized = false;

  // Reactivity
  $: llmStatus = $llmStore;
  $: canUseAI = llmStatus.hasConfiguredProvider && !llmStatus.isLoading;
  $: totalCardsToGenerate = logic.getTotalCardsToGenerate(
    formData.selectedMaterials,
  );

  $: currentStepComplete = (() => {
    if (currentStep === 1) {
      return formData.name.trim() !== "";
    } else if (currentStep === 2) {
      return (
        cards.length > 0 &&
        cards.every(
          (card) => card.front.trim() !== "" && card.back.trim() !== "",
        )
      );
    }
    return false;
  })();

  $: isValid =
    cards.length > 0 &&
    cards.every(
      (card) => card.front.trim() !== "" && card.back.trim() !== "",
    ) &&
    formData.name.trim() !== "";

  $: stepsComplete = {
    1: formData.name.trim() !== "",
    2:
      cards.length > 0 &&
      cards.every(
        (card) => card.front.trim() !== "" && card.back.trim() !== "",
      ),
  };

  $: if (isFormDataInitialized && formData) {
    const dataToStore = {
      name: formData.name,
      language: formData.language,
      concept: formData.concept,
    };
    sessionStorage.setItem(
      "flashcard-creator-form",
      JSON.stringify(dataToStore),
    );
  }

  // Update interval management
  $: if (isGenerating) {
    if (!updateInterval) {
      updateInterval = setInterval(() => {
        currentStep = logic.getCurrentStep;
        cards = logic.getCards;
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

  // Navigation functions
  function handleClose() {
    sessionStorage.removeItem("flashcard-creator-form");
    onClose();
  }

  async function handleNextStep() {
    console.log("handleNextStep called, currentStep:", currentStep);

    if (currentStep === 1 && formData.selectedMaterials.length > 0) {
      if (!canUseAI) {
        error =
          "No AI provider configured. Skipping automatic generation - you can create flashcards manually in the next step.";
        logic.moveToNextStep(formData);
        currentStep = logic.getCurrentStep;
        return;
      }

      isGenerating = true;
      try {
        await logic.moveToNextStep(formData);
        currentStep = logic.getCurrentStep;
        cards = logic.getCards;
        isGenerating = logic.getIsGenerating;
        error = logic.getError;
        generationStatus = logic.getGenerationStatus;
      } catch (err) {
        console.error("Generation failed:", err);
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

  // Card management functions
  function handleAddCard() {
    logic.addCard();
    cards = logic.getCards;
  }

  function handleRemoveCard(index: number) {
    logic.removeCard(index);
    cards = logic.getCards;
  }

  function handleCardChange(
    index: number,
    side: "front" | "back",
    value: string,
  ) {
    logic.updateCard(index, side, value);
    cards = logic.getCards;
  }

  // Modal functions
  function handleOpenModal() {
    isModalOpen = true;
  }

  function handleCloseModal() {
    isModalOpen = false;
  }

  function handleModalAddCard(card: CardData) {
    const { front, back } = card;
    logic.addCardWithContent(front, back);
    cards = logic.getCards;
  }

  // Material management functions
  function handleShowMaterialSelector() {
    console.log("FlashcardCreator: handleShowMaterialSelector called");
    console.log("About to call onShowMaterialsSelector with:", {
      fromFlashcardCreator: true,
      currentSelectedMaterials: formData.selectedMaterials || [],
    });
    onShowMaterialsSelector({
      fromFlashcardCreator: true,
      currentSelectedMaterials: formData.selectedMaterials || [],
    });
  }

  function handleRemoveMaterial(material: StudyMaterial) {
    formData = {
      ...formData,
      selectedMaterials: logic.removeMaterial(
        material,
        formData.selectedMaterials,
      ),
    };
  }

  function handleMaterialCardCountChange(materialId: string, count: number) {
    formData = {
      ...formData,
      selectedMaterials: logic.updateMaterialCardCount(
        materialId,
        count,
        formData.selectedMaterials,
      ),
    };
  }

  // Deck creation function
  async function handleCreateDeck() {
  try {
    // Generate UUID and add temp material to store first
    const deckId = await materialsStore.addReviewMaterial({
      name: formData.name,
      type: 'flashcard_deck',
      tags: formData.tags
    });
    
    // Now create the deck with the pre-generated ID
    const newDeck = await logic.createFlashcardDeck(formData, deckId);
    sessionStorage.removeItem("flashcard-creator-form");
    onCreated(newDeck);
    onClose();
  } catch (err) {
    error = logic.getError;
  }
}

  // Lifecycle functions
  onMount(() => {
    const stored = sessionStorage.getItem("flashcard-creator-form");
    if (stored) {
      try {
        const parsed = JSON.parse(stored);
        formData = {
          ...formData,
          name: parsed.name || formData.name,
          language: parsed.language || formData.language,
          concept: parsed.concept || formData.concept,
        };
      } catch (e) {
        console.warn("Failed to restore form data:", e);
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

  // Exported functions
  export function handleMaterialsSelected(materials: StudyMaterial[]) {
    formData = {
      ...formData,
      selectedMaterials: materials.map((material) => ({
        ...material,
        cardsToGenerate: material.cardsToGenerate || 5,
      })),
    };
  }
</script>

<div class="creator-container" transition:fade={{ duration: 200 }}>
  <div class="creator-header">
    <h3>Create Flashcard Deck</h3>
    <button class="close-button" on:click={handleClose}>×</button>
  </div>

  <div class="creator-progress">
    <div
      class="progress-step"
      class:active={currentStep >= 1}
      class:complete={stepsComplete[1]}
    >
      <div class="step-number">1</div>
      <div class="step-label">Info & Materials</div>
    </div>
    <div class="progress-line" class:active={currentStep >= 2}></div>
    <div
      class="progress-step"
      class:active={currentStep >= 2}
      class:complete={stepsComplete[2]}
    >
      <div class="step-number">2</div>
      <div class="step-label">Add & Edit Cards</div>
    </div>
    <div class="progress-line" class:active={currentStep >= 3}></div>
    <div class="progress-step" class:active={currentStep >= 3}>
      <div class="step-number">3</div>
      <div class="step-label">Review Deck</div>
    </div>
  </div>

  {#if error}
    <div class="error-message" transition:fade>
      {error}
    </div>
  {/if}

  {#if isGenerating}
    <div class="fdc-generation-status" transition:fade>
      <div class="loading-spinner"></div>
      <div class="fdc-status-text">
        <strong>Generating Flashcards...</strong>
        <p>{generationStatus || "Processing your study materials..."}</p>
      </div>
    </div>
  {/if}

  <div class="creator-content">
    {#if currentStep === 1}
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
                placeholder="Enter deck name..."
                disabled={isGenerating}
                class="input"
                on:keydown={(e) => {
                  e.stopPropagation();
                }}
              />
            </div>

            <div class="fdc-form-field">
              <label for="language">Language</label>
              <select
                id="language"
                bind:value={formData.language}
                class="fdc-language-select"
                disabled={isGenerating}
              >
                {#each LANGUAGE_OPTIONS as option}
                  <option value={option.value}>{option.label}</option>
                {/each}
              </select>
            </div>
          </div>
        </div>

        <div class="fdc-generation-section">
          <div class="fdc-section-header">
            <h4>AI Generation (Optional)</h4>
            <p>Generate flashcards automatically from your study materials</p>
            {#if !canUseAI}
              <p class="fdc-ai-warning">
                ⚠️ No AI provider configured. Configure one in Settings to
                enable automatic generation, or create cards manually in the
                next step.
              </p>
            {/if}
          </div>

          <div
            class="fdc-select-materials-wrapper"
            title={!canUseAI
              ? "Configure an AI provider in Settings to use automatic generation"
              : ""}
          >
            <Button
              variant="secondary"
              text="Select Study Materials"
              onClick={handleShowMaterialSelector}
              icon={BotIcon}
              disabled={isGenerating || !canUseAI}
            ></Button>
          </div>

          {#if formData.selectedMaterials.length > 0}
            <div class="fdc-selected-materials-compact">
              <div class="fdc-materials-header">
                <span class="fdc-materials-count"
                  >{formData.selectedMaterials.length} materials selected</span
                >
                <span class="fdc-total-cards"
                  >Total: {totalCardsToGenerate} cards</span
                >
              </div>

              <div class="fdc-materials-grid">
                {#each formData.selectedMaterials as material}
                  <div class="fdc-material-chip">
                    <div class="fdc-material-info">
                      <FileIcon size={14}/>
                      <span class="fdc-material-name">
                        {material.name ||
                          material.display_name}
                      </span>
                    </div>

                    <div class="fdc-material-controls">
                      <input
                        type="number"
                        value={material.cardsToGenerate || 5}
                        on:input={(e) =>
                          handleMaterialCardCountChange(
                            material.id,
                            parseInt((e.target as HTMLInputElement).value),
                          )}
                        min="1"
                        max="20"
                        class="fdc-card-count-input"
                        title="Cards to generate"
                        disabled={isGenerating || !canUseAI}
                      />
                      <button
                        aria-label="Remove Material"
                        class="fdc-remove-material-btn"
                        on:click={() => handleRemoveMaterial(material)}
                        title="Remove material"
                        disabled={isGenerating}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                        >
                          <line x1="18" y1="6" x2="6" y2="18"></line>
                          <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                      </button>
                    </div>
                  </div>
                {/each}
              </div>

              <div class="fdc-concept-field">
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
                <p class="fdc-concept-help">
                  Specify the particular concept or topic you want the
                  flashcards to focus on from the selected materials.
                </p>
              </div>
            </div>
          {:else}
            <div class="fdc-no-materials">
              <p>
                No materials selected. Select materials and specify a concept to
                enable AI generation, or create flashcards manually in the next
                step.
              </p>
            </div>
          {/if}
        </div>
      </div>
    {:else if currentStep === 2}
      <div class="fdc-step-content" transition:fade>
        <div class="fdc-cards-header">
          <h4>Edit Your Flashcards</h4>
          <div class="fdc-cards-header-actions">
            <button
              class="fdc-add-card-modal-button"
              on:click={handleOpenModal}
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="12" y1="9" x2="12" y2="15"></line>
                <line x1="9" y1="12" x2="15" y2="12"></line>
              </svg>
              Add Card
            </button>
            <span class="fdc-cards-count"
              >{cards.length} card{cards.length !== 1 ? "s" : ""}</span
            >
          </div>
        </div>

        {#if cards.length === 0}
          <div class="fdc-no-cards">
            <div class="fdc-no-cards-content">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="48"
                height="48"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="12" y1="9" x2="12" y2="15"></line>
                <line x1="9" y1="12" x2="15" y2="12"></line>
              </svg>
              <p>Click "Add Card" above to create your first flashcard</p>
            </div>
          </div>
        {:else}
          <div class="fdc-cards-list">
            {#each cards as card, index (index)}
              <div
                class="fdc-card-item"
                animate:flip={{ duration: 300 }}
                transition:slide|local
              >
                <div class="fdc-card-header">
                  <span>Card {index + 1}</span>
                  <button
                    class="fdc-remove-card"
                    on:click={() => handleRemoveCard(index)}
                  >
                    Remove Card
                  </button>
                </div>

                <div class="fdc-card-content">
                  <div class="fdc-card-side">
                    <p>Front (Question)</p>
                    <textarea
                      value={card.front}
                      on:input={(e) =>
                        handleCardChange(
                          index,
                          "front",
                          (e.target as HTMLTextAreaElement).value,
                        )}
                      placeholder="Enter the question or term..."
                    ></textarea>
                  </div>
                  <div class="fdc-card-side">
                    <p>Back (Answer)</p>
                    <textarea
                      value={card.back}
                      on:input={(e) =>
                        handleCardChange(
                          index,
                          "back",
                          (e.target as HTMLTextAreaElement).value,
                        )}
                      placeholder="Enter the answer or definition..."
                    ></textarea>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else if currentStep === 3}
      <div class="fdc-step-content" transition:fade>
        <div class="fdc-review-section">
          <h4>Deck Information</h4>
          <div class="fdc-review-item">
            <span class="fdc-review-label">Name</span>
            <span class="fdc-review-value">{formData.name}</span>
          </div>
          <div class="fdc-review-item">
            <span class="fdc-review-label">Language</span>
            <span class="fdc-review-value">{formData.language}</span>
          </div>
          {#if formData.concept.trim()}
            <div class="fdc-review-item">
              <span class="fdc-review-label">Focus Concept</span>
              <span class="fdc-review-value fdc-concept-value"
                >{formData.concept}</span
              >
            </div>
          {/if}
          <div class="fdc-review-item">
            <span class="fdc-review-label">Cards</span>
            <span class="fdc-review-value"
              >{cards.length} flashcard{cards.length !== 1 ? "s" : ""}</span
            >
          </div>

          {#if formData.selectedMaterials.length > 0}
            <div class="fdc-review-item">
              <span class="fdc-review-label">Materials</span>
              <div class="fdc-review-materials">
                {#each formData.selectedMaterials as material}
                  <span class="fdc-material-item"
                    >{material.name ||
                      material.display_name}</span
                  >
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <div class="fdc-review-cards-preview">
          <h4>Cards Preview</h4>
          <div class="fdc-cards-preview-list">
            {#each cards.slice(0, 3) as card, index}
              <div class="fdc-preview-card">
                <div class="fdc-preview-card-header">
                  Card {index + 1}
                </div>
                <div class="fdc-preview-card-content">
                  <div class="fdc-preview-front">
                    <span class="fdc-preview-label">Front</span>
                    <div class="fdc-preview-content">
                      {card.front.length > 80
                        ? card.front.substring(0, 80) + "..."
                        : card.front}
                    </div>
                  </div>
                  <div class="fdc-preview-back">
                    <span class="fdc-preview-label">Back</span>
                    <div class="fdc-preview-content">
                      {card.back.length > 80
                        ? card.back.substring(0, 80) + "..."
                        : card.back}
                    </div>
                  </div>
                </div>
              </div>
            {/each}
            {#if cards.length > 3}
              <div class="fdc-more-cards">
                +{cards.length - 3} more card{cards.length - 3 !== 1 ? "s" : ""}
                not shown
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
      {@const canProceed =
        currentStep === 1
          ? formData.name.trim() !== ""
          : cards.length > 0 &&
            cards.every(
              (card) => card.front.trim() !== "" && card.back.trim() !== "",
            )}
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
        text={isCreating ? "Creating..." : "Create Deck"}
        changed={isCreating}
        disabled={!isValid || isCreating}
        onClick={handleCreateDeck}
      />
    {/if}
  </div>
</div>

<FlashcardModal
  bind:isOpen={isModalOpen}
  onAddCard={handleModalAddCard}
  onClose={handleCloseModal}
/>