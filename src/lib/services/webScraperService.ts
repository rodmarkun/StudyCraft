import TurndownService from 'turndown';



export async function scrapeWebsite(url: string): Promise<string> {
  try {
    const html = await window.electronAPI.fetchWebContent(url);
    let cleanHtml = html
      .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
      .replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, '')
      .replace(/<nav\b[^<]*(?:(?!<\/nav>)<[^<]*)*<\/nav>/gi, '')
      .replace(/<header\b[^<]*(?:(?!<\/header>)<[^<]*)*<\/header>/gi, '')
      .replace(/<footer\b[^<]*(?:(?!<\/footer>)<[^<]*)*<\/footer>/gi, '')
      .replace(/<aside\b[^<]*(?:(?!<\/aside>)<[^<]*)*<\/aside>/gi, '');

    // Extract main content
    let mainContent = extractContent(cleanHtml, 'main') ||
                      extractContent(cleanHtml, 'article') ||
                      extractContent(cleanHtml, 'div[id="mw-content-text"]') || // Wikipedia-specific
                      extractContent(cleanHtml, 'div[class="mw-parser-output"]') || // Wikipedia-specific
                      cleanHtml;

    const turndownService = new TurndownService({
      headingStyle: 'atx',
      codeBlockStyle: 'fenced'
    });

    let markdown = turndownService.turndown(mainContent);

    // Clean up the markdown
    markdown = markdown
      // Remove "From Wikipedia, the free encyclopedia"
      .replace(/^From Wikipedia,?\s?the free encyclopedia\s*/i, '')
      // Remove image descriptions
      .replace(/!\[.*?\]\(.*?\)/g, '')
      // Remove citations
      .replace(/\[citation needed\]/gi, '')
      .replace(/\[\d+\]/g, '')
      // Remove edit links
      .replace(/\[edit\]/gi, '')
      // Remove "Contents" or "Table of contents" headers
      .replace(/^(?:Contents|Table of contents)$\n[-=]+\n\n/gim, '')
      // Remove pronunciation guides
      .replace(/\(\/.*?\/\s+.*?\)/g, '')
      // Remove multiple consecutive newlines
      .replace(/\n{3,}/g, '\n\n')
      // Remove lines that are just punctuation (often artifacts from removing other elements)
      .replace(/^\s*[^\w\s]+\s*$/gm, '')
      // Trim each line
      .split('\n').map(line => line.trim()).join('\n')
      // Final trim of the entire text
      .trim();

    return markdown;
  } catch (error) {
    console.error('Error scraping website:', error);
    throw new Error(`Failed to scrape website: ${error.message}`);
  }
}

function extractContent(html: string, selector: string): string {
  const regex = new RegExp(`<${selector}\\b[^>]*>(.*?)<\/${selector.split('[')[0]}>`, 'is');
  const match = html.match(regex);
  return match ? match[1] : '';
}

export async function getWebsiteTitle(url: string): Promise<string> {
    try {
      const response = await window.electronAPI.fetchWebContent(url);
      const titleMatch = response.match(/<title>(.*?)<\/title>/i);
      return titleMatch ? titleMatch[1].trim() : url;
    } catch (error) {
      console.error('Error fetching website title:', error);
      return url;
    }
  }