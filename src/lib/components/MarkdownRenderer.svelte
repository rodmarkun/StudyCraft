<!-- src/lib/components/MarkdownRenderer.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';

  export let content: string = '';

  let renderedContent: string = '';

  onMount(() => {
    marked.use({
      breaks: true,
      gfm: true,
      headerIds: true,
      headerPrefix: 'heading-'
    });
  });

  $: {
    if (content) {
      const rawHtml = marked.parse(content);
      renderedContent = DOMPurify.sanitize(rawHtml, { 
        USE_PROFILES: { html: true },
        ADD_TAGS: ['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'ul', 'ol', 'li']
      });
    } else {
      renderedContent = '';
    }
  }
</script>

<div class="markdown-body prose dark:prose-invert max-w-none">
  {@html renderedContent}
</div>

<style>
  .markdown-body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
    font-size: 16px;
    line-height: 1.5;
    word-wrap: break-word;
  }

  /* Header styles */
  :global(.markdown-body h1) { font-size: 2em; margin-top: 0.67em; margin-bottom: 0.67em; font-weight: bold; }
  :global(.markdown-body h2) { font-size: 1.5em; margin-top: 0.83em; margin-bottom: 0.83em; font-weight: bold; }
  :global(.markdown-body h3) { font-size: 1.17em; margin-top: 1em; margin-bottom: 1em; font-weight: bold; }
  :global(.markdown-body h4) { font-size: 1em; margin-top: 1.33em; margin-bottom: 1.33em; font-weight: bold; }
  :global(.markdown-body h5) { font-size: .83em; margin-top: 1.67em; margin-bottom: 1.67em; font-weight: bold; }
  :global(.markdown-body h6) { font-size: .67em; margin-top: 2.33em; margin-bottom: 2.33em; font-weight: bold; }

  /* List styles */
  :global(.markdown-body ul), :global(.markdown-body ol) {
    padding-left: 2em;
    margin-top: 0;
    margin-bottom: 16px;
  }

  :global(.markdown-body li) {
    margin-top: 0.25em;
    margin-bottom: 0.25em;
  }

  :global(.markdown-body ul ul), :global(.markdown-body ul ol), 
  :global(.markdown-body ol ul), :global(.markdown-body ol ol) {
    margin-top: 0;
    margin-bottom: 0;
  }

  :global(.markdown-body li > p) {
    margin-top: 16px;
  }

  :global(.markdown-body li + li) {
    margin-top: 0.25em;
  }

  /* Keep other existing styles */
  .markdown-body p { margin-top: 1em; margin-bottom: 1em; }
  .markdown-body a { color: #0366d6; text-decoration: none; }
  .markdown-body a:hover { text-decoration: underline; }
  .markdown-body strong { font-weight: bold; }
  .markdown-body em { font-style: italic; }
  .markdown-body code { padding: 0.2em 0.4em; margin: 0; font-size: 85%; background-color: rgba(27,31,35,0.05); border-radius: 3px; }
  .markdown-body pre { padding: 16px; overflow: auto; font-size: 85%; line-height: 1.45; background-color: #f6f8fa; border-radius: 3px; }
  .markdown-body blockquote { padding: 0 1em; color: #6a737d; border-left: 0.25em solid #dfe2e5; }
  .markdown-body table { border-spacing: 0; border-collapse: collapse; }
  .markdown-body table th, .markdown-body table td { padding: 6px 13px; border: 1px solid #dfe2e5; }
</style>