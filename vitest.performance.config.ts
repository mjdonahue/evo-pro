import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/__tests__/setup.ts'],
    include: ['./src/**/*.performance.{test,spec}.{ts,tsx}'],
    exclude: ['./src/**/*.{unit,integration,e2e}.{test,spec}.{ts,tsx}'],
    testTimeout: 30000, // Longer timeout for performance tests
    hookTimeout: 30000,
    reporters: ['default', 'json'],
    outputFile: {
      json: './performance-results/results.json',
    },
    pool: 'forks', // Use separate processes for better isolation
    poolOptions: {
      forks: {
        isolate: true,
      },
    },
    benchmark: {
      include: ['**/*.bench.{ts,tsx}'],
      outputFile: './performance-results/benchmark.json',
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
});