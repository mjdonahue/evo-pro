/**
 * Retry mechanism for API client
 * 
 * This module provides utilities for retrying operations that encounter transient failures,
 * with configurable retry strategies and backoff algorithms.
 */

import { ApiError, ErrorUtils } from './errors';

/**
 * Retry configuration options
 */
export interface RetryOptions {
  /** Maximum number of retry attempts (default: 3) */
  maxRetries?: number;
  /** Initial delay in milliseconds before the first retry (default: 300) */
  initialDelay?: number;
  /** Maximum delay in milliseconds between retries (default: 5000) */
  maxDelay?: number;
  /** Backoff factor for exponential backoff (default: 2) */
  backoffFactor?: number;
  /** Whether to add jitter to the delay to prevent thundering herd (default: true) */
  jitter?: boolean;
  /** Custom function to determine if an error is retryable */
  isRetryable?: (error: unknown) => boolean;
  /** Callback function called before each retry attempt */
  onRetry?: (error: unknown, attempt: number, delay: number) => void;
}

/**
 * Default retry options
 */
const DEFAULT_RETRY_OPTIONS: Required<RetryOptions> = {
  maxRetries: 3,
  initialDelay: 300,
  maxDelay: 5000,
  backoffFactor: 2,
  jitter: true,
  isRetryable: ErrorUtils.isRetryable,
  onRetry: () => {}
};

/**
 * Calculates the delay for the next retry attempt using exponential backoff
 * @param attempt - The current attempt number (0-based)
 * @param options - Retry options
 * @returns The delay in milliseconds
 */
function calculateDelay(attempt: number, options: Required<RetryOptions>): number {
  // Calculate exponential backoff: initialDelay * (backoffFactor ^ attempt)
  let delay = options.initialDelay * Math.pow(options.backoffFactor, attempt);
  
  // Apply maximum delay limit
  delay = Math.min(delay, options.maxDelay);
  
  // Add jitter if enabled (±25% randomization)
  if (options.jitter) {
    const jitterFactor = 0.5 + Math.random();
    delay = Math.floor(delay * jitterFactor);
  }
  
  return delay;
}

/**
 * Waits for the specified delay
 * @param delay - The delay in milliseconds
 * @returns A promise that resolves after the delay
 */
function wait(delay: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, delay));
}

/**
 * Retries a function when it encounters retryable errors
 * @param fn - The function to retry
 * @param options - Retry options
 * @returns A promise that resolves with the function result or rejects with the last error
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  options?: RetryOptions
): Promise<T> {
  const opts: Required<RetryOptions> = { ...DEFAULT_RETRY_OPTIONS, ...options };
  let lastError: unknown;
  
  for (let attempt = 0; attempt <= opts.maxRetries; attempt++) {
    try {
      // First attempt or retry
      return await fn();
    } catch (error) {
      lastError = error;
      
      // Check if we've reached the maximum number of retries
      if (attempt >= opts.maxRetries) {
        break;
      }
      
      // Check if the error is retryable
      if (!opts.isRetryable(error)) {
        break;
      }
      
      // Calculate delay for this retry attempt
      const delay = calculateDelay(attempt, opts);
      
      // Call the onRetry callback
      opts.onRetry(error, attempt + 1, delay);
      
      // Wait before retrying
      await wait(delay);
    }
  }
  
  // If we get here, all retries failed or the error wasn't retryable
  throw lastError;
}

/**
 * Configuration for the global retry behavior
 */
export interface GlobalRetryConfig {
  /** Default retry options for all operations */
  defaultOptions: RetryOptions;
  /** Whether retry is enabled globally */
  enabled: boolean;
  /** Custom logger for retry events */
  logger?: (message: string, data?: any) => void;
}

/**
 * Global retry configuration
 */
const globalRetryConfig: GlobalRetryConfig = {
  defaultOptions: DEFAULT_RETRY_OPTIONS,
  enabled: true,
  logger: console.log
};

/**
 * Retry manager for configuring global retry behavior
 */
export const RetryManager = {
  /**
   * Sets the default retry options
   * @param options - The new default options
   */
  setDefaultOptions(options: RetryOptions): void {
    globalRetryConfig.defaultOptions = { ...DEFAULT_RETRY_OPTIONS, ...options };
  },
  
  /**
   * Enables or disables retry globally
   * @param enabled - Whether retry should be enabled
   */
  setEnabled(enabled: boolean): void {
    globalRetryConfig.enabled = enabled;
  },
  
  /**
   * Sets a custom logger for retry events
   * @param logger - The logger function
   */
  setLogger(logger: (message: string, data?: any) => void): void {
    globalRetryConfig.logger = logger;
  },
  
  /**
   * Gets the current global retry configuration
   * @returns The current configuration
   */
  getConfig(): GlobalRetryConfig {
    return { ...globalRetryConfig };
  }
};

/**
 * Creates a retry-enabled version of a function
 * @param fn - The function to wrap with retry capability
 * @param options - Retry options
 * @returns A function that will retry on transient failures
 */
export function createRetryableFunction<T extends (...args: any[]) => Promise<any>>(
  fn: T,
  options?: RetryOptions
): T {
  return ((...args: Parameters<T>): ReturnType<T> => {
    if (!globalRetryConfig.enabled) {
      return fn(...args) as ReturnType<T>;
    }
    
    const retryOptions: RetryOptions = {
      ...globalRetryConfig.defaultOptions,
      ...options,
      onRetry: (error, attempt, delay) => {
        // Call the custom onRetry if provided
        if (options?.onRetry) {
          options.onRetry(error, attempt, delay);
        }
        
        // Log the retry attempt
        if (globalRetryConfig.logger) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          globalRetryConfig.logger(
            `Retrying operation (attempt ${attempt}/${(options?.maxRetries || DEFAULT_RETRY_OPTIONS.maxRetries)}) after error: ${errorMessage}. Retrying in ${delay}ms.`,
            { error, attempt, delay, functionName: fn.name }
          );
        }
      }
    };
    
    return withRetry(() => fn(...args), retryOptions) as ReturnType<T>;
  }) as T;
}