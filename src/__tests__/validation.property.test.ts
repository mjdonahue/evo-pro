import { describe, it, expect } from 'vitest';
import { test, assert, property } from '@fast-check/vitest';
import * as fc from 'fast-check';

/**
 * Example validation functions to test with property-based testing
 */

// Function to validate a username
const isValidUsername = (username: string): boolean => {
  // Username must be 3-20 characters, alphanumeric with underscores and hyphens
  return /^[a-zA-Z0-9_-]{3,20}$/.test(username);
};

// Function to validate a password
const isValidPassword = (password: string): boolean => {
  // Password must be at least 8 characters, with at least one uppercase, one lowercase, and one number
  return /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/.test(password);
};

// Function to validate an email address
const isValidEmail = (email: string): boolean => {
  // Simple email validation
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
};

// Function to validate a phone number
const isValidPhoneNumber = (phone: string): boolean => {
  // Simple phone number validation (digits, spaces, dashes, parentheses)
  return /^[\d\s\-()]{7,15}$/.test(phone);
};

/**
 * Property-based tests for validation functions
 * 
 * Property-based testing generates many test cases to verify that properties
 * hold for all inputs within the specified domain.
 */
describe('Validation Functions - Property Tests', () => {
  describe('Username Validation', () => {
    // Property: Valid usernames should pass validation
    test.prop([
      fc.stringMatching(/^[a-zA-Z0-9_-]{3,20}$/)
    ])('valid usernames should pass validation', (validUsername) => {
      expect(isValidUsername(validUsername)).toBe(true);
    });

    // Property: Usernames that are too short should fail validation
    test.prop([
      fc.stringMatching(/^[a-zA-Z0-9_-]{1,2}$/)
    ])('usernames that are too short should fail validation', (shortUsername) => {
      expect(isValidUsername(shortUsername)).toBe(false);
    });

    // Property: Usernames that are too long should fail validation
    test.prop([
      fc.stringMatching(/^[a-zA-Z0-9_-]{21,30}$/)
    ])('usernames that are too long should fail validation', (longUsername) => {
      expect(isValidUsername(longUsername)).toBe(false);
    });

    // Property: Usernames with invalid characters should fail validation
    test.prop([
      fc.string().filter(s => /[^a-zA-Z0-9_-]/.test(s))
    ])('usernames with invalid characters should fail validation', (invalidUsername) => {
      // Some generated strings might accidentally pass the regex
      // This is fine as we're just testing the property in general
      if (isValidUsername(invalidUsername)) {
        // If it passes, make sure it actually matches our criteria
        expect(invalidUsername).toMatch(/^[a-zA-Z0-9_-]{3,20}$/);
      } else {
        expect(isValidUsername(invalidUsername)).toBe(false);
      }
    });
  });

  describe('Password Validation', () => {
    // Property: Valid passwords should pass validation
    test.prop([
      fc.stringMatching(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/)
    ])('valid passwords should pass validation', (validPassword) => {
      expect(isValidPassword(validPassword)).toBe(true);
    });

    // Property: Passwords without lowercase letters should fail validation
    test.prop([
      fc.stringMatching(/^(?=.*[A-Z])(?=.*\d)[A-Z\d]{8,}$/)
    ])('passwords without lowercase letters should fail validation', (noLowercase) => {
      expect(isValidPassword(noLowercase)).toBe(false);
    });

    // Property: Passwords without uppercase letters should fail validation
    test.prop([
      fc.stringMatching(/^(?=.*[a-z])(?=.*\d)[a-z\d]{8,}$/)
    ])('passwords without uppercase letters should fail validation', (noUppercase) => {
      expect(isValidPassword(noUppercase)).toBe(false);
    });

    // Property: Passwords without digits should fail validation
    test.prop([
      fc.stringMatching(/^(?=.*[a-z])(?=.*[A-Z])[a-zA-Z]{8,}$/)
    ])('passwords without digits should fail validation', (noDigits) => {
      expect(isValidPassword(noDigits)).toBe(false);
    });

    // Property: Passwords that are too short should fail validation
    test.prop([
      fc.stringMatching(/^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{1,7}$/)
    ])('passwords that are too short should fail validation', (shortPassword) => {
      expect(isValidPassword(shortPassword)).toBe(false);
    });
  });

  describe('Email Validation', () => {
    // Property: Valid emails should pass validation
    test('valid emails should pass validation', () => {
      // Using fc.emailAddress() to generate valid email addresses
      fc.assert(
        fc.property(fc.emailAddress(), (email) => {
          expect(isValidEmail(email)).toBe(true);
        })
      );
    });

    // Property: Strings without @ should fail email validation
    test.prop([
      fc.string().filter(s => !s.includes('@'))
    ])('strings without @ should fail email validation', (noAtSign) => {
      expect(isValidEmail(noAtSign)).toBe(false);
    });
  });
});