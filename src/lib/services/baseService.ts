import { apiClient, ApiClientError } from '../api/client';

/**
 * Base class for all services in the application.
 * Services provide higher-level abstractions over the API client,
 * organizing functionality around business domains rather than entity types.
 */
export abstract class BaseService {
  /**
   * Reference to the API client for making API calls
   */
  protected api = apiClient;

  /**
   * Creates a new service instance
   */
  constructor() {}

  /**
   * Handles errors from API calls, providing consistent error handling across services
   * @param error - The error to handle
   * @param context - Additional context about the operation that failed
   * @throws {ApiClientError} - Rethrows the error with additional context
   */
  protected handleError(error: unknown, context: Record<string, any> = {}): never {
    if (error instanceof ApiClientError) {
      // Add additional context to the error
      error.details = { ...error.details, ...context };
      throw error;
    }

    // If it's not an ApiClientError, wrap it in one
    throw new ApiClientError(
      'service_error',
      error instanceof Error ? error.message : 'Unknown service error',
      { ...context, originalError: error }
    );
  }
}