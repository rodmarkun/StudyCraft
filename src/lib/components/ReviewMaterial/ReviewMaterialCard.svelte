<script>
  import { onMount } from 'svelte';
  import { spring } from 'svelte/motion';
  import { invoke } from '@tauri-apps/api/core';
  import { save } from '@tauri-apps/plugin-dialog';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import "../../styles/variables.css";

  // Assets
  import flashcardDeckIcon from '../../../assets/flashcard_deck_icon.png';
  import testIcon from '../../../assets/test_icon.png';
  
  // Props
  export let material;
  export let zoomLevel = 1;
  export let onDelete = (id) => {};
  export let onReview = (material) => {};
  export let onEdit = (material) => {};
  export let onEditTest = (material) => {};
  export let onNameUpdated = (data) => {};

  // State
  let fontSize = `${Math.max(0.75, 0.5 + (zoomLevel * 0.5))}rem`;
  let tagSize = `${0.8 * zoomLevel}rem`;
  let buttonSize = Math.max(16, Math.floor(16 * zoomLevel));
  $: transformScale = $size >= 1 ? 
    `scale(${1 + ($size - 1) * 0.15})` : 
    `scale(${1 - (1 - $size) * 0.1})`;
  
  let showDeleteConfirm = false;
  let isEditing = false;
  let editingName = '';
  let nameElement;
  let isHovering = false;
  let cardCounts = null;
  let isLoadingCounts = false;
  let isExporting = false;
  const size = spring(1, {
    stiffness: 0.2,
    damping: 0.7
  });
  
  // Reactivity
  $: isFlashcardDeck = (
    material.type === 'flashcard_deck' || 
    (material.review_material_type && (
      String(material.review_material_type) === 'FlashcardDeck' ||
      String(material.review_material_type).toLowerCase() === 'flashcarddeck' ||
      material.review_material_type === 'flashcard_deck'
    ))
  );
  $: isTest = (
    material.type === 'test' || 
    (material.review_material_type && (
      String(material.review_material_type) === 'Test' ||
      String(material.review_material_type).toLowerCase() === 'test' ||
      material.review_material_type === 'test'
    ))
  );
  $: materialId = (() => {
    if (material.id) return String(material.id);
    if (material.review_material_id) return String(material.review_material_id);
    return undefined;
  })();
  
  $: materialName = material.name || material.review_material_name;  
  $: isAllCaughtUp = cardCounts && cardCounts.total_count > 0 && cardCounts.pending_count === 0;
  $: {
    size.set(zoomLevel);
    fontSize = `${Math.max(0.75, 0.5 + (zoomLevel * 0.5))}rem`;
    tagSize = `${0.8 * zoomLevel}rem`;
    buttonSize = Math.max(16, Math.floor(16 * zoomLevel));
  }
  $: if (isFlashcardDeck && materialId) {
    loadCardCounts();
  }
  
  async function loadCardCounts() {
    if (isLoadingCounts) {
      return;
    }
    try {
      isLoadingCounts = true;
      const deckIdString = String(materialId);
      
      const counts = await invoke('get_flashcard_counts', { 
        deckId: deckIdString
      });
      
      cardCounts = counts;
    } catch (error) {
      console.error('Failed to load card counts:', error);
      console.error('Error details:', {
        message: error.message,
        stack: error.stack
      });
      
      cardCounts = {
        total_count: material.cards_count || 0,
        pending_count: 0,
        new_count: 0,
        learning_count: 0,
        review_count: 0
      };
      console.log('Using fallback cardCounts:', cardCounts);
    } finally {
      isLoadingCounts = false;
    }
  }
  
  async function handleExportToAnki(event) {
    event.stopPropagation();
    
    if (isExporting) return;
    
    try {
      isExporting = true;
      
      const ankiContent = await invoke('export_flashcard_deck_to_anki', {
        deckId: String(materialId)
      });
      
      const defaultFileName = `${materialName.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.txt`;
      
      const filePath = await save({
        defaultPath: defaultFileName,
        filters: [{
          name: 'Anki Text File',
          extensions: ['txt']
        }]
      });
      
      if (filePath) {
        await writeTextFile(filePath, ankiContent);
        console.log('Anki export saved to:', filePath);
      }
    } catch (error) {
      console.error('Failed to export to Anki:', error);
      alert('Failed to export deck to Anki format. Please try again.');
    } finally {
      isExporting = false;
    }
  }
  
  function startEditing(event) {
    event.stopPropagation();
    isEditing = true;
    editingName = materialName;
    setTimeout(() => nameElement?.focus(), 50);
  }
  
  async function finishEditing() {
    if (editingName.trim() && editingName !== materialName) {
      try {
        if (isFlashcardDeck) {
          await invoke('update_flashcard_deck_name', {
            deckId: String(materialId),
            name: editingName.trim()
            });
          
          if (material.name) {
            material.name = editingName.trim();
          } else {
            material.review_material_name = editingName.trim();
          }
          
          onNameUpdated({ id: materialId, newName: editingName.trim() });
        }
      } catch (error) {
        console.error('Failed to update material name:', error);
        editingName = materialName;
      }
    }
    isEditing = false;
  }

  function formatLastReview(lastReview) {
    if (!lastReview || lastReview === null || lastReview === undefined) {
      return 'Never';
    }
    
    try {
      const date = new Date(lastReview);
      
      if (isNaN(date.getTime())) {
        return 'Never';
      }
      
      const now = new Date();
      const diffTime = now.getTime() - date.getTime();
      const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));
      
      if (diffDays < 0) {
        return 'Recently';
      } else if (diffDays === 0) {
        return 'Today';
      } else if (diffDays === 1) {
        return 'Yesterday';
      } else if (diffDays < 7) {
        return `${diffDays} days ago`;
      } else if (diffDays < 30) {
        const weeks = Math.floor(diffDays / 7);
        return `${weeks} week${weeks > 1 ? 's' : ''} ago`;
      } else if (diffDays < 365) {
        const months = Math.floor(diffDays / 30);
        return `${months} month${months > 1 ? 's' : ''} ago`;
      } else {
        return date.toLocaleDateString();
      }
    } catch (error) {
      console.error('Error formatting date:', error, lastReview);
      return 'Unknown';
    }
  }
  
  function getCardCountDisplay() {
    if (!cardCounts) {
      if (isFlashcardDeck) {
        const totalCards = material.cards_count || 0;
        console.log('Using fallback for flashcard deck, totalCards:', totalCards);
        return totalCards > 0 ? `${totalCards} cards` : 'No cards';
      } else {
        const totalQuestions = material.questions_count || 0;
        console.log('Using fallback for test, totalQuestions:', totalQuestions);
        return totalQuestions > 0 ? `${totalQuestions} questions` : 'No questions';
      }
    }
    
    const { total_count, pending_count } = cardCounts;
    console.log('Using cardCounts:', { total_count, pending_count });
    
    if (total_count === 0) {
      return 'No cards';
    }
    
    if (pending_count === 0) {
      return `${total_count} cards`;
    }
    
    if (pending_count === total_count) {
      return `${total_count} cards pending`;
    }
    
    return `${pending_count}/${total_count} cards pending`;
  }
  
  function getCardCountColor() {
    if (!cardCounts || !isFlashcardDeck) {
      return 'var(--text-secondary)';
    }
    
    const { total_count, pending_count } = cardCounts;
    
    if (total_count === 0) {
      return 'var(--text-secondary)'; 
    }
    
    if (pending_count === 0) {
      return 'var(--success)'; 
    }
    
    return 'var(--error)';
  }
  
  function handleKeydown(event) {
    if (event.key === 'Enter') {
      event.preventDefault();
      finishEditing();
    } else if (event.key === 'Escape') {
      isEditing = false;
    }
  }
  
  function handleDelete() {
    onDelete(materialId);
  }
  
  function handleReview(event) {
    event.stopPropagation();
    onReview(material);
  }

  function handleEdit(event) {
    event.stopPropagation();
    
    if (isFlashcardDeck) {
      onEdit(material);
    } else if (isTest) {
      onEditTest(material);
    } else {
      console.log('Edit functionality not yet implemented for material type:', material.type || material.review_material_type);
    }
  }

  function handleCardClick(event) {
    if (event.target.closest('.rmc-action-button') || 
        event.target.tagName === 'H3' ||
        event.target.closest('.rmc-name-input') ||
        isEditing) {
      return;
    }
    
    handleReview(event);
  }

  function handleCardKeydown(event) {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      handleCardClick(event);
    }
  }

  onMount(() => {
    console.log('ReviewMaterialCard mounted with material:', material);
  });
</script>

<div 
  class="rmc-material-card" 
  class:rmc-flashcard-deck={isFlashcardDeck} 
  class:rmc-test={!isFlashcardDeck}
  class:rmc-all-caught-up={isAllCaughtUp}
  class:rmc-hovering={isHovering}
  style="transform: {transformScale} rotate({isHovering ? '2deg' : '0deg'});"
  role="button"
  tabindex="0"
  on:click={handleCardClick}
  on:keydown={handleCardKeydown}
  on:mouseenter={() => isHovering = true}
  on:mouseleave={() => isHovering = false}
>
  <div class="rmc-action-buttons">
    <button 
      class="rmc-action-button rmc-delete"
      title="Delete material"
      aria-label="Delete material"
      on:click={(e) => {
        e.stopPropagation();
        showDeleteConfirm = true;
      }}
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
      >
        <path d="M3 6h18"></path>
        <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
        <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
      </svg>
    </button>
    <button 
      class="rmc-action-button rmc-edit"
      title="Edit material"
      aria-label="Edit material"
      on:click={handleEdit}
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
      >
        <path d="M17 3a2.85 2.85 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3Z"></path>
      </svg>
    </button>
        {#if isFlashcardDeck}
      <button 
        class="rmc-action-button rmc-export"
        title="Export to Anki"
        aria-label="Export to Anki"
        disabled={isExporting}
        on:click={handleExportToAnki}
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
        >
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
          <polyline points="7 10 12 15 17 10"></polyline>
          <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
      </button>
    {/if}
  </div>

  <div class="rmc-cover">
    <div class="rmc-card-icon">
      {#if isFlashcardDeck}
        <img 
          src={flashcardDeckIcon} 
          alt="Flashcard deck" 
          width="64" 
          height="64"
          class="rmc-flashcard-icon"
        />
      {:else}
        <img 
          src={testIcon} 
          alt="Test" 
          width="48" 
          height="48"
          class="rmc-flashcard-icon"
        />
      {/if}
    </div>
    
    <div 
      class="rmc-play-button-container" 
      class:rmc-visible={isHovering} 
      role="button"
      tabindex="0"
      on:click={handleReview}
      on:keydown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          handleReview(e);
        }
      }}
    >
      <button class="rmc-play-button" title="Start review">
        <svg 
          xmlns="http://www.w3.org/2000/svg" 
          width="48" 
          height="48" 
          viewBox="0 0 24 24" 
          fill="none" 
          stroke="currentColor" 
          stroke-width="2" 
          stroke-linecap="round" 
          stroke-linejoin="round"
        >
          <polygon points="5 3 19 12 5 21 5 3"></polygon>
        </svg>
        <span>Review</span>
      </button>
    </div>
  </div>

  <div class="rmc-info">
    {#if isEditing}
      <input
        bind:this={nameElement}
        bind:value={editingName}
        on:blur={finishEditing}
        on:keydown={handleKeydown}
        class="rmc-name-input"
        style="font-size: {fontSize}"
      />
    {:else}
      <h3 
        style="font-size: {fontSize}"
        on:dblclick={startEditing}
      >
        {materialName}
      </h3>
    {/if}

    <div class="rmc-card-meta">
      <div class="meta-item rmc-card-count" style="color: {getCardCountColor()}">
        <span>{getCardCountDisplay()}</span>
      </div>
      <div class="meta-item">
        <span>Last: {formatLastReview(material.last_review || material.last_review)}</span>
      </div>
    </div>
    
    {#if material.tags && material.tags.length > 0}
      <div class="tags">
        {#each material.tags as tag}
          <span class="tag" style="font-size: {tagSize}">{tag}</span>
        {/each}
      </div>
    {/if}
  </div>
</div>

{#if showDeleteConfirm}
  <div class="modal-backdrop" on:click={() => showDeleteConfirm = false} role="button" tabindex="0" aria-label="Close Dialog">
    <div class="modal-content" on:click|stopPropagation>
      <h4>Delete Material</h4>
      <p>Are you sure you want to delete "{materialName}"? This action cannot be undone.</p>
      <div class="modal-actions">
        <button 
          class="cancel-button" 
          on:click={() => showDeleteConfirm = false}
        >
          Cancel
        </button>
        <button 
          class="confirm-button" 
          on:click={() => {
            handleDelete();
            showDeleteConfirm = false;
          }}
        >
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
.rmc-material-card {
 position: relative;
 width: 100%;
 cursor: pointer;
 display: flex;
 flex-direction: column;
 gap: var(--space-sm);
 transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
 will-change: transform, opacity;
}

.rmc-material-card.rmc-hovering {
 z-index: 5;
}

.rmc-material-card.rmc-flashcard-deck .rmc-cover {
 background: linear-gradient(135deg, 
   var(--accent), 
   color-mix(in srgb, var(--accent) 80%, var(--surface))
 );
}

.rmc-material-card.rmc-test .rmc-cover {
 background: linear-gradient(135deg, 
   var(--accent-second), 
   color-mix(in srgb, var(--accent-second) 80%, var(--surface))
 );
}

.rmc-action-buttons {
 position: absolute;
 top: var(--space-sm);
 right: var(--space-sm);
 display: flex;
 flex-direction: column;
 gap: var(--space-xs);
 opacity: 0;
 transform: translateX(10px);
 transition: all 0.3s ease;
 z-index: 10;
}

.rmc-material-card:hover .rmc-action-buttons {
 opacity: 1;
 transform: translateX(0);
}

.rmc-action-button {
 width: 32px;
 height: 32px;
 border-radius: 50%;
 border: none;
 display: flex;
 align-items: center;
 justify-content: center;
 background: var(--surface);
 color: var(--text);
 cursor: pointer;
 transition: all 0.2s ease;
 padding: 0;
 box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.rmc-action-button:hover {
 transform: scale(1.1);
}

.rmc-action-button:disabled {
 opacity: 0.5;
 cursor: not-allowed;
}

.rmc-action-button:disabled:hover {
 transform: none;
}

.rmc-action-button.rmc-export:hover {
 background: color-mix(in srgb, var(--accent) 15%, var(--surface));
 color: var(--accent);
}

.rmc-action-button.rmc-delete:hover {
 background: color-mix(in srgb, var(--error) 15%, var(--surface));
 color: var(--error);
}

.rmc-action-button.rmc-edit:hover {
 background: color-mix(in srgb, var(--accent) 15%, var(--surface));
 color: var(--accent);
}

.rmc-cover {
 position: relative;
 width: 100%;
 aspect-ratio: 3/4;
 border-radius: var(--border-radius);
 box-shadow: 
     0 0.5rem 1rem rgba(0, 0, 0, 0.1),
     0 0.25rem 0.5rem rgba(0, 0, 0, 0.05);
 transition: box-shadow 0.3s ease, transform 0.3s ease;
 overflow: hidden;
 display: flex;
 align-items: center;
 justify-content: center;
}

.rmc-material-card:hover .rmc-cover {
 box-shadow: 
     0 1rem 2rem rgba(0, 0, 0, 0.15),
     0 0.5rem 1rem rgba(0, 0, 0, 0.1);
}

.rmc-card-icon {
 color: rgba(255, 255, 255, 0.9);
 transition: transform 0.3s ease, opacity 0.3s ease;
}

.rmc-material-card:hover .rmc-card-icon {
 transform: scale(1.1);
 opacity: 0.2;
}

.rmc-play-button-container {
 position: absolute;
 top: 0;
 left: 0;
 width: 100%;
 height: 100%;
 display: flex;
 align-items: center;
 justify-content: center;
 opacity: 0;
 transition: opacity 0.3s ease;
 background: rgba(0, 0, 0, 0.2);
 border-radius: var(--border-radius);
}

.rmc-play-button-container.rmc-visible {
 opacity: 1;
}

.rmc-play-button {
 display: flex;
 flex-direction: column;
 align-items: center;
 gap: var(--space-xs);
 padding: var(--space-md);
 background: var(--accent);
 border: none;
 border-radius: 50%;
 color: white;
 cursor: pointer;
 transition: all 0.2s ease;
 box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
 width: 70px;
 height: 70px;
 justify-content: center;
 transform: scale(0.9);
}

.rmc-play-button:hover {
 transform: scale(1);
 box-shadow: 0 6px 16px rgba(0, 0, 0, 0.3);
 background: color-mix(in srgb, var(--accent) 80%, white);
}

.rmc-play-button svg {
 margin-left: 4px;
 width: 32px;
 height: 32px;
}

.rmc-play-button span {
 position: absolute;
 bottom: -30px;
 font-weight: 600;
 text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
 font-size: 1.1rem;
 letter-spacing: 0.03em;
}

.rmc-info {
 padding: 0 var(--space-sm);
 display: flex;
 flex-direction: column;
 gap: var(--space-sm);
}

.rmc-info h3 {
 margin: 0;
 text-align: center;
 color: var(--text);
 overflow: hidden;
 text-overflow: ellipsis;
 display: -webkit-box;
 -webkit-line-clamp: 2;
 -webkit-box-orient: vertical;
 transition: font-size 0.3s ease;
 font-weight: 600;
}

.rmc-name-input {
 width: 100%;
 font-family: inherit;
 text-align: center;
 padding: var(--space-xs);
 background: var(--surface);
 border: 1px solid var(--accent);
 border-radius: var(--border-radius);
 color: var(--text);
 transition: font-size 0.3s ease;
 font-weight: 600;
}

.rmc-name-input:focus {
 outline: none;
 box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 25%, transparent);
}

.rmc-card-meta {
 display: flex;
 flex-direction: column;
 align-items: center;
 gap: var(--space-xs);
 color: var(--text-secondary);
 font-size: 0.85rem;
}

.rmc-card-count {
 font-weight: 600;
 padding: 2px 8px;
 border-radius: 12px;
 background: color-mix(in srgb, currentColor 10%, transparent);
 transition: all 0.3s ease;
}

.meta-item {
 display: flex;
 align-items: center;
 gap: var(--space-xs);
}

.meta-item span {
 font-size: 0.8rem;
}

.tags {
 display: flex;
 flex-wrap: wrap;
 justify-content: center;
 gap: 0.25rem;
 margin-top: var(--space-xs);
}

.tag {
 background: color-mix(in srgb, var(--accent) 20%, var(--surface));
 color: var(--accent);
 padding: 0.25rem 0.5rem;
 border-radius: var(--border-radius);
 transition: font-size 0.3s ease;
 font-size: 0.75rem;
 font-weight: 500;
 border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
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
 backdrop-filter: blur(4px);
}

.modal-content {
 background: var(--surface);
 padding: var(--space-lg);
 border-radius: var(--border-radius);
 max-width: 400px;
 width: 90%;
 box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
 border: 1px solid var(--border);
}

.modal-content h4 {
 margin: 0;
 margin-bottom: var(--space-md);
 font-family: var(--font-heading);
 font-size: 1.25rem;
 font-weight: 600;
 color: var(--text);
}

.modal-content p {
 margin-bottom: var(--space-lg);
 color: var(--text-secondary);
 line-height: 1.5;
}

.modal-actions {
 display: flex;
 justify-content: flex-end;
 gap: var(--space-md);
}

.cancel-button, .confirm-button {
 padding: var(--space-sm) var(--space-lg);
 border-radius: var(--border-radius);
 border: none;
 cursor: pointer;
 font-family: var(--font-body);
 font-weight: 500;
 transition: all 0.2s ease;
}

.cancel-button {
 background: var(--background);
 border: 1px solid var(--border);
 color: var(--text);
}

.confirm-button {
 background: var(--error);
 color: white;
}

.cancel-button:hover {
 background: color-mix(in srgb, var(--text) 5%, var(--background));
 border-color: var(--text-secondary);
}

.confirm-button:hover {
 background: color-mix(in srgb, var(--error) 85%, black);
}

.rmc-flashcard-icon {
 opacity: 0.9;
 transition: transform 0.3s ease, opacity 0.3s ease;
 filter: brightness(0) invert(1);
 object-fit: contain;
}

.rmc-material-card:hover .rmc-flashcard-icon {
 transform: scale(1.1);
 opacity: 0.2;
}

.rmc-action-button:focus,
.rmc-play-button:focus,
.rmc-name-input:focus,
.cancel-button:focus,
.confirm-button:focus {
 outline: 2px solid var(--accent);
 outline-offset: 2px;
}

.rmc-material-card:focus {
 outline: 2px solid var(--accent);
 outline-offset: 4px;
 border-radius: var(--border-radius);
}

@media (max-width: 768px) {
 .rmc-play-button {
   width: 60px;
   height: 60px;
 }
 
 .rmc-play-button svg {
   width: 24px;
   height: 24px;
 }
 
 .rmc-play-button span {
   bottom: -25px;
   font-size: 0.9rem;
 }
 
 .rmc-card-meta {
   font-size: 0.8rem;
 }
 
 .meta-item span {
   font-size: 0.75rem;
 }
}

@media (max-width: 480px) {
 .rmc-action-buttons {
   top: var(--space-xs);
   right: var(--space-xs);
 }
 
 .rmc-action-button {
   width: 28px;
   height: 28px;
 }
 
 .rmc-play-button {
   width: 50px;
   height: 50px;
 }
 
 .rmc-play-button svg {
   width: 20px;
   height: 20px;
 }
 
 .rmc-play-button span {
   bottom: -20px;
   font-size: 0.8rem;
 }
 
 .rmc-card-meta {
   font-size: 0.75rem;
   gap: var(--space-xs);
 }
 
 .meta-item span {
   font-size: 0.7rem;
 }
 
 .rmc-info {
   padding: 0 var(--space-xs);
   gap: var(--space-xs);
 }
 
 .tag {
   font-size: 0.7rem;
   padding: 0.2rem 0.4rem;
 }
}

@media (prefers-contrast: high) {
 .rmc-cover {
   border: 2px solid var(--text);
 }
 
 .rmc-action-button {
   border: 2px solid var(--border);
 }
 
 .modal-content {
   border: 2px solid var(--border);
 }
}

@media (prefers-reduced-motion: reduce) {
 .rmc-material-card {
   transition: none;
 }
 
 .rmc-action-buttons {
   transition: none;
 }
 
 .rmc-play-button-container {
   transition: none;
 }
 
 .rmc-action-button:hover,
 .rmc-play-button:hover {
   transform: none;
 }
}
</style>