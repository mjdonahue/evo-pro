import { ipcInvoke } from "@evo/common";
import { withRetry, RetryOptions, createRetryableFunction } from '../lib/api/retry';
import { ErrorUtils, ErrorCategory, ApiError } from '../lib/api/errors';
import { withOfflineSupport, offlineQueueManager } from '../lib/api/offline';

/**
 * Default retry options for IPC calls
 */
const DEFAULT_IPC_RETRY_OPTIONS: RetryOptions = {
  maxRetries: 3,
  initialDelay: 300,
  maxDelay: 5000,
  backoffFactor: 2,
  jitter: true
};

/**
 * Invokes an IPC command with standardized error handling and response formatting
 * 
 * @param method - The name of the IPC command to invoke
 * @param params - The parameters to pass to the command
 * @param entityType - The type of entity being operated on (for offline queue)
 * @returns A promise that resolves to the result of the command
 * @throws An error if the command fails
 */
export async function ipc_invoke<T>(
  method: string,
  params?: object,
  entityType?: string
): Promise<T> {
  try {
    // If we're offline and this is a modifying operation, queue it
    if (!offlineQueueManager.isNetworkOnline() && entityType && 
        (method.startsWith('create_') || method.startsWith('update_') || method.startsWith('delete_'))) {
      return await withOfflineSupport(ipcInvoke, entityType)<T>(method, params);
    }

    // Otherwise, try to execute directly
    return await ipcInvoke<T>(method, params);
  } catch (error) {
    // If the error is due to network connectivity, try to queue the operation
    if (error instanceof Error && 
        (error.message.includes('network') || error.message.includes('connection') || 
         error.message.includes('offline') || error.message.includes('timeout'))) {

      // Convert to an offline error
      const offlineError = new ApiError(
        'network_offline',
        'Network connection unavailable',
        ErrorCategory.OFFLINE,
        { originalError: error }
      );

      // If we have an entity type, try to queue the operation
      if (entityType && (method.startsWith('create_') || method.startsWith('update_') || method.startsWith('delete_'))) {
        return await withOfflineSupport(ipcInvoke, entityType)<T>(method, params);
      }

      throw offlineError;
    }

    // Rethrow other errors
    throw error;
  }
}

/**
 * Invokes an IPC command with retry capability for transient failures
 * 
 * @param method - The name of the IPC command to invoke
 * @param params - The parameters to pass to the command
 * @param retryOptions - Options for the retry mechanism
 * @param entityType - The type of entity being operated on (for offline queue)
 * @returns A promise that resolves to the result of the command
 * @throws An error if the command fails after all retry attempts
 */
export async function ipc_invoke_with_retry<T>(
  method: string,
  params?: object,
  retryOptions?: RetryOptions,
  entityType?: string
): Promise<T> {
  return await withRetry(
    () => ipc_invoke<T>(method, params, entityType),
    {
      ...DEFAULT_IPC_RETRY_OPTIONS,
      ...retryOptions,
      onRetry: (error, attempt, delay) => {
        console.log(
          `Retrying IPC call to ${method} (attempt ${attempt}) after error: ${error instanceof Error ? error.message : String(error)}. Retrying in ${delay}ms.`
        );
        retryOptions?.onRetry?.(error, attempt, delay);
      },
      // Don't retry if we're offline - let the offline queue handle it
      isRetryable: (error) => {
        if (!offlineQueueManager.isNetworkOnline()) {
          return false;
        }
        return retryOptions?.isRetryable?.(error) ?? ErrorUtils.isRetryable(error);
      }
    }
  );
}

/**
 * Creates a retryable version of the ipc_invoke function with custom retry options
 * 
 * @param retryOptions - Options for the retry mechanism
 * @param entityType - The type of entity being operated on (for offline queue)
 * @returns A function that invokes IPC commands with retry capability
 */
export function createRetryableIpcInvoke<T>(
  retryOptions?: RetryOptions,
  entityType?: string
): (method: string, params?: object) => Promise<T> {
  return (method: string, params?: object) => ipc_invoke_with_retry<T>(method, params, retryOptions, entityType);
}
