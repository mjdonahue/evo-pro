import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/__tests__/integration/setup.ts'],
    include: ['./src/**/*.integration.{test,spec}.{ts,tsx}'],
    exclude: ['./src/**/*.{unit,e2e}.{test,spec}.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov', 'clover'],
      exclude: [
        'node_modules/',
        'src/__tests__/',
        '**/*.d.ts',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        '**/index.{ts,tsx}',
      ],
      reportsDirectory: './coverage/integration',
      all: true,
      branches: 70,
      functions: 70,
      lines: 70,
      statements: 70,
    },
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
});
