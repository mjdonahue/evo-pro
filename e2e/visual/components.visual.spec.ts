import { test, expect } from '@playwright/test';

/**
 * Visual regression tests for UI components
 * 
 * These tests capture screenshots of UI components and compare them to baseline images.
 * If the visual appearance changes, the test will fail and show the differences.
 */

test.describe('Visual Regression Tests', () => {
  // Set up for each test
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
    
    // Wait for the application to be fully loaded
    await page.waitForLoadState('networkidle');
  });
  
  /**
   * Test for the main layout
   */
  test('main layout appearance', async ({ page }) => {
    // Verify that the main layout appears correctly
    await expect(page.locator('body')).toHaveScreenshot('main-layout.png');
  });
  
  /**
   * Test for navigation components
   */
  test('navigation menu appearance', async ({ page }) => {
    // Find the navigation menu
    const navigationMenu = page.locator('nav');
    
    // Verify that the navigation menu appears correctly
    await expect(navigationMenu).toHaveScreenshot('navigation-menu.png');
  });
  
  /**
   * Test for responsive design
   */
  test('responsive design', async ({ page }) => {
    // Test on mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(500); // Wait for responsive changes to apply
    await expect(page).toHaveScreenshot('responsive-mobile.png');
    
    // Test on tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.waitForTimeout(500); // Wait for responsive changes to apply
    await expect(page).toHaveScreenshot('responsive-tablet.png');
    
    // Test on desktop viewport
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.waitForTimeout(500); // Wait for responsive changes to apply
    await expect(page).toHaveScreenshot('responsive-desktop.png');
  });
  
  /**
   * Test for dark mode
   */
  test('dark mode appearance', async ({ page }) => {
    // Find the theme toggle (assuming there is one)
    const themeToggle = page.locator('[data-testid="theme-toggle"]');
    
    // If the theme toggle exists, click it to switch to dark mode
    if (await themeToggle.count() > 0) {
      await themeToggle.click();
      await page.waitForTimeout(500); // Wait for theme change to apply
    } else {
      // If there's no theme toggle, try to set dark mode via localStorage
      await page.evaluate(() => {
        localStorage.setItem('theme', 'dark');
        document.documentElement.classList.add('dark');
      });
      await page.reload();
      await page.waitForLoadState('networkidle');
    }
    
    // Verify that dark mode appears correctly
    await expect(page).toHaveScreenshot('dark-mode.png');
  });
  
  /**
   * Test for specific UI components
   */
  test('button states', async ({ page }) => {
    // Find a button (adjust the selector as needed)
    const button = page.locator('button').first();
    
    // Verify the default state
    await expect(button).toHaveScreenshot('button-default.png');
    
    // Verify the hover state
    await button.hover();
    await page.waitForTimeout(300); // Wait for hover effect
    await expect(button).toHaveScreenshot('button-hover.png');
    
    // Verify the active state
    await page.mouse.down();
    await page.waitForTimeout(300); // Wait for active effect
    await expect(button).toHaveScreenshot('button-active.png');
    await page.mouse.up();
  });
  
  /**
   * Test for form elements
   */
  test('form elements appearance', async ({ page }) => {
    // Navigate to a page with form elements (adjust as needed)
    await page.goto('/');
    
    // Find form elements
    const inputField = page.locator('input').first();
    const selectField = page.locator('select').first();
    
    // Verify input field appearance
    if (await inputField.count() > 0) {
      await expect(inputField).toHaveScreenshot('input-field.png');
      
      // Verify input field with focus
      await inputField.focus();
      await page.waitForTimeout(300); // Wait for focus effect
      await expect(inputField).toHaveScreenshot('input-field-focus.png');
      
      // Verify input field with value
      await inputField.fill('Test Value');
      await expect(inputField).toHaveScreenshot('input-field-with-value.png');
    }
    
    // Verify select field appearance
    if (await selectField.count() > 0) {
      await expect(selectField).toHaveScreenshot('select-field.png');
    }
  });
});