/**
 * Background synchronization for offline operations
 * 
 * This module provides automatic background synchronization capabilities for the offline queue,
 * including periodic sync, app startup/resume sync, and battery/network-aware sync strategies.
 */

import { syncManager, SyncStatus, SyncResult, syncEvents, SyncEventType } from './sync';
import { offlineQueueManager } from './offline';

/**
 * Background sync strategy options
 */
export enum BackgroundSyncStrategy {
  /** Sync immediately when operations are queued */
  IMMEDIATE = 'immediate',
  /** Sync periodically based on a fixed interval */
  PERIODIC = 'periodic',
  /** Sync with exponential backoff after failures */
  EXPONENTIAL_BACKOFF = 'exponential_backoff',
  /** Sync only when the app is idle */
  IDLE = 'idle',
  /** Sync only when the device is charging and on WiFi */
  OPTIMAL_CONDITIONS = 'optimal_conditions',
  /** Sync based on the number of queued operations */
  QUEUE_THRESHOLD = 'queue_threshold',
  /** Custom sync strategy */
  CUSTOM = 'custom'
}

/**
 * Options for background sync
 */
export interface BackgroundSyncOptions {
  /** Whether background sync is enabled */
  enabled?: boolean;
  /** The sync strategy to use */
  strategy?: BackgroundSyncStrategy;
  /** Interval in milliseconds for periodic sync */
  syncInterval?: number;
  /** Maximum number of sync attempts before giving up */
  maxSyncAttempts?: number;
  /** Whether to sync on app startup */
  syncOnStartup?: boolean;
  /** Whether to sync when the app resumes from background */
  syncOnResume?: boolean;
  /** Minimum battery level required for sync (0-100) */
  minBatteryLevel?: number;
  /** Whether to require WiFi for sync */
  requireWifi?: number;
  /** Number of queued operations that triggers a sync */
  queueThreshold?: number;
  /** Custom sync condition function */
  customSyncCondition?: () => boolean;
  /** Callback when a sync is scheduled */
  onSyncScheduled?: (nextSyncTime: number) => void;
  /** Callback when a sync is started */
  onSyncStarted?: () => void;
  /** Callback when a sync is completed */
  onSyncCompleted?: (result: SyncResult) => void;
  /** Callback when a sync fails */
  onSyncFailed?: (error: Error) => void;
}

/**
 * Default options for background sync
 */
const DEFAULT_BACKGROUND_SYNC_OPTIONS: Required<BackgroundSyncOptions> = {
  enabled: true,
  strategy: BackgroundSyncStrategy.PERIODIC,
  syncInterval: 5 * 60 * 1000, // 5 minutes
  maxSyncAttempts: 10,
  syncOnStartup: true,
  syncOnResume: true,
  minBatteryLevel: 20,
  requireWifi: false,
  queueThreshold: 5,
  customSyncCondition: () => true,
  onSyncScheduled: () => {},
  onSyncStarted: () => {},
  onSyncCompleted: () => {},
  onSyncFailed: () => {}
};

/**
 * Background sync manager for automatically synchronizing offline operations
 */
export class BackgroundSyncManager {
  private options: Required<BackgroundSyncOptions>;
  private syncTimer: number | null = null;
  private nextSyncTime: number = 0;
  private syncAttempts: number = 0;
  private lastSyncResult: SyncResult | null = null;
  private isInitialized: boolean = false;

  /**
   * Creates a new background sync manager
   * @param options - Options for background sync
   */
  constructor(options: BackgroundSyncOptions = {}) {
    this.options = { ...DEFAULT_BACKGROUND_SYNC_OPTIONS, ...options };
  }

  /**
   * Initializes the background sync manager
   */
  initialize(): void {
    if (this.isInitialized) return;

    // Set up event listeners
    this.setupEventListeners();

    // Sync on startup if enabled
    if (this.options.syncOnStartup) {
      this.syncIfNeeded();
    }

    // Schedule the next sync based on the strategy
    this.scheduleNextSync();

    this.isInitialized = true;
  }

  /**
   * Sets up event listeners for various events
   */
  private setupEventListeners(): void {
    // Listen for online/offline events
    window.addEventListener('online', this.handleOnline);
    window.addEventListener('offline', this.handleOffline);

    // Listen for visibility change (app resume)
    document.addEventListener('visibilitychange', this.handleVisibilityChange);

    // Listen for queue changes
    document.addEventListener('queue:changed', this.handleQueueChanged);

    // Listen for sync events
    syncEvents.addEventListener(SyncEventType.COMPLETED, this.handleSyncCompleted);
    syncEvents.addEventListener(SyncEventType.FAILED, this.handleSyncFailed);
  }

  /**
   * Cleans up event listeners
   */
  destroy(): void {
    // Remove event listeners
    window.removeEventListener('online', this.handleOnline);
    window.removeEventListener('offline', this.handleOffline);
    document.removeEventListener('visibilitychange', this.handleVisibilityChange);
    document.removeEventListener('queue:changed', this.handleQueueChanged);

    // Remove sync event listeners
    syncEvents.removeEventListener(SyncEventType.COMPLETED, this.handleSyncCompleted);
    syncEvents.removeEventListener(SyncEventType.FAILED, this.handleSyncFailed);

    // Clear any scheduled sync
    this.clearScheduledSync();

    this.isInitialized = false;
  }

  /**
   * Handles the online event
   */
  private handleOnline = (): void => {
    // When we come back online, try to sync
    this.syncIfNeeded();
  };

  /**
   * Handles the offline event
   */
  private handleOffline = (): void => {
    // When we go offline, clear any scheduled sync
    this.clearScheduledSync();
  };

  /**
   * Handles visibility change (app resume)
   */
  private handleVisibilityChange = (): void => {
    if (document.visibilityState === 'visible' && this.options.syncOnResume) {
      // When the app resumes, try to sync
      this.syncIfNeeded();
    }
  };

  /**
   * Handles queue changes
   */
  private handleQueueChanged = (event: Event): void => {
    const customEvent = event as CustomEvent;
    const queueSize = customEvent.detail?.queueSize || 0;

    // If using queue threshold strategy and we've reached the threshold, sync
    if (
      this.options.strategy === BackgroundSyncStrategy.QUEUE_THRESHOLD &&
      queueSize >= this.options.queueThreshold
    ) {
      this.syncIfNeeded();
    }

    // If using immediate strategy, sync right away
    if (this.options.strategy === BackgroundSyncStrategy.IMMEDIATE && queueSize > 0) {
      this.syncIfNeeded();
    }
  };

  /**
   * Handles sync completed event
   */
  private handleSyncCompleted = (event: any): void => {
    const result = event.result as SyncResult;
    this.lastSyncResult = result;
    this.syncAttempts = 0; // Reset sync attempts on success
    this.options.onSyncCompleted(result);

    // Schedule the next sync
    this.scheduleNextSync();
  };

  /**
   * Handles sync failed event
   */
  private handleSyncFailed = (event: any): void => {
    const error = event.error as Error;
    this.syncAttempts++;
    this.options.onSyncFailed(error);

    // Schedule the next sync with backoff if using exponential backoff strategy
    this.scheduleNextSync();
  };

  /**
   * Schedules the next sync based on the current strategy
   */
  private scheduleNextSync(): void {
    // Clear any existing timer
    this.clearScheduledSync();

    // If sync is disabled, don't schedule anything
    if (!this.options.enabled) return;

    // If we're offline, don't schedule anything
    if (!offlineQueueManager.isNetworkOnline()) return;

    // If there are no operations to sync, don't schedule anything
    if (offlineQueueManager.getQueueLength() === 0) return;

    let delay: number;

    switch (this.options.strategy) {
      case BackgroundSyncStrategy.IMMEDIATE:
        // Immediate sync, schedule for the next tick
        delay = 0;
        break;

      case BackgroundSyncStrategy.EXPONENTIAL_BACKOFF:
        // Exponential backoff based on the number of attempts
        delay = Math.min(
          this.options.syncInterval * Math.pow(2, this.syncAttempts),
          30 * 60 * 1000 // Max 30 minutes
        );
        break;

      case BackgroundSyncStrategy.IDLE:
        // Use requestIdleCallback if available, otherwise use a longer interval
        if (typeof window.requestIdleCallback === 'function') {
          window.requestIdleCallback(() => this.syncIfNeeded(), { timeout: 60000 });
          return; // Early return as we're using requestIdleCallback
        }
        delay = 10 * 60 * 1000; // 10 minutes if requestIdleCallback not available
        break;

      case BackgroundSyncStrategy.QUEUE_THRESHOLD:
        // Check periodically if we've reached the threshold
        delay = 60 * 1000; // Check every minute
        break;

      case BackgroundSyncStrategy.OPTIMAL_CONDITIONS:
        // Check conditions periodically
        delay = 5 * 60 * 1000; // Check every 5 minutes
        break;

      case BackgroundSyncStrategy.CUSTOM:
        // Use the sync interval as a polling interval for the custom condition
        delay = this.options.syncInterval;
        break;

      case BackgroundSyncStrategy.PERIODIC:
      default:
        // Regular periodic sync
        delay = this.options.syncInterval;
        break;
    }

    // Schedule the next sync
    this.nextSyncTime = Date.now() + delay;
    this.syncTimer = window.setTimeout(() => this.syncIfNeeded(), delay);

    // Notify that a sync has been scheduled
    this.options.onSyncScheduled(this.nextSyncTime);
  }

  /**
   * Clears any scheduled sync
   */
  private clearScheduledSync(): void {
    if (this.syncTimer !== null) {
      clearTimeout(this.syncTimer);
      this.syncTimer = null;
    }
  }

  /**
   * Checks if sync is needed and conditions are met, then syncs
   */
  private syncIfNeeded(): void {
    // If sync is disabled, don't sync
    if (!this.options.enabled) return;

    // If we're offline, don't sync
    if (!offlineQueueManager.isNetworkOnline()) return;

    // If there are no operations to sync, don't sync
    if (offlineQueueManager.getQueueLength() === 0) return;

    // If we've reached the maximum number of attempts, don't sync
    if (this.syncAttempts >= this.options.maxSyncAttempts) return;

    // If sync is already in progress, don't start another one
    if (syncManager.isSynchronizing()) return;

    // Check strategy-specific conditions
    if (!this.checkSyncConditions()) return;

    // All conditions met, start the sync
    this.startSync();
  }

  /**
   * Checks if sync conditions are met based on the current strategy
   */
  private checkSyncConditions(): boolean {
    switch (this.options.strategy) {
      case BackgroundSyncStrategy.OPTIMAL_CONDITIONS:
        return this.checkOptimalConditions();

      case BackgroundSyncStrategy.QUEUE_THRESHOLD:
        return offlineQueueManager.getQueueLength() >= this.options.queueThreshold;

      case BackgroundSyncStrategy.CUSTOM:
        return this.options.customSyncCondition();

      default:
        return true;
    }
  }

  /**
   * Checks if optimal conditions for sync are met (battery, network)
   */
  private checkOptimalConditions(): boolean {
    // Check battery level if the Battery API is available
    if (typeof navigator.getBattery === 'function') {
      navigator.getBattery().then(battery => {
        if (battery.level * 100 < this.options.minBatteryLevel && !battery.charging) {
          return false;
        }
      });
    }

    // Check network type if the Network Information API is available
    if (this.options.requireWifi && 'connection' in navigator) {
      const connection = (navigator as any).connection;
      if (connection && connection.type !== 'wifi') {
        return false;
      }
    }

    return true;
  }

  /**
   * Starts the sync process
   */
  private startSync(): void {
    this.options.onSyncStarted();
    syncManager.synchronize();
  }

  /**
   * Gets the time of the next scheduled sync
   * @returns The timestamp of the next scheduled sync, or 0 if none is scheduled
   */
  getNextSyncTime(): number {
    return this.nextSyncTime;
  }

  /**
   * Gets the result of the last sync
   * @returns The result of the last sync, or null if no sync has been performed
   */
  getLastSyncResult(): SyncResult | null {
    return this.lastSyncResult;
  }

  /**
   * Gets the current sync status
   * @returns The current sync status
   */
  getSyncStatus(): SyncStatus {
    return syncManager.getProgress().status;
  }

  /**
   * Updates the background sync options
   * @param options - The new options
   */
  updateOptions(options: BackgroundSyncOptions): void {
    this.options = { ...this.options, ...options };

    // If the sync interval or strategy changed, reschedule the next sync
    this.scheduleNextSync();
  }

  /**
   * Enables background sync
   */
  enable(): void {
    if (!this.options.enabled) {
      this.options.enabled = true;
      this.scheduleNextSync();
    }
  }

  /**
   * Disables background sync
   */
  disable(): void {
    if (this.options.enabled) {
      this.options.enabled = false;
      this.clearScheduledSync();
    }
  }

  /**
   * Forces a sync regardless of conditions
   * @returns A promise that resolves when the sync is complete
   */
  forceSync(): Promise<SyncResult> {
    this.options.onSyncStarted();
    return syncManager.synchronize();
  }
}

// Create a singleton instance of the background sync manager
export const backgroundSyncManager = new BackgroundSyncManager();

// Initialize the background sync manager
if (typeof window !== 'undefined') {
  // Wait for the DOM to be fully loaded
  if (document.readyState === 'complete') {
    backgroundSyncManager.initialize();
  } else {
    window.addEventListener('load', () => {
      backgroundSyncManager.initialize();
    });
  }
}