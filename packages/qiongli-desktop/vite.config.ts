import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [tailwindcss(), sveltekit(), svelteTesting()],
  clearScreen: false,
  server: {
    strictPort: true
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts']
  }
});
