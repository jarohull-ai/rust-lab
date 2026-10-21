import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [react(), tailwindcss()],
  build: {
    license: { fileName: 'third-party-licenses.md' },
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: 'react-flow',
              test: /node_modules[\\/]@xyflow/,
              priority: 30,
            },
            {
              name: 'code-editor-react',
              test: /node_modules[\\/]@uiw/,
              priority: 35,
            },
            {
              name: 'code-editor-core',
              test: /node_modules[\\/](@codemirror|codemirror)/,
              priority: 30,
            },
            {
              name: 'react-vendor',
              test: /node_modules[\\/](react|react-dom|scheduler)[\\/]/,
              priority: 20,
            },
            {
              name: 'vendor',
              test: /node_modules/,
              minSize: 20_000,
              maxSize: 400_000,
              priority: 10,
            },
          ],
        },
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:3030',
        changeOrigin: true,
        headers: process.env.RUST_LAB_API_TOKEN
          ? {
              'X-Requested-With': 'Polyglot-Lab',
              'X-Lab-Token': process.env.RUST_LAB_API_TOKEN,
            }
          : undefined,
      },
    },
  },
});
