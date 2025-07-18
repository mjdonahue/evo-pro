/**
 * Selective Synchronization Options
 * 
 * This module provides functionality for configuring which data types
 * are synchronized across devices, allowing users to control what data
 * is shared between their devices.
 */

import { crossDeviceSyncManager, CrossDeviceSyncOptions } from './crossDeviceSync';

/**
 * Selective synchronization configuration for an entity type
 */
export interface EntitySyncConfig {
  /** Entity type identifier */
  entityType: string;
  /** Display name for the entity type */
  displayName: string;
  /** Description of the entity type */
  description: string;
  /** Whether synchronization is enabled for this entity type */
  enabled: boolean;
  /** Icon identifier for the entity type */
  icon?: string;
  /** Estimated storage size per entity (in bytes) */
  estimatedSizePerEntity?: number;
  /** Approximate count of entities of this type */
  approximateCount?: number;
  /** Whether this entity type is required for core functionality */
  required?: boolean;
  /** Dependencies on other entity types */
  dependencies?: string[];
  /** Custom synchronization options for this entity type */
  options?: Partial<CrossDeviceSyncOptions>;
}

/**
 * Selective synchronization preferences
 */
export interface SelectiveSyncPreferences {
  /** Global synchronization enabled flag */
  syncEnabled: boolean;
  /** Entity type configurations */
  entityConfigs: Record<string, EntitySyncConfig>;
  /** Default configuration for new entity types */
  defaultConfig: Partial<EntitySyncConfig>;
  /** Global synchronization options */
  globalOptions: Partial<CrossDeviceSyncOptions>;
  /** Last updated timestamp */
  lastUpdated: number;
}

/**
 * Default selective sync preferences
 */
export const DEFAULT_SELECTIVE_SYNC_PREFERENCES: SelectiveSyncPreferences = {
  syncEnabled: true,
  entityConfigs: {
    // Core entity types with default configurations
    'note': {
      entityType: 'note',
      displayName: 'Notes',
      description: 'Text notes and documents',
      enabled: true,
      icon: 'note',
      required: false
    },
    'task': {
      entityType: 'task',
      displayName: 'Tasks',
      description: 'To-do items and task lists',
      enabled: true,
      icon: 'task',
      required: false
    },
    'contact': {
      entityType: 'contact',
      displayName: 'Contacts',
      description: 'Contact information for people',
      enabled: true,
      icon: 'contact',
      required: false
    },
    'calendar': {
      entityType: 'calendar',
      displayName: 'Calendar',
      description: 'Calendar events and appointments',
      enabled: true,
      icon: 'calendar',
      required: false
    },
    'setting': {
      entityType: 'setting',
      displayName: 'Settings',
      description: 'Application settings and preferences',
      enabled: true,
      icon: 'setting',
      required: true
    },
    'attachment': {
      entityType: 'attachment',
      displayName: 'Attachments',
      description: 'Files attached to notes, tasks, etc.',
      enabled: true,
      icon: 'attachment',
      required: false,
      dependencies: ['note', 'task']
    }
  },
  defaultConfig: {
    enabled: true,
    required: false
  },
  globalOptions: {
    includeDeleted: true,
    compressionLevel: 6,
    encryptTransfer: true,
    verifyIntegrity: true
  },
  lastUpdated: Date.now()
};

/**
 * Storage key for selective sync preferences
 */
const STORAGE_KEY = 'selective_sync_preferences';

/**
 * Selective synchronization manager
 */
export class SelectiveSyncManager {
  private preferences: SelectiveSyncPreferences;
  private listeners: Set<(preferences: SelectiveSyncPreferences) => void> = new Set();
  
  /**
   * Creates a new selective sync manager
   * @param initialPreferences - Initial preferences (if not provided, loads from storage or uses defaults)
   */
  constructor(initialPreferences?: SelectiveSyncPreferences) {
    if (initialPreferences) {
      this.preferences = initialPreferences;
    } else {
      this.preferences = this.loadPreferences();
    }
  }
  
  /**
   * Loads preferences from storage
   * @returns The loaded preferences, or defaults if none are stored
   */
  private loadPreferences(): SelectiveSyncPreferences {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored) as SelectiveSyncPreferences;
        return {
          ...DEFAULT_SELECTIVE_SYNC_PREFERENCES,
          ...parsed,
          // Ensure we have all default entity configs
          entityConfigs: {
            ...DEFAULT_SELECTIVE_SYNC_PREFERENCES.entityConfigs,
            ...parsed.entityConfigs
          }
        };
      }
    } catch (error) {
      console.error('Failed to load selective sync preferences:', error);
    }
    
    return DEFAULT_SELECTIVE_SYNC_PREFERENCES;
  }
  
  /**
   * Saves preferences to storage
   */
  private savePreferences(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.preferences));
    } catch (error) {
      console.error('Failed to save selective sync preferences:', error);
    }
  }
  
  /**
   * Gets the current preferences
   * @returns The current preferences
   */
  getPreferences(): SelectiveSyncPreferences {
    return { ...this.preferences };
  }
  
  /**
   * Updates the preferences
   * @param updates - The updates to apply
   * @returns The updated preferences
   */
  updatePreferences(updates: Partial<SelectiveSyncPreferences>): SelectiveSyncPreferences {
    this.preferences = {
      ...this.preferences,
      ...updates,
      lastUpdated: Date.now()
    };
    
    this.savePreferences();
    this.notifyListeners();
    
    return this.preferences;
  }
  
  /**
   * Gets the configuration for an entity type
   * @param entityType - The entity type
   * @returns The entity configuration, or a default if not found
   */
  getEntityConfig(entityType: string): EntitySyncConfig {
    const config = this.preferences.entityConfigs[entityType];
    
    if (!config) {
      // Create a default config for this entity type
      const defaultConfig: EntitySyncConfig = {
        entityType,
        displayName: entityType.charAt(0).toUpperCase() + entityType.slice(1),
        description: `${entityType.charAt(0).toUpperCase() + entityType.slice(1)} data`,
        enabled: this.preferences.defaultConfig.enabled ?? true,
        required: this.preferences.defaultConfig.required ?? false
      };
      
      // Add to preferences
      this.preferences.entityConfigs[entityType] = defaultConfig;
      this.savePreferences();
      
      return defaultConfig;
    }
    
    return config;
  }
  
  /**
   * Updates the configuration for an entity type
   * @param entityType - The entity type
   * @param updates - The updates to apply
   * @returns The updated entity configuration
   */
  updateEntityConfig(entityType: string, updates: Partial<EntitySyncConfig>): EntitySyncConfig {
    const current = this.getEntityConfig(entityType);
    const updated = { ...current, ...updates };
    
    this.preferences.entityConfigs[entityType] = updated;
    this.preferences.lastUpdated = Date.now();
    
    this.savePreferences();
    this.notifyListeners();
    
    return updated;
  }
  
  /**
   * Enables synchronization for an entity type
   * @param entityType - The entity type
   * @returns The updated entity configuration
   */
  enableEntitySync(entityType: string): EntitySyncConfig {
    return this.updateEntityConfig(entityType, { enabled: true });
  }
  
  /**
   * Disables synchronization for an entity type
   * @param entityType - The entity type
   * @returns The updated entity configuration
   */
  disableEntitySync(entityType: string): EntitySyncConfig {
    return this.updateEntityConfig(entityType, { enabled: false });
  }
  
  /**
   * Gets all entity types that are enabled for synchronization
   * @returns Array of enabled entity types
   */
  getEnabledEntityTypes(): string[] {
    return Object.values(this.preferences.entityConfigs)
      .filter(config => config.enabled)
      .map(config => config.entityType);
  }
  
  /**
   * Gets all entity types that are disabled for synchronization
   * @returns Array of disabled entity types
   */
  getDisabledEntityTypes(): string[] {
    return Object.values(this.preferences.entityConfigs)
      .filter(config => !config.enabled)
      .map(config => config.entityType);
  }
  
  /**
   * Enables global synchronization
   * @returns The updated preferences
   */
  enableSync(): SelectiveSyncPreferences {
    return this.updatePreferences({ syncEnabled: true });
  }
  
  /**
   * Disables global synchronization
   * @returns The updated preferences
   */
  disableSync(): SelectiveSyncPreferences {
    return this.updatePreferences({ syncEnabled: false });
  }
  
  /**
   * Updates global synchronization options
   * @param options - The options to update
   * @returns The updated preferences
   */
  updateGlobalOptions(options: Partial<CrossDeviceSyncOptions>): SelectiveSyncPreferences {
    return this.updatePreferences({
      globalOptions: {
        ...this.preferences.globalOptions,
        ...options
      }
    });
  }
  
  /**
   * Adds a listener for preference changes
   * @param listener - The listener function
   */
  addListener(listener: (preferences: SelectiveSyncPreferences) => void): void {
    this.listeners.add(listener);
  }
  
  /**
   * Removes a listener
   * @param listener - The listener function to remove
   */
  removeListener(listener: (preferences: SelectiveSyncPreferences) => void): void {
    this.listeners.delete(listener);
  }
  
  /**
   * Notifies all listeners of preference changes
   */
  private notifyListeners(): void {
    const preferences = this.getPreferences();
    for (const listener of this.listeners) {
      listener(preferences);
    }
  }
  
  /**
   * Applies selective sync preferences to a sync options object
   * @param options - The base sync options
   * @returns The updated sync options with selective sync preferences applied
   */
  applySyncPreferences(options: CrossDeviceSyncOptions = {}): CrossDeviceSyncOptions {
    if (!this.preferences.syncEnabled) {
      throw new Error('Synchronization is disabled in preferences');
    }
    
    // Get enabled entity types
    const enabledEntityTypes = this.getEnabledEntityTypes();
    
    // Apply global options
    const updatedOptions: CrossDeviceSyncOptions = {
      ...options,
      ...this.preferences.globalOptions,
      // Override with entity types from preferences
      entityTypes: enabledEntityTypes
    };
    
    return updatedOptions;
  }
  
  /**
   * Resets preferences to defaults
   * @returns The default preferences
   */
  resetToDefaults(): SelectiveSyncPreferences {
    this.preferences = { ...DEFAULT_SELECTIVE_SYNC_PREFERENCES };
    this.savePreferences();
    this.notifyListeners();
    return this.preferences;
  }
}

// Create a singleton instance of the selective sync manager
export const selectiveSyncManager = new SelectiveSyncManager();

/**
 * Hook for using selective synchronization
 */
export function useSelectiveSync() {
  return {
    manager: selectiveSyncManager,
    preferences: selectiveSyncManager.getPreferences(),
    enabledEntityTypes: selectiveSyncManager.getEnabledEntityTypes(),
    disabledEntityTypes: selectiveSyncManager.getDisabledEntityTypes(),
    updatePreferences: (updates: Partial<SelectiveSyncPreferences>) => 
      selectiveSyncManager.updatePreferences(updates),
    enableEntitySync: (entityType: string) => 
      selectiveSyncManager.enableEntitySync(entityType),
    disableEntitySync: (entityType: string) => 
      selectiveSyncManager.disableEntitySync(entityType),
    enableSync: () => selectiveSyncManager.enableSync(),
    disableSync: () => selectiveSyncManager.disableSync(),
    updateGlobalOptions: (options: Partial<CrossDeviceSyncOptions>) => 
      selectiveSyncManager.updateGlobalOptions(options),
    resetToDefaults: () => selectiveSyncManager.resetToDefaults()
  };
}

/**
 * Creates sync options with selective sync preferences applied
 * @param baseOptions - Base sync options
 * @returns Sync options with selective sync preferences applied
 */
export function createSelectiveSyncOptions(baseOptions: CrossDeviceSyncOptions = {}): CrossDeviceSyncOptions {
  return selectiveSyncManager.applySyncPreferences(baseOptions);
}