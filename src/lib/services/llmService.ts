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
    const { aiConfig, customPrompts } = currentOptions;

    if (aiConfig.provider === 'none') {
        return "LLM is not configured. Please set up an LLM provider in the options.";
    }

    const prompt = customPrompts.explanationPrompt
        .replace('{question}', question)
        .replace('{answer}', answer);
        
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

export async function generateFlashcards(content: string, numberOfCards: number): Promise<Array<{question: string, answer: string}>> {
    // Remove the PDF cleaning step here, as it's now done before calling this function
    const chunkSize = 2000;
    const chunks = splitContent(content, chunkSize);
    const flashcards: Array<{question: string, answer: string}> = [];
    const usedQuestions = new Set<string>();

    const currentOptions = getCurrentOptions();
    const { customPrompts } = currentOptions;

    for (const chunk of chunks) {
        const remainingCards = numberOfCards - flashcards.length;
        if (remainingCards <= 0) break;

        const chunkPrompt = customPrompts.flashcardQuestionPrompt
            .replace('{numberOfCards}', Math.min(remainingCards, 5).toString())
            .replace('{content}', chunk);

        const questionsResponse = await getLLMResponse(chunkPrompt);
        const questions = questionsResponse.split('\n')
            .map(q => cleanResponse(q))
            .filter(q => q.trim() !== '');

        for (const question of questions) {
            if (usedQuestions.has(question.toLowerCase())) continue;

            const answerPrompt = customPrompts.flashcardAnswerPrompt
                .replace('{question}', question)
                .replace('{content}', chunk);

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

export async function generateTestQuestions(content: string, numberOfQuestions: number): Promise<Array<{question: string, options: string[], correctOptionIndex: number}>> {
    const chunkSize = 4000;
    const chunks = splitContent(content, chunkSize);
    const questions: Array<{question: string, options: string[], correctOptionIndex: number}> = [];
    const usedQuestions = new Set<string>();

    const currentOptions = getCurrentOptions();
    const { customPrompts } = currentOptions;

    for (const chunk of chunks) {
        const remainingQuestions = numberOfQuestions - questions.length;
        if (remainingQuestions <= 0) break;

        const questionsPrompt = customPrompts.testQuestionPrompt
            .replace('{numberOfQuestions}', Math.min(remainingQuestions, 3).toString())
            .replace('{content}', chunk);

        const questionsResponse = await getLLMResponse(questionsPrompt);
        const generatedQuestions = questionsResponse.split('\n')
            .map(q => cleanResponse(q))
            .filter(q => q.trim() !== '');

        for (const questionText of generatedQuestions) {
            if (usedQuestions.has(questionText.toLowerCase())) continue;

            const falseOptionsPrompt = customPrompts.testFalseOptionsPrompt
                .replace('{question}', questionText)
                .replace('{content}', chunk);

            const falseOptionsResponse = await getLLMResponse(falseOptionsPrompt);
            const falseOptions = falseOptionsResponse.split('\n')
                .map(o => cleanResponse(o))
                .filter(o => o.trim() !== '')
                .slice(0, 3);

            const trueOptionPrompt = customPrompts.testTrueOptionPrompt
                .replace('{question}', questionText)
                .replace('{content}', chunk);

            const trueOptionResponse = await getLLMResponse(trueOptionPrompt);
            const trueOption = cleanResponse(trueOptionResponse);

            const options = [...new Set([...falseOptions, trueOption])];
            if (options.length < 4) continue; // Skip if we don't have enough unique options

            let correctOptionIndex = options.indexOf(trueOption);

            // Shuffle the options
            for (let i = options.length - 1; i > 0; i--) {
                const j = Math.floor(Math.random() * (i + 1));
                [options[i], options[j]] = [options[j], options[i]];
                if (i === correctOptionIndex) correctOptionIndex = j;
                else if (j === correctOptionIndex) correctOptionIndex = i;
            }

            questions.push({ question: questionText, options, correctOptionIndex });
            usedQuestions.add(questionText.toLowerCase());

            if (questions.length >= numberOfQuestions) break;
        }
    }

    return questions;
}

function splitContent(content: string, chunkSize: number): string[] {
    const chunks: string[] = [];
    for (let i = 0; i < content.length; i += chunkSize) {
        chunks.push(content.slice(i, i + chunkSize));
    }
    return chunks;
}