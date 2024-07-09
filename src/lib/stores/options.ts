import { writable } from 'svelte/store';

interface LLMConfig {
  provider: 'none' | 'ollama';
  // Add more LLM-specific options here in the future
}

interface Options {
  openMaterialsInDefaultApp: boolean;
  simplifiedMaterialView: boolean;
  llmConfig: LLMConfig;
}

const defaultOptions: Options = {
  openMaterialsInDefaultApp: false,
  simplifiedMaterialView: false,
  llmConfig: {
    provider: 'none',
  },
};

function createOptionsStore() {
  const { subscribe, set, update } = writable<Options>(defaultOptions);

  return {
    subscribe,
    setOption: <K extends keyof Options>(key: K, value: Options[K]) => 
      update(opts => ({ ...opts, [key]: value })),
    setLLMOption: <K extends keyof LLMConfig>(key: K, value: LLMConfig[K]) => 
      update(opts => ({ ...opts, llmConfig: { ...opts.llmConfig, [key]: value } })),
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