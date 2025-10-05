  export interface ProviderConfig {
    provider_id?: string;
    api_key?: string;
    enabled?: boolean;
    available_models?: string[]; 
    is_configured?: boolean;
  }

  export interface ValidationResult {
    valid: boolean;
    error_message?: string;
  }

  export interface AgentModelConfig {
    agent_type?: string;
    provider_models?: Record<string, string>;
  }