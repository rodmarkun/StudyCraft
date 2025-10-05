<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";

  import Button from "../Shared/Button.svelte";
  import MaterialsGrid from "./MaterialsGrid.svelte";
  import FlashcardDeckCreator from "../ReviewMaterial/FlashcardDeckCreator.svelte";
  import FlashcardDeckEditor from "../ReviewMaterial/FlashcardDeckEditor.svelte";
  import FlashcardReview from "../ReviewMaterial/FlashcardReview.svelte";
  import TestCreator from "../ReviewMaterial/TestCreator.svelte";
  import TestEditor from "../ReviewMaterial/TestEditor.svelte";
  import TestReview from "../ReviewMaterial/TestReview.svelte";
  import CreateMenu from "./CreateMenu.svelte";

  import { MaterialsManager } from "../../logic/materialsManager";

  import {
    materialsStore,
    type StudyMaterial,
    type ReviewMaterial,
    type MaterialType,
    type SortOption,
  } from "../../stores/materialsStore";
  import { tagsStore } from "../../stores/tags";

  export let displayMode = "visual";
  export let zoomLevel = 1;
  export let materialType = "study" as MaterialType;
  export let selectionMode = false;
  export let initialSelectedMaterials: (StudyMaterial | ReviewMaterial)[] = [];
  export let sortOption = "date_asc" as SortOption;
  export let searchTerm = "";

  export let onMaterialsSelected = (
    materials: (StudyMaterial | ReviewMaterial)[],
  ) => {};
  export let onSelectionCancelled = () => {};
  export let onShowMaterialsSelector = (details: any) => {};

  let tauriUnlisten;
  let tauriUnlistenResized;
  let dragDropUnlisten;
  let resizeTrigger = 0;
  const manager = new MaterialsManager();

  let containerElement: HTMLElement;
  let showFlashcardCreator = false;
  let showFlashcardDeckEditor = false;
  let showTestCreator = false;
  let showTestEditor = false;
  let showingReview = false;
  let currentReviewDeck: ReviewMaterial | null = null;
  let currentEditingDeck: ReviewMaterial | null = null;
  let currentEditingTest: ReviewMaterial | null = null;
  let flashcardCreatorSelectedMaterials = [...initialSelectedMaterials];
  let testCreatorSelectedMaterials: (StudyMaterial | ReviewMaterial)[] = [];
  let flashcardCreatorComponent: any = null;
  let flashcardDeckEditorComponent: any = null;
  let testCreatorComponent: any = null;

  let pendingMaterialsForFlashcardCreator: (StudyMaterial | ReviewMaterial)[] =
    [];
  let pendingMaterialsForTestCreator: (StudyMaterial | ReviewMaterial)[] = [];

  let showCreateMenu = false;
  let createMenuButton: HTMLElement;

  let currentReviewMaterial: ReviewMaterial | null = null;
  let reviewMaterialType: "flashcard_deck" | "test" | null = null;
  let selectionPromises = new Map<string, Promise<boolean>>();

  let isDragOver = false;

  $: if (materialType) {
    materialsStore.setMaterialType(materialType as MaterialType);
  }

  $: if (sortOption) {
    materialsStore.setSortOption(sortOption as SortOption);
  }

  $: if (searchTerm !== $materialsStore.searchTerm) {
    materialsStore.setSearchTerm(searchTerm);
  }

  $: if (selectionMode !== $materialsStore.selectionMode) {
    console.log(
      "Setting selection mode:",
      selectionMode,
      "with initial materials:",
      initialSelectedMaterials,
    );
    const materialsToPreselect = selectionMode ? initialSelectedMaterials : [];
    materialsStore.setSelectionMode(selectionMode, materialsToPreselect);
  }

  $: ({ columns, gridGap, gridStyle } = manager.calculateGridProperties(
    containerElement,
    zoomLevel,
  ));

  $: resizeTrigger,
    ({ columns, gridGap, gridStyle } = manager.calculateGridProperties(
      containerElement,
      zoomLevel,
    ));

  $: if (
    flashcardCreatorComponent &&
    pendingMaterialsForFlashcardCreator.length > 0
  ) {
    console.log(
      "FlashcardCreator component became available, delivering pending materials:",
      pendingMaterialsForFlashcardCreator,
    );
    if (
      typeof flashcardCreatorComponent.handleMaterialsSelected === "function"
    ) {
      flashcardCreatorComponent.handleMaterialsSelected(
        pendingMaterialsForFlashcardCreator,
      );
      pendingMaterialsForFlashcardCreator = [];
    }
  }

  $: if (testCreatorComponent && pendingMaterialsForTestCreator.length > 0) {
    console.log(
      "TestCreator component became available, delivering pending materials:",
      pendingMaterialsForTestCreator,
    );
    if (typeof testCreatorComponent.handleMaterialsSelected === "function") {
      testCreatorComponent.handleMaterialsSelected(
        pendingMaterialsForTestCreator,
      );
      pendingMaterialsForTestCreator = [];
    }
  }

  $: if (
    $materialsStore.selectionMode !== selectionMode ||
    $materialsStore.selectedMaterials
  ) {
    selectionPromises.clear();
  }

  function handleClickOutside(event: MouseEvent) {
    if (
      showCreateMenu &&
      createMenuButton &&
      !createMenuButton.contains(event.target as Node)
    ) {
      showCreateMenu = false;
    }
  }

  async function selectAndAddFile() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Study Materials',
          extensions: ['pdf', 'md', 'txt']
        }]
      });

      if (selected) {
        selected.forEach(async (material) => {
          await materialsStore.addStudyMaterialFromPath(material);
        });
      }
    } catch (error) {
      console.error('File selection failed:', error);
    }
  }

  function handleResize() {
    console.log("handleResize called - forcing grid recalculation");
    manager.handleResize();
    resizeTrigger++;
  }

  function triggerFileSelect() {
    console.log("triggerFileSelect called with materialType:", materialType);

    if (materialType === "study") {
      selectAndAddFile();
    } else {
      toggleCreateMenu();
    }
  }

  function toggleCreateMenu() {
    showCreateMenu = !showCreateMenu;
  }

  function handleCreateFlashcardDeck() {
    showCreateMenu = false;
    showFlashcardCreator = true;
    if (pendingMaterialsForFlashcardCreator.length > 0) {
      flashcardCreatorSelectedMaterials = [
        ...pendingMaterialsForFlashcardCreator,
      ];
      pendingMaterialsForFlashcardCreator = [];
    }
  }

  function handleCreateTest() {
    showCreateMenu = false;
    showTestCreator = true;
    if (pendingMaterialsForTestCreator.length > 0) {
      testCreatorSelectedMaterials = [...pendingMaterialsForTestCreator];
      pendingMaterialsForTestCreator = [];
    }
  }

  function closeCreators() {
    showFlashcardCreator = false;
    showFlashcardDeckEditor = false;
    showTestCreator = false;
    showTestEditor = false;
    currentEditingDeck = null;
    currentEditingTest = null;
    testCreatorSelectedMaterials = [];
    flashcardCreatorSelectedMaterials = [];
  }

  async function handleMaterialCreated(event: CustomEvent<ReviewMaterial>) {
    console.log("Material created:", event.detail);
    closeCreators();

    try {
      console.log("Current material type:", materialType);
      console.log(
        "Current materials count before refresh:",
        $materialsStore.materials.length,
      );

      if (materialType === "review") {
        await materialsStore.refreshReviewMaterials();
      } else {
        await materialsStore.initialize();
      }

      console.log(
        "Materials count after refresh:",
        $materialsStore.materials.length,
      );
      console.log("Materials refreshed after creation");
    } catch (error) {
      console.error("Failed to refresh materials after creation:", error);
    }
  }

  function handleEditMaterial(material: ReviewMaterial) {
    console.log("=== handleEditMaterial called ===");
    console.log("Material:", material);

    if (
      material?.type === "flashcard_deck" ||
      material?.review_material_type === "FlashcardDeck"
    ) {
      currentEditingDeck = material;
      showFlashcardDeckEditor = true;
    } else {
      console.log(
        "Edit functionality not yet implemented for material type:",
        material?.type,
      );
    }
  }

  function handleEditTest(material: ReviewMaterial) {
    console.log("=== handleEditTest called ===");
    console.log("Test material:", material);

    if (
      material?.type === "test" ||
      material?.review_material_type === "Test"
    ) {
      currentEditingTest = material;
      showTestEditor = true;
      console.log("Opening test editor for test:", material);
    } else {
      console.log(
        "Invalid material type for test editing:",
        material?.type || material?.review_material_type,
      );
    }
  }

  function handleDeckUpdated(details: any) {
    const { deckId, name, cardsCount } = details;
    console.log("Deck updated:", details);
    materialsStore.refreshReviewMaterials();
  }

  function handleTestUpdated(details: any) {
    const { testId, name, questionsCount } = details;
    console.log("Test updated:", details);
    materialsStore.refreshReviewMaterials();
  }

  async function handleFileSelect(event: Event) {
    console.log("handleFileSelect called");

    try {
      const inputElement = event.target as HTMLInputElement;
      const file = inputElement.files?.[0];
      if (file) {
        console.log("Selected file:", file.name, file.type, file.size);
        await materialsStore.addStudyMaterial(file);
        console.log("File added successfully");
      }
    } catch (error) {
      console.error("Error in handleFileSelect:", error);
      alert("Failed to add study material: " + error);
    }
  }

  async function handleFileDrop(filePaths: string[]) {
    console.log("handleFileDrop called with paths:", filePaths);

    if (materialType !== "study") {
      console.log("File drop only allowed for study materials");
      return;
    }

    try {
      await Promise.all(
        filePaths.map(async (filePath) => {
          console.log("Processing dropped file:", filePath);
          await materialsStore.addStudyMaterialFromPath(filePath);
          console.log("Dropped file added successfully");
        }),
      );
    } catch (error) {
      console.error("Error processing dropped files:", error);
      alert("Failed to add some study materials: " + error);
    }
  }

  async function handleDeleteMaterial(identifier: string) {
    try {
      const material = $materialsStore.materials.find(
        (m) => manager.getMaterialIdentifier(m) === identifier,
      );

      if (material) {
        await materialsStore.deleteMaterial(material);
      }
    } catch (error) {
      alert(`Failed to delete material: ${error}`);
    }
  }

  function handleReviewMaterial(material: ReviewMaterial) {
    currentReviewMaterial = material;
    reviewMaterialType =
      material.type === "flashcard_deck" ? "flashcard_deck" : "test";
    showingReview = true;
  }

  async function handleReviewCompleted(data: any) {
    console.log("Review completed for deck:", data);

    try {
      await materialsStore.refreshReviewMaterials();
      console.log("Materials refreshed after review completion");
    } catch (error) {
      console.error("Failed to refresh materials after review:", error);
    }
  }

  function handleCloseReview() {
    showingReview = false;
    currentReviewMaterial = null;
    reviewMaterialType = null;
    currentReviewDeck = null;
  }

  function confirmMaterialSelection() {
    const selectedMaterials = $materialsStore.selectedMaterials;

    console.log("Confirming material selection:", selectedMaterials);
    onMaterialsSelected(selectedMaterials);

    materialsStore.setSelectionMode(false);
  }

  function cancelMaterialSelection() {
    materialsStore.setSelectionMode(false);
    onSelectionCancelled();
  }

  function handleShowMaterialsSelector(detail: any) {
    console.log("MaterialsBoard: Show materials selector requested:", detail);

    if (detail.fromFlashcardCreator) {
      onShowMaterialsSelector({
        fromFlashcardCreator: true,
        currentSelectedMaterials: detail.currentSelectedMaterials || [],
      });
    } else if (detail.fromTestCreator) {
      console.log("Dispatching showMaterialsSelector for TestCreator");
      onShowMaterialsSelector({
        fromTestCreator: true,
        currentSelectedMaterials: detail.currentSelectedMaterials || [],
      });
    }
  }

  onMount(async () => {
    console.log("MaterialsBoard mounted");
    tagsStore.loadTags();
    await materialsStore.initialize();

    window.addEventListener("resize", handleResize);
    window.addEventListener("click", handleClickOutside);

    const currentWindow = getCurrentWindow();

    tauriUnlisten = await currentWindow.onResized(({ payload: size }) => {
      console.log("Tauri window resized:", size);
      setTimeout(handleResize, 100);
    });

    tauriUnlistenResized = await currentWindow.onScaleChanged(({ payload }) => {
      console.log("Tauri scale changed:", payload.scaleFactor, payload.size);
      setTimeout(handleResize, 100);
    });

    const webview = getCurrentWebview();
    dragDropUnlisten = await webview.onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        if (materialType === "study") {
          isDragOver = true;
        }
      } else if (event.payload.type === "drop") {
        isDragOver = false;
        if (materialType === "study" && event.payload.paths) {
          handleFileDrop(event.payload.paths);
        }
      } else {
        isDragOver = false;
      }
    });

    setTimeout(handleResize, 0);

    console.log("Initial state:", {
      materialType,
      displayMode,
      selectionMode,
      materialsCount: $materialsStore.materials.length,
    });
  });

  onDestroy(() => {
    materialsStore.destroy();
    window.removeEventListener("resize", handleResize);
    window.removeEventListener("click", handleClickOutside);

    if (tauriUnlisten) {
      tauriUnlisten();
    }
    if (tauriUnlistenResized) {
      tauriUnlistenResized();
    }
    if (dragDropUnlisten) {
      dragDropUnlisten();
    }
  });

  export function updateFlashcardCreator(
    materials: (StudyMaterial | ReviewMaterial)[],
  ) {
    console.log(
      "MaterialsBoard: updateFlashcardCreator called with:",
      materials,
    );
    flashcardCreatorSelectedMaterials = materials;
    pendingMaterialsForFlashcardCreator = materials;

    if (
      showFlashcardCreator &&
      flashcardCreatorComponent &&
      typeof flashcardCreatorComponent.handleMaterialsSelected === "function"
    ) {
      console.log(
        "Calling flashcardCreatorComponent.handleMaterialsSelected immediately",
      );
      flashcardCreatorComponent.handleMaterialsSelected(materials);
      pendingMaterialsForFlashcardCreator = [];
    } else {
      console.log("FlashcardCreator not ready, storing materials as pending");
      if (
        materialType === "review" &&
        !showFlashcardCreator &&
        !showTestCreator
      ) {
        showFlashcardCreator = true;
      }
    }
  }

  export function updateFlashcardDeckEditor(
    materials: (StudyMaterial | ReviewMaterial)[],
  ) {
    console.log(
      "MaterialsBoard: updateFlashcardDeckEditor called with:",
      materials,
    );

    if (
      showFlashcardDeckEditor &&
      flashcardDeckEditorComponent &&
      typeof flashcardDeckEditorComponent.handleMaterialsSelected === "function"
    ) {
      console.log(
        "Calling flashcardDeckEditorComponent.handleMaterialsSelected",
      );
      flashcardDeckEditorComponent.handleMaterialsSelected(materials);
    }
  }

  export function updateTestCreator(
    materials: (StudyMaterial | ReviewMaterial)[],
  ) {
    console.log("MaterialsBoard: updateTestCreator called with:", materials);

    testCreatorSelectedMaterials = materials;
    pendingMaterialsForTestCreator = materials;

    if (
      showTestCreator &&
      testCreatorComponent &&
      typeof testCreatorComponent.handleMaterialsSelected === "function"
    ) {
      console.log("Calling testCreatorComponent.handleMaterialsSelected");
      testCreatorComponent.handleMaterialsSelected(materials);
      pendingMaterialsForTestCreator = [];
    } else {
      console.log("TestCreator not ready, storing materials as pending");
      if (
        materialType === "review" &&
        !showTestCreator &&
        !showFlashcardCreator
      ) {
        showTestCreator = true;
      }
    }
  }
</script>

<section
  class="materials-section"
  class:drag-over={isDragOver && materialType === "study"}
>
  {#if isDragOver && materialType === "study"}
    <div class="drag-overlay">
      <div class="drag-content">
        <h3>Drop files here</h3>
        <p>PDF, Markdown, or TXT files</p>
      </div>
    </div>
  {/if}

  {#if $materialsStore.selectionMode}
    <div class="selection-header">
      <h3>Select Study Materials for Questions</h3>
      <p>Choose the materials you want to use for generating questions</p>
      <div class="selection-actions">
        <span
          >{$materialsStore.selectedMaterials.length} materials selected</span
        >
        <div class="buttons">
          <Button
            variant="secondary"
            onClick={cancelMaterialSelection}
            text="Cancel"
          />
          <Button
            variant="primary"
            onClick={confirmMaterialSelection}
            disabled={$materialsStore.selectedMaterials.length === 0}
            text="Use Selected Materials"
          />
        </div>
      </div>
    </div>
  {/if}

  {#if $materialsStore.isLoading}
    <div class="loading">Loading {materialType} materials...</div>
  {/if}

  {#if !$materialsStore.isLoading && $materialsStore.materials.length === 0 && !showFlashcardCreator && !showFlashcardDeckEditor && !showTestCreator && !showTestEditor}
    <div class="empty-state">
      <h3>No {materialType} materials yet</h3>
      {#if materialType === "study"}
        <p>Add PDF, Markdown or TXT files to start studying.<br>You can also add from links in the bar above!</p>
        <p class="drag-hint">Drag files here or browse</p>
        <Button
          variant="primary"
          onClick={triggerFileSelect}
          text="Browse Files"
        />
      {:else}
        <p>Create flashcard decks or tests to start reviewing</p>
        <div class="empty-actions">
          <div class="create-button-container" bind:this={createMenuButton}>
            <Button
              variant="primary"
              onClick={toggleCreateMenu}
              text="Create"
            ></Button>

            <CreateMenu
              show={showCreateMenu}
              position="below"
              onCreateFlashcardDeck={handleCreateFlashcardDeck}
              onCreateTest={handleCreateTest}
            />
          </div>
        </div>
      {/if}
    </div>
  {/if}

  {#if materialType === "review" && showFlashcardCreator}
    <FlashcardDeckCreator
      bind:this={flashcardCreatorComponent}
      onClose={closeCreators}
      onCreated={handleMaterialCreated}
      onShowMaterialsSelector={handleShowMaterialsSelector}
      initialSelectedMaterials={flashcardCreatorSelectedMaterials}
    />
  {/if}

  {#if materialType === "review" && showFlashcardDeckEditor && currentEditingDeck}
    <FlashcardDeckEditor
      bind:this={flashcardDeckEditorComponent}
      deck={currentEditingDeck}
      isOpen={showFlashcardDeckEditor}
      onClose={closeCreators}
      onUpdated={handleDeckUpdated}
    />
  {/if}

  {#if materialType === "review" && showTestCreator}
    <TestCreator
      bind:this={testCreatorComponent}
      onClose={closeCreators}
      onCreated={handleMaterialCreated}
      onShowMaterialsSelector={handleShowMaterialsSelector}
      initialSelectedMaterials={testCreatorSelectedMaterials}
    />
  {/if}

  {#if materialType === "review" && showTestEditor && currentEditingTest}
    <TestEditor
      test={currentEditingTest}
      isOpen={showTestEditor}
      onClose={closeCreators}
      onUpdated={handleTestUpdated}
    />
  {/if}


<div style="display: {$materialsStore.materials.length === 0 || (materialType === 'review' && (showFlashcardCreator || showFlashcardDeckEditor || showTestCreator || showTestEditor) && !$materialsStore.selectionMode) ? 'none' : 'block'}">      
  <MaterialsGrid
        {materialType}
        {zoomLevel}
        {columns}
        {gridGap}
        {showCreateMenu}
        bind:createMenuButton
        onDeleteMaterial={handleDeleteMaterial}
        onReviewMaterial={handleReviewMaterial}
        onEditMaterial={handleEditMaterial}
        onEditTest={handleEditTest}
        onAddMaterial={triggerFileSelect}
        onToggleCreateMenu={toggleCreateMenu}
        onCreateFlashcardDeck={handleCreateFlashcardDeck}
        onCreateTest={handleCreateTest}
      />

      {#if !$materialsStore.selectionMode}
        <div class="grid-info">
          <span
            >{columns} × {Math.ceil(
              ($materialsStore.materials.length + 1) / columns,
            )}</span
          >
        </div>
      {/if}
    </div>

  {#if showingReview && currentReviewMaterial}
    {#if reviewMaterialType === "flashcard_deck"}
      <FlashcardReview
        deck={currentReviewMaterial}
        isOpen={showingReview}
        onClose={handleCloseReview}
        onReviewCompleted={handleReviewCompleted}
      />
    {:else if reviewMaterialType === "test"}
      <TestReview
        test={currentReviewMaterial}
        isOpen={showingReview}
        onClose={handleCloseReview}
        onReviewCompleted={handleReviewCompleted}
      />
    {/if}
  {/if}
</section>

<style>
  :global(html) {
    scroll-behavior: auto !important;
  }

  .materials-section {
    padding: 1rem 0;
    position: relative;
    min-height: 60vh;
    width: 100%;
    transition: background-color 0.2s ease;
  }

  .materials-section.drag-over {
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
  }

  .drag-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: color-mix(in srgb, var(--accent) 10%, var(--background));
    border: 2px dashed var(--accent);
    border-radius: var(--border-radius);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    pointer-events: none;
  }

  .drag-content {
    text-align: center;
    color: var(--accent);
  }

  .drag-content h3 {
    margin: 0 0 0.5rem 0;
    font-family: var(--font-heading);
    font-weight: 500;
    font-size: 1.5rem;
  }

  .drag-content p {
    margin: 0;
    opacity: 0.8;
  }

  .selection-header {
    padding: var(--space-md) var(--space-lg);
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    border-bottom: 1px solid var(--border);
    margin-bottom: var(--space-md);
    border-radius: 8px;
  }

  .selection-header h3 {
    margin: 0;
    margin-bottom: var(--space-xs);
    font-family: var(--font-heading);
    font-weight: 500;
  }

  .selection-header p {
    margin: 0;
    margin-bottom: var(--space-md);
    color: var(--text-secondary);
  }

  .selection-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .selection-actions .buttons {
    display: flex;
    gap: var(--space-md);
  }

  .loading {
    text-align: center;
    padding: 1rem 2rem;
    color: var(--text-secondary);
    font-family: var(--font-body);
  }

  .empty-state {
    text-align: center;
    padding: 2rem;
    border: 2px dashed var(--border);
    border-radius: var(--border-radius);
    margin: 1rem 2rem;
    background: var(--surface);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .empty-state h3 {
    font-size: 1.5rem;
    margin-bottom: 1rem;
    font-family: var(--font-heading);
    font-weight: 400;
  }

  .empty-state p {
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .drag-hint {
    color: var(--accent) !important;
    font-weight: 500;
    margin-bottom: 1.5rem !important;
  }

  .empty-actions {
    display: flex;
    justify-content: center;
    gap: var(--space-md);
  }

  .create-button-container {
    position: relative;
    display: inline-block;
  }

  .grid-info {
    position: fixed;
    bottom: 2rem;
    left: 2rem;
    background: var(--surface);
    color: var(--text-secondary);
    padding: 0.25rem 0.75rem;
    border-radius: var(--border-radius);
    font-size: 0.9rem;
    opacity: 0.6;
    pointer-events: none;
    z-index: 100;
    border: 1px solid var(--border);
    font-family: var(--font-body);
    transition: opacity 0.3s ease;
  }

  @media (max-width: 768px) {
    .grid-info {
      bottom: 4rem;
      left: 1rem;
    }

    .selection-actions {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-sm);
    }

    .selection-actions .buttons {
      width: 100%;
    }

    .empty-state {
      margin: 1rem;
      padding: 1.5rem;
    }
  }
</style>