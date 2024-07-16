import { writable } from 'svelte/store';

export interface OllamaConfig {
  model: string;
  port: number;
}

export interface RunpodConfig {
  apiKey: string;
  serverlessApiId: string;
}

export interface OpenAIConfig {
  apiKey: string;
  model: string;
}

export interface AIConfig {
  provider: 'none' | 'ollama' | 'runpod' | 'openai';
  ollama: OllamaConfig;
  runpod: RunpodConfig;
  openai: OpenAIConfig;
}

export interface VectorDBConfig {
  provider: 'none' | 'chroma' | 'pinecone';
  pineconeApiKey: string;
  pineconeEnvironment: string;
  pineconeIndex: string;
}

export interface Options {
  openMaterialsInDefaultApp: boolean;
  simplifiedMaterialView: boolean;
  aiConfig: AIConfig;
  vectorDBConfig: VectorDBConfig;
}

const defaultOptions: Options = {
  openMaterialsInDefaultApp: false,
  simplifiedMaterialView: false,
  aiConfig: {
    provider: 'none',
    ollama: { model: 'llama2', port: 11434 },
    runpod: { apiKey: '', serverlessApiId: '' },
    openai: { apiKey: '', model: 'text-davinci-003' },
  },
  vectorDBConfig: {
    provider: 'none',
    pineconeApiKey: '',
    pineconeEnvironment: '',
    pineconeIndex: '',
  },
};

function createOptionsStore() {
  const { subscribe, set, update } = writable<Options>(defaultOptions);

  return {
    subscribe,
    setOption: <K extends keyof Options>(key: K, value: Options[K]) => 
      update(opts => ({ ...opts, [key]: value })),
    setAIOption: <K extends keyof AIConfig>(key: K, value: AIConfig[K]) => 
      update(opts => ({ ...opts, aiConfig: { ...opts.aiConfig, [key]: value } })),
    setAIProviderOption: (
      provider: AIConfig['provider'],
      key: string,
      value: any
    ) => 
      update(opts => ({
        ...opts,
        aiConfig: {
          ...opts.aiConfig,
          [provider]: { ...opts.aiConfig[provider], [key]: value }
        }
      })),
    setVectorDBOption: <K extends keyof VectorDBConfig>(key: K, value: VectorDBConfig[K]) => 
      update(opts => ({ ...opts, vectorDBConfig: { ...opts.vectorDBConfig, [key]: value } })),
    resetToDefaults: () => set(defaultOptions),
    loadOptions: () => {
      const savedOptions = localStorage.getItem('studycraft_options');
      if (savedOptions) {
        set(JSON.parse(savedOptions));
      }
    },
    saveOptions: (options: Options) => {
      localStorage.setItem('studycraft_options', JSON.stringify(options));
    },
  };
}

export const options = createOptionsStore();