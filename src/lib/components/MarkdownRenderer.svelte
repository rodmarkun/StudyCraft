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
  :root {
    --md-text-color: #333;
    --md-link-color: #0366d6;
    --md-code-bg: rgba(27,31,35,0.05);
    --md-border-color: #dfe2e5;
    --md-blockquote-color: #6a737d;
    --md-table-border-color: #dfe2e5;
    --md-table-bg-alt: #f6f8fa;
  }

  :global(.dark) {
    --md-text-color: #e0e0e0;
    --md-link-color: #58a6ff;
    --md-code-bg: rgba(240,246,252,0.15);
    --md-border-color: #30363d;
    --md-blockquote-color: #8b949e;
    --md-table-border-color: #30363d;
    --md-table-bg-alt: rgba(255,255,255,0.05);
  }

  .markdown-body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue', sans-serif;
    font-size: 16px;
    line-height: 1.6;
    word-wrap: break-word;
    color: var(--md-text-color);
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  /* Header styles */
  :global(.markdown-body h1, .markdown-body h2, .markdown-body h3, .markdown-body h4, .markdown-body h5, .markdown-body h6) {
    margin-top: 24px;
    margin-bottom: 16px;
    font-weight: 600;
    line-height: 1.25;
    color: var(--md-text-color);
  }
  :global(.markdown-body h1, .markdown-body h2) {
    border-bottom: 1px solid var(--md-border-color);
    padding-bottom: .3em;
  }
  :global(.markdown-body h1) { font-size: 2em; }
  :global(.markdown-body h2) { font-size: 1.5em; }
  :global(.markdown-body h3) { font-size: 1.25em; }
  :global(.markdown-body h4) { font-size: 1em; }
  :global(.markdown-body h5) { font-size: 0.875em; }
  :global(.markdown-body h6) { font-size: 0.85em; color: var(--md-blockquote-color); }

  /* List styles */
  :global(.markdown-body ul, .markdown-body ol) {
    padding-left: 2em;
    margin-top: 0;
    margin-bottom: 16px;
  }
  :global(.markdown-body ul) { list-style-type: disc; }
  :global(.markdown-body ol) { list-style-type: decimal; }
  :global(.markdown-body li) { word-wrap: break-all; }
  :global(.markdown-body li > p) { margin-top: 16px; }
  :global(.markdown-body li + li) { margin-top: 0.25em; }

  /* Paragraph and text styles */
  :global(.markdown-body p) {
    margin-top: 0;
    margin-bottom: 16px;
  }
  :global(.markdown-body a) {
    color: var(--md-link-color);
    text-decoration: none;
  }
  :global(.markdown-body a:hover) { text-decoration: underline; }
  :global(.markdown-body strong) { font-weight: 600; }
  :global(.markdown-body em) { font-style: italic; }

  /* Code styles */
  :global(.markdown-body code) {
    padding: 0.2em 0.4em;
    margin: 0;
    font-size: 85%;
    background-color: var(--md-code-bg);
    border-radius: 3px;
    font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, Courier, monospace;
  }
  :global(.markdown-body pre) {
    padding: 16px;
    overflow: auto;
    font-size: 85%;
    line-height: 1.45;
    background-color: var(--md-code-bg);
    border-radius: 3px;
  }

  /* Blockquote styles */
  :global(.markdown-body blockquote) {
    padding: 0 1em;
    color: var(--md-blockquote-color);
    border-left: 0.25em solid var(--md-border-color);
    margin: 0 0 16px 0;
  }

  /* Table styles */
  :global(.markdown-body table) {
    display: block;
    width: 100%;
    overflow: auto;
    border-spacing: 0;
    border-collapse: collapse;
    margin-bottom: 16px;
  }
  :global(.markdown-body table th) { font-weight: 600; }
  :global(.markdown-body table th, .markdown-body table td) {
    padding: 6px 13px;
    border: 1px solid var(--md-table-border-color);
  }
  :global(.markdown-body table tr:nth-child(2n)) {
    background-color: var(--md-table-bg-alt);
  }

  /* Image styles */
  :global(.markdown-body img) {
    max-width: 100%;
    box-sizing: content-box;
  }

  /* Horizontal rule styles */
  :global(.markdown-body hr) {
    height: 0.25em;
    padding: 0;
    margin: 24px 0;
    background-color: var(--md-border-color);
    border: 0;
  }
</style>