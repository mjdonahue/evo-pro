import { test, expect } from '@playwright/test';

/**
 * Example end-to-end test for the application
 * 
 * This test verifies that the application loads correctly
 * and basic navigation works.
 */
test('application loads and basic navigation works', async ({ page }) => {
  // Navigate to the application
  await page.goto('/');

  // Verify that the application has loaded
  await expect(page).toHaveTitle(/Evo Pro/);

  // Verify that the main content is visible
  await expect(page.locator('main')).toBeVisible();

  // Example of interacting with the application
  // This would need to be adjusted based on the actual application structure
  const navigationMenu = page.locator('nav');
  await expect(navigationMenu).toBeVisible();

  // Example of clicking a navigation item
  // Replace with actual navigation elements from your application
  const firstNavItem = navigationMenu.locator('a').first();
  if (await firstNavItem.isVisible()) {
    await firstNavItem.click();
    
    // Wait for navigation to complete
    await page.waitForLoadState('networkidle');
    
    // Verify that the URL has changed
    expect(page.url()).not.toBe('/');
  }
});

/**
 * Test for responsive design
 */
test('application is responsive', async ({ page }) => {
  // Navigate to the application
  await page.goto('/');

  // Test on mobile viewport
  await page.setViewportSize({ width: 375, height: 667 });
  await expect(page.locator('main')).toBeVisible();

  // Test on tablet viewport
  await page.setViewportSize({ width: 768, height: 1024 });
  await expect(page.locator('main')).toBeVisible();

  // Test on desktop viewport
  await page.setViewportSize({ width: 1440, height: 900 });
  await expect(page.locator('main')).toBeVisible();
});