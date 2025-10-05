<script>
  import { fade, fly } from "svelte/transition";

  // Props
  export let show = false;
</script>

{#if show}
  <div
    class="shortcuts-overlay"
    transition:fade={{ duration: 200 }}
    on:click|self={() => (show = false)}
  >
    <div class="shortcuts-panel" transition:fly={{ y: 20, duration: 300 }}>
      <div class="panel-header">
        <h3>Keyboard Shortcuts</h3>
        <button class="close-button" on:click={() => (show = false)}>×</button>
      </div>

      <div class="shortcuts-content">
        <div class="shortcut-group">
          <h4>Zoom Controls</h4>
          <div class="shortcut-item">
            <div class="keys">
              <span class="key">Ctrl</span> <span class="plus">+</span>
              <span class="key">+</span>
            </div>
            <div class="action">Zoom in</div>
          </div>
          <div class="shortcut-item">
            <div class="keys">
              <span class="key">Ctrl</span> <span class="plus">+</span>
              <span class="key">-</span>
            </div>
            <div class="action">Zoom out</div>
          </div>
          <div class="shortcut-item">
            <div class="keys">
              <span class="key">Ctrl</span> <span class="plus">+</span>
              <span class="key">0</span>
            </div>
            <div class="action">Reset zoom to 100%</div>
          </div>
          <div class="shortcut-item">
            <div class="keys">
              <span class="key">Ctrl</span> <span class="plus">+</span>
              <span class="key">Scroll</span>
            </div>
            <div class="action">Zoom in/out</div>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .shortcuts-overlay {
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

  .shortcuts-panel {
    background: var(--surface);
    border-radius: var(--border-radius);
    width: 90%;
    max-width: 500px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
    overflow: hidden;
    border: 1px solid var(--border);
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid var(--border);
  }

  .shortcuts-content {
    padding: 1rem;
    max-height: 70vh;
    overflow-y: auto;
  }

  .shortcut-group {
    margin-bottom: 1.5rem;
  }

  .shortcut-group h4 {
    margin: 0 0 0.75rem 0;
    color: var(--text);
    font-size: 1rem;
    font-weight: 500;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
  }

  .shortcut-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .key {
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    border: 1px solid var(--border);
    padding: 0.25rem 0.5rem;
    border-radius: var(--border-radius);
    font-size: 0.85rem;
    color: var(--text);
    min-width: 1.5rem;
    text-align: center;
  }

  .plus {
    color: var(--text-secondary);
  }

  .action {
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  h3 {
    font-size: 1.2rem;
    margin-bottom: var(--space-md);
    font-family: var(--font-heading);
    margin: 0;
    font-weight: 500;
    color: var(--text);
  }

  .close-button {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 1.5rem;
    cursor: pointer;
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    transition: all 0.2s ease;
  }

  .close-button:hover {
    background: color-mix(in srgb, var(--accent) 10%, transparent);
    color: var(--text);
  }
</style>
