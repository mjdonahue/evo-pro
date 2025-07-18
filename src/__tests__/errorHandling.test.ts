import { describe, it, expect, vi, beforeEach } from 'vitest';
import { controllers, ControllerError } from '../api/controllers';
import { ApiError, ErrorCategory, ErrorUtils } from '../lib/api/errors';
import { ipc_invoke } from '../api/ipc';

// Mock the ipc_invoke function
vi.mock('../api/ipc', () => ({
  ipc_invoke: vi.fn()
}));

describe('Error Handling System', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  describe('BaseController Error Handling', () => {
    it('should handle not found errors correctly', async () => {
      // Mock a not found response
      (ipc_invoke as any).mockResolvedValueOnce({
        success: false,
        error: 'User not found',
        errorCode: 'user_not_found'
      });

      // Attempt to get a non-existent user
      try {
        await controllers.users.get('non-existent-id');
        // If we get here, the test should fail
        expect(true).toBe(false);
      } catch (error) {
        // Verify the error is an ApiError with the correct category
        expect(error).toBeInstanceOf(ApiError);
        expect(error).toBeInstanceOf(ControllerError);
        expect((error as ApiError).category).toBe(ErrorCategory.NOT_FOUND);
        expect((error as ApiError).code).toBe('not_found_error');
        expect((error as ApiError).context.entityType).toBe('user');
        expect((error as ApiError).context.entityId).toBe('non-existent-id');
      }
    });

    it('should handle validation errors correctly', async () => {
      // Mock a validation error response
      (ipc_invoke as any).mockResolvedValueOnce({
        success: false,
        error: 'Invalid user data',
        errorCode: 'validation_error'
      });

      // Attempt to create a user with invalid data
      try {
        await controllers.users.create({ name: '' });
        // If we get here, the test should fail
        expect(true).toBe(false);
      } catch (error) {
        // Verify the error is an ApiError with the correct category
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).category).toBe(ErrorCategory.VALIDATION);
        expect((error as ApiError).code).toBe('validation_error');
        expect((error as ApiError).context.entityType).toBe('user');
        expect((error as ApiError).context.operation).toBe('create');
      }
    });

    it('should handle network errors correctly', async () => {
      // Mock a network error
      (ipc_invoke as any).mockRejectedValueOnce(new Error('Network error'));

      // Attempt to list users
      try {
        await controllers.users.list();
        // If we get here, the test should fail
        expect(true).toBe(false);
      } catch (error) {
        // Verify the error is an ApiError with the correct category
        expect(error).toBeInstanceOf(ApiError);
        expect((error as ApiError).category).toBe(ErrorCategory.NETWORK);
        expect((error as ApiError).code).toBe('invoke_error');
        expect((error as ApiError).context.entityType).toBe('user');
        expect((error as ApiError).context.operation).toBe('list');
        expect((error as ApiError).retryable).toBe(true);
      }
    });
  });

  describe('Error Utilities', () => {
    it('should convert unknown errors to ApiError', () => {
      const originalError = new Error('Original error');
      const apiError = ErrorUtils.fromUnknown(originalError, { operation: 'test' });

      expect(apiError).toBeInstanceOf(ApiError);
      expect(apiError.message).toBe('Original error');
      expect(apiError.category).toBe(ErrorCategory.UNKNOWN);
      expect(apiError.context.operation).toBe('test');
      expect(apiError.context.originalError).toBe(originalError);
    });

    it('should determine if an error is retryable', () => {
      const nonRetryableError = new ApiError(
        'validation_error',
        'Validation failed',
        ErrorCategory.VALIDATION,
        {},
        { retryable: false }
      );

      const retryableError = new ApiError(
        'network_error',
        'Network failed',
        ErrorCategory.NETWORK,
        {},
        { retryable: true }
      );

      expect(ErrorUtils.isRetryable(nonRetryableError)).toBe(false);
      expect(ErrorUtils.isRetryable(retryableError)).toBe(true);
      expect(ErrorUtils.isRetryable(new Error('Regular error'))).toBe(false);
    });

    it('should get user-friendly messages and recovery actions', () => {
      const error = new ApiError(
        'test_error',
        'Technical error message',
        ErrorCategory.BUSINESS_LOGIC,
        {},
        {
          userMessage: 'User-friendly message',
          recoveryAction: 'Try this to fix it'
        }
      );

      expect(ErrorUtils.getUserMessage(error)).toBe('User-friendly message');
      expect(ErrorUtils.getRecoveryAction(error)).toBe('Try this to fix it');

      // Test fallbacks
      expect(ErrorUtils.getUserMessage(new Error('Regular error'))).toBe('An unexpected error occurred.');
      expect(ErrorUtils.getRecoveryAction(new Error('Regular error'))).toBe('Please try again or contact support if the problem persists.');
    });
  });
});