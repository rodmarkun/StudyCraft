<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-shell';

  // Stores
  import { llmStore } from '../../stores/llmStore';

  // Types
  import {type ProviderConfig, type ValidationResult} from '../../logic/Settings/types'

  // Props
  export let handleError = (error) => {};
  export let clearError = () => {};

  // States
  let providers = [];
  let providerConfigs = {};
  let validatingKeys = {};

  // Functions
  async function loadProviders() {
    try {
      const rawProviders: [] = await invoke('get_providers');
      // Sort providers alphabetically
      providers = rawProviders.sort();
      
      providerConfigs = {};
      validatingKeys = {};
      
      // Load existing configs
      for (const provider of providers) {
        try {
          const config: ProviderConfig = await invoke('get_provider_config', { provider });
          providerConfigs[provider] = {
            provider_id: provider,
            api_key: config.api_key || '',
            enabled: config.enabled || false,
            is_configured: config.is_configured || false,
            keyValidated: config.api_key ? true : false
          };
        } catch (error) {
          console.error(`Failed to load config for ${provider}:`, error);
          providerConfigs[provider] = {
            provider_id: provider,
            api_key: '',
            enabled: false,
            is_configured: false,
            keyValidated: false
          };
        }
        validatingKeys[provider] = false;
      }
      
      providerConfigs = { ...providerConfigs };
      clearError();
    } catch (error) {
      console.error('Failed to load providers:', error);
      handleError(`Failed to load providers: ${error.message || error}`);
    }
  }

  async function validateApiKey(provider) {
    const config = providerConfigs[provider];
    
    if (!config.api_key.trim() && provider.toLowerCase() !== 'ollama') {
      alert('Please enter an API key first');
      return;
    }
    
    try {
      validatingKeys[provider] = true;
      validatingKeys = { ...validatingKeys };
      
      const result: ValidationResult = await invoke('validate_api_key_and_fetch_models', {
        provider,
        apiKey: provider.toLowerCase() === 'ollama' ? '' : config.api_key
      });
      
      if (result.valid) {
        config.keyValidated = true;
        providerConfigs = { ...providerConfigs };
        
        llmStore.refresh();
      } else {
        config.keyValidated = false;
        providerConfigs = { ...providerConfigs };
        alert(`${provider === 'ollama' ? 'Connection failed' : 'API key validation failed'}: ${result.error_message || 'Unknown error'}`);
      }
    } catch (error) {
      console.error(`Failed to validate API key for ${provider}:`, error);
      config.keyValidated = false;
      providerConfigs = { ...providerConfigs };
      alert(`Failed to ${provider === 'ollama' ? 'connect' : 'validate API key'}: ${error.message || error}`);
    } finally {
      validatingKeys[provider] = false;
      validatingKeys = { ...validatingKeys };
    }
  }

  async function updateApiKey(provider, apiKey) {
    const config = providerConfigs[provider];
    config.api_key = apiKey;
    
    if (!apiKey.trim()) {
      config.keyValidated = false;
      config.enabled = false;
      await invoke('clear_api_key', { provider });
    } else {
      // Save the API key
      try {
        await invoke('set_api_key', { provider, apiKey });
        clearError();
      } catch (error) {
        console.error(`Failed to save API key for ${provider}:`, error);
        handleError(`Failed to save API key: ${error.message || error}`);
        return;
      }
    }
    
    providerConfigs = { ...providerConfigs };
  }

  async function updateProviderEnabled(provider, enabled) {
    try {
      await invoke('set_provider_enabled', { provider, enabled });
      providerConfigs[provider].enabled = enabled;
      providerConfigs = { ...providerConfigs };
      llmStore.refresh();
      clearError();
    } catch (error) {
      console.error(`Failed to update provider enabled status for ${provider}:`, error);
      handleError(`Failed to update provider status: ${error.message || error}`);
    }
  }

  async function openExternalLink(url: string) {
    try {
      await open(url);
    } catch (error) {
      console.error('Failed to open external link:', error);
      window.open(url, '_blank');
    }
  }

  function isProviderConfigured(provider) {
    const config = providerConfigs[provider];
    return config.keyValidated && config.enabled;
  }

  function canEnableProvider(provider) {
    const config = providerConfigs[provider];
    return config.keyValidated;
  }

  function getProviderApiUrl(provider) {
    const providerName = provider.toLowerCase();
    switch (providerName) {
      case 'openai':
        return 'https://platform.openai.com/api-keys';
      case 'anthropic':
        return 'https://console.anthropic.com/';
      case 'google':
        return 'https://aistudio.google.com/app/apikey';
      case 'mistral':
        return 'https://console.mistral.ai/';
      default:
        return null;
    }
  }

  function getProviderDisplayName(provider) {
    const providerName = provider.toLowerCase();
    switch (providerName) {
      case 'openai':
        return 'OpenAI Platform';
      case 'anthropic':
        return 'Anthropic Console';
      case 'google':
        return 'Google AI Studio';
      case 'mistral':
        return 'Mistral Console';
    }
}

onMount(async () => {
    await loadProviders();
  });
</script>

<div class="section-header">
  <h3>LLM Settings</h3>
</div>

<div class="tab-content">
  <div class="info-banner">
    <div>
      <strong>Provider Configuration</strong>
      <p>Configure API keys and enable providers here. The more providers you have configured, the faster generation will be as the system can distribute requests across multiple services. Model selection is handled in the 'Agents' tab.</p>
    </div>
  </div>
  
  <div class="providers-grid">
    {#each providers as provider (provider)}
      {@const config = providerConfigs[provider]}
      {#if config}
        <div class="provider-card" class:configured={isProviderConfigured(provider)}>
          <div class="provider-header">
            <div class="provider-title">
              <h3>{provider.charAt(0).toUpperCase() + provider.slice(1)}</h3>
              <div class="status-indicator">
                {#if isProviderConfigured(provider)}
                  <div class="status-dot configured" title="Configured and enabled"></div>
                {:else if config.keyValidated}
                  <div class="status-dot partial" title="Valid API key, but disabled"></div>
                {:else}
                  <div class="status-dot unconfigured" title="Not configured"></div>
                {/if}
              </div>
            </div>
            
            <div class="enable-toggle">
              <label class="toggle-label">
                <input 
                  type="checkbox" 
                  checked={config.enabled}
                  on:change={(e) => {
                    const target = e.target as HTMLInputElement;
                    updateProviderEnabled(provider, target.checked);
                  }}
                  disabled={!canEnableProvider(provider)}
                  class="toggle-checkbox"
                />
                <span class="toggle-slider"></span>
              </label>
              <span class="toggle-text" class:disabled={!canEnableProvider(provider)}>
                {config.enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
          </div>
          
          <!-- API Key Section -->
          <div class="config-row">
            <p class="config-label">
              {provider.toLowerCase() === 'ollama' ? 'Connection' : 'API Key'}
            </p>
            <div class="input-group">
              <input 
                type="password"
                value={config.api_key}
                on:input={(e) => {
                  const target = e.target as HTMLInputElement;
                  updateApiKey(provider, target.value);
                }}
                placeholder={provider.toLowerCase() === 'ollama' ? 'Not required for Ollama' : 'Enter your API key'}
                disabled={provider.toLowerCase() === 'ollama'}
                class="config-input"
              />
              <button 
                class="action-button validate"
                on:click={() => validateApiKey(provider)}
                disabled={validatingKeys[provider] || (!config.api_key.trim() && provider.toLowerCase() !== 'ollama')}
                title={provider.toLowerCase() === 'ollama' ? 'Test connection to Ollama' : 'Validate API key'}
              >
                {#if validatingKeys[provider]}
                  <svg class="spinner" viewBox="0 0 24 24" width="16" height="16">
                    <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2" />
                  </svg>
                {:else if provider.toLowerCase() === 'ollama'}
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M8 3H5a2 2 0 0 0-2 2v3M21 8V5a2 2 0 0 0-2-2h-3M16 21h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/>
                  </svg>
                {:else}
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                    <polyline points="22 4 12 14.01 9 11.01"></polyline>
                  </svg>
                {/if}
              </button>
            </div>
            
            {#if config.keyValidated}
              <div class="validation-status success">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                {provider.toLowerCase() === 'ollama' ? 'Connection successful' : 'API key is valid'}
              </div>
            {:else if config.api_key && !validatingKeys[provider]}
              <div class="validation-status pending">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"></circle>
                  <path d="M12 16v-4"></path>
                  <path d="M12 8h.01"></path>
                </svg>
                Click validate to test {provider.toLowerCase() === 'ollama' ? 'connection' : 'API key'}
              </div>
            {/if}
          </div>
          
          <!-- Configuration Help -->
          <div class="config-help">
            {#if provider.toLowerCase() === 'ollama'}
              <p>Make sure Ollama is running locally on port 11434.</p>
            {:else}
              {@const apiUrl = getProviderApiUrl(provider)}
              {@const displayName = getProviderDisplayName(provider)}
              {#if apiUrl}
                <p>Get your API key from <button class="link-button" on:click={() => openExternalLink(apiUrl)}>{displayName}</button></p>
              {:else}
                <p>Get your API key from the {displayName}</p>
              {/if}
            {/if}
          </div>
        </div>
      {/if}
    {/each}
  </div>
</div>

<style>
  .tab-content {
    height: 100%;
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

  .providers-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: var(--space-md);
  }

  .provider-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--border-radius);
    padding: var(--space-md);
    transition: all var(--transition-speed) ease;
    display: flex;
    flex-direction: column;
  }

  .provider-card.configured {
    border-color: var(--success);
    background: color-mix(in srgb, var(--success) 3%, var(--surface));
  }

  .provider-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: var(--space-md);
  }

  .provider-title {
    display: flex;
    align-items: center;
    gap: var(--space-md);
  }

  .provider-title h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
  }

  .status-indicator {
    display: flex;
    align-items: center;
  }

  .status-dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    transition: all var(--transition-speed) ease;
  }

  .status-dot.configured {
    background: var(--success);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--success) 30%, transparent);
  }

  .status-dot.partial {
    background: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 30%, transparent);
  }

  .status-dot.unconfigured {
    background: var(--text-secondary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--text-secondary) 30%, transparent);
  }

  .enable-toggle {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
  }

  .toggle-label {
    display: flex;
    align-items: center;
    cursor: pointer;
  }

  .toggle-checkbox {
    display: none;
  }

  .toggle-slider {
    position: relative;
    width: 44px;
    height: 24px;
    background: var(--border);
    border-radius: 24px;
    transition: all var(--transition-speed) ease;
    border: 2px solid transparent;
  }

  .toggle-slider::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
    transition: all var(--transition-speed) ease;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }

  .toggle-checkbox:checked + .toggle-slider {
    background: var(--success);
    border-color: var(--success);
  }

  .toggle-checkbox:checked + .toggle-slider::after {
    transform: translateX(20px);
  }

  .toggle-checkbox:disabled + .toggle-slider {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-text {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text);
    transition: all var(--transition-speed) ease;
  }

  .toggle-text.disabled {
    opacity: 0.5;
  }

  .config-row {
    margin-bottom: var(--space-md);
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .config-label {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text);
    margin: 0;
  }

  .input-group {
    display: flex;
    gap: var(--space-sm);
    align-items: center;
  }

  .config-input {
    flex: 1;
    background: var(--background);
    border: 1px solid var(--border);
    padding: var(--space-md);
    border-radius: var(--border-radius);
    color: var(--text);
    font-family: var(--font-body);
    font-size: 0.9rem;
    transition: all var(--transition-speed) ease;
  }

  .config-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent) 20%, transparent);
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

  .validation-status {
    display: flex;
    align-items: center;
    gap: var(--space-sm);
    font-size: 0.85rem;
    margin-top: var(--space-xs);
  }

  .validation-status.success {
    color: var(--success);
  }

  .validation-status.pending {
    color: var(--text-secondary);
  }

  .config-help {
    margin-top: var(--space-sm);
    padding-top: var(--space-sm);
    border-top: 1px solid var(--border);
  }

  .config-help p {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .link-button {
    background: none;
    border: none;
    color: var(--accent);
    text-decoration: underline;
    cursor: pointer;
    font-size: inherit;
    font-family: inherit;
    padding: 0;
  }

  .link-button:hover {
    color: var(--accent-hover, var(--accent));
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
    .providers-grid {
      grid-template-columns: 1fr;
    }
  }
</style>