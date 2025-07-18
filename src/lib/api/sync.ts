/**
 * Synchronization mechanisms for offline operations
 * 
 * This module provides advanced synchronization capabilities for the offline queue,
 * including dependency tracking, ordered execution, and conflict resolution.
 */

import { ApiResponse } from './types';
import { ErrorCategory, ApiError, Errors } from './errors';
import { 
  OfflineQueueManager, 
  QueuedOperation, 
  OperationType,
  offlineQueueManager
} from './offline';
import {
  ConflictManager,
  Conflict,
  ConflictType,
  ConflictStrategy,
  ConflictResolution,
  ConflictResolutionOptions,
  conflictManager
} from './conflict';

/**
 * Synchronization status
 */
export enum SyncStatus {
  IDLE = 'idle',
  SYNCING = 'syncing',
  COMPLETED = 'completed',
  FAILED = 'failed',
  PARTIALLY_COMPLETED = 'partially_completed'
}

/**
 * Synchronization progress information
 */
export interface SyncProgress {
  /** Total number of operations to sync */
  total: number;
  /** Number of operations successfully synced */
  completed: number;
  /** Number of operations that failed to sync */
  failed: number;
  /** Current operation being processed */
  currentOperation?: QueuedOperation;
  /** Current status of the synchronization */
  status: SyncStatus;
  /** Error information if the synchronization failed */
  error?: Error;
  /** Timestamp when the synchronization started */
  startTime: number;
  /** Timestamp when the synchronization ended (if completed) */
  endTime?: number;
}

/**
 * Dependency information for an operation
 */
export interface OperationDependency {
  /** ID of the operation that this operation depends on */
  dependsOnId: string;
  /** Type of dependency */
  type: 'entity' | 'order' | 'custom';
  /** Entity ID that this dependency is related to (for entity dependencies) */
  entityId?: string;
  /** Custom validation function to check if the dependency is satisfied */
  validate?: (operation: QueuedOperation, dependsOn: QueuedOperation) => boolean;
}

/**
 * Extended queued operation with dependency information
 */
export interface SyncOperation extends QueuedOperation {
  /** Dependencies that must be satisfied before this operation can be processed */
  dependencies?: OperationDependency[];
  /** Result of the operation after processing (if successful) */
  result?: any;
  /** Whether this operation has been processed during synchronization */
  synced?: boolean;
  /** Whether this operation has been skipped during synchronization */
  skipped?: boolean;
  /** Reason why this operation was skipped (if applicable) */
  skipReason?: string;
}

/**
 * Options for the synchronization manager
 */
export interface SyncOptions {
  /** Whether to continue synchronization if an operation fails */
  continueOnError?: boolean;
  /** Maximum number of retry attempts for failed operations */
  maxRetries?: number;
  /** Callback for synchronization progress updates */
  onProgress?: (progress: SyncProgress) => void;
  /** Callback when synchronization is completed */
  onComplete?: (result: SyncResult) => void;
  /** Callback when an operation is successfully synced */
  onOperationSynced?: (operation: SyncOperation, result: any) => void;
  /** Callback when an operation fails to sync */
  onOperationFailed?: (operation: SyncOperation, error: Error) => void;
  /** Whether to automatically resolve dependencies between operations */
  autoResolveDependencies?: boolean;
  /** Whether to automatically retry failed operations */
  autoRetry?: boolean;
  /** Options for conflict resolution */
  conflictOptions?: ConflictResolutionOptions;
  /** Callback when a conflict is detected */
  onConflictDetected?: (conflict: Conflict) => void;
  /** Callback when a conflict is resolved */
  onConflictResolved?: (conflict: Conflict, resolution: ConflictResolution) => void;
}

/**
 * Default options for synchronization
 */
const DEFAULT_SYNC_OPTIONS: Required<SyncOptions> = {
  continueOnError: true,
  maxRetries: 3,
  onProgress: () => {},
  onComplete: () => {},
  onOperationSynced: () => {},
  onOperationFailed: () => {},
  autoResolveDependencies: true,
  autoRetry: true,
  conflictOptions: {},
  onConflictDetected: () => {},
  onConflictResolved: () => {}
};

/**
 * Result of a synchronization operation
 */
export interface SyncResult {
  /** Whether the synchronization was successful */
  success: boolean;
  /** Total number of operations processed */
  total: number;
  /** Number of operations successfully synced */
  completed: number;
  /** Number of operations that failed to sync */
  failed: number;
  /** Number of operations that were skipped */
  skipped: number;
  /** Number of conflicts detected during synchronization */
  conflicts: number;
  /** List of operations that failed to sync */
  failedOperations: SyncOperation[];
  /** List of operations that were skipped */
  skippedOperations: SyncOperation[];
  /** List of conflicts detected during synchronization */
  detectedConflicts: Conflict[];
  /** Error information if the synchronization failed */
  error?: Error;
  /** Duration of the synchronization in milliseconds */
  duration: number;
}

/**
 * Synchronization manager for offline operations
 */
export class SyncManager {
  private options: Required<SyncOptions>;
  private progress: SyncProgress;
  private operations: SyncOperation[] = [];
  private isSyncing: boolean = false;
  private syncPromise: Promise<SyncResult> | null = null;
  private abortController: AbortController | null = null;

  /**
   * Creates a new synchronization manager
   * @param options - Options for the synchronization manager
   */
  constructor(options: SyncOptions = {}) {
    this.options = { ...DEFAULT_SYNC_OPTIONS, ...options };
    this.progress = {
      total: 0,
      completed: 0,
      failed: 0,
      status: SyncStatus.IDLE,
      startTime: 0
    };
  }

  /**
   * Prepares operations for synchronization by analyzing dependencies
   * @param operations - The operations to prepare
   * @returns The prepared operations with resolved dependencies
   */
  private prepareOperations(operations: QueuedOperation[]): SyncOperation[] {
    const syncOperations: SyncOperation[] = operations.map(op => ({
      ...op,
      dependencies: [],
      synced: false,
      skipped: false
    }));

    if (this.options.autoResolveDependencies) {
      // Automatically resolve dependencies between operations
      this.resolveDependencies(syncOperations);
    }

    return syncOperations;
  }

  /**
   * Resolves dependencies between operations
   * @param operations - The operations to resolve dependencies for
   */
  private resolveDependencies(operations: SyncOperation[]): void {
    // Create a map of entity IDs to operations
    const entityMap: Record<string, SyncOperation[]> = {};

    // First pass: build the entity map
    operations.forEach(op => {
      if (op.entityType && op.entityId) {
        if (!entityMap[`${op.entityType}:${op.entityId}`]) {
          entityMap[`${op.entityType}:${op.entityId}`] = [];
        }
        entityMap[`${op.entityType}:${op.entityId}`].push(op);
      }
    });

    // Second pass: resolve dependencies
    operations.forEach(op => {
      // For update and delete operations, add dependencies on create operations for the same entity
      if ((op.type === OperationType.UPDATE || op.type === OperationType.DELETE) && op.entityId) {
        const key = `${op.entityType}:${op.entityId}`;
        const relatedOps = entityMap[key] || [];

        // Find create operations for this entity
        const createOps = relatedOps.filter(relOp => 
          relOp.type === OperationType.CREATE && 
          relOp.id !== op.id
        );

        // Add dependencies on create operations
        createOps.forEach(createOp => {
          if (!op.dependencies) {
            op.dependencies = [];
          }

          op.dependencies.push({
            dependsOnId: createOp.id,
            type: 'entity',
            entityId: op.entityId
          });
        });
      }

      // For operations on the same entity, maintain order based on timestamp
      if (op.entityId) {
        const key = `${op.entityType}:${op.entityId}`;
        const relatedOps = entityMap[key] || [];

        // Find operations that happened before this one
        const earlierOps = relatedOps.filter(relOp => 
          relOp.timestamp < op.timestamp && 
          relOp.id !== op.id
        );

        // Add dependencies to maintain order
        earlierOps.forEach(earlierOp => {
          if (!op.dependencies) {
            op.dependencies = [];
          }

          // Check if this dependency already exists
          const dependencyExists = op.dependencies.some(dep => 
            dep.dependsOnId === earlierOp.id
          );

          if (!dependencyExists) {
            op.dependencies.push({
              dependsOnId: earlierOp.id,
              type: 'order',
              entityId: op.entityId
            });
          }
        });
      }
    });
  }

  /**
   * Checks if an operation's dependencies are satisfied
   * @param operation - The operation to check
   * @param processedOperations - Map of processed operations by ID
   * @returns Whether the operation's dependencies are satisfied
   */
  private areDependenciesSatisfied(
    operation: SyncOperation,
    processedOperations: Map<string, SyncOperation>
  ): boolean {
    if (!operation.dependencies || operation.dependencies.length === 0) {
      return true;
    }

    return operation.dependencies.every(dependency => {
      const dependsOn = processedOperations.get(dependency.dependsOnId);

      // If the dependency doesn't exist or wasn't processed successfully, it's not satisfied
      if (!dependsOn || !dependsOn.synced) {
        return false;
      }

      // For custom dependencies, use the validate function
      if (dependency.type === 'custom' && dependency.validate) {
        return dependency.validate(operation, dependsOn);
      }

      // For entity and order dependencies, the dependency is satisfied if it was processed successfully
      return true;
    });
  }

  /**
   * Processes a single operation
   * @param operation - The operation to process
   * @param ipcInvoke - The IPC invoke function to use
   * @returns The result of the operation
   */
  private async processOperation(
    operation: SyncOperation,
    ipcInvoke: (method: string, params?: Record<string, any>) => Promise<any>
  ): Promise<any> {
    try {
      // Update progress
      this.progress.currentOperation = operation;
      this.options.onProgress(this.progress);

      // Process the operation
      const result = await ipcInvoke(operation.method, operation.params);

      // Mark as synced and store the result
      operation.synced = true;
      operation.result = result;

      // Update progress
      this.progress.completed++;
      this.options.onProgress(this.progress);
      this.options.onOperationSynced(operation, result);

      return result;
    } catch (error) {
      // Mark as failed
      operation.synced = false;
      operation.retryCount++;

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

      // Update progress
      this.progress.failed++;
      this.options.onProgress(this.progress);
      this.options.onOperationFailed(operation, error instanceof Error ? error : new Error(String(error)));

      throw error;
    }
  }

  /**
   * Synchronizes offline operations with the server
   * @param operations - The operations to synchronize (if not provided, uses the offline queue)
   * @param ipcInvoke - The IPC invoke function to use
   * @returns A promise that resolves to the synchronization result
   */
  async synchronize(
    operations?: QueuedOperation[],
    ipcInvoke?: (method: string, params?: Record<string, any>) => Promise<any>
  ): Promise<SyncResult> {
    // If already syncing, return the existing promise
    if (this.isSyncing && this.syncPromise) {
      return this.syncPromise;
    }

    this.isSyncing = true;
    this.abortController = new AbortController();

    // Create a new promise for this synchronization cycle
    this.syncPromise = (async () => {
      const startTime = Date.now();

      // Initialize progress
      this.progress = {
        total: 0,
        completed: 0,
        failed: 0,
        status: SyncStatus.SYNCING,
        startTime
      };

      try {
        // Get operations from the offline queue if not provided
        const queuedOperations = operations || offlineQueueManager.getQueuedOperations();

        // Prepare operations for synchronization
        this.operations = this.prepareOperations(queuedOperations);
        this.progress.total = this.operations.length;
        this.options.onProgress(this.progress);

        // If there are no operations, return immediately
        if (this.operations.length === 0) {
          this.progress.status = SyncStatus.COMPLETED;
          this.progress.endTime = Date.now();
          this.options.onProgress(this.progress);

          const result: SyncResult = {
            success: true,
            total: 0,
            completed: 0,
            failed: 0,
            skipped: 0,
            failedOperations: [],
            skippedOperations: [],
            duration: this.progress.endTime - startTime
          };

          this.options.onComplete(result);
          return result;
        }

        // Use the provided IPC invoke function or a mock one
        const invokeFunction = ipcInvoke || ((method: string, params?: Record<string, any>) => {
          console.warn('No IPC invoke function provided, using mock implementation');
          return Promise.resolve({ success: true, data: { id: 'mock-id' } });
        });

        // Process operations in dependency order
        const processedOperations = new Map<string, SyncOperation>();
        const failedOperations: SyncOperation[] = [];
        const skippedOperations: SyncOperation[] = [];

        // Continue processing until all operations are processed or we can't make progress
        let progress = true;
        while (progress && !this.abortController.signal.aborted) {
          progress = false;

          for (const operation of this.operations) {
            // Skip operations that have already been processed
            if (operation.synced || operation.skipped) {
              continue;
            }

            // Check if dependencies are satisfied
            if (!this.areDependenciesSatisfied(operation, processedOperations)) {
              // If dependencies failed, skip this operation
              const failedDependencies = operation.dependencies?.filter(dep => {
                const dependsOn = processedOperations.get(dep.dependsOnId);
                return dependsOn && !dependsOn.synced;
              });

              if (failedDependencies && failedDependencies.length > 0) {
                operation.skipped = true;
                operation.skipReason = 'Failed dependencies';
                skippedOperations.push(operation);
                progress = true;
                continue;
              }

              // Dependencies not processed yet, skip for now
              continue;
            }

            try {
              // Process the operation
              await this.processOperation(operation, invokeFunction);
              processedOperations.set(operation.id, operation);
              progress = true;
            } catch (error) {
              // If we should continue on error, mark as failed and continue
              if (this.options.continueOnError) {
                failedOperations.push(operation);
                processedOperations.set(operation.id, operation);
                progress = true;
              } else {
                // Otherwise, stop processing
                throw error;
              }
            }
          }
        }

        // Check if we were aborted
        if (this.abortController.signal.aborted) {
          throw new Error('Synchronization aborted');
        }

        // Check if there are operations that couldn't be processed due to circular dependencies
        const remainingOperations = this.operations.filter(op => !op.synced && !op.skipped);
        if (remainingOperations.length > 0) {
          // Mark as skipped due to circular dependencies
          remainingOperations.forEach(op => {
            op.skipped = true;
            op.skipReason = 'Circular dependency';
            skippedOperations.push(op);
          });
        }

        // Update progress
        this.progress.status = failedOperations.length > 0 ? SyncStatus.PARTIALLY_COMPLETED : SyncStatus.COMPLETED;
        this.progress.endTime = Date.now();
        this.options.onProgress(this.progress);

        // Create result
        const result: SyncResult = {
          success: failedOperations.length === 0,
          total: this.operations.length,
          completed: this.progress.completed,
          failed: failedOperations.length,
          skipped: skippedOperations.length,
          failedOperations,
          skippedOperations,
          duration: this.progress.endTime - startTime
        };

        this.options.onComplete(result);
        return result;
      } catch (error) {
        // Update progress
        this.progress.status = SyncStatus.FAILED;
        this.progress.endTime = Date.now();
        this.progress.error = error instanceof Error ? error : new Error(String(error));
        this.options.onProgress(this.progress);

        // Create result
        const result: SyncResult = {
          success: false,
          total: this.operations.length,
          completed: this.progress.completed,
          failed: this.progress.failed,
          skipped: 0,
          failedOperations: this.operations.filter(op => !op.synced && !op.skipped),
          skippedOperations: this.operations.filter(op => op.skipped),
          error: this.progress.error,
          duration: this.progress.endTime - startTime
        };

        this.options.onComplete(result);
        return result;
      } finally {
        this.isSyncing = false;
        this.syncPromise = null;
        this.abortController = null;
      }
    })();

    return this.syncPromise;
  }

  /**
   * Aborts the current synchronization process
   */
  abort(): void {
    if (this.abortController) {
      this.abortController.abort();
    }
  }

  /**
   * Gets the current synchronization progress
   * @returns The current progress
   */
  getProgress(): SyncProgress {
    return { ...this.progress };
  }

  /**
   * Checks if synchronization is in progress
   * @returns Whether synchronization is in progress
   */
  isSynchronizing(): boolean {
    return this.isSyncing;
  }
}

// Create a singleton instance of the sync manager
export const syncManager = new SyncManager();

/**
 * Event types for synchronization
 */
export enum SyncEventType {
  PROGRESS = 'sync_progress',
  COMPLETED = 'sync_completed',
  FAILED = 'sync_failed',
  OPERATION_SYNCED = 'operation_synced',
  OPERATION_FAILED = 'operation_failed',
  CONFLICT_DETECTED = 'conflict_detected',
  CONFLICT_RESOLVED = 'conflict_resolved'
}

/**
 * Base event interface for synchronization events
 */
export interface SyncEvent {
  type: SyncEventType;
  timestamp: number;
}

/**
 * Progress event for synchronization
 */
export interface SyncProgressEvent extends SyncEvent {
  type: SyncEventType.PROGRESS;
  progress: SyncProgress;
}

/**
 * Completed event for synchronization
 */
export interface SyncCompletedEvent extends SyncEvent {
  type: SyncEventType.COMPLETED;
  result: SyncResult;
}

/**
 * Failed event for synchronization
 */
export interface SyncFailedEvent extends SyncEvent {
  type: SyncEventType.FAILED;
  error: Error;
  progress: SyncProgress;
}

/**
 * Operation synced event
 */
export interface OperationSyncedEvent extends SyncEvent {
  type: SyncEventType.OPERATION_SYNCED;
  operation: SyncOperation;
  result: any;
}

/**
 * Operation failed event
 */
export interface OperationFailedEvent extends SyncEvent {
  type: SyncEventType.OPERATION_FAILED;
  operation: SyncOperation;
  error: Error;
}

/**
 * Conflict detected event
 */
export interface ConflictDetectedEvent extends SyncEvent {
  type: SyncEventType.CONFLICT_DETECTED;
  conflict: Conflict;
}

/**
 * Conflict resolved event
 */
export interface ConflictResolvedEvent extends SyncEvent {
  type: SyncEventType.CONFLICT_RESOLVED;
  conflict: Conflict;
  resolution: ConflictResolution;
}

/**
 * Union type for all synchronization events
 */
export type SyncEventUnion = 
  | SyncProgressEvent
  | SyncCompletedEvent
  | SyncFailedEvent
  | OperationSyncedEvent
  | OperationFailedEvent
  | ConflictDetectedEvent
  | ConflictResolvedEvent;

/**
 * Event listener for synchronization events
 */
export type SyncEventListener = (event: SyncEventUnion) => void;

/**
 * Event emitter for synchronization events
 */
export class SyncEventEmitter {
  private listeners: Map<SyncEventType, Set<SyncEventListener>> = new Map();

  /**
   * Adds an event listener
   * @param type - The event type to listen for
   * @param listener - The listener function
   */
  addEventListener(type: SyncEventType, listener: SyncEventListener): void {
    if (!this.listeners.has(type)) {
      this.listeners.set(type, new Set());
    }
    this.listeners.get(type)!.add(listener);
  }

  /**
   * Removes an event listener
   * @param type - The event type to remove the listener from
   * @param listener - The listener function to remove
   */
  removeEventListener(type: SyncEventType, listener: SyncEventListener): void {
    if (this.listeners.has(type)) {
      this.listeners.get(type)!.delete(listener);
    }
  }

  /**
   * Emits an event
   * @param event - The event to emit
   */
  emit(event: SyncEventUnion): void {
    // Call listeners for the specific event type
    if (this.listeners.has(event.type)) {
      for (const listener of this.listeners.get(event.type)!) {
        listener(event);
      }
    }

    // Call listeners for all event types
    if (this.listeners.has(SyncEventType.PROGRESS) && 
        event.type !== SyncEventType.PROGRESS) {
      for (const listener of this.listeners.get(SyncEventType.PROGRESS)!) {
        listener(event);
      }
    }
  }
}

// Create a singleton instance of the event emitter
export const syncEvents = new SyncEventEmitter();

// Configure the sync manager to emit events
syncManager.synchronize({
  onProgress: (progress) => {
    syncEvents.emit({
      type: SyncEventType.PROGRESS,
      timestamp: Date.now(),
      progress
    });
  },
  onComplete: (result) => {
    syncEvents.emit({
      type: SyncEventType.COMPLETED,
      timestamp: Date.now(),
      result
    });
  },
  onOperationSynced: (operation, result) => {
    syncEvents.emit({
      type: SyncEventType.OPERATION_SYNCED,
      timestamp: Date.now(),
      operation,
      result
    });
  },
  onOperationFailed: (operation, error) => {
    syncEvents.emit({
      type: SyncEventType.OPERATION_FAILED,
      timestamp: Date.now(),
      operation,
      error
    });
  }
});
