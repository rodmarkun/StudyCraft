// vite.config.js
import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      'marked': 'marked',
      'dompurify': 'dompurify'
    }
  },
  build: {
    rollupOptions: {
      external: ['electron']
    }
  },
  define: {
    'process.env': {}
  }
});