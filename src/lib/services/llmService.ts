export interface LLMResponse {
    response: string;
}

const API_URL = 'http://localhost:11434/api/generate';

export async function getLLMExplanation(question: string, answer: string): Promise<string> {
  const prompt = `Please explain the following answer to the question "${question}": ${answer}. Provide a rich but brief explanation that is easy to understand. Answer with only the explanation and nothing else. Explanation:`;

  try {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        "model": "llama3",
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
    return "Sorry, I couldn't fetch an explanation at this time.";
  }
}