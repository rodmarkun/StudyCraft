<script lang="ts">
  // Icons
  // TODO - change for studycraft's new svg icons
  import {
    SmartphoneCharging as SmartphoneChargingIcon,
    ListCheck as ListCheckIcon,
  } from "lucide-svelte";

  // Props
  export let show = false;
  export let position = "below"; // 'below' | 'above'

  // Callbacks
  export let onCreateFlashcardDeck = () => {};
  export let onCreateTest = () => {};
</script>

{#if show}
  <div class="create-menu" class:phantom-menu={position === "above"}>
    <button class="create-menu-item" on:click={onCreateFlashcardDeck}>
      <SmartphoneChargingIcon size={18} />
      Flashcard Deck
    </button>
    <button class="create-menu-item" on:click={onCreateTest}>
      <ListCheckIcon size={18} />
      Test
    </button>
  </div>
{/if}

<style>
  .create-menu {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: white;
    border: 1px solid var(--border-color, #e1e5e9);
    border-radius: 8px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    z-index: 1000;
    overflow: hidden;
    min-width: 180px;
    margin-top: 4px;
  }
  .create-menu-item {
    width: 100%;
    padding: 12px 16px;
    border: none;
    background: white;
    color: var(--text-primary, #2c3e50);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 8px;
    transition: background-color 0.2s ease;
    text-align: left;
  }

  .create-menu-item:hover {
    background-color: var(--hover-bg, #f8f9fa);
  }

  .create-menu-item:first-child {
    border-top-left-radius: 8px;
    border-top-right-radius: 8px;
  }

  .create-menu-item:last-child {
    border-bottom-left-radius: 8px;
    border-bottom-right-radius: 8px;
  }

  .create-menu-item + .create-menu-item {
    border-top: 1px solid var(--border-color, #e1e5e9);
  }
</style>
