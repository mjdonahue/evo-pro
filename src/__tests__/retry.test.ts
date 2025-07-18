import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { withRetry, RetryOptions, RetryManager } from '../lib/api/retry';
import { ApiError, ErrorCategory } from '../lib/api/errors';
import { ipc_invoke_with_retry } from '../api/ipc';

// Mock the ipcInvoke function
vi.mock('@evo/common', () => ({
  ipcInvoke: vi.fn()
}));

// Import the mocked function
import { ipcInvoke } from '@evo/common';

describe('Retry Mechanism', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    // Reset the RetryManager to default settings
    RetryManager.setEnabled(true);
    RetryManager.setDefaultOptions({
      maxRetries: 3,
      initialDelay: 10, // Use small values for testing
      maxDelay: 100,
      backoffFactor: 2,
      jitter: false // Disable jitter for predictable tests
    });
    // Mock setTimeout to execute immediately
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('withRetry function', () => {
    it('should retry on retryable errors', async () => {
      // Create a function that fails with a retryable error twice, then succeeds
      const mockFn = vi.fn()
        .mockRejectedValueOnce(new ApiError('network_error', 'Network error', ErrorCategory.NETWORK, {}, { retryable: true }))
        .mockRejectedValueOnce(new ApiError('timeout_error', 'Timeout error', ErrorCategory.TIMEOUT, {}, { retryable: true }))
        .mockResolvedValueOnce('success');

      const result = withRetry(mockFn);
      
      // Fast-forward timers after each rejection
      await vi.runOnlyPendingTimersAsync();
      await vi.runOnlyPendingTimersAsync();
      
      await expect(result).resolves.toBe('success');
      expect(mockFn).toHaveBeenCalledTimes(3);
    });

    it('should not retry on non-retryable errors', async () => {
      // Create a function that fails with a non-retryable error
      const mockFn = vi.fn()
        .mockRejectedValueOnce(new ApiError('validation_error', 'Validation error', ErrorCategory.VALIDATION, {}, { retryable: false }));

      await expect(withRetry(mockFn)).rejects.toThrow('Validation error');
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    it('should respect maxRetries option', async () => {
      // Create a function that always fails with a retryable error
      const mockFn = vi.fn()
        .mockRejectedValue(new ApiError('network_error', 'Network error', ErrorCategory.NETWORK, {}, { retryable: true }));

      const retryOptions: RetryOptions = {
        maxRetries: 2,
        initialDelay: 10,
        jitter: false
      };

      const resultPromise = withRetry(mockFn, retryOptions);
      
      // Fast-forward timers for each retry
      await vi.runOnlyPendingTimersAsync();
      await vi.runOnlyPendingTimersAsync();
      
      await expect(resultPromise).rejects.toThrow('Network error');
      expect(mockFn).toHaveBeenCalledTimes(3); // Initial + 2 retries
    });

    it('should use exponential backoff', async () => {
      const delays: number[] = [];
      const onRetry = (error: unknown, attempt: number, delay: number) => {
        delays.push(delay);
      };

      // Create a function that always fails with a retryable error
      const mockFn = vi.fn()
        .mockRejectedValue(new ApiError('network_error', 'Network error', ErrorCategory.NETWORK, {}, { retryable: true }));

      const retryOptions: RetryOptions = {
        maxRetries: 3,
        initialDelay: 10,
        backoffFactor: 2,
        jitter: false,
        onRetry
      };

      const resultPromise = withRetry(mockFn, retryOptions);
      
      // Fast-forward timers for each retry
      await vi.runOnlyPendingTimersAsync();
      await vi.runOnlyPendingTimersAsync();
      await vi.runOnlyPendingTimersAsync();
      
      await expect(resultPromise).rejects.toThrow('Network error');
      
      // Check that delays follow exponential backoff pattern
      expect(delays[0]).toBe(10); // Initial delay
      expect(delays[1]).toBe(20); // Initial * backoffFactor
      expect(delays[2]).toBe(40); // Initial * backoffFactor^2
    });
  });

  describe('ipc_invoke_with_retry function', () => {
    it('should retry IPC calls on transient failures', async () => {
      // Mock the ipcInvoke function to fail twice, then succeed
      (ipcInvoke as any)
        .mockRejectedValueOnce(new Error('Network error'))
        .mockRejectedValueOnce(new Error('Timeout error'))
        .mockResolvedValueOnce({ success: true, data: 'success' });

      const result = ipc_invoke_with_retry('test_method', { param: 'value' });
      
      // Fast-forward timers after each rejection
      await vi.runOnlyPendingTimersAsync();
      await vi.runOnlyPendingTimersAsync();
      
      await expect(result).resolves.toEqual({ success: true, data: 'success' });
      expect(ipcInvoke).toHaveBeenCalledTimes(3);
      expect(ipcInvoke).toHaveBeenCalledWith('test_method', { param: 'value' });
    });

    it('should respect global retry configuration', async () => {
      // Disable retries globally
      RetryManager.setEnabled(false);

      // Mock the ipcInvoke function to fail
      (ipcInvoke as any).mockRejectedValueOnce(new Error('Network error'));

      // Should not retry when globally disabled
      await expect(ipc_invoke_with_retry('test_method', {})).rejects.toThrow('Network error');
      expect(ipcInvoke).toHaveBeenCalledTimes(1);

      // Re-enable retries
      RetryManager.setEnabled(true);
      
      // Set custom global options
      RetryManager.setDefaultOptions({
        maxRetries: 1,
        initialDelay: 10,
        jitter: false
      });

      // Reset the mock
      vi.resetAllMocks();
      (ipcInvoke as any)
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({ success: true, data: 'success' });

      // Should use the global options
      const result = ipc_invoke_with_retry('test_method', {});
      
      // Fast-forward timer
      await vi.runOnlyPendingTimersAsync();
      
      await expect(result).resolves.toEqual({ success: true, data: 'success' });
      expect(ipcInvoke).toHaveBeenCalledTimes(2); // Only 1 retry as per global config
    });
  });

  describe('RetryManager', () => {
    it('should allow configuring global retry behavior', () => {
      // Set custom options
      const customOptions: RetryOptions = {
        maxRetries: 5,
        initialDelay: 100,
        maxDelay: 2000,
        backoffFactor: 3,
        jitter: true
      };
      
      RetryManager.setDefaultOptions(customOptions);
      RetryManager.setEnabled(false);
      
      // Get the config and verify
      const config = RetryManager.getConfig();
      expect(config.enabled).toBe(false);
      expect(config.defaultOptions.maxRetries).toBe(5);
      expect(config.defaultOptions.initialDelay).toBe(100);
      expect(config.defaultOptions.maxDelay).toBe(2000);
      expect(config.defaultOptions.backoffFactor).toBe(3);
      expect(config.defaultOptions.jitter).toBe(true);
    });
  });
});