/**
 * Cross-Device Synchronization API
 * 
 * This module provides types and functions for synchronizing data across devices,
 * including status tracking, session management, and device discovery.
 */

import { SyncOperation, SyncResult as BaseSyncResult, SyncProgress as BaseSyncProgress, SyncStatus } from './sync';
import { Conflict } from './conflict';
import { CrossDeviceConflict } from './crossDeviceConflict';

/**
 * Device information
 */
export interface Device {
  /** Unique device ID */
  id: string;
  /** Device name */
  name: string;
  /** Device type */
  type: 'desktop' | 'mobile' | 'tablet' | 'web' | 'other';
  /** Last seen timestamp */
  lastSeen: number;
  /** Whether this is the current device */
  isCurrent: boolean;
  /** Device capabilities */
  capabilities?: {
    /** Whether the device supports push notifications */
    pushNotifications?: boolean;
    /** Whether the device supports background sync */
    backgroundSync?: boolean;
    /** Maximum storage capacity in bytes */
    storageCapacity?: number;
    /** Available storage in bytes */
    availableStorage?: number;
    /** Network connection type */
    networkType?: 'wifi' | 'cellular' | 'ethernet' | 'unknown';
    /** Battery level (0-100) */
    batteryLevel?: number;
  };
}

/**
 * Synchronization session between devices
 */
export interface SyncSession {
  /** Unique session ID */
  id: string;
  /** Source device */
  sourceDevice: Device;
  /** Target device */
  targetDevice: Device;
  /** Session start time */
  startTime: number;
  /** Session end time (if completed) */
  endTime?: number;
  /** Current status */
  status: SyncStatus;
  /** Synchronization progress */
  progress: SyncProgress;
  /** Synchronization result (if completed) */
  result?: SyncResult;
  /** Detected conflicts */
  conflicts: CrossDeviceConflict[];
  /** Session metadata */
  metadata?: Record<string, any>;
}

/**
 * Re-export SyncStatus from sync.ts
 */
export { SyncStatus };

/**
 * Extended synchronization progress for cross-device sync
 */
export interface SyncProgress extends BaseSyncProgress {
  /** Source device ID */
  sourceDeviceId: string;
  /** Target device ID */
  targetDeviceId: string;
  /** Number of conflicts detected */
  conflicts: number;
  /** Network transfer statistics */
  networkStats?: {
    /** Bytes sent */
    bytesSent: number;
    /** Bytes received */
    bytesReceived: number;
    /** Current transfer rate in bytes per second */
    transferRate: number;
  };
}

/**
 * Extended synchronization result for cross-device sync
 */
export interface SyncResult extends BaseSyncResult {
  /** Source device ID */
  sourceDeviceId: string;
  /** Target device ID */
  targetDeviceId: string;
  /** Session ID */
  sessionId: string;
  /** Network statistics */
  networkStats?: {
    /** Total bytes sent */
    totalBytesSent: number;
    /** Total bytes received */
    totalBytesReceived: number;
    /** Average transfer rate in bytes per second */
    averageTransferRate: number;
  };
}

/**
 * Options for cross-device synchronization
 */
export interface CrossDeviceSyncOptions {
  /** Entity types to synchronize (if empty, all types are synchronized) */
  entityTypes?: string[];
  /** Whether to include deleted entities */
  includeDeleted?: boolean;
  /** Maximum number of operations to sync in a batch */
  batchSize?: number;
  /** Whether to continue if a conflict is detected */
  continueOnConflict?: boolean;
  /** Whether to automatically resolve conflicts */
  autoResolveConflicts?: boolean;
  /** Compression level (0-9) */
  compressionLevel?: number;
  /** Whether to encrypt data during transfer */
  encryptTransfer?: boolean;
  /** Whether to verify data integrity after transfer */
  verifyIntegrity?: boolean;
  /** Timeout in milliseconds */
  timeout?: number;
  /** Callback for progress updates */
  onProgress?: (progress: SyncProgress) => void;
  /** Callback when synchronization is completed */
  onComplete?: (result: SyncResult) => void;
  /** Callback when a conflict is detected */
  onConflictDetected?: (conflict: CrossDeviceConflict) => void;
}

/**
 * Default options for cross-device synchronization
 */
export const DEFAULT_CROSS_DEVICE_SYNC_OPTIONS: CrossDeviceSyncOptions = {
  entityTypes: [],
  includeDeleted: true,
  batchSize: 100,
  continueOnConflict: true,
  autoResolveConflicts: false,
  compressionLevel: 6,
  encryptTransfer: true,
  verifyIntegrity: true,
  timeout: 300000, // 5 minutes
  onProgress: () => {},
  onComplete: () => {},
  onConflictDetected: () => {}
};

/**
 * Cross-device synchronization manager
 */
export class CrossDeviceSyncManager {
  private sessions: Map<string, SyncSession> = new Map();
  private devices: Map<string, Device> = new Map();
  private currentDevice: Device | null = null;
  private listeners: Set<(sessions: SyncSession[]) => void> = new Set();
  
  /**
   * Creates a new cross-device synchronization manager
   */
  constructor() {
    // Initialize the current device
    this.initializeCurrentDevice();
  }
  
  /**
   * Initializes the current device information
   */
  private initializeCurrentDevice() {
    // In a real implementation, this would get the actual device information
    this.currentDevice = {
      id: 'current-device-' + Math.random().toString(36).substring(2, 9),
      name: 'Current Device',
      type: 'desktop',
      lastSeen: Date.now(),
      isCurrent: true,
      capabilities: {
        pushNotifications: true,
        backgroundSync: true,
        storageCapacity: 1000000000, // 1 GB
        availableStorage: 500000000, // 500 MB
        networkType: 'wifi',
        batteryLevel: 100
      }
    };
    
    this.devices.set(this.currentDevice.id, this.currentDevice);
  }
  
  /**
   * Gets the current device
   * @returns The current device
   */
  getCurrentDevice(): Device {
    if (!this.currentDevice) {
      this.initializeCurrentDevice();
    }
    return this.currentDevice!;
  }
  
  /**
   * Gets all known devices
   * @returns Array of devices
   */
  getDevices(): Device[] {
    return Array.from(this.devices.values());
  }
  
  /**
   * Gets a device by ID
   * @param deviceId - The device ID
   * @returns The device, or undefined if not found
   */
  getDevice(deviceId: string): Device | undefined {
    return this.devices.get(deviceId);
  }
  
  /**
   * Adds or updates a device
   * @param device - The device to add or update
   */
  addDevice(device: Device): void {
    this.devices.set(device.id, device);
  }
  
  /**
   * Removes a device
   * @param deviceId - The device ID to remove
   */
  removeDevice(deviceId: string): void {
    this.devices.delete(deviceId);
    
    // Remove any sessions involving this device
    for (const [sessionId, session] of this.sessions.entries()) {
      if (session.sourceDevice.id === deviceId || session.targetDevice.id === deviceId) {
        this.sessions.delete(sessionId);
      }
    }
    
    // Notify listeners
    this.notifyListeners();
  }
  
  /**
   * Gets all synchronization sessions
   * @returns Array of synchronization sessions
   */
  getSessions(): SyncSession[] {
    return Array.from(this.sessions.values());
  }
  
  /**
   * Gets a synchronization session by ID
   * @param sessionId - The session ID
   * @returns The session, or undefined if not found
   */
  getSession(sessionId: string): SyncSession | undefined {
    return this.sessions.get(sessionId);
  }
  
  /**
   * Creates a new synchronization session
   * @param targetDeviceId - The target device ID
   * @param options - Synchronization options
   * @returns The created session
   */
  createSession(targetDeviceId: string, options?: CrossDeviceSyncOptions): SyncSession {
    const targetDevice = this.devices.get(targetDeviceId);
    if (!targetDevice) {
      throw new Error(`Device with ID ${targetDeviceId} not found`);
    }
    
    if (!this.currentDevice) {
      this.initializeCurrentDevice();
    }
    
    const sessionId = 'session-' + Math.random().toString(36).substring(2, 9);
    const session: SyncSession = {
      id: sessionId,
      sourceDevice: this.currentDevice!,
      targetDevice,
      startTime: Date.now(),
      status: SyncStatus.IDLE,
      progress: {
        total: 0,
        completed: 0,
        failed: 0,
        status: SyncStatus.IDLE,
        startTime: Date.now(),
        sourceDeviceId: this.currentDevice!.id,
        targetDeviceId,
        conflicts: 0
      },
      conflicts: [],
      metadata: {
        options
      }
    };
    
    this.sessions.set(sessionId, session);
    this.notifyListeners();
    
    return session;
  }
  
  /**
   * Updates a synchronization session
   * @param sessionId - The session ID
   * @param updates - The updates to apply
   * @returns The updated session
   */
  updateSession(sessionId: string, updates: Partial<SyncSession>): SyncSession {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Session with ID ${sessionId} not found`);
    }
    
    const updatedSession = { ...session, ...updates };
    this.sessions.set(sessionId, updatedSession);
    this.notifyListeners();
    
    return updatedSession;
  }
  
  /**
   * Removes a synchronization session
   * @param sessionId - The session ID to remove
   */
  removeSession(sessionId: string): void {
    this.sessions.delete(sessionId);
    this.notifyListeners();
  }
  
  /**
   * Starts synchronization for a session
   * @param sessionId - The session ID
   * @param options - Synchronization options
   * @returns A promise that resolves when synchronization is complete
   */
  async startSync(sessionId: string, options?: CrossDeviceSyncOptions): Promise<SyncResult> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Session with ID ${sessionId} not found`);
    }
    
    // Merge options with defaults and session metadata
    const mergedOptions: CrossDeviceSyncOptions = {
      ...DEFAULT_CROSS_DEVICE_SYNC_OPTIONS,
      ...session.metadata?.options,
      ...options
    };
    
    // Update session status
    this.updateSession(sessionId, {
      status: SyncStatus.SYNCING,
      progress: {
        ...session.progress,
        status: SyncStatus.SYNCING,
        startTime: Date.now()
      }
    });
    
    try {
      // In a real implementation, this would perform the actual synchronization
      // For this example, we'll simulate synchronization with a delay
      
      // Simulate progress updates
      const totalItems = 100;
      const updateInterval = setInterval(() => {
        const currentSession = this.sessions.get(sessionId);
        if (!currentSession || currentSession.status !== SyncStatus.SYNCING) {
          clearInterval(updateInterval);
          return;
        }
        
        const newCompleted = Math.min(
          currentSession.progress.completed + Math.floor(Math.random() * 5) + 1,
          totalItems
        );
        
        const updatedProgress: SyncProgress = {
          ...currentSession.progress,
          total: totalItems,
          completed: newCompleted,
          status: newCompleted === totalItems ? SyncStatus.COMPLETED : SyncStatus.SYNCING,
          networkStats: {
            bytesSent: newCompleted * 1000,
            bytesReceived: newCompleted * 500,
            transferRate: 50000
          }
        };
        
        this.updateSession(sessionId, {
          progress: updatedProgress,
          status: updatedProgress.status
        });
        
        if (mergedOptions.onProgress) {
          mergedOptions.onProgress(updatedProgress);
        }
        
        if (newCompleted === totalItems) {
          clearInterval(updateInterval);
        }
      }, 200);
      
      // Simulate completion after a delay
      await new Promise(resolve => setTimeout(resolve, 5000));
      
      clearInterval(updateInterval);
      
      // Create result
      const result: SyncResult = {
        success: true,
        total: totalItems,
        completed: totalItems,
        failed: 0,
        skipped: 0,
        conflicts: 0,
        failedOperations: [],
        skippedOperations: [],
        detectedConflicts: [],
        duration: 5000,
        sourceDeviceId: session.sourceDevice.id,
        targetDeviceId: session.targetDevice.id,
        sessionId,
        networkStats: {
          totalBytesSent: totalItems * 1000,
          totalBytesReceived: totalItems * 500,
          averageTransferRate: 50000
        }
      };
      
      // Update session with result
      this.updateSession(sessionId, {
        status: SyncStatus.COMPLETED,
        endTime: Date.now(),
        result,
        progress: {
          ...session.progress,
          total: totalItems,
          completed: totalItems,
          status: SyncStatus.COMPLETED,
          endTime: Date.now()
        }
      });
      
      if (mergedOptions.onComplete) {
        mergedOptions.onComplete(result);
      }
      
      return result;
    } catch (error) {
      // Update session with error
      const errorObj = error instanceof Error ? error : new Error(String(error));
      
      const result: SyncResult = {
        success: false,
        total: session.progress.total,
        completed: session.progress.completed,
        failed: session.progress.total - session.progress.completed,
        skipped: 0,
        conflicts: session.conflicts.length,
        failedOperations: [],
        skippedOperations: [],
        detectedConflicts: session.conflicts,
        error: errorObj,
        duration: Date.now() - session.progress.startTime,
        sourceDeviceId: session.sourceDevice.id,
        targetDeviceId: session.targetDevice.id,
        sessionId
      };
      
      this.updateSession(sessionId, {
        status: SyncStatus.FAILED,
        endTime: Date.now(),
        result,
        progress: {
          ...session.progress,
          status: SyncStatus.FAILED,
          endTime: Date.now(),
          error: errorObj
        }
      });
      
      throw error;
    }
  }
  
  /**
   * Cancels synchronization for a session
   * @param sessionId - The session ID
   */
  cancelSync(sessionId: string): void {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error(`Session with ID ${sessionId} not found`);
    }
    
    if (session.status !== SyncStatus.SYNCING) {
      return;
    }
    
    this.updateSession(sessionId, {
      status: SyncStatus.FAILED,
      endTime: Date.now(),
      progress: {
        ...session.progress,
        status: SyncStatus.FAILED,
        endTime: Date.now()
      }
    });
  }
  
  /**
   * Adds a listener for session changes
   * @param listener - The listener function
   */
  addListener(listener: (sessions: SyncSession[]) => void): void {
    this.listeners.add(listener);
  }
  
  /**
   * Removes a listener
   * @param listener - The listener function to remove
   */
  removeListener(listener: (sessions: SyncSession[]) => void): void {
    this.listeners.delete(listener);
  }
  
  /**
   * Notifies all listeners of session changes
   */
  private notifyListeners(): void {
    const sessions = this.getSessions();
    for (const listener of this.listeners) {
      listener(sessions);
    }
  }
}

// Create a singleton instance of the cross-device sync manager
export const crossDeviceSyncManager = new CrossDeviceSyncManager();

/**
 * Hook for using cross-device synchronization
 */
export function useCrossDeviceSync() {
  return {
    manager: crossDeviceSyncManager,
    currentDevice: crossDeviceSyncManager.getCurrentDevice(),
    devices: crossDeviceSyncManager.getDevices(),
    sessions: crossDeviceSyncManager.getSessions(),
    createSession: (targetDeviceId: string, options?: CrossDeviceSyncOptions) => 
      crossDeviceSyncManager.createSession(targetDeviceId, options),
    startSync: (sessionId: string, options?: CrossDeviceSyncOptions) => 
      crossDeviceSyncManager.startSync(sessionId, options),
    cancelSync: (sessionId: string) => 
      crossDeviceSyncManager.cancelSync(sessionId)
  };
}