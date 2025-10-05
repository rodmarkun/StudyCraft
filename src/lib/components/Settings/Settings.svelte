<script>
  import { fade, fly } from 'svelte/transition';

  //Icons
  import { Settings as SettingsIcon } from 'lucide-svelte';
  import { Sparkles as SparklesIcon } from 'lucide-svelte';
  import { Bot as BotIcon } from 'lucide-svelte';
  import { Heart as HeartIcon } from 'lucide-svelte';

  // Components
  import GeneralSettings from './GeneralSettings.svelte';
  import LLMSettings from './LLMSettings.svelte';
  import AgentSettings from './AgentSettings.svelte';
  import SupportSettings from './SupportSettings.svelte';

  // Props
  export let isOpen = false;
  export let onClose = () => {};

  // State
  let activeTab = 'general';
  let saveError = '';

  // Functions
  function closeSettings() {
    onClose();
  }
  
  function handleKeydown(event) {
    if (event.key === 'Escape') {
      closeSettings();
    }
  }

  function setActiveTab(tab) {
    activeTab = tab;
  }

  function handleError(error) {
    saveError = error;
  }

  function clearError() {
    saveError = '';
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
<div 
  class="settings-backdrop" 
  transition:fade={{ duration: 200 }} 
  on:click={closeSettings}
  role="dialog"
  aria-modal="true"
  aria-labelledby="settings-title"
  tabindex="-1"
>
  <div 
    class="settings-container" 
    transition:fly={{ y: -20, duration: 200 }} 
    on:click|stopPropagation
    role="document"
    tabindex="0"
  >
    <div class="settings-header">
      <h2 id="settings-title">Settings</h2>
      <button 
        class="close-button" 
        on:click={closeSettings}
        aria-label="Close settings"
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
    
    <!-- Settings Tabs -->
    <div class="settings-tabs">
      <button 
        class="tab-button" 
        class:active={activeTab === 'general'}
        on:click={() => setActiveTab('general')}
      >
        <SettingsIcon size={18}/>
        General
      </button>
      
      <div class="tab-separator"></div>
      
      <button 
        class="tab-button" 
        class:active={activeTab === 'llms'}
        on:click={() => setActiveTab('llms')}
      >
        <SparklesIcon size={18}/>
        LLMs
      </button>
      
      <div class="tab-separator"></div>
      
      <button 
        class="tab-button" 
        class:active={activeTab === 'agents'}
        on:click={() => setActiveTab('agents')}
      >
        <BotIcon size={18}/>
        Agents
      </button>
      
      <div class="tab-separator"></div>
      
      <button 
        class="tab-button" 
        class:active={activeTab === 'support'}
        on:click={() => setActiveTab('support')}
      >
        <HeartIcon size={18}/>
        Support
      </button>
    </div>
    
    <div class="settings-content">
      <div class="tab-content" class:active={activeTab === 'general'}>
        <GeneralSettings {handleError} {clearError} />
      </div>
      <div class="tab-content" class:active={activeTab === 'llms'}>
        <LLMSettings {handleError} {clearError} />
      </div>
      <div class="tab-content" class:active={activeTab === 'agents'}>
        <AgentSettings {handleError} {clearError} />
      </div>
      <div class="tab-content" class:active={activeTab === 'support'}>
        <SupportSettings />
      </div>
      
      {#if saveError}
        <div class="save-status error">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
          {saveError}
        </div>
      {/if}
    </div>
  </div>
</div>
{/if}

<style>
  :root {
    --background: #ffffff;
    --surface: #f8f9fa;
    --border: #e9ecef;
    --text: #212529;
    --text-secondary: #6c757d;
    --accent: #0066cc;
    --error: #dc3545;
    --space-xs: 4px;
    --space-sm: 8px;
    --space-md: 12px;
    --space-lg: 20px;
    --space-xl: 32px;
    --border-radius: 8px;
    --transition-speed: 0.2s;
    --font-body: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      --background: #1a1a1a;
      --surface: #2d2d2d;
      --border: #404040;
      --text: #ffffff;
      --text-secondary: #a0a0a0;
      --accent: #4da6ff;
    }
  }

  .settings-backdrop {
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
    padding: var(--space-lg);
  }

  .settings-container {
    background: var(--background);
    border-radius: var(--border-radius);
    width: 100%;
    max-width: 1000px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04);
  }

  .settings-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-md) var(--space-lg);
    border-bottom: 1px solid var(--border);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text);
  }

  .close-button {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: var(--space-sm);
    border-radius: var(--border-radius);
    transition: all var(--transition-speed) ease;
  }

  .close-button:hover {
    background: var(--surface);
    color: var(--text);
  }

  .settings-tabs {
    display: flex;
    justify-content: center;
    align-items: center;
    border-bottom: 1px solid var(--border);
    background: var(--background);
    padding: 2px;
    gap: var(--space-xs);
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

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-lg);
  }

  .tab-content {
    display: none;
  }

  .tab-content.active {
    display: block;
    animation: fadeIn 0.2s ease-in-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .save-status {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 0.9rem;
    padding: var(--space-md);
    border-radius: var(--border-radius);
    margin-bottom: var(--space-lg);
    position: relative;
    z-index: 10;
  }

  .save-status.error {
    color: var(--error);
    background: color-mix(in srgb, var(--error) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--error) 30%, transparent);
  }

  @media (max-width: 768px) {
    .settings-container {
      max-width: 100%;
      max-height: 95vh;
    }
    
    .settings-tabs {
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
  }
</style>