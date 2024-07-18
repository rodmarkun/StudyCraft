import TurndownService from 'turndown';

export async function scrapeWebsite(url: string): Promise<string> {
  try {
    // Request the HTML content from the main process
    const html = await window.electronAPI.fetchWebContent(url);
    
    // Extract main content (this is a simple example and might need adjustment)
    const mainContentMatch = html.match(/<main.*?>([\s\S]*?)<\/main>/i) 
      || html.match(/<article.*?>([\s\S]*?)<\/article>/i)
      || [null, html];
    
    const mainContent = mainContentMatch[1] || html;

    // Convert HTML to Markdown
    const turndownService = new TurndownService();
    const markdown = turndownService.turndown(mainContent);

    return markdown;
  } catch (error) {
    console.error('Error scraping website:', error);
    throw new Error(`Failed to scrape website: ${error.message}`);
  }
}