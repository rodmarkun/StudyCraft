import { writable } from 'svelte/store';

interface AIConfig {
  provider: 'none' | 'ollama';
  ollamaModel: string;
  ollamaPort: number;
}

interface VectorDBConfig {
  provider: 'none' | 'chroma' | 'pinecone';
  pineconeApiKey: string;
  pineconeEnvironment: string;
  pineconeIndex: string;
}

interface Options {
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
    ollamaModel: 'llama2',
    ollamaPort: 11434,
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