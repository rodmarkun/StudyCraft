<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';

  // Stores
  import { llmStore } from '../../stores/llmStore';

  // Types
  import {type ProviderConfig, type AgentModelConfig} from '../../logic/Settings/types';

  // Callbacks
  export let handleError = (error) => {};
  export let clearError = () => {};

  // State
  let agentSettings = null;
  let originalAgentSettings = null;
  let agentSettingsChanged = false;
  let availableModels = {};
  let enabledProviders = [];

  // Store subscription
  let unsubscribe;

  // Functions
  async function loadAgentSettings() {
    try {
      agentSettings = await invoke('get_agent_settings');
      originalAgentSettings = JSON.parse(JSON.stringify(agentSettings));
      agentSettingsChanged = false;
      clearError();
    } catch (error) {
      console.error('Failed to load agent settings:', error);
      handleError(`Failed to load agent settings: ${error.message || error}`);
    }
  }

  async function loadAvailableModels() {
    try {
      const rawProviders: [] = await invoke('get_enabled_providers');
      enabledProviders = rawProviders.sort();
      availableModels = {};
      
      for (const provider of enabledProviders) {
        try {
          console.log("Getting provider config for ", provider);
          const config: ProviderConfig = await invoke('get_provider_config', { provider });
          console.log("Config: ", config);
          if (config.available_models && Array.isArray(config.available_models)) {
            availableModels[provider] = config.available_models;
          }
        } catch (error) {
          console.warn(`Failed to load models for ${provider}:`, error);
        }
      }
      
      availableModels = { ...availableModels };
      enabledProviders = [...enabledProviders];
    } catch (error) {
      console.error('Failed to load available models:', error);
    }
  }

  async function loadCurrentAgentModels() {
    if (!agentSettings) return;
    
    try {
      // Load current models for each agent and provider
      const agentModelConfigs: AgentModelConfig[] = await invoke('get_all_agent_model_configs');
      
      // Update the agentSettings with current model selections
      for (const agentConfig of agentModelConfigs) {
        const agentType = agentConfig.agent_type;
        if (!agentSettings.agent_models_per_provider[agentType]) {
          agentSettings.agent_models_per_provider[agentType] = {};
        }
        
        // Merge provider models into agent settings
        for (const [provider, model] of Object.entries(agentConfig.provider_models)) {
          agentSettings.agent_models_per_provider[agentType][provider] = model;
        }
      }
      
      // Force reactivity update
      agentSettings = { ...agentSettings };
    } catch (error) {
      console.error('Failed to load current agent models:', error);
    }
  }

  async function refreshModelsAndSettings() {
    await loadAvailableModels();
    await loadCurrentAgentModels();
  }

  async function saveAgentSettings() {
    if (!agentSettings || !agentSettingsChanged) return;
    
    try {
      await invoke('save_agent_settings', { settings: agentSettings });
      originalAgentSettings = JSON.parse(JSON.stringify(agentSettings));
      agentSettingsChanged = false;
      clearError();
    } catch (error) {
      console.error('Failed to save agent settings:', error);
      handleError(`Failed to save agent settings: ${error.message || error}`);
    }
  }

  async function resetAgentSettings() {
    if (confirm('Are you sure you want to reset all agent settings to their default values?')) {
      try {
        await invoke('reset_agent_settings_to_defaults');
        await loadAgentSettings();
        clearError();
      } catch (error) {
        console.error('Failed to reset agent settings:', error);
        handleError(`Failed to reset agent settings: ${error.message || error}`);
      }
    }
  }

  function updateAgentSetting(settingType, agentType, value, provider = null) {
    if (!agentSettings) return;

    switch (settingType) {
      case 'prompt':
        agentSettings.agent_prompts[agentType] = value;
        break;
      case 'max_tokens':
        agentSettings.agent_max_tokens[agentType] = parseInt(value) || 3000;
        break;
      case 'temperature':
        agentSettings.agent_temperature[agentType] = parseFloat(value) || 0.3;
        break;
      case 'model':
        if (provider) {
          if (!agentSettings.agent_models_per_provider[agentType]) {
            agentSettings.agent_models_per_provider[agentType] = {};
          }
          agentSettings.agent_models_per_provider[agentType][provider] = value;
        }
        break;
    }

    agentSettings = { ...agentSettings };
    agentSettingsChanged = JSON.stringify(agentSettings) !== JSON.stringify(originalAgentSettings);
  }

  function getAgentTypeDisplayName(agentType) {
    // Convert snake_case or PascalCase to readable format
    return agentType
      .replace(/([A-Z])/g, ' $1')
      .replace(/_/g, ' ')
      .toLowerCase()
      .split(' ')
      .map(word => word.charAt(0).toUpperCase() + word.slice(1))
      .join(' ');
  }

  onMount(async () => {
    await loadAgentSettings();
    await loadAvailableModels();
    await loadCurrentAgentModels();

    // Subscribe to store changes to refresh models when providers are updated
    unsubscribe = llmStore.subscribe(async (state) => {
      if (!state.isLoading && state.lastChecked) {
        // Refresh models when the store updates (e.g., after provider validation)
        await refreshModelsAndSettings();
      }
    });
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
  });
</script>

<div class="tab-content">
  {#if agentSettings}
    <div class="settings-section">
      <div class="section-header">
        <h3>Agent Configuration</h3>
        <div class="section-actions">
          <button 
            class="action-button save"
            class:changed={agentSettingsChanged}
            on:click={saveAgentSettings}
            disabled={!agentSettingsChanged}
            title="Save changes"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
              <polyline points="17 21 17 13 7 13 7 21"></polyline>
              <polyline points="7 3 7 8 15 8"></polyline>
            </svg>
            <span>{agentSettingsChanged ? 'Save Changes' : 'Saved'}</span>
          </button>
          <button 
            class="action-button danger"
            on:click={resetAgentSettings}
            title="Reset to defaults"
          >
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="1 4 1 10 7 10"></polyline>
              <polyline points="23 20 23 14 17 14"></polyline>
              <path d="M20.49 9A9 9 0 0 0 5.64 5.64L1 10m22 4-4.64 4.36A9 9 0 0 1 3.51 15"></path>
            </svg>
            <span>Reset</span>
          </button>
        </div>
      </div>

      <div class="info-banner">
        <div>
          <strong>Agent Configuration</strong>
          <p>Configure how different AI agents behave, including their prompts, model preferences, and generation parameters. Each agent type serves a specific purpose in the study workflow. Beware that changing the prompts might end up in unexpected behavior.</p>
        </div>
      </div>

      <div class="agents-grid">
        {#each Object.keys(agentSettings.agent_prompts).sort() as agentType (agentType)}
          <div class="agent-card">
            <div class="agent-header">
              <h4>{getAgentTypeDisplayName(agentType)}</h4>
            </div>

            <div class="agent-settings">
              <!-- Model Selection per Provider -->
              <div class="setting-item">
                <label class="setting-label">
                  Model Selection
                  <span class="setting-description">Choose models for each enabled provider</span>
                </label>
                {#if enabledProviders.length > 0}
                  <div class="models-grid">
                    {#each enabledProviders as provider}
                      {#if availableModels[provider] && availableModels[provider].length > 0}
                        <div class="model-selection">
                          <label class="model-provider">{provider}</label>
                          <select 
                            class="setting-input model-select"
                            value={agentSettings.agent_models_per_provider[agentType]?.[provider] || ''}
                            on:change={(e) => updateAgentSetting('model', agentType, (e.target as HTMLSelectElement).value, provider)}
                          >
                            <option value="">Select model</option>
                            {#each availableModels[provider] as model}
                              <option value={model}>{model}</option>
                            {/each}
                          </select>
                        </div>
                      {/if}
                    {/each}
                  </div>
                {:else}
                  <div class="no-models-message">
                    <p>No providers enabled. Configure providers in the LLMs tab first.</p>
                  </div>
                {/if}
              </div>

              <!-- Parameters -->
              <div class="setting-row">
                <div class="setting-item">
                  <label class="setting-label">
                    Max Tokens
                    <span class="setting-description">Maximum response length</span>
                  </label>
                  <input 
                    type="number" 
                    class="setting-input"
                    min="100"
                    max="8000"
                    step="100"
                    value={agentSettings.agent_max_tokens[agentType] || 3000}
                    on:input={(e) => updateAgentSetting('max_tokens', agentType, (e.target as HTMLInputElement).value)}
                    on:blur={saveAgentSettings}
                  />
                </div>

                <div class="setting-item">
                  <label class="setting-label">
                    Temperature
                    <span class="setting-description">Response creativity (0.0 - 1.0)</span>
                  </label>
                  <input 
                    type="number" 
                    class="setting-input"
                    min="0.0"
                    max="1.0"
                    step="0.1"
                    value={agentSettings.agent_temperature[agentType] || 0.3}
                    on:input={(e) => updateAgentSetting('temperature', agentType, (e.target as HTMLInputElement).value)}
                    on:blur={saveAgentSettings}
                  />
                </div>
              </div>

              <!-- Prompt Template -->
              <div class="setting-item">
                <label class="setting-label">
                  Prompt Template
                  <span class="setting-description">The system prompt that defines the agent's behavior</span>
                </label>
                <textarea 
                  class="setting-textarea"
                  rows="8"
                  value={agentSettings.agent_prompts[agentType] || ''}
                  on:input={(e) => updateAgentSetting('prompt', agentType, (e.target as HTMLTextAreaElement).value)}
                  on:blur={saveAgentSettings}
                  placeholder="Enter the system prompt for this agent..."
                ></textarea>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="loading-state">
      <p>Loading agent settings...</p>
    </div>
  {/if}
</div>

<style>
  .tab-content {
    height: 100%;
  }

  .settings-section {
    margin-bottom: var(--space-xl);
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: var(--space-lg);
    padding-bottom: var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .section-header h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .section-actions {
    display: flex;
    gap: var(--space-sm);
  }

  .action-button {
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

  .action-button:hover:not(:disabled) {
    filter: brightness(1.1);
    transform: translateY(-1px);
  }

  .action-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }

  .action-button.save.changed {
    background: var(--success);
    animation: pulse 2s infinite;
  }

  .action-button.danger {
    background: var(--error);
    color: white;
  }

  .info-banner {
    display: flex;
    align-items: flex-start;
    gap: var(--space-sm);
    background: color-mix(in srgb, var(--info) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--info) 30%, transparent);
    border-radius: var(--border-radius);
    padding: var(--space-md);
    margin-bottom: var(--space-lg);
    color: var(--info);
  }

  .info-banner svg {
    flex-shrink: 0;
    margin-top: 2px;
  }

  .info-banner strong {
    display: block;
    color: var(--text);
    margin-bottom: var(--space-xs);
  }

  .info-banner p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .agents-grid {
    display: grid;
    gap: var(--space-lg);
  }

  .agent-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-md);
    transition: all var(--transition-speed) ease;
  }

  .agent-card:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .agent-header {
    margin-bottom: var(--space-lg);
    padding-bottom: var(--space-md);
    border-bottom: 1px solid var(--border);
  }

  .agent-header h4 {
    margin: 0;
    font-size: 1.2rem;
    font-weight: 600;
    color: var(--text);
  }

  .agent-settings {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .setting-item {
    margin-bottom: var(--space-md);
  }

  .setting-item:last-child {
    margin-bottom: 0;
  }

  .setting-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--space-md);
  }

  .setting-label {
    display: block;
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text);
    margin-bottom: var(--space-sm);
  }

  .setting-description {
    display: block;
    font-size: 0.8rem;
    font-weight: 400;
    color: var(--text-secondary);
    margin-top: var(--space-xs);
  }

  .models-grid {
    display: grid;
    gap: var(--space-md);
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  }

  .model-selection {
    display: flex;
    flex-direction: column;
    gap: var(--space-xs);
  }

  .model-provider {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary);
    text-transform: capitalize;
  }

  .setting-input {
    width: 100%;
    background: var(--background);
    border: 1px solid var(--border);
    padding: var(--space-md);
    border-radius: var(--border-radius);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 0.9rem;
    transition: all var(--transition-speed) ease;
  }

  .setting-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .model-select {
    background: var(--background);
    border: 1px solid var(--border);
    padding: var(--space-sm) var(--space-md);
    border-radius: var(--border-radius);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 0.9rem;
    transition: all var(--transition-speed) ease;
    cursor: pointer;
  }

  .model-select:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .model-select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .no-models-message {
    background: color-mix(in srgb, var(--warning) 10%, transparent);
    border: 1px solid color-mix(in srgb, var(--warning) 30%, transparent);
    border-radius: var(--border-radius);
    padding: var(--space-md);
    text-align: center;
  }

  .no-models-message p {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .setting-textarea {
    width: 100%;
    background: var(--background);
    border: 1px solid var(--border);
    padding: var(--space-md);
    border-radius: var(--border-radius);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 0.9rem;
    line-height: 1.5;
    resize: vertical;
    min-height: 120px;
    transition: all var(--transition-speed) ease;
  }

  .setting-textarea:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent);
  }

  .setting-textarea::placeholder {
    color: var(--text-secondary);
  }

  .loading-state {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: var(--space-xl);
    color: var(--text-secondary);
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

  @media (max-width: 768px) {
    .models-grid {
      grid-template-columns: 1fr;
    }
    
    .agent-settings {
      gap: var(--space-md);
    }
    
    .setting-textarea {
      min-height: 100px;
    }
    
    .section-header {
      flex-direction: column;
      align-items: stretch;
      gap: var(--space-md);
    }
    
    .section-actions {
      justify-content: stretch;
    }
    
    .section-actions .action-button {
      flex: 1;
    }

    .setting-row {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 480px) {
    .agent-card {
      padding: var(--space-md);
    }
    
    .models-grid {
      gap: var(--space-sm);
    }
    
    .setting-textarea {
      font-size: 0.85rem;
      min-height: 80px;
    }
  }
</style>