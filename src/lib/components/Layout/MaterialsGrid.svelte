<script lang="ts">
  import { onMount } from 'svelte';
  
  import { materialsStore } from "../../stores/materialsStore";
  import { filteredMaterials } from "../../stores/materialsStore";
  import {
    isStudyMaterial,
    isReviewMaterial,
  } from "../../stores/materialsStore";

  import {
    type StudyMaterial,
    type ReviewMaterial,
  } from "../../stores/materialsStore";

  import { MaterialsManager } from "../../logic/materialsManager";
  const manager = new MaterialsManager();

  import StudyMaterialCard from "../StudyMaterials/StudyMaterialCard.svelte";
  import ReviewMaterialCard from "../ReviewMaterial/ReviewMaterialCard.svelte";
  import CreateMenu from "./CreateMenu.svelte";

  export let materialType: "study" | "review";
  export let zoomLevel = 1;
  export let columns: number;
  export let gridGap: string;
  export let showCreateMenu = false;
  export let createMenuButton: HTMLElement | null = null;

  export let onDeleteMaterial = (identifier: string) => {};
  export let onReviewMaterial = (material: ReviewMaterial) => {};
  export let onEditMaterial = (material: ReviewMaterial) => {};
  export let onEditTest = (material: ReviewMaterial) => {};
  export let onAddMaterial = () => {};
  export let onToggleCreateMenu = () => {};
  export let onCreateFlashcardDeck = () => {};
  export let onCreateTest = () => {};

  let containerElement: HTMLElement;
  let selectionPromises = new Map<string, Promise<boolean>>();

  function isMaterialSelected(material: StudyMaterial | ReviewMaterial) {
    const id = manager.getMaterialIdentifier(material);
    if (!selectionPromises.has(id)) {
      selectionPromises.set(id, materialsStore.isMaterialSelected(material));
    }
    return selectionPromises.get(id) as Promise<boolean>;
  }

  function toggleMaterialSelection(material: StudyMaterial | ReviewMaterial) {
    materialsStore.toggleMaterialSelection(material);
    const id = manager.getMaterialIdentifier(material);
    selectionPromises.delete(id);
  }

  function handleDeleteMaterial(identifier: string) {
    onDeleteMaterial(identifier);
  }

  function handleReviewMaterial(material: ReviewMaterial) {
    onReviewMaterial(material);
  }

  function handleEditMaterial(review_material) {
    onEditMaterial(review_material);
  }

  function handleEditTest(review_material) {
    onEditTest(review_material);
  }

  function triggerFileSelect() {
    onAddMaterial();
  }

  function toggleCreateMenu() {
    onToggleCreateMenu();
  }

  function handleCreateFlashcardDeck() {
    onCreateFlashcardDeck();
  }

  function handleCreateTest() {
    onCreateTest();
  }

  $: if ($materialsStore.selectionMode || $materialsStore.selectedMaterials) {
    selectionPromises.clear();
  }

  onMount(() => {
  console.log('MaterialsGrid MOUNTED');
  return () => {
    console.log('MaterialsGrid DESTROYED');
  };
});
</script>

<div
  class="materials-grid"
  bind:this={containerElement}
  style="grid-template-columns: repeat({columns}, minmax(0, 1fr)); gap: {gridGap};"
>
{#each $materialsStore.materials as material, index (manager.getMaterialIdentifier(material))}    
{@const isSelected = $materialsStore.selectionMode && $materialsStore.selectedMaterials.some(m => manager.getMaterialIdentifier(m) === manager.getMaterialIdentifier(material))}
    <div
      class="material-wrapper"
      class:selectable={$materialsStore.selectionMode}
      class:selected={$materialsStore.selectionMode && isSelected}
      role="button"
      tabindex={$materialsStore.selectionMode ? 0 : -1}
      on:click={() =>
        $materialsStore.selectionMode && toggleMaterialSelection(material)}
      on:keydown={(e) => {
        if (
          $materialsStore.selectionMode &&
          (e.key === "Enter" || e.key === " ")
        ) {
          e.preventDefault();
          toggleMaterialSelection(material);
        }
      }}
    >
      {#if materialType === "study" && isStudyMaterial(material)}
        <StudyMaterialCard
          {material}
          {zoomLevel}
          onDelete={(id) => handleDeleteMaterial(id)}
          selectionMode={$materialsStore.selectionMode}
          selected={isSelected}
        />
      {:else if materialType === "review" && isReviewMaterial(material)}
        <ReviewMaterialCard
          {material}
          {zoomLevel}
          onDelete={(id) => handleDeleteMaterial(id)}
          onReview={(materialOrEvent) => {
            const material = materialOrEvent.detail || materialOrEvent;
            handleReviewMaterial(material);
          }}
          onEdit={handleEditMaterial}
          onEditTest={handleEditTest}
        />
      {/if}

      {#if $materialsStore.selectionMode}
        <div class="selection-indicator">
          <div class="checkbox">
            {#if isSelected}
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
              >
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/each}

  {#if !$materialsStore.selectionMode}
    {#if materialType === "study"}
      <button class="phantom-card" on:click={triggerFileSelect} type="button">
        <div class="phantom-content">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>Add Study Material</span>
        </div>
      </button>
    {:else}
      <div
        class="phantom-card create-button-container"
        bind:this={createMenuButton}
      >
        <button
          class="phantom-content"
          on:click={toggleCreateMenu}
          type="button"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="32"
            height="32"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          <span>Create</span>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            style="margin-left: 4px; transition: transform 0.2s; transform: rotate({showCreateMenu
              ? 180
              : 0}deg);"
          >
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>

        <CreateMenu
          show={showCreateMenu}
          position="above"
          onCreateFlashcardDeck={handleCreateFlashcardDeck}
          onCreateTest={handleCreateTest}
        />
      </div>
    {/if}
  {/if}
</div>

<style>
  .materials-grid {
    display: grid;
    gap: 2rem;
    padding: 1rem 2rem;
    width: 100%;
    max-width: 100%;
  }

  .material-wrapper {
    position: relative;
    width: 100%;
  }

  .material-wrapper.selectable {
    cursor: pointer;
  }

  .material-wrapper.selected {
    outline: 3px solid var(--accent);
    border-radius: var(--border-radius);
    box-shadow: 0 0 0 5px rgba(var(--accent-rgb), 0.2);
  }

  .checkbox {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid var(--accent);
    background: var(--surface);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .material-wrapper.selected .checkbox {
    background: var(--accent);
    color: var(--surface);
  }

  .selection-indicator {
    position: absolute;
    top: 10px;
    left: 10px;
    z-index: 10;
  }

  .phantom-card {
    aspect-ratio: 3/4;
    width: 100%;
    background: transparent;
    border: 2px dashed var(--border);
    border-radius: var(--border-radius);
    cursor: pointer;
    transition: all var(--transition-speed) cubic-bezier(0.34, 1.56, 0.64, 1);
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
  }

  .phantom-card:focus {
    outline: none;
  }

  .phantom-card:hover {
    border-color: var(--text);
    transform: scale(1.05) rotate(2deg);
    color: var(--text);
  }

.phantom-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    transition: all var(--transition-speed) ease;
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    padding: 16px;
    width: 100%;
    height: 100%;
  }

  .phantom-content span {
    font-size: 1rem;
    font-family: var(--font-body);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .phantom-card.create-button-container {
    position: relative;
  }
</style>