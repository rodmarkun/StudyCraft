<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { slide } from "svelte/transition";

  import { CircleCheckBig as CircleCheckBigIcon } from "lucide-svelte";

  import { llmStore } from "../../stores/llmStore";
  import { materialsStore } from "../../stores/materialsStore";

  interface ChunkMetadata {
    chunk_id: string;
    material_id: string;
    file_display_name: string;
    chunk_index: number;
    chunk_text: string;
  }

  interface RelevantChunk {
    chunk_id: string;
    text: string;
    score: number;
    metadata: ChunkMetadata;
  }

  interface SearchAgentResponse {
    answer: string;
    sources: RelevantChunk[];
  }

  let activeTab = 'add';
  let searchQuery = "";
  let isSearching = false;
  let searchResult = "";
  let searchChunks: RelevantChunk[] = [];
  let searchSources: RelevantChunk[] = [];
  let error = "";
  let showResult = false;

  $: llmStatus = $llmStore;
  $: canSearch =
    hasStudyMaterials &&
    !llmStatus.isLoading;
  $: canQuestionAsk =
    hasStudyMaterials &&
    llmStatus.hasConfiguredProvider &&
    !llmStatus.isLoading;
  $: hasStudyMaterials = $materialsStore.materials.some(
    (material) => "id" in material,
  );

  function setActiveTab(tab) {
    activeTab = tab;
    clearSearch();
  }

  async function checkForStudyMaterials() {
    try {
      const studyMaterials = await invoke("get_study_materials", {
        sortBy: "date_asc",
      });
      hasStudyMaterials =
        Array.isArray(studyMaterials) && studyMaterials.length > 0;
    } catch (err) {
      console.error("Failed to check for study materials:", err);
      hasStudyMaterials = false;
    }
  }

  function isValidUrl(string: string): boolean {
    try {
      new URL(string);
      return true;
    } catch (_) {
      return false;
    }
  }

  function formatText(text: string): string {
    let formatted = text;
    
    // Bold text (**text**)
    formatted = formatted.replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>');
    
    // Italic text (*text*) - but not if already part of bold
    formatted = formatted.replace(/(?<!\*)\*([^*]+?)\*(?!\*)/g, '<em>$1</em>');
    
    // Handle lists (lines starting with - or * followed by space)
    formatted = formatted.replace(/^[\-\*]\s+(.+)$/gm, '<li>$1</li>');
    
    // Wrap consecutive list items in ul tags
    formatted = formatted.replace(/((<li>.*?<\/li>\s*)+)/gs, '<ul>$1</ul>');
    
    // Handle numbered lists (lines starting with number. followed by space)
    formatted = formatted.replace(/^\d+\.\s+(.+)$/gm, '<li>$1</li>');
    
    // Convert line breaks to <br> tags, but preserve list formatting
    formatted = formatted.replace(/\n(?!<\/?(ul|li))/g, '<br>');
    
    return formatted;
  }

  async function handleChunkClick(chunk: RelevantChunk) {
    try {
      await invoke("open_material", {
        id: chunk.metadata.material_id
      });
    } catch (err) {
      console.error("Failed to open material:", err);
      error = "Failed to open material. Please try again.";
    }
  }

  async function handleSourceClick(source: RelevantChunk) {
    try {
      await invoke("open_material", {
        id: source.metadata.material_id
      });
    } catch (err) {
      console.error("Failed to open material:", err);
      error = "Failed to open material. Please try again.";
    }
  }

  async function handleSearch() {
    if (!searchQuery.trim()) {
      return;
    }

    if (activeTab === 'add') {
      if (!isValidUrl(searchQuery.trim())) {
        error = "Please enter a valid URL (e.g., https://example.com or https://github.com/user/repo)";
        return;
      }

      isSearching = true;
      error = "";
      searchResult = "";
      searchChunks = [];
      searchSources = [];
      showResult = false;

      try {
        await materialsStore.addStudyMaterialFromLink(searchQuery.trim());
        searchQuery = "";
      } catch (err) {
        console.error("Failed to add material from link:", err);
        error = (err as string) || "Failed to process link. Please try again.";
      } finally {
        isSearching = false;
      }
      return;
    }


    if (activeTab === 'question' && !canQuestionAsk) {
      if (!llmStatus.hasConfiguredProvider) {
        error =
          "No AI provider configured. Please configure an AI provider in Settings to use this feature.";
      }
      return;
    }

    isSearching = true;
    error = "";
    searchResult = "";
    searchChunks = [];
    searchSources = [];
    showResult = false;

    try {
      if (activeTab === 'search') {
        const chunks = await invoke("search_study_materials_global", {
          query: searchQuery.trim(),
          maxChunks: 10
        });
        searchChunks = chunks as RelevantChunk[];
        showResult = true;
      } else if (activeTab === 'question') {
        const result = await invoke("search_in_study_materials", {
          query: searchQuery.trim(),
        });
        const agentResponse = result as SearchAgentResponse;
        searchResult = agentResponse.answer;
        searchSources = agentResponse.sources;
        showResult = true;
      }
    } catch (err) {
      console.error("Search failed:", err);
      error = (err as string) || "Search failed. Please try again.";
    } finally {
      isSearching = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (
      event.key === "Enter" &&
      !isSearching &&
      searchQuery.trim() &&
      ((activeTab === 'add') || (activeTab === 'search' && canSearch) || (activeTab === 'question' && canQuestionAsk))
    ) {
      event.preventDefault();
      handleSearch();
    }
  }

  function clearSearch() {
    searchQuery = "";
    searchResult = "";
    searchChunks = [];
    searchSources = [];
    error = "";
    showResult = false;
  }

  onMount(() => {
    checkForStudyMaterials();
  });
</script>

<div class="search-container" transition:slide={{ duration: 200 }}>
  <div class="search-tabs">
    <button 
      class="tab-button" 
      class:active={activeTab === 'add'}
      on:click={() => setActiveTab('add')}
    >
      Add
    </button>
    
    <div class="tab-separator"></div>
    
    <button 
      class="tab-button" 
      class:active={activeTab === 'search'}
      on:click={() => setActiveTab('search')}
    >
      Search
    </button>
    
    <div class="tab-separator"></div>
    
    <button 
      class="tab-button" 
      class:active={activeTab === 'question'}
      on:click={() => setActiveTab('question')}
    >
      Question
    </button>
  </div>

  <div class="search-input-wrapper">
    <div class="input-container">
      <input
        type="text"
        bind:value={searchQuery}
        on:keydown={handleKeydown}
        placeholder={activeTab === 'add' ? 'Add a website or github repository link...' : activeTab === 'search' ? 'Search semantically across your study materials...' : 'Ask a question to your study materials...'}
        disabled={isSearching || (activeTab === 'search' && !hasStudyMaterials) || (activeTab === 'question' && !canQuestionAsk)}
        title={(activeTab === 'search' && !hasStudyMaterials)
          ? "Add study materials first to use this feature"
          : (activeTab === 'question' && !canQuestionAsk)
          ? "Add study materials and configure an AI provider in Settings to use this feature"
          : ""}
        class="search-input"
      />
      {#if searchQuery.trim()}
        <button
          type="button"
          on:click={clearSearch}
          class="clear-button"
          title="Clear search"
        >
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
          >
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      {/if}
    </div>
    <button
      type="button"
      on:click={handleSearch}
      disabled={isSearching || !searchQuery.trim() || (activeTab === 'search' && !hasStudyMaterials) || (activeTab === 'question' && !canQuestionAsk)}
      class="search-button"
      title={(activeTab === 'search' && !hasStudyMaterials)
        ? "Add study materials first to use this feature"
        : (activeTab === 'question' && !canQuestionAsk)
        ? "Add study materials and configure an AI provider in Settings to use this feature"
        : ""}
    >
      {#if isSearching}
        <svg class="spinner" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 12a9 9 0 11-6.219-8.56"/>
        </svg>
        {activeTab === 'add' ? 'Adding...' : activeTab === 'search' ? 'Searching...' : 'Asking...'}
      {:else}
        <CircleCheckBigIcon size={16} />
        {activeTab === 'add' ? 'Add' : activeTab === 'search' ? 'Search' : 'Ask'}
      {/if}
    </button>
  </div>
</div>

{#if error}
  <div class="search-error" transition:slide={{ duration: 300 }}>
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
    >
      <circle cx="12" cy="12" r="10"></circle>
      <line x1="15" y1="9" x2="9" y2="15"></line>
      <line x1="9" y1="9" x2="15" y2="15"></line>
    </svg>
    {error}
  </div>
{/if}

{#if showResult && searchChunks.length > 0}
  <div class="search-result" transition:slide={{ duration: 300 }}>
    <div class="result-header">
      <h4>Search results from your study materials:</h4>
      <button
        type="button"
        on:click={() => (showResult = false)}
        class="close-result"
        title="Close result"
      >
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
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
    <div class="chunks-container">
      {#each searchChunks as chunk}
        <div class="chunk-item" on:click={() => handleChunkClick(chunk)}>
          <div class="chunk-header">
            <span class="chunk-score">Score: {chunk.score}</span>
            <span class="chunk-file">{chunk.metadata.file_display_name}</span>
          </div>
          <div class="chunk-text">{chunk.text}</div>
        </div>
      {/each}
    </div>
  </div>
{/if}

{#if showResult && searchResult && activeTab === 'question'}
  <div class="search-result" transition:slide={{ duration: 300 }}>
    <div class="result-header">
      <h4>Answer from your study materials:</h4>
      <button
        type="button"
        on:click={() => (showResult = false)}
        class="close-result"
        title="Close result"
      >
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
        >
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>
    <div class="result-content">
      {@html formatText(searchResult)}
    </div>
    {#if searchSources.length > 0}
      <div class="sources-section">
        <h5>Sources:</h5>
        <div class="sources-container">
          {#each searchSources as source}
            <button
              type="button"
              class="source-item"
              on:click={() => handleSourceClick(source)}
              title="Click to open this material"
            >
              {source.metadata.file_display_name}
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .search-container {
    position: relative;
    margin: calc(-1 * var(--space-md)) auto var(--space-md) auto;
    max-width: 800px;
  }

  .search-tabs {
    display: flex;
    justify-content: center;
    align-items: center;
    background: var(--background);
    padding: 2px;
    gap: var(--space-xs);
    margin-bottom: var(--space-sm);
  }

  .tab-button {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    background: var(--background);
    border: none;
    padding: var(--space-sm) var(--space-md);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all var(--transition-speed) ease;
    border-radius: var(--border-radius);
    font-weight: 500;
    font-size: 0.9rem;
    min-height: 36px;
  }

  .tab-button:hover {
    color: var(--text);
    background: var(--surface);
  }

  .tab-button:focus {
    outline: none;
  }

  .tab-button.active {
    color: var(--accent);
    background: var(--background);
  }

  .tab-separator {
    width: 1px;
    height: 20px;
    background: var(--border);
  }

  .search-input-wrapper {
    display: flex;
    gap: var(--space-sm);
    align-items: stretch;
    margin-bottom: var(--space-xs);
  }

  .input-container {
    position: relative;
    flex: 1;
  }

  .search-input {
    width: 100%;
    padding: var(--space-sm);
    padding-right: calc(var(--space-md) * 2 + 16px);
    border: 2px solid var(--border);
    border-radius: var(--border-radius);
    background: var(--background);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 1rem;
    transition: all var(--transition-speed) ease;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(var(--accent-rgb), 0.1);
  }

  .search-input:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .search-input::placeholder {
    color: var(--text-secondary);
  }

  .clear-button {
    position: absolute;
    right: var(--space-md);
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: var(--space-xs);
    border-radius: var(--border-radius);
    transition: all var(--transition-speed) ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .clear-button:hover {
    color: var(--text);
    background: color-mix(in srgb, var(--border) 50%, transparent);
  }

  .search-button {
    background: var(--accent);
    color: white;
    border: none;
    padding: var(--space-sm);
    border-radius: var(--border-radius);
    font-family: var(--font-body);
    font-size: 1rem;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-speed) ease;
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    white-space: nowrap;
  }

  .search-button:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 80%, black);
  }

  .search-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .search-error {
    margin: var(--space-md) auto 0 auto;
    max-width: 800px;
    padding: var(--space-md);
    background: color-mix(in srgb, #ef4444 10%, var(--surface));
    border: 1px solid color-mix(in srgb, #ef4444 30%, var(--border));
    border-radius: var(--border-radius);
    color: #ef4444;
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .search-result {
    margin: var(--space-md) auto 0 auto;
    max-width: 800px;
    background: color-mix(in srgb, var(--accent) 5%, var(--surface));
    border: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border));
    border-radius: var(--border-radius);
    overflow: hidden;
  }

  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md);
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    border-bottom: 1px solid
      color-mix(in srgb, var(--accent) 20%, var(--border));
  }

  .result-header h4 {
    margin: 0;
    font-family: var(--font-heading);
    font-weight: 500;
    color: var(--text);
    font-size: 1rem;
  }

  .close-result {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: var(--space-xs);
    border-radius: var(--border-radius);
    transition: all var(--transition-speed) ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .close-result:hover {
    color: var(--text);
    background: color-mix(in srgb, var(--border) 50%, transparent);
  }

  .result-content {
    padding: var(--space-md);
    color: var(--text);
    line-height: 1.6;
  }

  .result-content :global(strong) {
    font-weight: 600;
    color: var(--text);
  }

  .result-content :global(em) {
    font-style: italic;
    color: var(--text);
  }

  .result-content :global(ul) {
    margin: var(--space-sm) 0;
    padding-left: var(--space-md);
  }

  .result-content :global(li) {
    margin: var(--space-xs) 0;
    list-style-type: disc;
  }

  .sources-section {
    border-top: 1px solid color-mix(in srgb, var(--accent) 20%, var(--border));
    padding: var(--space-md);
  }

  .sources-section h5 {
    margin: 0 0 var(--space-sm) 0;
    font-family: var(--font-heading);
    font-weight: 500;
    color: var(--text);
    font-size: 0.9rem;
  }

  .sources-container {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-xs);
  }

  .source-item {
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-xs) var(--space-sm);
    color: var(--text);
    font-size: 0.8rem;
    cursor: pointer;
    transition: all var(--transition-speed) ease;
    text-decoration: none;
    font-family: var(--font-body);
  }

  .source-item:hover {
    background: color-mix(in srgb, var(--accent) 10%, var(--background));
    border-color: var(--accent);
    color: var(--accent);
  }

  .chunks-container {
    padding: var(--space-sm);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .chunk-item {
    background: var(--background);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-md);
    cursor: pointer;
    transition: all var(--transition-speed) ease;
  }

  .chunk-item:hover {
    background: color-mix(in srgb, var(--accent) 5%, var(--background));
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .chunk-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-sm);
  }

  .chunk-score {
    background: var(--accent);
    color: white;
    padding: 2px 8px;
    border-radius: 12px;
    font-size: 0.8rem;
    font-weight: 500;
  }

  .chunk-file {
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 500;
  }

  .chunk-text {
    color: var(--text);
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .spinner {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  @media (max-width: 768px) {
    .search-tabs {
      flex-direction: column;
      gap: var(--space-xs);
    }
    
    .tab-separator {
      width: 100%;
      height: 1px;
    }
    
    .tab-button {
      justify-content: flex-start;
      width: 100%;
    }

    .search-input-wrapper {
      flex-direction: column;
      gap: var(--space-sm);
    }

    .search-button {
      flex: 1;
    }

    .result-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-sm);
    }

    .close-result {
      align-self: flex-end;
    }

    .chunk-header {
      flex-direction: column;
      align-items: flex-start;
      gap: var(--space-xs);
    }
  }
</style>