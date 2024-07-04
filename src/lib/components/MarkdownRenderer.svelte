<!-- src/lib/components/MarkdownRenderer.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import * as marked from 'marked';
  import DOMPurify from 'dompurify';

  export let content: string = ''; // Provide a default empty string

  let renderedContent: string = '';

  $: {
    console.log('Content received:', content);
    if (content) {
      const rawHtml = marked.parse(content);
      console.log('Raw HTML:', rawHtml);
      renderedContent = DOMPurify.sanitize(rawHtml);
      console.log('Sanitized HTML:', renderedContent);
    } else {
      renderedContent = ''; // Set to empty string if content is falsy
      console.log('No content to render');
    }
  }
</script>

<div class="markdown-body">
  {@html renderedContent}
</div>