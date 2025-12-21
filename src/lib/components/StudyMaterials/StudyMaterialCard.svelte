<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { spring } from "svelte/motion";

  // Components
  import MarkdownEditor from "./MarkdownEditor.svelte";

  // Stores
  import { coverImageCache } from "../../stores/coverCache";
  import { toastStore } from "../../stores/toastStore";

  // Props
  export let material;
  export let zoomLevel = 1;
  export let onDelete = (id) => {};
  export let selectionMode = false;
  export let selected = false;

  // Reactivity
  $: tags = material.tags || [];
  $: isProcessing = material.is_processing;

  const size = spring(1, {
    stiffness: 0.2,
    damping: 0.7,
  });
  $: {
      size.set(zoomLevel);
  }
  
  $: {
    fontSize = `${Math.max(0.75, 0.5 + (zoomLevel * 0.5))}rem`;
    tagSize = `${0.8 * zoomLevel}rem`;
    buttonSize = Math.max(16, Math.floor(16 * zoomLevel));
  }
  $: transformScale =
    $size >= 1
      ? `scale(${1 + ($size - 1) * 0.15})`
      : `scale(${1 - (1 - $size) * 0.1})`;
  $: if (material.cover_path && !coverLoaded && !coverLoadAttempted) {
    loadCover();
  }

  // State
  let fontSize = `${Math.max(0.75, 0.5 + (zoomLevel * 0.5))}rem`;
  let tagSize = `${0.8 * zoomLevel}rem`;
  let buttonSize = Math.max(16, Math.floor(16 * zoomLevel));
  let coverUrl = null;
  let isEditing = false;
  let editingName = "";
  let nameElement;
  let coverLoaded = false;
  let showDeleteConfirm = false;
  let isHovering = false;
  let coverLoadError = null;
  let coverLoadAttempted = false;
  let showMarkdownEditor = false;
  let markdownFilePath = "";

  // Functions
  async function handleMaterialClick(event) {
    if (selectionMode) return;

    if (
      event.target.closest(".action-button") ||
      event.target.closest(".material-action-button") ||
      event.target.closest(".action-buttons") ||
      event.target.tagName === "H3" ||
      event.target.closest(".name-input") ||
      event.target.tagName === "BUTTON" ||
      event.target.tagName === "SVG" ||
      event.target.tagName === "PATH" ||
      isEditing
    ) {
      return;
    }

    try {
      await invoke("open_material", {
        id: material.id,
      });
    } catch (error) {
      console.error("Failed to open material:", error);
    }
  }

  async function loadCover() {
    // Skip if already loaded or no cover available
    if (!material.cover_path || coverLoaded || coverLoadAttempted) {
      return;
    }

    coverLoadAttempted = true;

    if (coverImageCache.has(material.cover_path)) {
      coverUrl = coverImageCache.get(material.cover_path);
      coverLoaded = true;
      return;
    }

    try {
      const imageData = await invoke("get_cover_image_raw", {
        path: material.cover_path,
      });

      const blob = new Blob([new Uint8Array(imageData)], { type: "image/png" });
      const url = URL.createObjectURL(blob);
      coverImageCache.setItem(material.cover_path, url);

      coverUrl = url;
      coverLoaded = true;
    } catch (e) {
      console.error("Failed to read cover:", e);
      coverLoadError = e.toString();
      coverUrl = null;
    }
  }

  function startEditing() {
    if (!material?.is_processing && !selectionMode) {
      isEditing = true;
      editingName = material?.display_name || material?.name || '';
      setTimeout(() => nameElement?.focus(), 50);
    }
  }

  async function handleKeydown(event) {
    if (event.key === "Enter") {
      event.preventDefault();
      await finishEditing();
    } else if (event.key === "Escape") {
      isEditing = false;
    }
  }

  async function finishEditing() {
    if (editingName.trim() && editingName !== material.display_name) {
      try {
        await invoke("update_material_name", {
          id: material.id,
          newName: editingName.trim(),
        });
        material.display_name = editingName.trim();
      } catch (error) {
        console.error("Failed to update name:", error);
      }
    }
    isEditing = false;
  }

  async function handleDelete(event) {
    // Prevent event bubbling
    event.stopPropagation();
    event.preventDefault();

    try {
      // Remove from cache if exists
      if (material.cover_path && coverImageCache.has(material.cover_path)) {
        // Release the blob URL before removing from cache
        URL.revokeObjectURL(coverImageCache.get(material.cover_path));
        coverImageCache.delete(material.cover_path);
      }

      onDelete(material.id);
    } catch (error) {
      console.error("Failed to handle delete:", error);
    }
  }

  async function openMarkdownEditor(event) {
    // Prevent event bubbling
    event.stopPropagation();
    event.preventDefault();

    if (material.is_processing || selectionMode) return;

    try {
      const result = await invoke("get_material_markdown_path", {
        id: material.id,
      });

      markdownFilePath = result;
      showMarkdownEditor = true;
    } catch (error) {
      console.error("Failed to get markdown path:", error);
      toastStore.error("Could not open the markdown editor. Please try again later.");
    }
  }

  function handleEditorClose() {
    showMarkdownEditor = false;
  }

  function handleEditorSaved() {}

  function showDeleteModal(event) {
    // Prevent event bubbling
    event.stopPropagation();
    event.preventDefault();

    showDeleteConfirm = true;
  }

  let cardElement;

  onMount(() => {
    if (material.cover_path) {
      loadCover();
    }
  });
</script>

<div
  bind:this={cardElement}
  class="material-card"
  class:processing={isProcessing}
  class:hovering={isHovering && !selectionMode}
  class:selectionMode
  class:selected
  style="transform: {transformScale} rotate({isHovering && !selectionMode
    ? '2deg'
    : '0deg'});"
  on:click={handleMaterialClick}
  on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleMaterialClick(e)}
  on:mouseenter={() => !selectionMode && (isHovering = true)}
  on:mouseleave={() => !selectionMode && (isHovering = false)}
  role="button"
  tabindex={selectionMode ? 0 : -1}
  aria-label={`${selectionMode ? (selected ? 'Deselect' : 'Select') : 'Open'} ${material?.display_name || material?.name || 'material'}`}
>
  {#if !selectionMode}
    <div class="action-buttons">
      <button
        class="material-action-button delete"
        title="Delete material"
        aria-label="Delete material"
        on:click={showDeleteModal}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width={buttonSize}
          height={buttonSize}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M3 6h18"></path>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
        </svg>
      </button>
      <button
        class="material-action-button edit"
        title="Edit markdown"
        aria-label="Edit markdown"
        on:click={openMarkdownEditor}
        disabled={isProcessing}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width={buttonSize}
          height={buttonSize}
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3Z"
          ></path>
        </svg>
      </button>
    </div>
  {/if}

  <div class="cover" class:has-image={coverUrl}>
    {#if coverUrl}
      <div
        class="cover-image"
        style="background-image: url('{coverUrl}')"
      ></div>
    {:else if coverLoadError}
      <div class="cover-error">
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
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="12" y1="8" x2="12" y2="12"></line>
          <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
        <span>Failed to load cover</span>
      </div>
    {/if}
  </div>

  <div class="info">
    {#if isEditing}
      <input
        bind:this={nameElement}
        bind:value={editingName}
        on:blur={finishEditing}
        on:keydown={handleKeydown}
        class="name-input"
        style="font-size: {fontSize}"
      />
    {:else}
      <h3
        style="font-size: {fontSize}"
        on:dblclick|stopPropagation={startEditing}
        title={material?.display_name || material?.name || 'Untitled'}
      >
        {material?.display_name || material?.name || 'Untitled'}
      </h3>
    {/if}
    {#if tags.length > 0}
      <div class="tags">
        {#each tags as tag}
          <span class="tag" style="font-size: {tagSize}">{tag}</span>
        {/each}
      </div>
    {/if}
  </div>

  {#if isProcessing}
    <div class="processing-overlay">
      <div
        class="spinner"
        style="width: {2 * zoomLevel}rem; height: {2 * zoomLevel}rem;"
      ></div>
      <span style="font-size: {fontSize}">Converting...</span>
    </div>
  {/if}
</div>

{#if showDeleteConfirm}
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="modal-backdrop"
    on:click={() => (showDeleteConfirm = false)}
    on:keydown={(e) => e.key === 'Escape' && (showDeleteConfirm = false)}
    role="dialog"
    aria-modal="true"
    aria-labelledby="delete-modal-title"
    tabindex="-1"
  >
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions a11y_click_events_have_key_events -->
    <div class="modal-content" on:click|stopPropagation role="document">
      <h4 id="delete-modal-title">Delete Material</h4>
      <p>
        Are you sure you want to delete "{material?.display_name || material?.name || 'this material'}"? This action
        cannot be undone.
      </p>
      <div class="modal-actions">
        <button
          class="cancel-button"
          on:click={() => (showDeleteConfirm = false)}
        >
          Cancel
        </button>
        <button
          class="confirm-button"
          on:click={(e) => {
            handleDelete(e);
            showDeleteConfirm = false;
          }}
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<MarkdownEditor
  isOpen={showMarkdownEditor}
  filePath={markdownFilePath}
  materialName={material.display_name}
  onClose={handleEditorClose}
  onSaved={handleEditorSaved}
/>

<style>
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background: var(--surface);
    max-width: 400px;
    width: 90%;
    padding: 1.5rem;
    border-radius: 8px;
  }

  .modal-content h4 {
    margin: 0;
    margin-bottom: 1rem;
    font-family: var(--font-heading);
    font-size: 1.5rem;
  }

  .modal-content p {
    margin-bottom: 1.5rem;
    color: var(--text-secondary);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
  }

  .cancel-button,
  .confirm-button {
    padding: 0.5rem 1.5rem;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    font-family: var(--font-body);
    transition: all 0.2s ease;
  }

  .cancel-button {
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
  }

  .confirm-button {
    background: #ef4444;
    color: white;
  }

  .cancel-button:hover {
    background: color-mix(in srgb, var(--accent) 5%, var(--surface));
  }

  .confirm-button:hover {
    filter: brightness(1.1);
  }

  .material-card {
    position: relative;
    width: 100%;
    transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    will-change: transform, opacity;
  }

  .material-card.hovering {
    z-index: 5;
  }

  .action-buttons {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    opacity: 0;
    transform: translateX(10px);
    transition: all 0.3s ease;
    z-index: 10;
  }

  .material-card:hover .action-buttons {
    opacity: 1;
    transform: translateX(0);
  }

  .material-action-button {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface);
    color: var(--text);
    transition: all 0.2s ease;
    padding: 0;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    cursor: pointer;
  }

  .material-action-button:hover {
    transform: scale(1.1);
  }

  .material-action-button.delete:hover {
    background: color-mix(in srgb, #ef4444 15%, var(--surface));
    color: #ef4444;
  }

  .material-action-button.edit:hover:not(:disabled) {
    background: color-mix(in srgb, var(--accent) 15%, var(--surface));
    color: var(--accent);
  }

  .cover {
    position: relative;
    width: 100%;
    aspect-ratio: 3/4;
    background: color-mix(in srgb, var(--accent) 30%, var(--surface));
    border-radius: 8px;
    box-shadow:
      0 0.5rem 1rem rgba(0, 0, 0, 0.1),
      0 0.25rem 0.5rem rgba(0, 0, 0, 0.05);
    transition:
      box-shadow 0.3s ease,
      transform 0.3s ease;
    overflow: hidden;
  }

  .cover.has-image {
    padding: 0;
  }

  .cover-image {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    transition: transform 0.3s ease;
  }

  .material-card:hover .cover-image {
    transform: scale(1.05);
  }

  .material-card:hover .cover {
    box-shadow:
      0 1rem 2rem rgba(0, 0, 0, 0.15),
      0 0.5rem 1rem rgba(0, 0, 0, 0.1);
  }

  .cover-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #ff5555;
    opacity: 0.7;
    gap: 0.5rem;
  }

  .cover-error span {
    font-size: 0.9rem;
  }

  .info {
    padding: 0 0.5rem;
  }

  h3 {
    margin: 0;
    text-align: center;
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    transition: font-size 0.3s ease;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 0.25rem;
    margin-top: 0.25rem;
  }

  .tag {
    background: color-mix(in srgb, var(--accent) 20%, var(--surface));
    padding: 0.25rem 0.5rem;
    border-radius: 8px;
    color: var(--text);
    transition: font-size 0.3s ease;
  }

  .processing-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    border-radius: 8px;
    z-index: 2;
    transition: opacity 0.3s ease;
    backdrop-filter: blur(4px);
  }

  .spinner {
    border-radius: 50%;
    animation: spin 1s linear infinite;
    border: 0.1875rem solid var(--accent);
    border-top: 0.1875rem solid transparent;
    transition:
      width 0.3s ease,
      height 0.3s ease;
  }

  .material-card.selectionMode {
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .material-card.selectionMode:hover {
    transform: scale(1.02);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .material-card.selected .cover {
    border: 2px solid var(--accent);
  }

  .name-input {
    width: 100%;
    font-family: inherit;
    text-align: center;
    padding: 0.25rem;
    background: var(--surface);
    border: 1px solid var(--accent);
    border-radius: 8px;
    color: var(--text);
    transition: font-size 0.3s ease;
  }

  .name-input:focus {
    outline: none;
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 25%, transparent);
  }
</style>
