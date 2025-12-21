<script lang="ts">
  import {onMount} from 'svelte';
  import {slide} from 'svelte/transition';
  import {theme} from './lib/stores/theme';
  import {invoke} from '@tauri-apps/api/core';
  import './lib/styles/app.css';

  // Components
  import Header from './lib/components/Layout/Header.svelte';
  import Page from './lib/components/Layout/Page.svelte';
  import MaterialsBoard from './lib/components/Layout/MaterialsBoard.svelte';
  import StatsView from './lib/components/Stats/StatsView.svelte';
  import Settings from './lib/components/Settings/Settings.svelte';
  import Toast from './lib/components/Shared/Toast.svelte';
  import KeyboardShortcutsGuide from './lib/components/Shared/KeyboardShortcutsGuide.svelte';
  import StudyMaterialsBar from './lib/components/StudyMaterials/StudyMaterialsBar.svelte';
  import ZoomControls from './lib/components/Shared/ZoomControls.svelte';
  import FirstTimeModal from './lib/components/Shared/FirstTimeModal.svelte';

  // Icons
  import {Search as SearchIcon, Keyboard as KeyboardIcon} from 'lucide-svelte';

  // Stores
  import {llmStore} from './lib/stores/llmStore';
  import {toastStore} from './lib/stores/toastStore';

  // Types
  import type { SortOption, StudyMaterial, ReviewMaterial } from './lib/stores/materialsStore';
  import type { ToastType } from './lib/components/Shared/Toast.svelte';

  type ViewMode = 'study' | 'review' | 'stats';

  interface MaterialsBoardRef {
    updateTestCreator: (materials: (StudyMaterial | ReviewMaterial)[]) => void;
    updateFlashcardCreator: (materials: (StudyMaterial | ReviewMaterial)[]) => void;
  }

  interface MaterialsSelectorContext {
    fromFlashcardCreator?: boolean;
    fromTestCreator?: boolean;
  }

  interface MaterialSelectorDetail {
    fromFlashcardCreator?: boolean;
    fromTestCreator?: boolean;
    currentSelectedMaterials?: (StudyMaterial | ReviewMaterial)[];
  }

  const sortOptions = [
    { value: "date_asc", label: "Oldest first" },
    { value: "date_desc", label: "Newest first" },
    { value: "name_asc", label: "Name (A-Z)" },
    { value: "name_desc", label: "Name (Z-A)" }
  ] as const;

  // State
  let zoomLevel: number = 1;
  let currViewMode: ViewMode = "study";
  let showKeyboardShortcuts: boolean = false;
  let showSettings: boolean = false;
  let showingMaterialSelector: boolean = false;
  let selectedStudyMaterials: (StudyMaterial | ReviewMaterial)[] = [];
  let showFirstTimeModal: boolean = false;
  let sortOption: SortOption = "date_asc";
  let showSearch: boolean = false;
  let searchTerm: string = '';
  let materialsBoardRef: MaterialsBoardRef | null = null;
  let appIsReady: boolean = false;

  // Which creator requested materials from the MaterialsBoard
  let materialsSelectorContext: MaterialsSelectorContext = {};

  // Functions
  function clearToast() {
    toastStore.clear();
  }

  function handleViewChange(mode: ViewMode) {
    currViewMode = mode;
  }

  function handleOpenSettings() {
    showSettings = true;
  }

  function handleCloseSettings() {
    showSettings = false;
    llmStore.refresh();
  }

  function handleShowMaterialsSelector(detail: MaterialSelectorDetail | undefined) {
    console.log("App: handleShowMaterialsSelector called with:", detail);
    // Store the context of which creator requested materials
    materialsSelectorContext = {
      fromFlashcardCreator: detail?.fromFlashcardCreator,
      fromTestCreator: detail?.fromTestCreator
    };

    showingMaterialSelector = true;
    currViewMode = 'study';

    if (detail?.currentSelectedMaterials && detail.currentSelectedMaterials.length > 0) {
      selectedStudyMaterials = [...detail.currentSelectedMaterials];
    } else if (selectedStudyMaterials.length > 0) {
      console.log('Keeping existing selectedStudyMaterials:', selectedStudyMaterials);
    } else {
      selectedStudyMaterials = [];
    }
  }

  function handleMaterialsSelected(materials: (StudyMaterial | ReviewMaterial)[]) {
    console.log("Materials selected in App:", materials);
    selectedStudyMaterials = materials;
    showingMaterialSelector = false;
    currViewMode = 'review';

    // Use requestAnimationFrame for more reliable timing than setTimeout(0)
    requestAnimationFrame(() => {
      if (materialsBoardRef) {
        if (materialsSelectorContext.fromTestCreator) {
          materialsBoardRef.updateTestCreator(selectedStudyMaterials);
        } else if (materialsSelectorContext.fromFlashcardCreator) {
          materialsBoardRef.updateFlashcardCreator(selectedStudyMaterials);
        }

        materialsSelectorContext = {};
      }
    });
  }

  function handleSelectionCancelled() {
    showingMaterialSelector = false;
    currViewMode = 'review';
    materialsSelectorContext = {};
  }

  function handleZoomChange(zoomValue: number) {
    zoomLevel = zoomValue;
  }

  function handleSortChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    sortOption = target.value as SortOption;
  }

  function toggleSearch() {
    showSearch = !showSearch;
    if (!showSearch) {
      searchTerm = '';
    }
  }

  function clearSearch() {
    searchTerm = '';
  }

  async function checkEmbeddingModel() {
    try {
      const embeddingReady: boolean = await invoke('is_embedding_model_ready');
      
      appIsReady = embeddingReady;
      
      // Show first time modal only if either system is not ready (first time user)
      if (!appIsReady) {
        showFirstTimeModal = true;
      }
    } catch (error) {
      console.error('Failed to check system status:', error);
      showFirstTimeModal = true;
    }
  }

  function handleFirstTimeComplete() {
    appIsReady = true;
    showFirstTimeModal = false;
    console.log('First time setup completed! Enjoy StudyCraft!');
  }

  onMount(async () => {
    llmStore.initialize();
    await checkEmbeddingModel();
  });

</script>

<div class="app" data-theme={$theme}>

  <Toast
    message={$toastStore.current?.message ?? ''}
    type={$toastStore.current?.type ?? 'error'}
    onClose={clearToast}
  />

  <Header
    {currViewMode}
    onViewChange={handleViewChange}
    onOpenSettings={handleOpenSettings}/>

  <Page>
    {#if currViewMode !== 'stats'}
      <div class="content-header">
        <h2>{currViewMode === 'study' ? 'Study' : 'Review'}</h2>
        <div class="view-controls">
          
          <!-- Material sorting controls -->
          <div class="sort-controls">
            <select id="sort-select" value={sortOption} on:change={handleSortChange}>
              {#each sortOptions as option}
                <option value={option.value}>{option.label}</option>
              {/each}
            </select>
          </div>

          <!-- Controls for searching for a material -->
          <div class="search-controls">
            <button 
              class="view-button" 
              on:click={toggleSearch}
              class:active={showSearch}
              title="Search"
              aria-label="Search for materials"
            >
            <SearchIcon size={18}/>
            </button>

             {#if showSearch}
              <div class="search-input" transition:slide|local={{ duration: 200 }}>
                <input 
                  type="text" 
                  placeholder="Search by name..." 
                  bind:value={searchTerm}
                  autofocus
                />
                {#if searchTerm}
                  <button class="clear-search" on:click={clearSearch}>×</button>
                {/if}
              </div>
            {/if}
          </div>

          <!-- Zoom -->
          <ZoomControls onZoomChange={handleZoomChange}/>

          <!-- Keyboard shortcut button -->
          <button 
            class="keyboard-shortcuts-button" 
            title="Keyboard shortcuts"
            on:click={() => showKeyboardShortcuts = true}
            aria-label="Toggle keyboard shortcuts guide"
          >
            <KeyboardIcon size={18}/>
          </button>

        </div>
      </div>
    {/if}
    
    <!-- Ask stuff and get answer from your Study Materials -->
    {#if currViewMode === 'study'}
      <StudyMaterialsBar/>
    {/if}

    <!-- Main content -->
    {#if currViewMode === 'stats'}
      <StatsView />
    {:else}
      <MaterialsBoard 
        {zoomLevel}
        {sortOption}
        {searchTerm}
        materialType={currViewMode === 'study' ? 'study' : 'review'}
        selectionMode={currViewMode === 'study' && showingMaterialSelector}
        initialSelectedMaterials={selectedStudyMaterials}
        bind:this={materialsBoardRef}
        onMaterialsSelected={handleMaterialsSelected}
        onSelectionCancelled={handleSelectionCancelled}
        onShowMaterialsSelector={handleShowMaterialsSelector}
      />
    {/if}
  </Page>

  <!-- Comps that gets on top of the app itself -->
  <KeyboardShortcutsGuide bind:show={showKeyboardShortcuts} />
  <Settings isOpen={showSettings} onClose={handleCloseSettings} />
  <FirstTimeModal 
    bind:show={showFirstTimeModal} 
    on:complete={handleFirstTimeComplete}
  />
</div>