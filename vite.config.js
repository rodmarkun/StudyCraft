import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig(async () => {
  const { svelte } = await import('@sveltejs/vite-plugin-svelte');

  return {
    plugins: [svelte()],
    base: './',
    build: {
      outDir: 'dist',
      emptyOutDir: true,
      rollupOptions: {
        input: {
          main: path.resolve(__dirname, 'index.html'),
          'electron-main': path.resolve(__dirname, 'electron/main.cjs'),
        },
        output: {
          entryFileNames: (chunkInfo) => {
            return chunkInfo.name === 'electron-main' ? '[name].cjs' : '[name]-[hash].js';
          },
        },
        external: ['electron']
      }
    }
  };
});