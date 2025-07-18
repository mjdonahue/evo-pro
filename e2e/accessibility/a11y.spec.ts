import { test, expect } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

/**
 * Accessibility tests using axe-core
 * 
 * These tests check for accessibility issues on various pages and components.
 * They use the axe-core library to perform automated accessibility testing.
 */

test.describe('Accessibility Tests', () => {
  // Set up for each test
  test.beforeEach(async ({ page }) => {
    // Navigate to the application
    await page.goto('/');
    
    // Wait for the application to be fully loaded
    await page.waitForLoadState('networkidle');
  });
  
  /**
   * Test for the main page accessibility
   */
  test('main page should not have any automatically detectable accessibility issues', async ({ page }) => {
    // Run axe against the page
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();
    
    // Assert that there are no violations
    expect(accessibilityScanResults.violations).toEqual([]);
  });
  
  /**
   * Test for specific components
   */
  test('navigation menu should be accessible', async ({ page }) => {
    // Find the navigation menu
    const navigationMenu = page.locator('nav');
    
    // Ensure the navigation menu exists
    await expect(navigationMenu).toBeVisible();
    
    // Run axe against the navigation menu
    const accessibilityScanResults = await new AxeBuilder({ page })
      .include('nav')
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();
    
    // Assert that there are no violations
    expect(accessibilityScanResults.violations).toEqual([]);
  });
  
  /**
   * Test for form accessibility
   */
  test('forms should be accessible', async ({ page }) => {
    // Navigate to a page with forms (adjust as needed)
    await page.goto('/');
    
    // Find forms
    const forms = page.locator('form');
    
    // If forms exist, test them
    if (await forms.count() > 0) {
      // Run axe against the forms
      const accessibilityScanResults = await new AxeBuilder({ page })
        .include('form')
        .withTags(['wcag2a', 'wcag2aa'])
        .analyze();
      
      // Assert that there are no violations
      expect(accessibilityScanResults.violations).toEqual([]);
    } else {
      test.skip('No forms found to test');
    }
  });
  
  /**
   * Test for color contrast
   */
  test('page should have sufficient color contrast', async ({ page }) => {
    // Run axe against the page, focusing on color contrast
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2aa'])
      .options({
        rules: {
          'color-contrast': { enabled: true }
        }
      })
      .analyze();
    
    // Assert that there are no color contrast violations
    const contrastViolations = accessibilityScanResults.violations.filter(
      violation => violation.id === 'color-contrast'
    );
    
    expect(contrastViolations).toEqual([]);
  });
  
  /**
   * Test for keyboard navigation
   */
  test('page should be navigable with keyboard', async ({ page }) => {
    // Find all interactive elements
    const interactiveElements = page.locator('a, button, [role="button"], input, select, textarea, [tabindex]:not([tabindex="-1"])');
    
    // Count the number of interactive elements
    const count = await interactiveElements.count();
    
    // Skip if no interactive elements found
    if (count === 0) {
      test.skip('No interactive elements found to test');
      return;
    }
    
    // Press Tab to navigate through elements
    for (let i = 0; i < Math.min(count, 10); i++) { // Limit to 10 tabs to avoid infinite loops
      await page.keyboard.press('Tab');
      
      // Get the active element
      const activeElement = await page.evaluate(() => {
        const active = document.activeElement;
        return {
          tagName: active?.tagName.toLowerCase(),
          isVisible: active !== document.body && active !== document.documentElement
        };
      });
      
      // Verify that an element is focused and it's not the body or document
      expect(activeElement.isVisible).toBe(true);
    }
  });
  
  /**
   * Test for ARIA attributes
   */
  test('ARIA attributes should be used correctly', async ({ page }) => {
    // Run axe against the page, focusing on ARIA
    const accessibilityScanResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa', 'cat.aria'])
      .analyze();
    
    // Assert that there are no ARIA violations
    const ariaViolations = accessibilityScanResults.violations.filter(
      violation => violation.tags.includes('cat.aria')
    );
    
    expect(ariaViolations).toEqual([]);
  });
  
  /**
   * Test for responsive accessibility
   */
  test('page should be accessible on different viewport sizes', async ({ page }) => {
    // Test on mobile viewport
    await page.setViewportSize({ width: 375, height: 667 });
    await page.waitForTimeout(500); // Wait for responsive changes to apply
    
    let mobileResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();
    
    expect(mobileResults.violations).toEqual([]);
    
    // Test on tablet viewport
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.waitForTimeout(500); // Wait for responsive changes to apply
    
    let tabletResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();
    
    expect(tabletResults.violations).toEqual([]);
    
    // Test on desktop viewport
    await page.setViewportSize({ width: 1440, height: 900 });
    await page.waitForTimeout(500); // Wait for responsive changes to apply
    
    let desktopResults = await new AxeBuilder({ page })
      .withTags(['wcag2a', 'wcag2aa'])
      .analyze();
    
    expect(desktopResults.violations).toEqual([]);
  });
});