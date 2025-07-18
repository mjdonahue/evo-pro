/**
 * Cross-Device Conflict Management
 * 
 * This module provides types and utilities for handling conflicts that occur
 * during cross-device synchronization.
 */

import { Conflict, ConflictType, ConflictResolution, ConflictStrategy } from './conflict';
import { SyncOperation } from './sync';
import { Device } from './crossDeviceSync';

/**
 * Extended conflict type for cross-device synchronization
 */
export interface CrossDeviceConflict extends Omit<Conflict, 'serverState'> {
  /** The operation that caused the conflict */
  operation: SyncOperation;
  /** The remote state that conflicts with the operation */
  serverState?: any;
  /** The local state before the operation */
  localState?: any;
  /** The source device where the conflict originated */
  sourceDevice: Device;
  /** The target device where the conflict was detected */
  targetDevice?: Device;
  /** The sync session ID where the conflict was detected */
  sessionId: string;
  /** Additional metadata about the conflict */
  metadata?: Record<string, any>;
}

/**
 * Options for cross-device conflict resolution
 */
export interface CrossDeviceConflictOptions {
  /** Default strategy for resolving conflicts */
  defaultStrategy?: ConflictStrategy;
  /** Device priority map (device ID to priority, higher number = higher priority) */
  devicePriorities?: Record<string, number>;
  /** Whether to prefer the current device when resolving conflicts */
  preferCurrentDevice?: boolean;
  /** Whether to use timestamps for conflict resolution */
  useTimestamps?: boolean;
  /** Whether to prompt the user for manual resolution */
  promptForResolution?: boolean;
  /** Custom resolution strategies by entity type */
  entityStrategies?: Record<string, ConflictStrategy>;
}

/**
 * Default options for cross-device conflict resolution
 */
export const DEFAULT_CROSS_DEVICE_CONFLICT_OPTIONS: CrossDeviceConflictOptions = {
  defaultStrategy: ConflictStrategy.SERVER_WINS,
  devicePriorities: {},
  preferCurrentDevice: true,
  useTimestamps: true,
  promptForResolution: false,
  entityStrategies: {}
};

/**
 * Cross-device conflict manager
 */
export class CrossDeviceConflictManager {
  private options: CrossDeviceConflictOptions;
  private conflicts: CrossDeviceConflict[] = [];
  private currentDevice: Device | null = null;
  
  /**
   * Creates a new cross-device conflict manager
   * @param options - Options for conflict resolution
   * @param currentDevice - The current device
   */
  constructor(options: CrossDeviceConflictOptions = {}, currentDevice: Device | null = null) {
    this.options = { ...DEFAULT_CROSS_DEVICE_CONFLICT_OPTIONS, ...options };
    this.currentDevice = currentDevice;
  }
  
  /**
   * Sets the current device
   * @param device - The current device
   */
  setCurrentDevice(device: Device): void {
    this.currentDevice = device;
  }
  
  /**
   * Gets all conflicts
   * @returns All conflicts
   */
  getConflicts(): CrossDeviceConflict[] {
    return [...this.conflicts];
  }
  
  /**
   * Gets unresolved conflicts
   * @returns Unresolved conflicts
   */
  getUnresolvedConflicts(): CrossDeviceConflict[] {
    return this.conflicts.filter(conflict => !conflict.resolved);
  }
  
  /**
   * Gets conflicts for a specific session
   * @param sessionId - The session ID
   * @returns Conflicts for the session
   */
  getConflictsForSession(sessionId: string): CrossDeviceConflict[] {
    return this.conflicts.filter(conflict => conflict.sessionId === sessionId);
  }
  
  /**
   * Adds a conflict
   * @param conflict - The conflict to add
   */
  addConflict(conflict: CrossDeviceConflict): void {
    this.conflicts.push(conflict);
  }
  
  /**
   * Resolves a conflict
   * @param conflict - The conflict to resolve
   * @param strategy - The strategy to use (if not provided, uses the default strategy)
   * @returns The resolution result
   */
  async resolveConflict(
    conflict: CrossDeviceConflict, 
    strategy?: ConflictStrategy
  ): Promise<ConflictResolution> {
    // If already resolved, return the existing resolution
    if (conflict.resolved && conflict.resolution) {
      return conflict.resolution;
    }
    
    // Determine the strategy to use
    const resolutionStrategy = strategy || this.getResolutionStrategy(conflict);
    
    try {
      // Apply the strategy
      let resolution: ConflictResolution;
      
      switch (resolutionStrategy) {
        case ConflictStrategy.CLIENT_WINS:
          resolution = this.resolveClientWins(conflict);
          break;
        case ConflictStrategy.SERVER_WINS:
          resolution = this.resolveServerWins(conflict);
          break;
        case ConflictStrategy.MERGE:
          resolution = this.resolveMerge(conflict);
          break;
        case ConflictStrategy.MANUAL:
          resolution = await this.resolveManual(conflict);
          break;
        default:
          // Default to device priority-based resolution
          resolution = this.resolveByDevicePriority(conflict);
      }
      
      // Update the conflict with the resolution
      conflict.resolved = resolution.success;
      conflict.resolution = resolution;
      
      return resolution;
    } catch (error) {
      // If resolution fails, mark as unresolved
      const resolution: ConflictResolution = {
        strategy: resolutionStrategy,
        success: false,
        error: error instanceof Error ? error : new Error(String(error)),
        timestamp: Date.now()
      };
      
      conflict.resolution = resolution;
      
      return resolution;
    }
  }
  
  /**
   * Gets the appropriate resolution strategy for a conflict
   * @param conflict - The conflict to resolve
   * @returns The resolution strategy
   */
  private getResolutionStrategy(conflict: CrossDeviceConflict): ConflictStrategy {
    // Check for entity-specific strategy
    if (conflict.operation.entityType && 
        this.options.entityStrategies?.[conflict.operation.entityType]) {
      return this.options.entityStrategies[conflict.operation.entityType];
    }
    
    // If manual resolution is enabled, use that
    if (this.options.promptForResolution) {
      return ConflictStrategy.MANUAL;
    }
    
    // Use default strategy
    return this.options.defaultStrategy || ConflictStrategy.SERVER_WINS;
  }
  
  /**
   * Resolves a conflict using the client-wins strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private resolveClientWins(conflict: CrossDeviceConflict): ConflictResolution {
    return {
      strategy: ConflictStrategy.CLIENT_WINS,
      data: conflict.operation.params.data,
      success: true,
      timestamp: Date.now()
    };
  }
  
  /**
   * Resolves a conflict using the server-wins strategy
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private resolveServerWins(conflict: CrossDeviceConflict): ConflictResolution {
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
  private resolveMerge(conflict: CrossDeviceConflict): ConflictResolution {
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
   * Resolves a conflict using manual resolution
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private async resolveManual(conflict: CrossDeviceConflict): Promise<ConflictResolution> {
    // In a real implementation, this would prompt the user for resolution
    // For this example, we'll just use server-wins
    return this.resolveServerWins(conflict);
  }
  
  /**
   * Resolves a conflict based on device priority
   * @param conflict - The conflict to resolve
   * @returns The resolution result
   */
  private resolveByDevicePriority(conflict: CrossDeviceConflict): ConflictResolution {
    // Get device priorities
    const sourceDevicePriority = this.getDevicePriority(conflict.sourceDevice);
    const targetDevicePriority = conflict.targetDevice 
      ? this.getDevicePriority(conflict.targetDevice) 
      : 0;
    
    // If source device has higher priority, use client-wins
    if (sourceDevicePriority > targetDevicePriority) {
      return this.resolveClientWins(conflict);
    }
    
    // Otherwise, use server-wins
    return this.resolveServerWins(conflict);
  }
  
  /**
   * Gets the priority of a device
   * @param device - The device
   * @returns The device priority
   */
  private getDevicePriority(device: Device): number {
    // Check if there's an explicit priority set
    if (this.options.devicePriorities && device.id in this.options.devicePriorities) {
      return this.options.devicePriorities[device.id];
    }
    
    // If preferCurrentDevice is true and this is the current device, give it higher priority
    if (this.options.preferCurrentDevice && 
        this.currentDevice && 
        device.id === this.currentDevice.id) {
      return 100;
    }
    
    // Default priority based on device type
    switch (device.type) {
      case 'desktop':
        return 50;
      case 'tablet':
        return 40;
      case 'mobile':
        return 30;
      case 'web':
        return 20;
      default:
        return 10;
    }
  }
}

// Create a singleton instance of the cross-device conflict manager
export const crossDeviceConflictManager = new CrossDeviceConflictManager();

/**
 * Creates a cross-device conflict from a base conflict
 * @param conflict - The base conflict
 * @param sourceDevice - The source device
 * @param targetDevice - The target device
 * @param sessionId - The session ID
 * @returns A cross-device conflict
 */
export function createCrossDeviceConflict(
  conflict: Conflict,
  sourceDevice: Device,
  targetDevice: Device,
  sessionId: string
): CrossDeviceConflict {
  return {
    ...conflict,
    sourceDevice,
    targetDevice,
    sessionId,
    metadata: {}
  };
}

/**
 * Hook for using cross-device conflict resolution
 */
export function useCrossDeviceConflicts() {
  return {
    manager: crossDeviceConflictManager,
    conflicts: crossDeviceConflictManager.getConflicts(),
    unresolvedConflicts: crossDeviceConflictManager.getUnresolvedConflicts(),
    resolveConflict: (conflict: CrossDeviceConflict, strategy?: ConflictStrategy) => 
      crossDeviceConflictManager.resolveConflict(conflict, strategy),
    getConflictsForSession: (sessionId: string) => 
      crossDeviceConflictManager.getConflictsForSession(sessionId)
  };
}