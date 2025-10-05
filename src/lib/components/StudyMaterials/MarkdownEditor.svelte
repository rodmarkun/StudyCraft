<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { fade, slide } from "svelte/transition";
  import {
    Save,
    X,
    RefreshCw,
    Eye,
    Split,
    Maximize2,
    Minimize2,
    FileText,
  } from "lucide-svelte";
  import Button from '../Shared/Button.svelte';
  import { marked } from "marked";
  
  export let filePath = "";
  export let materialName = "";
  export let isOpen = false;
  export let onSaved: (() => void) | undefined = undefined;
  export let onClose: (() => void) | undefined = undefined;
  
  let editorContent = "";
  let originalContent = "";
  let renderedChunks: string[] = [];
  let isLoading = true;
  let isSaving = false;
  let isDirty = false;
  let editor: HTMLTextAreaElement;
  let viewMode = "split";
  let isFullscreen = false;
  let wordCount = 0;
  let charCount = 0;
  let autoSave = false;
  let autoSaveTimeout;
  let renderTimeout;
  let statsTimeout;
  let isRendering = false;
  let fileSize = 0;
  
  marked.setOptions({
    breaks: true,
    gfm: true,
  });

  const CHUNK_SIZE = 5000;
  const LARGE_FILE_THRESHOLD = 500000;

  function chunkMarkdown(content: string): string[] {
    const chunks: string[] = [];
    for (let i = 0; i < content.length; i += CHUNK_SIZE) {
      chunks.push(content.slice(i, Math.min(i + CHUNK_SIZE, content.length)));
    }
    return chunks;
  }

  async function renderMarkdownProgressive() {
    if (isRendering || !editorContent) {
      renderedChunks = [];
      return;
    }
    isRendering = true;
    
    const chunks = chunkMarkdown(editorContent);
    renderedChunks = new Array(chunks.length).fill("");
    
    for (let i = 0; i < chunks.length; i++) {
      try {
        const result = marked.parse(chunks[i]);
        renderedChunks[i] = typeof result === 'string' ? result : await result;
        
        if (i % 3 === 0) {
          await new Promise(resolve => setTimeout(resolve, 0));
        }
      } catch (error) {
        console.error("Markdown parsing error:", error);
        renderedChunks[i] = "<p>Error rendering chunk</p>";
      }
    }
    
    isRendering = false;
  }

  function getDebounceTime(content: string): number {
    const size = content.length;
    if (size < 10000) return 100;
    if (size < 100000) return 300;
    return 500;
  }

  function debounceRender() {
    clearTimeout(renderTimeout);
    renderTimeout = setTimeout(async () => {
      if (viewMode === "preview" || viewMode === "split") {
        await renderMarkdownProgressive();
      }
    }, getDebounceTime(editorContent));
  }

  function debounceStats() {
    clearTimeout(statsTimeout);
    statsTimeout = setTimeout(() => {
      updateStats();
    }, 100);
  }

  function updateStats() {
    const text = editorContent || "";
    charCount = text.length;
    wordCount = text.trim() ? text.trim().split(/\s+/).length : 0;
  }

  $: {
    if (filePath && isOpen) {
      loadMarkdownContent();
    }
  }

  $: isDirty = editorContent !== originalContent;

  $: if (autoSave && isDirty && editorContent !== "") {
    clearTimeout(autoSaveTimeout);
    autoSaveTimeout = setTimeout(() => {
      saveMarkdownContent();
    }, 1000);
  }

  async function loadMarkdownContent() {
    isLoading = true;
    try {
      const params = {
        filePath: filePath,
      };
      const content: string = await invoke("get_markdown_content", params);
      editorContent = content;
      originalContent = content;
      fileSize = content.length;
      
      if (fileSize > LARGE_FILE_THRESHOLD) {
        viewMode = "edit";
      }
      
      await tick();
      debounceRender();
      debounceStats();
      
      isLoading = false;
    } catch (error) {
      console.error("Failed to load markdown content:", error);
      editorContent =
        "# Failed to load content\n\nThere was an error loading the content for this file.";
      isLoading = false;
    }
  }

  async function saveMarkdownContent() {
    if (!isDirty) return;
    isSaving = true;
    try {
      await invoke("save_markdown_content", {
        filePath: filePath,
        content: editorContent,
      });
      originalContent = editorContent;
      onSaved?.();
      isSaving = false;
    } catch (error) {
      console.error("Failed to save markdown content:", error);
      isSaving = false;
    }
  }

  function handleClose() {
    if (isDirty) {
      if (
        confirm(
          "You have unsaved changes. Are you sure you want to close the editor?",
        )
      ) {
        closeEditor();
      }
    } else {
      closeEditor();
    }
  }

  function closeEditor() {
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }
    if (renderTimeout) {
      clearTimeout(renderTimeout);
    }
    if (statsTimeout) {
      clearTimeout(statsTimeout);
    }
    onClose?.();
  }

  function toggleFullscreen() {
    isFullscreen = !isFullscreen;
  }

  function setViewMode(mode) {
    viewMode = mode;
    if (mode === "preview" || mode === "split") {
      debounceRender();
    }
  }

  function handleInput(event) {
    editorContent = event.target.value;
    debounceRender();
    debounceStats();
  }

  function handleKeydown(event) {
    if ((event.ctrlKey || event.metaKey) && event.key === "s") {
      event.preventDefault();
      saveMarkdownContent();
      return;
    }
    if (event.key === "Escape") {
      handleClose();
      return;
    }
    if (event.key === "F11") {
      event.preventDefault();
      toggleFullscreen();
      return;
    }
  }

  onMount(() => {
    document.addEventListener("keydown", handleKeydown);
  });

  onDestroy(() => {
    document.removeEventListener("keydown", handleKeydown);
    if (autoSaveTimeout) {
      clearTimeout(autoSaveTimeout);
    }
    if (renderTimeout) {
      clearTimeout(renderTimeout);
    }
    if (statsTimeout) {
      clearTimeout(statsTimeout);
    }
  });
</script>

{#if isOpen}
  <div
    class="editor-modal"
    class:fullscreen={isFullscreen}
    transition:fade={{ duration: 200 }}
  >
    <div
      class="editor-container"
      class:fullscreen={isFullscreen}
      transition:slide={{ duration: 200 }}
    >
      <div class="editor-header">
        <div class="header-left">
          <h2>{materialName}</h2>
          {#if fileSize > LARGE_FILE_THRESHOLD}
            <span class="file-size-warning">Large file ({Math.round(fileSize / 1024)}KB)</span>
          {/if}
          <div class="view-mode-switcher">
            <button
              class="mode-button"
              class:active={viewMode === "edit"}
              on:click={() => setViewMode("edit")}
              title="Edit only"
            >
              <svelte:component this={FileText} size={18} />
            </button>
            <button
              class="mode-button"
              class:active={viewMode === "split"}
              on:click={() => setViewMode("split")}
              title="Split view"
            >
              <svelte:component this={Split} size={18} />
            </button>
            <button
              class="mode-button"
              class:active={viewMode === "preview"}
              on:click={() => setViewMode("preview")}
              title="Preview only"
            >
              <svelte:component this={Eye} size={18} />
            </button>
          </div>
        </div>
        <div class="editor-controls">
          <Button
            variant="save"
            icon={Save}
            text={isSaving ? "Reindexing..." : "Save"}
            changedText={isSaving ? "Reindexing..." : "Save Changes"}
            changed={isDirty}
            disabled={isSaving || !isDirty}
            title="Save changes (Ctrl+S)"
            onClick={saveMarkdownContent}
          />
          <button
            class="control-button"
            on:click={toggleFullscreen}
            title="Toggle fullscreen (F11)"
          >
            <svelte:component
              this={isFullscreen ? Minimize2 : Maximize2}
              size={16}
            />
          </button>
          <button
            class="control-button close"
            on:click={handleClose}
            title="Close editor (Esc)"
            aria-label="Close editor"
          >
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
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>
      <div class="editor-body">
        {#if isLoading}
          <div class="loading-indicator">
            <div class="spinner"></div>
            <span>Loading content...</span>
          </div>
        {:else}
          <div
            class="editor-content"
            class:split={viewMode === "split"}
            class:edit-only={viewMode === "edit"}
            class:preview-only={viewMode === "preview"}
          >
            {#if viewMode === "edit" || viewMode === "split"}
              <div class="editor-pane">
                <textarea
                  bind:this={editor}
                  value={editorContent}
                  class="markdown-textarea"
                  spellcheck="false"
                  placeholder=""
                  on:input={handleInput}
                  on:keydown={handleKeydown}
                ></textarea>
              </div>
            {/if}
            {#if viewMode === "preview" || viewMode === "split"}
              <div class="preview-pane">
                <div class="preview-content">
                  {#each renderedChunks as chunk, i}
                    <div class="chunk" data-chunk-index={i}>
                      {@html chunk}
                    </div>
                  {/each}
                  {#if isRendering}
                    <div class="rendering-indicator">Rendering preview...</div>
                  {/if}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
      <div class="status-bar">
        <div class="status-left">
          {#if isDirty}
            <span class="status-indicator unsaved">● Unsaved changes</span>
          {:else}
            <span class="status-indicator saved">✓ Saved</span>
          {/if}
          <label class="auto-save-toggle">
            <input type="checkbox" bind:checked={autoSave} />
            Auto-save
          </label>
        </div>
        <div class="status-center">
          <span class="stats">
            {wordCount} words • {charCount} characters
          </span>
        </div>
        <div class="status-right">
          <span class="hint"
            >Ctrl+S to save • F11 for fullscreen • Esc to close</span
          >
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .editor-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
  }

  .editor-modal.fullscreen {
    background: var(--background);
  }

  .editor-container {
    width: 90%;
    max-width: 1200px;
    height: 90vh;
    background: var(--surface);
    border-radius: var(--border-radius);
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-container.fullscreen {
    width: 100%;
    height: 100vh;
    max-width: none;
    border-radius: 0;
  }

  .editor-header {
    padding: var(--space-md);
    border-bottom: 1px solid var(--border);
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
    background: var(--background);
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: var(--space-lg);
  }

  .editor-header h2 {
    margin: 0;
    font-size: 1.25rem;
    font-family: var(--font-heading);
    color: var(--text);
  }

  .file-size-warning {
    font-size: 0.8rem;
    color: var(--warning, orange);
    padding: 0.25rem 0.5rem;
    background: var(--surface);
    border-radius: var(--border-radius);
    border: 1px solid var(--warning, orange);
  }

  .view-mode-switcher {
    display: flex;
    background: var(--surface);
    border-radius: var(--border-radius);
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .mode-button {
    padding: var(--space-sm);
    border: none;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
    height: 40px;
    border-right: 1px solid var(--border);
    border-radius: 0;
  }

  .mode-button:last-child {
    border-right: none;
  }

  .mode-button:hover {
    background: var(--background);
    color: var(--text);
  }

  .mode-button.active {
    background: var(--accent);
    color: white;
  }

  .editor-controls {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .control-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: var(--space-sm);
    border-radius: var(--border-radius);
    transition: all var(--transition-speed) ease;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 40px;
    height: 40px;
  }

  .control-button:hover {
    background: var(--surface);
    color: var(--text);
  }

  .control-button.close {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: var(--space-sm);
    border-radius: var(--border-radius);
    transition: all var(--transition-speed) ease;
  }

  .control-button.close:hover {
    background: var(--surface);
    color: var(--text);
  }

  .editor-body {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .editor-content {
    height: 100%;
    display: flex;
    contain: layout style paint;
  }

  .editor-content.edit-only .editor-pane {
    flex: 1;
  }

  .editor-content.preview-only .preview-pane {
    flex: 1;
  }

  .editor-content.split .editor-pane,
  .editor-content.split .preview-pane {
    flex: 1;
  }

  .editor-content.split .editor-pane {
    border-right: 1px solid var(--border);
  }

  .editor-pane {
    height: 100%;
    display: flex;
    flex-direction: column;
    contain: layout style;
  }

  .markdown-textarea {
    flex: 1;
    width: 100%;
    padding: 2rem;
    border: none;
    resize: none;
    background: var(--surface);
    color: var(--text);
    font-family: "Fira Code", "JetBrains Mono", "SF Mono", "Monaco",
      "Inconsolata", "Roboto Mono", "Source Code Pro", monospace;
    font-size: 16px;
    line-height: 1.7;
    tab-size: 2;
    outline: none;
    letter-spacing: 0.02em;
    overflow-y: auto;
    word-wrap: break-word;
    white-space: pre-wrap;
    font-weight: 400;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-rendering: optimizeLegibility;
    caret-color: var(--accent);
    contain: strict;
    will-change: scroll-position;
  }

  .markdown-textarea::placeholder {
    color: var(--text-secondary);
    opacity: 0.6;
    font-style: italic;
  }

  .markdown-textarea:focus {
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--accent);
  }

  .markdown-textarea::-webkit-scrollbar {
    width: 8px;
  }

  .markdown-textarea::-webkit-scrollbar-track {
    background: var(--background);
  }

  .markdown-textarea::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }

  .markdown-textarea::-webkit-scrollbar-thumb:hover {
    background: var(--text-secondary);
  }

  .preview-pane {
    height: 100%;
    background: var(--background);
    overflow-y: auto;
    border-left: 1px solid var(--border);
    contain: layout style paint;
  }

  .preview-pane::-webkit-scrollbar {
    width: 8px;
  }

  .preview-pane::-webkit-scrollbar-track {
    background: var(--background);
  }

  .preview-pane::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
  }

  .preview-pane::-webkit-scrollbar-thumb:hover {
    background: var(--text-secondary);
  }

  .preview-content {
    padding: var(--space-lg);
    font-family: var(--font-body);
    line-height: 1.6;
    color: var(--text);
    max-width: none;
    contain: layout style;
  }

  .chunk {
    contain: layout style;
  }

  .rendering-indicator {
    text-align: center;
    padding: var(--space-md);
    color: var(--text-secondary);
    font-style: italic;
  }

  .preview-content :global(h1),
  .preview-content :global(h2),
  .preview-content :global(h3) {
    font-family: var(--font-heading);
    margin: 1.5em 0 0.5em 0;
    color: var(--text);
    line-height: 1.3;
  }

  .preview-content :global(h1) {
    font-size: 2em;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.3em;
  }

  .preview-content :global(h2) {
    font-size: 1.5em;
  }

  .preview-content :global(h3) {
    font-size: 1.25em;
  }

  .preview-content :global(p) {
    margin: 1em 0;
  }

  .preview-content :global(code) {
    background: var(--surface);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: "SF Mono", "Monaco", monospace;
    font-size: 0.9em;
    border: 1px solid var(--border);
  }

  .preview-content :global(pre) {
    background: var(--surface);
    padding: 1em;
    border-radius: var(--border-radius);
    overflow-x: auto;
    margin: 1em 0;
    border: 1px solid var(--border);
  }

  .preview-content :global(pre code) {
    background: none;
    padding: 0;
    border: none;
  }

  .preview-content :global(ul),
  .preview-content :global(ol) {
    padding-left: 2em;
    margin: 1em 0;
  }

  .preview-content :global(li) {
    margin: 0.5em 0;
  }

  .preview-content :global(a) {
    color: var(--accent);
    text-decoration: none;
  }

  .preview-content :global(a:hover) {
    text-decoration: underline;
  }

  .status-bar {
    padding: var(--space-sm) var(--space-md);
    background: var(--background);
    border-top: 1px solid var(--border);
    font-size: 0.85rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-md);
  }

  .status-left {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .status-center {
    flex: 1;
    text-align: center;
  }

  .status-right {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .status-indicator.unsaved {
    color: var(--warning, orange);
  }

  .status-indicator.saved {
    color: var(--success, green);
  }

  .stats {
    color: var(--text-secondary);
  }

  .hint {
    color: var(--text-secondary);
    font-size: 0.8rem;
  }

  .auto-save-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .auto-save-toggle input {
    margin: 0;
  }

  .loading-indicator {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-md);
    background: var(--surface);
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--accent);
    border-top: 3px solid transparent;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  :global(.action-button) {
    background: var(--accent);
    color: white;
    border: none;
    padding: var(--space-md);
    border-radius: var(--border-radius);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-sm);
    transition: all var(--transition-speed) ease;
    min-width: 40px;
    height: 40px;
  }

  :global(.action-button:hover:not(:disabled)) {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  :global(.action-button:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  :global(.action-button.save.changed) {
    background: var(--success);
    animation: pulse 2s infinite;
  }

  :global(.action-button.secondary) {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }

  @keyframes pulse {
    0% {
      transform: scale(1);
      box-shadow: 0 0 0 0 color-mix(in srgb, var(--success) 50%, transparent);
    }
    70% {
      transform: scale(1.02);
      box-shadow: 0 0 0 8px color-mix(in srgb, var(--success) 0%, transparent);
    }
    100% {
      transform: scale(1);
      box-shadow: 0 0 0 0 color-mix(in srgb, var(--success) 0%, transparent);
    }
  }
</style>