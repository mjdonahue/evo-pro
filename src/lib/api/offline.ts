/**
 * Offline queue system for API client
 * 
 * This module provides functionality for queuing operations when the application
 * is offline and processing them when connectivity is restored.
 */

import { ApiResponse } from './types';
import { ErrorCategory, ApiError, Errors } from './errors';

/**
 * Operation types that can be queued
 */
export enum OperationType {
  CREATE = 'create',
  UPDATE = 'update',
  DELETE = 'delete'
}

/**
 * Queued operation interface
 */
export interface QueuedOperation {
  /** Unique identifier for the operation */
  id: string;
  /** Type of operation */
  type: OperationType;
  /** IPC method to call */
  method: string;
  /** Parameters for the IPC call */
  params: Record<string, any>;
  /** Timestamp when the operation was queued */
  timestamp: number;
  /** Entity type the operation is for */
  entityType: string;
  /** Entity ID if available (for updates and deletes) */
  entityId?: string;
  /** Number of retry attempts */
  retryCount: number;
  /** Whether the operation is currently being processed */
  processing: boolean;
  /** Error information if the operation failed */
  error?: {
    message: string;
    code: string;
    timestamp: number;
  };
}

/**
 * Queue storage interface
 */
export interface QueueStorage {
  /** Get all queued operations */
  getAll(): QueuedOperation[];
  /** Get a specific operation by ID */
  get(id: string): QueuedOperation | undefined;
  /** Add an operation to the queue */
  add(operation: QueuedOperation): void;
  /** Update an operation in the queue */
  update(operation: QueuedOperation): void;
  /** Remove an operation from the queue */
  remove(id: string): void;
  /** Clear all operations from the queue */
  clear(): void;
}

/**
 * LocalStorage implementation of queue storage
 */
export class LocalStorageQueueStorage implements QueueStorage {
  private storageKey = 'offline_operation_queue';

  getAll(): QueuedOperation[] {
    if (typeof window === 'undefined') return [];

    try {
      const data = localStorage.getItem(this.storageKey);
      if (!data) return [];
      return JSON.parse(data) as QueuedOperation[];
    } catch (e) {
      console.error('Failed to parse offline queue from localStorage:', e);
      return [];
    }
  }

  get(id: string): QueuedOperation | undefined {
    return this.getAll().find(op => op.id === id);
  }

  add(operation: QueuedOperation): void {
    if (typeof window === 'undefined') return;

    const operations = this.getAll();
    operations.push(operation);
    this.saveOperations(operations);
  }

  update(operation: QueuedOperation): void {
    if (typeof window === 'undefined') return;

    const operations = this.getAll();
    const index = operations.findIndex(op => op.id === operation.id);
    if (index >= 0) {
      operations[index] = operation;
      this.saveOperations(operations);
    }
  }

  remove(id: string): void {
    if (typeof window === 'undefined') return;

    const operations = this.getAll();
    const filteredOperations = operations.filter(op => op.id !== id);
    if (filteredOperations.length !== operations.length) {
      this.saveOperations(filteredOperations);
    }
  }

  clear(): void {
    if (typeof window === 'undefined') return;
    localStorage.removeItem(this.storageKey);
  }

  private saveOperations(operations: QueuedOperation[]): void {
    try {
      localStorage.setItem(this.storageKey, JSON.stringify(operations));
    } catch (e) {
      console.error('Failed to save offline queue to localStorage:', e);
    }
  }
}

/**
 * Options for the offline queue manager
 */
export interface OfflineQueueOptions {
  /** Maximum number of retry attempts for failed operations */
  maxRetries?: number;
  /** Whether to automatically process the queue when coming online */
  autoProcess?: boolean;
  /** Callback when an operation is added to the queue */
  onOperationQueued?: (operation: QueuedOperation) => void;
  /** Callback when an operation is successfully processed */
  onOperationProcessed?: (operation: QueuedOperation, result: any) => void;
  /** Callback when an operation fails to process */
  onOperationFailed?: (operation: QueuedOperation, error: Error) => void;
  /** Callback when the online/offline status changes */
  onStatusChange?: (online: boolean) => void;
}

/**
 * Default options for the offline queue
 */
const DEFAULT_OFFLINE_QUEUE_OPTIONS: Required<OfflineQueueOptions> = {
  maxRetries: 3,
  autoProcess: true,
  onOperationQueued: () => {},
  onOperationProcessed: () => {},
  onOperationFailed: () => {},
  onStatusChange: () => {}
};

/**
 * Offline queue manager
 */
export class OfflineQueueManager {
  private storage: QueueStorage;
  private options: Required<OfflineQueueOptions>;
  private isOnline: boolean = true;
  private isProcessing: boolean = false;
  private processingPromise: Promise<void> | null = null;

  /**
   * Creates a new offline queue manager
   * @param storage - Storage implementation for the queue
   * @param options - Options for the queue manager
   */
  constructor(
    storage: QueueStorage = new LocalStorageQueueStorage(),
    options: OfflineQueueOptions = {}
  ) {
    this.storage = storage;
    this.options = { ...DEFAULT_OFFLINE_QUEUE_OPTIONS, ...options };

    // Initialize online status
    if (typeof window !== 'undefined') {
      this.isOnline = navigator.onLine;

      // Set up event listeners for online/offline events
      window.addEventListener('online', this.handleOnline);
      window.addEventListener('offline', this.handleOffline);
    }
  }

  /**
   * Cleans up event listeners
   */
  destroy(): void {
    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.handleOnline);
      window.removeEventListener('offline', this.handleOffline);
    }
  }

  /**
   * Handles the online event
   */
  private handleOnline = (): void => {
    if (!this.isOnline) {
      this.isOnline = true;
      this.options.onStatusChange(true);

      if (this.options.autoProcess) {
        this.processQueue();
      }
    }
  };

  /**
   * Handles the offline event
   */
  private handleOffline = (): void => {
    if (this.isOnline) {
      this.isOnline = false;
      this.options.onStatusChange(false);
    }
  };

  /**
   * Checks if the application is currently online
   * @returns Whether the application is online
   */
  isNetworkOnline(): boolean {
    return this.isOnline;
  }

  /**
   * Adds an operation to the queue
   * @param operation - The operation to queue
   */
  enqueue(operation: Omit<QueuedOperation, 'id' | 'timestamp' | 'retryCount' | 'processing'>): string {
    const id = this.generateOperationId();
    const queuedOperation: QueuedOperation = {
      ...operation,
      id,
      timestamp: Date.now(),
      retryCount: 0,
      processing: false
    };

    this.storage.add(queuedOperation);
    this.options.onOperationQueued(queuedOperation);

    // Dispatch event for UI updates
    this.dispatchQueueChangedEvent();

    return id;
  }

  /**
   * Dispatches a custom event when the queue changes
   * This is used by UI components to update their state
   */
  private dispatchQueueChangedEvent(): void {
    const event = new CustomEvent('queue:changed', {
      detail: {
        queueSize: this.storage.getAll().length,
        timestamp: Date.now()
      }
    });
    document.dispatchEvent(event);
  }

  /**
   * Generates a unique ID for an operation
   * @returns A unique ID
   */
  private generateOperationId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Processes the queue of operations
   * @returns A promise that resolves when all operations have been processed
   */
  async processQueue(): Promise<void> {
    // If already processing or offline, don't start another processing cycle
    if (this.isProcessing || !this.isOnline) {
      return this.processingPromise || Promise.resolve();
    }

    this.isProcessing = true;

    // Create a new promise for this processing cycle
    this.processingPromise = (async () => {
      const operations = this.storage.getAll()
        .filter(op => !op.processing)
        .sort((a, b) => a.timestamp - b.timestamp);

      for (const operation of operations) {
        try {
          // Mark as processing
          operation.processing = true;
          this.storage.update(operation);

          // Process the operation
          const result = await this.processOperation(operation);

          // Operation succeeded, remove from queue
          this.storage.remove(operation.id);
          this.options.onOperationProcessed(operation, result);

          // Dispatch event for UI updates
          this.dispatchQueueChangedEvent();
        } catch (error) {
          // Operation failed
          operation.processing = false;
          operation.retryCount += 1;

          if (error instanceof Error) {
            operation.error = {
              message: error.message,
              code: error instanceof ApiError ? error.code : 'unknown_error',
              timestamp: Date.now()
            };
          } else {
            operation.error = {
              message: String(error),
              code: 'unknown_error',
              timestamp: Date.now()
            };
          }

          // If we've reached the maximum retry count, mark as failed
          if (operation.retryCount >= this.options.maxRetries) {
            this.options.onOperationFailed(operation, error instanceof Error ? error : new Error(String(error)));
          }

          // Update the operation in storage
          this.storage.update(operation);

          // If we're offline, stop processing
          if (!this.isOnline) {
            break;
          }
        }
      }

      this.isProcessing = false;
      this.processingPromise = null;
    })();

    return this.processingPromise;
  }

  /**
   * Processes a single operation
   * @param operation - The operation to process
   * @returns The result of the operation
   */
  private async processOperation(operation: QueuedOperation): Promise<any> {
    // This would be implemented to actually perform the IPC call
    // For now, we'll just return a mock response
    if (!this.isOnline) {
      throw new ApiError(
        'network_offline',
        'Cannot process operation while offline',
        ErrorCategory.OFFLINE
      );
    }

    // In a real implementation, this would call ipcInvoke
    // return await ipcInvoke(operation.method, operation.params);

    // Mock implementation for now
    return { success: true, data: { id: 'mock-id' } };
  }

  /**
   * Gets all operations in the queue
   * @returns All queued operations
   */
  getQueuedOperations(): QueuedOperation[] {
    return this.storage.getAll();
  }

  /**
   * Gets the number of operations in the queue
   * @returns The queue length
   */
  getQueueLength(): number {
    return this.storage.getAll().length;
  }

  /**
   * Clears all operations from the queue
   */
  clearQueue(): void {
    this.storage.clear();
  }

  /**
   * Removes a specific operation from the queue
   * @param id - The ID of the operation to remove
   */
  removeOperation(id: string): void {
    this.storage.remove(id);
  }
}

// Create a singleton instance of the offline queue manager
export const offlineQueueManager = new OfflineQueueManager();

/**
 * Wraps an IPC invoke function with offline queue support
 * @param fn - The IPC invoke function to wrap
 * @returns A function that will queue operations when offline
 */
export function withOfflineSupport<T>(
  fn: (method: string, params?: Record<string, any>) => Promise<ApiResponse<T>>,
  entityType: string
): (method: string, params?: Record<string, any>) => Promise<ApiResponse<T>> {
  return async (method: string, params?: Record<string, any>): Promise<ApiResponse<T>> => {
    // If online, try to execute the operation directly
    if (offlineQueueManager.isNetworkOnline()) {
      try {
        return await fn(method, params);
      } catch (error) {
        // If the error is due to being offline, queue the operation
        if (error instanceof ApiError && error.category === ErrorCategory.OFFLINE) {
          return handleOfflineOperation<T>(method, params, entityType);
        }
        throw error;
      }
    } else {
      // If offline, queue the operation
      return handleOfflineOperation<T>(method, params, entityType);
    }
  };
}

/**
 * Handles an operation when offline
 * @param method - The IPC method
 * @param params - The parameters for the method
 * @param entityType - The type of entity being operated on
 * @returns A mock response for the operation
 */
function handleOfflineOperation<T>(
  method: string,
  params?: Record<string, any>,
  entityType: string
): ApiResponse<T> {
  // Determine the operation type from the method name
  let operationType: OperationType;
  let entityId: string | undefined;

  if (method.startsWith('create_')) {
    operationType = OperationType.CREATE;
  } else if (method.startsWith('update_')) {
    operationType = OperationType.UPDATE;
    entityId = params?.id;
  } else if (method.startsWith('delete_')) {
    operationType = OperationType.DELETE;
    entityId = params?.id;
  } else {
    // For non-modifying operations, we can't queue them
    throw Errors.network(
      'Cannot perform read operation while offline',
      {
        userMessage: 'This operation requires an internet connection.',
        recoveryAction: 'Please check your connection and try again.'
      }
    );
  }

  // Queue the operation
  const operationId = offlineQueueManager.enqueue({
    type: operationType,
    method,
    params: params || {},
    entityType,
    entityId
  });

  // Return a mock successful response
  // In a real implementation, this would need to be more sophisticated
  // to handle different types of operations and provide appropriate mock data
  return {
    success: true,
    data: { id: operationId } as any as T,
    _offlineOperation: true // Special flag to indicate this was an offline operation
  };
}

/**
 * Checks if a response was generated for an offline operation
 * @param response - The API response to check
 * @returns Whether the response was generated for an offline operation
 */
export function isOfflineResponse<T>(response: ApiResponse<T>): boolean {
  return (response as any)._offlineOperation === true;
}
