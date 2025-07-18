/**
 * Conflict resolution for offline operations
 * 
 * This module provides mechanisms for detecting and resolving conflicts
 * that occur when synchronizing offline operations with the server.
 */

import { ApiResponse } from './types';
import { ErrorCategory, ApiError, Errors } from './errors';
import { QueuedOperation, OperationType } from './offline';
import { SyncOperation } from './sync';

/**
 * Types of conflicts that can occur during synchronization
 */
export enum ConflictType {
  /** Local update conflicts with server update */
  UPDATE_UPDATE = 'update_update',
  /** Local update conflicts with server delete */
  UPDATE_DELETE = 'update_delete',
  /** Local delete conflicts with server update */
  DELETE_UPDATE = 'delete_update',
  /** Local create conflicts with server create (same ID) */
  CREATE_CREATE = 'create_create',
  /** Local operation conflicts with server state in an unspecified way */
  GENERIC = 'generic'
}

/**
 * Conflict information
 */
export interface Conflict {
  /** Type of conflict */
  type: ConflictType;
  /** The operation that caused the conflict */
  operation: SyncOperation;
  /** The server state that conflicts with the operation */
  serverState?: any;
  /** The local state before the operation */
  localState?: any;
  /** Timestamp when the conflict was detected */
  timestamp: number;
  /** Whether the conflict has been resolved */
  resolved: boolean;
  /** How the conflict was resolved (if resolved) */
  resolution?: ConflictResolution;
}

/**
 * Resolution strategies for conflicts
 */
export enum ConflictStrategy {
  /** Use the local changes */
  CLIENT_WINS = 'client_wins',
  /** Use the server changes */
  SERVER_WINS = 'server_wins',
  /** Merge the changes */
  MERGE = 'merge',
  /** Use three-way merge with base version */
  THREE_WAY_MERGE = 'three_way_merge',
  /** Use structural merge for complex data types */
  STRUCTURAL_MERGE = 'structural_merge',
  /** Use differential synchronization */
  DIFFERENTIAL = 'differential',
  /** Prompt the user to resolve the conflict */
  MANUAL = 'manual',
  /** Skip the operation */
  SKIP = 'skip'
}

/**
 * Result of conflict resolution
 */
export interface ConflictResolution {
  /** The strategy used to resolve the conflict */
  strategy: ConflictStrategy;
  /** The resolved data (if applicable) */
  data?: any;
  /** Whether the resolution was successful */
  success: boolean;
  /** Error information if the resolution failed */
  error?: Error;
  /** Timestamp when the resolution was applied */
  timestamp: number;
}

/**
 * Options for conflict resolution
 */
export interface ConflictResolutionOptions {
  /** Default strategy for resolving conflicts */
  defaultStrategy?: ConflictStrategy;
  /** Custom strategies for specific entity types */
  entityStrategies?: Record<string, ConflictStrategy>;
  /** Custom strategies for specific conflict types */
  conflictTypeStrategies?: Record<ConflictType, ConflictStrategy>;
  /** Custom resolution functions for specific entity types */
  customResolvers?: Record<string, ConflictResolver>;
  /** Whether to continue synchronization if a conflict cannot be resolved */
  continueOnUnresolved?: boolean;
  /** Callback when a conflict is detected */
  onConflictDetected?: (conflict: Conflict) => void;
  /** Callback when a conflict is resolved */
  onConflictResolved?: (conflict: Conflict, resolution: ConflictResolution) => void;
  /** Callback to prompt the user for manual resolution */
  manualResolutionPrompt?: (conflict: Conflict) => Promise<any>;
  /** Function to retrieve the base version for three-way merge */
  getBaseVersion?: (entityType: string, entityId: string) => Promise<any>;
  /** Options for structural merge */
  structuralMergeOptions?: {
    /** How to handle array conflicts (append, replace, merge) */
    arrayStrategy?: 'append' | 'replace' | 'merge';
    /** Whether to perform deep merge of nested objects */
    deepMerge?: boolean;
    /** Custom merge functions for specific fields */
    fieldMergeFunctions?: Record<string, (client: any, server: any, base?: any) => any>;
  };
  /** Options for differential synchronization */
  differentialOptions?: {
    /** Function to compute differences between versions */
    computeDiff?: (original: any, modified: any) => any;
    /** Function to apply a patch to an object */
    applyPatch?: (target: any, patch: any) => any;
  };
}

/**
 * Default options for conflict resolution
 */
export const DEFAULT_CONFLICT_OPTIONS: Required<ConflictResolutionOptions> = {
  defaultStrategy: ConflictStrategy.SERVER_WINS,
  entityStrategies: {},
  conflictTypeStrategies: {
    [ConflictType.UPDATE_UPDATE]: ConflictStrategy.SERVER_WINS,
    [ConflictType.UPDATE_DELETE]: ConflictStrategy.SERVER_WINS,
    [ConflictType.DELETE_UPDATE]: ConflictStrategy.SERVER_WINS,
    [ConflictType.CREATE_CREATE]: ConflictStrategy.CLIENT_WINS,
    [ConflictType.GENERIC]: ConflictStrategy.SERVER_WINS
  },
  customResolvers: {},
  continueOnUnresolved: true,
  onConflictDetected: () => {},
  onConflictResolved: () => {},
  manualResolutionPrompt: async () => { throw new Error('Manual resolution not implemented'); },
  getBaseVersion: async () => null,
  structuralMergeOptions: {
    arrayStrategy: 'merge',
    deepMerge: true,
    fieldMergeFunctions: {}
  },
  differentialOptions: {
    computeDiff: (original, modified) => {
      // Simple default diff implementation
      if (typeof original !== 'object' || typeof modified !== 'object') {
        return modified;
      }
      const diff: Record<string, any> = {};
      // Find added or modified properties
      for (const key in modified) {
        if (!original || original[key] !== modified[key]) {
          diff[key] = modified[key];
        }
      }
      return diff;
    },
    applyPatch: (target, patch) => {
      // Simple default patch implementation
      if (typeof target !== 'object' || typeof patch !== 'object') {
        return patch;
      }
      return { ...target, ...patch };
    }
  }
};

/**
 * Function type for custom conflict resolution
 */
export type ConflictResolver = (
  conflict: Conflict,
  options: ConflictResolutionOptions
) => Promise<ConflictResolution>;

/**
 * Conflict manager for handling conflicts during synchronization
 */
export class ConflictManager {
  private options: Required<ConflictResolutionOptions>;
  private conflicts: Conflict[] = [];

  /**
   * Creates a new conflict manager
   * @param options - Options for conflict resolution
   */
  constructor(options: ConflictResolutionOptions = {}) {
    this.options = { ...DEFAULT_CONFLICT_OPTIONS, ...options };
  }

  /**
   * Sets the options for conflict resolution
   * @param options - The new options
   */
  setOptions(options: ConflictResolutionOptions): void {
    this.options = { ...this.options, ...options };
  }

  /**
   * Gets all detected conflicts
   * @returns All conflicts
   */
  getConflicts(): Conflict[] {
    return [...this.conflicts];
  }

  /**
   * Gets unresolved conflicts
   * @returns Unresolved conflicts
   */
  getUnresolvedConflicts(): Conflict[] {
    return this.conflicts.filter(conflict => !conflict.resolved);
  }

  /**
   * Detects conflicts between an operation and the server state
   * @param operation - The operation to check
   * @param serverResponse - The server response
   * @returns A conflict if detected, undefined otherwise
   */
  detectConflict(
    operation: SyncOperation,
    serverResponse: ApiResponse<any>
  ): Conflict | undefined {
    // If the operation succeeded, there's no conflict
    if (serverResponse.success) {
      return undefined;
    }

    // Check for specific error codes that indicate conflicts
    const errorCode = serverResponse.errorCode || '';

    // Determine the conflict type based on the operation and error
    let conflictType = ConflictType.GENERIC;

    if (operation.type === OperationType.UPDATE) {
      if (errorCode.includes('version_conflict') || errorCode.includes('concurrent_modification')) {
        conflictType = ConflictType.UPDATE_UPDATE;
      } else if (errorCode.includes('not_found') || errorCode.includes('deleted')) {
        conflictType = ConflictType.UPDATE_DELETE;
      }
    } else if (operation.type === OperationType.DELETE) {
      if (errorCode.includes('version_conflict') || errorCode.includes('concurrent_modification')) {
        conflictType = ConflictType.DELETE_UPDATE;
      }
    } else if (operation.type === OperationType.CREATE) {
      if (errorCode.includes('duplicate') || errorCode.includes('already_exists')) {
        conflictType = ConflictType.CREATE_CREATE;
      }
    }

    // Create the conflict
    const conflict: Conflict = {
      type: conflictType,
      operation,
      serverState: serverResponse.data,
      timestamp: Date.now(),
      resolved: false
    };

    // Add to the list of conflicts
    this.conflicts.push(conflict);

    // Notify listeners
    this.options.onConflictDetected(conflict);

    return conflict;
  }

  /**
   * Resolves a conflict using the appropriate strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  async resolveConflict(conflict: Conflict): Promise<ConflictResolution> {
    // If already resolved, return the existing resolution
    if (conflict.resolved && conflict.resolution) {
      return conflict.resolution;
    }

    try {
      // Determine the strategy to use
      const strategy = this.getResolutionStrategy(conflict);

      // Apply the strategy
      let resolution: ConflictResolution;

      switch (strategy) {
        case ConflictStrategy.CLIENT_WINS:
          resolution = await this.resolveClientWins(conflict);
          break;
        case ConflictStrategy.SERVER_WINS:
          resolution = await this.resolveServerWins(conflict);
          break;
        case ConflictStrategy.MERGE:
          resolution = await this.resolveMerge(conflict);
          break;
        case ConflictStrategy.THREE_WAY_MERGE:
          resolution = await this.resolveThreeWayMerge(conflict);
          break;
        case ConflictStrategy.STRUCTURAL_MERGE:
          resolution = await this.resolveStructuralMerge(conflict);
          break;
        case ConflictStrategy.DIFFERENTIAL:
          resolution = await this.resolveDifferential(conflict);
          break;
        case ConflictStrategy.MANUAL:
          resolution = await this.resolveManual(conflict);
          break;
        case ConflictStrategy.SKIP:
          resolution = await this.resolveSkip(conflict);
          break;
        default:
          resolution = await this.resolveServerWins(conflict);
      }

      // Update the conflict with the resolution
      conflict.resolved = resolution.success;
      conflict.resolution = resolution;

      // Notify listeners
      this.options.onConflictResolved(conflict, resolution);

      return resolution;
    } catch (error) {
      // If resolution fails, mark as unresolved
      const resolution: ConflictResolution = {
        strategy: this.getResolutionStrategy(conflict),
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };

      conflict.resolution = resolution;

      // Notify listeners
      this.options.onConflictResolved(conflict, resolution);

      return resolution;
    }
  }

  /**
   * Gets the appropriate resolution strategy for a conflict
   * @param conflict - The conflict to resolve
   * @returns The resolution strategy
   */
  private getResolutionStrategy(conflict: Conflict): ConflictStrategy {
    // Check for a custom resolver
    if (conflict.operation.entityType && 
        this.options.customResolvers[conflict.operation.entityType]) {
      return ConflictStrategy.MERGE; // Custom resolvers use the merge strategy
    }

    // Check for entity-specific strategy
    if (conflict.operation.entityType && 
        this.options.entityStrategies[conflict.operation.entityType]) {
      return this.options.entityStrategies[conflict.operation.entityType];
    }

    // Check for conflict type strategy
    if (this.options.conflictTypeStrategies[conflict.type]) {
      return this.options.conflictTypeStrategies[conflict.type];
    }

    // Use default strategy
    return this.options.defaultStrategy;
  }

  /**
   * Resolves a conflict using the client-wins strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveClientWins(conflict: Conflict): Promise<ConflictResolution> {
    // In client-wins, we simply keep the local changes
    return {
      strategy: ConflictStrategy.CLIENT_WINS,
      data: conflict.operation.params,
      success: true,
      timestamp: Date.now()
    };
  }

  /**
   * Resolves a conflict using the server-wins strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveServerWins(conflict: Conflict): Promise<ConflictResolution> {
    // In server-wins, we discard the local changes
    return {
      strategy: ConflictStrategy.SERVER_WINS,
      data: conflict.serverState,
      success: true,
      timestamp: Date.now()
    };
  }

  /**
   * Resolves a conflict using the merge strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveMerge(conflict: Conflict): Promise<ConflictResolution> {
    // Check for a custom resolver
    if (conflict.operation.entityType && 
        this.options.customResolvers[conflict.operation.entityType]) {
      // Use the custom resolver
      return this.options.customResolvers[conflict.operation.entityType](conflict, this.options);
    }

    // Default merge strategy (simple field-level merge)
    try {
      let mergedData: any;

      if (conflict.type === ConflictType.UPDATE_UPDATE) {
        // For update-update conflicts, merge the fields
        const clientData = conflict.operation.params.data || {};
        const serverData = conflict.serverState || {};

        // Simple merge: take all fields from both, with server taking precedence for conflicts
        mergedData = { ...clientData, ...serverData };
      } else {
        // For other conflict types, default to server data
        mergedData = conflict.serverState;
      }

      return {
        strategy: ConflictStrategy.MERGE,
        data: mergedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.MERGE,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  }

  /**
   * Resolves a conflict using the three-way merge strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveThreeWayMerge(conflict: Conflict): Promise<ConflictResolution> {
    try {
      // Get the base version (common ancestor)
      const baseVersion = conflict.operation.entityType && conflict.operation.entityId
        ? await this.options.getBaseVersion(conflict.operation.entityType, conflict.operation.entityId)
        : null;

      // If no base version is available, fall back to regular merge
      if (!baseVersion) {
        return this.resolveMerge(conflict);
      }

      const clientData = conflict.operation.params.data || {};
      const serverData = conflict.serverState || {};

      // Perform three-way merge
      const mergedData: Record<string, any> = {};

      // Get all keys from all three versions
      const allKeys = new Set([
        ...Object.keys(baseVersion),
        ...Object.keys(clientData),
        ...Object.keys(serverData)
      ]);

      // Process each key
      for (const key of allKeys) {
        const baseValue = baseVersion[key];
        const clientValue = clientData[key];
        const serverValue = serverData[key];

        // Case 1: If client and server values are the same, use that value
        if (this.areValuesEqual(clientValue, serverValue)) {
          mergedData[key] = clientValue;
          continue;
        }

        // Case 2: If client value equals base value, server changed it, use server value
        if (this.areValuesEqual(clientValue, baseValue)) {
          mergedData[key] = serverValue;
          continue;
        }

        // Case 3: If server value equals base value, client changed it, use client value
        if (this.areValuesEqual(serverValue, baseValue)) {
          mergedData[key] = clientValue;
          continue;
        }

        // Case 4: Both client and server changed the value differently
        // For objects, try to recursively merge
        if (
          typeof clientValue === 'object' && clientValue !== null &&
          typeof serverValue === 'object' && serverValue !== null &&
          typeof baseValue === 'object' && baseValue !== null &&
          !Array.isArray(clientValue) && !Array.isArray(serverValue) && !Array.isArray(baseValue)
        ) {
          // Recursive three-way merge for nested objects
          const nestedConflict: Conflict = {
            ...conflict,
            operation: {
              ...conflict.operation,
              params: { data: clientValue }
            },
            serverState: serverValue,
            localState: baseValue
          };

          const nestedResolution = await this.resolveThreeWayMerge(nestedConflict);
          mergedData[key] = nestedResolution.data;
        } else {
          // For non-objects or arrays, use server value by default
          // This could be customized based on field-specific rules
          mergedData[key] = serverValue;
        }
      }

      return {
        strategy: ConflictStrategy.THREE_WAY_MERGE,
        data: mergedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.THREE_WAY_MERGE,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  }

  /**
   * Helper method to check if two values are equal
   * @param a - First value
   * @param b - Second value
   * @returns Whether the values are equal
   */
  private areValuesEqual(a: any, b: any): boolean {
    // Handle null/undefined
    if (a == null && b == null) return true;
    if (a == null || b == null) return false;

    // Handle primitive types
    if (typeof a !== 'object' && typeof b !== 'object') {
      return a === b;
    }

    // Handle arrays
    if (Array.isArray(a) && Array.isArray(b)) {
      if (a.length !== b.length) return false;
      return a.every((val, idx) => this.areValuesEqual(val, b[idx]));
    }

    // Handle objects
    if (typeof a === 'object' && typeof b === 'object') {
      const keysA = Object.keys(a);
      const keysB = Object.keys(b);

      if (keysA.length !== keysB.length) return false;

      return keysA.every(key => 
        keysB.includes(key) && this.areValuesEqual(a[key], b[key])
      );
    }

    return false;
  }

  /**
   * Resolves a conflict using the structural merge strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveStructuralMerge(conflict: Conflict): Promise<ConflictResolution> {
    try {
      const clientData = conflict.operation.params.data || {};
      const serverData = conflict.serverState || {};
      const options = this.options.structuralMergeOptions;

      // Perform structural merge
      const mergedData = this.mergeStructures(clientData, serverData, options);

      return {
        strategy: ConflictStrategy.STRUCTURAL_MERGE,
        data: mergedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.STRUCTURAL_MERGE,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  }

  /**
   * Merges two data structures recursively
   * @param client - Client data structure
   * @param server - Server data structure
   * @param options - Structural merge options
   * @param path - Current path in the object (for field-specific functions)
   * @returns Merged data structure
   */
  private mergeStructures(
    client: any,
    server: any,
    options: Required<ConflictResolutionOptions>['structuralMergeOptions'],
    path: string = ''
  ): any {
    // Handle null/undefined cases
    if (client == null) return server;
    if (server == null) return client;

    // Check for field-specific merge function
    if (path && options.fieldMergeFunctions[path]) {
      return options.fieldMergeFunctions[path](client, server);
    }

    // Handle arrays
    if (Array.isArray(client) && Array.isArray(server)) {
      return this.mergeArrays(client, server, options.arrayStrategy);
    }

    // Handle objects (but not arrays)
    if (
      typeof client === 'object' && !Array.isArray(client) &&
      typeof server === 'object' && !Array.isArray(server)
    ) {
      // If deep merge is disabled, just use shallow merge
      if (!options.deepMerge) {
        return { ...client, ...server };
      }

      // Deep merge objects
      const result: Record<string, any> = { ...client };

      // Process all keys from server object
      for (const key of Object.keys(server)) {
        const newPath = path ? `${path}.${key}` : key;

        // If key exists in both, merge recursively
        if (key in client) {
          result[key] = this.mergeStructures(client[key], server[key], options, newPath);
        } else {
          // Key only in server, add it
          result[key] = server[key];
        }
      }

      return result;
    }

    // For primitive types or incompatible types, prefer server value
    return server;
  }

  /**
   * Merges two arrays based on the specified strategy
   * @param clientArray - Client array
   * @param serverArray - Server array
   * @param strategy - Array merge strategy
   * @returns Merged array
   */
  private mergeArrays(
    clientArray: any[],
    serverArray: any[],
    strategy: 'append' | 'replace' | 'merge'
  ): any[] {
    switch (strategy) {
      case 'append':
        // Combine both arrays, removing duplicates
        return [...new Set([...clientArray, ...serverArray])];

      case 'replace':
        // Use server array
        return [...serverArray];

      case 'merge':
        // Try to merge items by position or ID
        if (clientArray.length === 0) return [...serverArray];
        if (serverArray.length === 0) return [...clientArray];

        // Check if items have IDs
        const hasIds = (
          typeof clientArray[0] === 'object' && clientArray[0] !== null && 'id' in clientArray[0] &&
          typeof serverArray[0] === 'object' && serverArray[0] !== null && 'id' in serverArray[0]
        );

        if (hasIds) {
          // Merge by ID
          const result: any[] = [];
          const clientMap = new Map(
            clientArray
              .filter(item => item && typeof item === 'object' && 'id' in item)
              .map(item => [item.id, item])
          );

          // Add all server items
          for (const serverItem of serverArray) {
            if (serverItem && typeof serverItem === 'object' && 'id' in serverItem) {
              const clientItem = clientMap.get(serverItem.id);

              if (clientItem) {
                // Item exists in both, merge objects
                result.push({ ...clientItem, ...serverItem });
                clientMap.delete(serverItem.id);
              } else {
                // Item only in server
                result.push(serverItem);
              }
            } else {
              // Item doesn't have ID, just add it
              result.push(serverItem);
            }
          }

          // Add remaining client items
          for (const [, clientItem] of clientMap) {
            result.push(clientItem);
          }

          return result;
        } else {
          // No IDs, merge by position up to the length of the shorter array,
          // then append remaining items from the longer array
          const result: any[] = [];
          const minLength = Math.min(clientArray.length, serverArray.length);

          // Merge items at same positions
          for (let i = 0; i < minLength; i++) {
            // Prefer server items for primitive values
            if (typeof clientArray[i] === 'object' && typeof serverArray[i] === 'object') {
              result.push({ ...clientArray[i], ...serverArray[i] });
            } else {
              result.push(serverArray[i]);
            }
          }

          // Append remaining items
          if (clientArray.length > minLength) {
            result.push(...clientArray.slice(minLength));
          } else if (serverArray.length > minLength) {
            result.push(...serverArray.slice(minLength));
          }

          return result;
        }

      default:
        // Default to server array
        return [...serverArray];
    }
  }

  /**
   * Resolves a conflict using differential synchronization
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveDifferential(conflict: Conflict): Promise<ConflictResolution> {
    try {
      // Get the base version (common ancestor)
      const baseVersion = conflict.operation.entityType && conflict.operation.entityId
        ? await this.options.getBaseVersion(conflict.operation.entityType, conflict.operation.entityId)
        : null;

      // If no base version is available, fall back to structural merge
      if (!baseVersion) {
        return this.resolveStructuralMerge(conflict);
      }

      const clientData = conflict.operation.params.data || {};
      const serverData = conflict.serverState || {};
      const options = this.options.differentialOptions;

      // Compute the differences between base and client, and base and server
      const clientDiff = options.computeDiff(baseVersion, clientData);
      const serverDiff = options.computeDiff(baseVersion, serverData);

      // Apply both sets of changes to the base version
      // First apply client changes
      let mergedData = { ...baseVersion };
      mergedData = options.applyPatch(mergedData, clientDiff);

      // Then apply server changes
      mergedData = options.applyPatch(mergedData, serverDiff);

      // Special handling for text fields
      if (conflict.operation.entityType) {
        mergedData = this.handleTextFieldMerges(baseVersion, clientData, serverData, mergedData);
      }

      return {
        strategy: ConflictStrategy.DIFFERENTIAL,
        data: mergedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.DIFFERENTIAL,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  }

  /**
   * Special handling for text field merges
   * @param base - Base version
   * @param client - Client version
   * @param server - Server version
   * @param merged - Current merged result
   * @returns Updated merged result with text fields properly merged
   */
  private handleTextFieldMerges(
    base: Record<string, any>,
    client: Record<string, any>,
    server: Record<string, any>,
    merged: Record<string, any>
  ): Record<string, any> {
    const result = { ...merged };

    // Find all string fields that were modified by both client and server
    for (const key of Object.keys(base)) {
      if (
        typeof base[key] === 'string' &&
        typeof client[key] === 'string' &&
        typeof server[key] === 'string' &&
        client[key] !== base[key] &&
        server[key] !== base[key] &&
        client[key] !== server[key]
      ) {
        // This is a text field with conflicting changes
        // Use a simple line-based merge algorithm
        result[key] = this.mergeTextChanges(base[key], client[key], server[key]);
      }
    }

    return result;
  }

  /**
   * Merges text changes from client and server
   * @param base - Base text
   * @param client - Client text
   * @param server - Server text
   * @returns Merged text
   */
  private mergeTextChanges(base: string, client: string, server: string): string {
    // Split into lines
    const baseLines = base.split('\n');
    const clientLines = client.split('\n');
    const serverLines = server.split('\n');

    // Create line maps for faster lookup
    const baseLineMap = new Map(baseLines.map((line, idx) => [line, idx]));

    // Find lines added by client
    const clientAdded = clientLines.filter(line => !baseLineMap.has(line));

    // Find lines added by server
    const serverAdded = serverLines.filter(line => !baseLineMap.has(line));

    // Combine all lines, removing duplicates
    const allLines = new Set([...baseLines, ...clientAdded, ...serverAdded]);

    // Convert back to string
    return Array.from(allLines).join('\n');
  }

  /**
   * Resolves a conflict using manual resolution
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveManual(conflict: Conflict): Promise<ConflictResolution> {
    try {
      // Prompt the user for resolution
      const resolvedData = await this.options.manualResolutionPrompt(conflict);

      return {
        strategy: ConflictStrategy.MANUAL,
        data: resolvedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.MANUAL,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  }

  /**
   * Resolves a conflict by skipping the operation
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveSkip(conflict: Conflict): Promise<ConflictResolution> {
    return {
      strategy: ConflictStrategy.SKIP,
      success: true,
      timestamp: Date.now()
    };
  }
}

// Create a singleton instance of the conflict manager
export const conflictManager = new ConflictManager();

/**
 * Creates a custom field-level merge resolver
 * @param fieldRules - Rules for merging specific fields
 * @returns A conflict resolver function
 */
export function createFieldMergeResolver(
  fieldRules: Record<string, 'client' | 'server' | 'newest' | 'oldest' | ((client: any, server: any) => any)>
): ConflictResolver {
  return async (conflict: Conflict): Promise<ConflictResolution> => {
    try {
      const clientData = conflict.operation.params.data || {};
      const serverData = conflict.serverState || {};
      const mergedData = { ...clientData };

      // Apply field-specific rules
      for (const [field, rule] of Object.entries(fieldRules)) {
        if (typeof rule === 'function') {
          // Custom merge function for this field
          mergedData[field] = rule(clientData[field], serverData[field]);
        } else if (rule === 'client') {
          // Keep client value
          mergedData[field] = clientData[field];
        } else if (rule === 'server') {
          // Use server value
          mergedData[field] = serverData[field];
        } else if (rule === 'newest' || rule === 'oldest') {
          // Time-based resolution requires timestamp fields
          const clientTime = clientData.updatedAt || clientData.timestamp || Date.now();
          const serverTime = serverData.updatedAt || serverData.timestamp || Date.now();

          if (rule === 'newest') {
            mergedData[field] = clientTime > serverTime ? clientData[field] : serverData[field];
          } else {
            mergedData[field] = clientTime < serverTime ? clientData[field] : serverData[field];
          }
        }
      }

      return {
        strategy: ConflictStrategy.MERGE,
        data: mergedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.MERGE,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  };
}

/**
 * Creates a last-write-wins resolver based on timestamps
 * @param timestampField - The field containing the timestamp
 * @returns A conflict resolver function
 */
export function createLastWriteWinsResolver(
  timestampField: string = 'updatedAt'
): ConflictResolver {
  return async (conflict: Conflict): Promise<ConflictResolution> => {
    try {
      const clientData = conflict.operation.params.data || {};
      const serverData = conflict.serverState || {};

      const clientTime = clientData[timestampField] || conflict.operation.timestamp;
      const serverTime = serverData[timestampField] || Date.now();

      // Use the data with the most recent timestamp
      const resolvedData = clientTime > serverTime ? clientData : serverData;

      return {
        strategy: ConflictStrategy.MERGE,
        data: resolvedData,
        success: true,
        timestamp: Date.now()
      };
    } catch (error) {
      return {
        strategy: ConflictStrategy.MERGE,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
    }
  };
}
