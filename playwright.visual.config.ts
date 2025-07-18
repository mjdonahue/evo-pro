import { defineConfig, devices } from '@playwright/test';
import path from 'path';

/**
 * Playwright configuration for visual regression testing
 * @see https://playwright.dev/docs/test-configuration
 */
export default defineConfig({
  testDir: './e2e/visual',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html'],
    ['list']
  ],
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'on',
  },
  // Visual comparison settings
  expect: {
    toHaveScreenshot: {
      maxDiffPixels: 100, // Allow small differences (e.g., for animations)
      threshold: 0.2, // 20% threshold for pixel differences
      animations: 'disabled', // Disable animations during screenshots
      caret: 'hide', // Hide text caret during screenshots
    },
    toMatchSnapshot: {
      maxDiffPixelRatio: 0.01, // 1% of pixels can be different
    },
  },
  // Store snapshots in a dedicated directory
  snapshotDir: path.join(__dirname, 'e2e/visual/__snapshots__'),
  // Define different viewport sizes for testing
  projects: [
    {
      name: 'desktop-chrome',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 5'] },
    },
    {
      name: 'tablet-safari',
      use: { ...devices['iPad (gen 7)'] },
    },
  ],
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
    timeout: 60000,
  },
});