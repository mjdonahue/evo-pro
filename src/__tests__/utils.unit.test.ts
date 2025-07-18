import { describe, it, expect } from 'vitest';

/**
 * Example utility function for formatting dates
 */
const formatDate = (date: Date, format: string = 'short'): string => {
  if (format === 'short') {
    return date.toLocaleDateString();
  } else if (format === 'long') {
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  } else if (format === 'iso') {
    return date.toISOString();
  }
  return date.toString();
};

/**
 * Example utility function for validating email addresses
 */
const isValidEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

/**
 * Example unit tests for utility functions
 * 
 * Unit tests focus on testing a single unit of code in isolation,
 * without dependencies on other parts of the system.
 */
describe('Utility Functions', () => {
  describe('formatDate', () => {
    it('formats date in short format by default', () => {
      const date = new Date('2023-01-15');
      const result = formatDate(date);
      
      // The exact format depends on the locale, so we'll just check that it's a string
      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);
    });

    it('formats date in long format', () => {
      const date = new Date('2023-01-15T12:30:00');
      const result = formatDate(date, 'long');
      
      expect(typeof result).toBe('string');
      expect(result.length).toBeGreaterThan(0);
      expect(result).toContain(date.toLocaleDateString());
    });

    it('formats date in ISO format', () => {
      const date = new Date('2023-01-15T12:30:00Z');
      const result = formatDate(date, 'iso');
      
      expect(result).toBe(date.toISOString());
    });

    it('returns string representation for unknown format', () => {
      const date = new Date('2023-01-15');
      const result = formatDate(date, 'unknown');
      
      expect(result).toBe(date.toString());
    });
  });

  describe('isValidEmail', () => {
    it('returns true for valid email addresses', () => {
      expect(isValidEmail('user@example.com')).toBe(true);
      expect(isValidEmail('user.name@example.co.uk')).toBe(true);
      expect(isValidEmail('user+tag@example.org')).toBe(true);
    });

    it('returns false for invalid email addresses', () => {
      expect(isValidEmail('user@')).toBe(false);
      expect(isValidEmail('user@example')).toBe(false);
      expect(isValidEmail('user@.com')).toBe(false);
      expect(isValidEmail('@example.com')).toBe(false);
      expect(isValidEmail('user example.com')).toBe(false);
      expect(isValidEmail('')).toBe(false);
    });
  });
});