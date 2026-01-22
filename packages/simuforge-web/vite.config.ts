import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
  server: {
    port: 3000,
    open: true,
    fs: {
      // Allow serving files from the renderer pkg directory
      allow: ['..', '../../packages/simuforge-renderer/pkg'],
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
  },
  optimizeDeps: {
    exclude: ['@simuforge/renderer'],
  },
  resolve: {
    alias: {
      '@simuforge/wasm': path.resolve(__dirname, '../simuforge-renderer/pkg'),
    },
  },
});
