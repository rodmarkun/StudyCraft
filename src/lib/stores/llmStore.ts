import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

interface ProviderConfig {
  provider_id: string;
  api_key: string;
  enabled: boolean;
  is_configured: boolean;
}

interface LLMState {
  hasConfiguredProvider: boolean;
  availableProviders: string[];
  configuredProviders: string[];
  isLoading: boolean;
  lastChecked: Date | null;
}

const initialState: LLMState = {
  hasConfiguredProvider: false,
  availableProviders: [],
  configuredProviders: [],
  isLoading: true,
  lastChecked: null,
};

function createLLMStore() {
  const { subscribe, set, update } = writable<LLMState>(initialState);

  async function get(): Promise<LLMState> {
    let state: LLMState = initialState;
    subscribe(s => { state = s; })();
    return { ...state };
  }

  async function checkProviders(): Promise<void> {
    update(state => ({ ...state, isLoading: true }));

    try {
      // Get all available providers
      const providers = await invoke<string[]>('get_providers');
      const configuredProviders: string[] = [];

      for (const provider of providers) {
        try {
          const config = await invoke<ProviderConfig>('get_provider_config', { provider });
          if (config.enabled && config.is_configured) {
            configuredProviders.push(provider);
          }
        } catch (error) {
          console.error(`Failed to check ${provider} configuration:`, error);
        }
      }
      
      set({
        hasConfiguredProvider: configuredProviders.length > 0,
        availableProviders: providers,
        configuredProviders,
        isLoading: false,
        lastChecked: new Date(),
      });
    } catch (error) {
      console.error('Failed to check LLM providers:', error);
      set({
        hasConfiguredProvider: false,
        availableProviders: [],
        configuredProviders: [],
        isLoading: false,
        lastChecked: new Date(),
      });
    }
  }

  async function initialize(): Promise<void> {
    try {
      await checkProviders();
    } catch (error) {
      console.error("Error initializing LLM store:", error);
    }
  }

  async function refresh(): Promise<void> {
    await checkProviders();
  }

  function canUseLLMFeature(): boolean {
    const state = get();
    return !state.isLoading && state.hasConfiguredProvider;
  }

  // Get user-friendly status message
  function getStatusMessage(): string {
    const state = get();
    
    if (state.isLoading) {
      return 'Checking AI provider configuration...';
    }
    
    if (!state.hasConfiguredProvider) {
      return 'No AI provider configured. Configure one in Settings to use AI features.';
    }
    
    const count = state.configuredProviders.length;
    return `${count} AI provider${count > 1 ? 's' : ''} configured: ${state.configuredProviders.join(', ')}`;
  }

  return {
    subscribe,
    initialize,
    checkProviders,
    refresh,
    canUseLLMFeature,
    getStatusMessage,
    get,
  };
}

export const llmStore = createLLMStore();

// Export types for use in components
export type { LLMState, ProviderConfig };