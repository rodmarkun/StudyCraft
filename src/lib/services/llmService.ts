import { get } from 'svelte/store';
import { options } from '../stores/options';

export interface LLMResponse {
    response: string;
}

function getCurrentOptions() {
    return get(options);
}

export async function getLLMExplanation(question: string, answer: string): Promise<string> {
    const currentOptions = getCurrentOptions();
    const { aiConfig } = currentOptions;

    if (aiConfig.provider === 'none') {
        return "LLM is not configured. Please set up an LLM provider in the options.";
    }

    const prompt = `Please explain the following answer to the question "${question}": ${answer}. Provide a rich but brief explanation that is easy to understand. Answer with only the explanation and nothing else. Explanation:`;

    if (aiConfig.provider === 'ollama') {
        const API_URL = `http://localhost:${aiConfig.ollamaPort}/api/generate`;

        try {
            const response = await fetch(API_URL, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    "model": aiConfig.ollamaModel,
                    "prompt": prompt,
                    "stream": false
                }),
            });

            if (!response.ok) {
                throw new Error('Network response was not ok');
            }

            const data: LLMResponse = await response.json();
            return data.response;
        } catch (error) {
            console.error('Error fetching explanation:', error);
            return "There was an error with the LLM. Please check if your current configuration is correct: Provider - " + aiConfig.provider + ", Model - " + aiConfig.ollamaModel + ", Port - " + aiConfig.ollamaPort;
        }
    }
    return "Selected LLM provider is not supported.";
}