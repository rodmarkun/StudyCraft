// src/lib/services/llmService.ts

import { get } from 'svelte/store';
import { options } from '../stores/options';
import { cleanPDFContent } from '../utils/fileUtils'

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

    try {
        let response;
        switch (aiConfig.provider) {
            case 'ollama':
                response = await getOllamaResponse(prompt, aiConfig.ollama);
                break;
            case 'runpod':
                response = await getRunpodResponse(prompt, aiConfig.runpod);
                break;
            case 'openai':
                response = await getOpenAIResponse(prompt, aiConfig.openai);
                break;
            default:
                throw new Error('Unsupported LLM provider');
        }
        return response;
    } catch (error) {
        console.error('Error fetching explanation:', error);
        return `There was an error with the LLM. Please check if your current configuration is correct: Provider - ${aiConfig.provider}`;
    }
}

async function getLLMResponse(prompt: string): Promise<string> {
    const currentOptions = getCurrentOptions();
    const { aiConfig } = currentOptions;

    if (aiConfig.provider === 'none') {
        throw new Error("LLM is not configured. Please set up an LLM provider in the options.");
    }

    try {
        switch (aiConfig.provider) {
            case 'ollama':
                return await getOllamaResponse(prompt, aiConfig.ollama);
            case 'runpod':
                return await getRunpodResponse(prompt, aiConfig.runpod);
            case 'openai':
                return await getOpenAIResponse(prompt, aiConfig.openai);
            default:
                throw new Error('Unsupported LLM provider');
        }
    } catch (error) {
        console.error('Error getting LLM response:', error);
        throw error;
    }
}

async function getOllamaResponse(prompt: string, config: { model: string; port: number }): Promise<string> {
    const API_URL = `http://localhost:${config.port}/api/generate`;
    const response = await fetch(API_URL, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ model: config.model, prompt, stream: false }),
    });
    if (!response.ok) throw new Error('Ollama API request failed');
    const data: LLMResponse = await response.json();
    return data.response;
}

async function getRunpodResponse(prompt: string, config: { apiKey: string; serverlessApiId: string }): Promise<string> {
    const API_URL = `https://api.runpod.ai/v2/${config.serverlessApiId}/runsync`;
    const response = await fetch(API_URL, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${config.apiKey}`
        },
        body: JSON.stringify({
            input: {
                prompt,
                max_new_tokens: 300,
                temperature: 0.7,
            }
        }),
    });
    if (!response.ok) throw new Error('Runpod API request failed');
    const data = await response.json();
    return data.output;
}

async function getOpenAIResponse(prompt: string, config: { apiKey: string; model: string }): Promise<string> {
    const API_URL = 'https://api.openai.com/v1/chat/completions';
    const response = await fetch(API_URL, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${config.apiKey}`
        },
        body: JSON.stringify({
            model: config.model,
            messages: [
                {
                    role: "system",
                    content: "You are a helpful assistant that provides brief explanations."
                },
                {
                    role: "user",
                    content: prompt
                }
            ],
            max_tokens: 300,
            temperature: 0.7,
        }),
    });
    
    if (!response.ok) {
        const errorData = await response.json();
        console.error('OpenAI API error:', errorData);
        throw new Error(`OpenAI API request failed: ${response.status} ${response.statusText}`);
    }
    
    const data = await response.json();
    return data.choices[0].message.content.trim();
}

function cleanResponse(response: string): string {
    // Remove numbering and bullet points
    return response.replace(/^\s*(\d+\.|-|\*)\s*/gm, '').trim();
}

export async function generateFlashcards(content: string, numberOfCards: number, materialType: string): Promise<Array<{question: string, answer: string}>> {
    let processedContent = content;

    if (materialType === 'pdf') {
        processedContent = await cleanPDFContent(content);
    }

    const chunkSize = 2000;
    const chunks = splitContent(processedContent, chunkSize);
    const flashcards: Array<{question: string, answer: string}> = [];
    const usedQuestions = new Set<string>();

    for (const chunk of chunks) {
        const remainingCards = numberOfCards - flashcards.length;
        if (remainingCards <= 0) break;

        const chunkPrompt = `Generate ${Math.min(remainingCards, 5)} unique and diverse flashcard questions based on the following content. Follow these guidelines:

- Each question should test a different key concept or fact from the content.
- Ensure questions cover a wide range of topics within the content.
- Use a variety of question types (e.g., "what", "how", "why", "explain", "compare", "analyze").
- Questions should be clear, concise, and specific.
- Avoid yes/no questions; prefer open-ended or fill-in-the-blank questions.
- Output only the questions, one per line, without numbering, bullets, or additional text.

Examples:
What is the primary purpose of [specific concept]?
How does [process/phenomenon] impact [related system]?
Why is [particular aspect] significant in the context of [broader topic]?
Explain the relationship between [concept A] and [concept B].
Compare and contrast [element 1] with [element 2].
Analyze the implications of [specific event or idea].

Content:
${chunk}

Questions:`;

        const questionsResponse = await getLLMResponse(chunkPrompt);
        const questions = questionsResponse.split('\n')
            .map(q => cleanResponse(q))
            .filter(q => q.trim() !== '');

        for (const question of questions) {
            if (usedQuestions.has(question.toLowerCase())) continue;

            const answerPrompt = `Provide a concise and accurate answer to the following question based on the given content. Follow these guidelines:

- Answer should be clear, informative, and directly address the question.
- Keep the answer concise, ideally 1-3 sentences.
- Include key details or examples if necessary for clarity.
- If the question asks to fill in a blank, provide the missing term or phrase.
- Output only the answer, without any additional text, explanation, or numbering.

Question: "${question}"

Content:
${chunk}

Answer:`;

            const answer = cleanResponse(await getLLMResponse(answerPrompt));

            if (answer !== '') {
                flashcards.push({ question, answer });
                usedQuestions.add(question.toLowerCase());
            }

            if (flashcards.length >= numberOfCards) break;
        }
    }

    return flashcards;
}

function splitContent(content: string, chunkSize: number): string[] {
    const chunks: string[] = [];
    for (let i = 0; i < content.length; i += chunkSize) {
        chunks.push(content.slice(i, i + chunkSize));
    }
    return chunks;
}