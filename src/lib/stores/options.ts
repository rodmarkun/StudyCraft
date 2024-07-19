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

export interface CustomPrompts {
  flashcardQuestionPrompt: string;
  flashcardAnswerPrompt: string;
  explanationPrompt: string;
}

export interface Options {
  openMaterialsInDefaultApp: boolean;
  simplifiedMaterialView: boolean;
  aiConfig: AIConfig;
  vectorDBConfig: VectorDBConfig;
  customPrompts: CustomPrompts;
}

const defaultPrompts: CustomPrompts = {
  flashcardQuestionPrompt: `Generate {numberOfCards} unique and diverse flashcard questions based on the following content. Follow these guidelines:

- Each question should test a different key concept or fact from the content.
- Ensure questions cover a wide range of topics within the content.
- Use a variety of question types (e.g., "what", "how", "why", "explain", "compare", "analyze").
- Questions should be clear, concise, and specific.
- Avoid yes/no questions; prefer open-ended or fill-in-the-blank questions.
- Output only the questions, one per line, without numbering, bullets, or additional text.

Content:
{content}

Questions:`,
  flashcardAnswerPrompt: `Provide a concise and accurate answer to the following question based on the given content. Follow these guidelines:

- Answer should be clear, informative, and directly address the question.
- Keep the answer concise, ideally 1-3 sentences.
- Include key details or examples if necessary for clarity.
- If the question asks to fill in a blank, provide the missing term or phrase.
- Output only the answer, without any additional text, explanation, or numbering.

Question: "{question}"

Content:
{content}

Answer:`,
  explanationPrompt: `Please explain the following answer to the question "{question}": {answer}. Provide a rich but brief explanation that is easy to understand. Answer with only the explanation and nothing else. Explanation:`,
};


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
  customPrompts: defaultPrompts,
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
        const parsedOptions = JSON.parse(savedOptions);
        // Merge saved options with default options
        const mergedOptions = {
          ...defaultOptions,
          ...parsedOptions,
          aiConfig: {
            ...defaultOptions.aiConfig,
            ...parsedOptions.aiConfig,
          },
          vectorDBConfig: {
            ...defaultOptions.vectorDBConfig,
            ...parsedOptions.vectorDBConfig,
          },
          customPrompts: {
            ...defaultOptions.customPrompts,
            ...parsedOptions.customPrompts,
          },
        };
        set(mergedOptions);
      }
    },
    saveOptions: (options: Options) => {
      localStorage.setItem('studycraft_options', JSON.stringify(options));
    },
    setCustomPrompt: (key: keyof CustomPrompts, value: string) =>
      update(opts => ({
        ...opts,
        customPrompts: { ...opts.customPrompts, [key]: value }
      })),
    resetCustomPrompts: () =>
      update(opts => ({ ...opts, customPrompts: defaultPrompts })),
  };
}

export const options = createOptionsStore();

// Initialize options when the module is imported
options.loadOptions();