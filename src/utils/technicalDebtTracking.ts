/**
 * Utilities for tracking technical debt in the codebase
 */

import { useState, useEffect } from 'react';

/**
 * Technical debt item representing a known issue or improvement needed
 */
export interface TechnicalDebtItem {
  /** Unique identifier for the debt item */
  id: string;
  /** Short title describing the debt */
  title: string;
  /** Detailed description of the technical debt */
  description: string;
  /** File or component affected by the debt */
  location: string;
  /** When the debt was identified */
  createdAt: number;
  /** When the debt was last updated */
  updatedAt: number;
  /** Estimated effort to resolve (in person-days) */
  estimatedEffort: number;
  /** Impact of the debt on the system (1-10) */
  impact: number;
  /** Urgency to resolve the debt (1-10) */
  urgency: number;
  /** Current status of the debt item */
  status: TechnicalDebtStatus;
  /** Type of technical debt */
  type: TechnicalDebtType;
  /** Tags for categorizing the debt */
  tags: string[];
  /** Person responsible for addressing the debt */
  assignee?: string;
  /** Target date for resolving the debt */
  targetDate?: number;
  /** Notes and additional context */
  notes?: string;
  /** Related pull requests or issues */
  relatedItems?: string[];
}

/**
 * Status of a technical debt item
 */
export enum TechnicalDebtStatus {
  /** Newly identified debt */
  IDENTIFIED = 'identified',
  /** Acknowledged but not yet scheduled */
  ACKNOWLEDGED = 'acknowledged',
  /** Scheduled for resolution */
  SCHEDULED = 'scheduled',
  /** Currently being addressed */
  IN_PROGRESS = 'in_progress',
  /** Resolved and no longer a concern */
  RESOLVED = 'resolved',
  /** Intentionally accepted as a trade-off */
  ACCEPTED = 'accepted',
  /** Deferred to a later time */
  DEFERRED = 'deferred'
}

/**
 * Types of technical debt
 */
export enum TechnicalDebtType {
  /** Code that needs refactoring */
  CODE = 'code',
  /** Architecture that needs improvement */
  ARCHITECTURE = 'architecture',
  /** Documentation that is missing or outdated */
  DOCUMENTATION = 'documentation',
  /** Tests that are missing or inadequate */
  TESTING = 'testing',
  /** Build or deployment issues */
  INFRASTRUCTURE = 'infrastructure',
  /** Dependencies that need updating */
  DEPENDENCIES = 'dependencies',
  /** Performance issues */
  PERFORMANCE = 'performance',
  /** Security vulnerabilities */
  SECURITY = 'security',
  /** Accessibility issues */
  ACCESSIBILITY = 'accessibility',
  /** User experience issues */
  UX = 'ux',
  /** Other types of debt */
  OTHER = 'other'
}

/**
 * Technical debt tracking configuration
 */
export interface TechnicalDebtConfig {
  /** Storage key for persisted debt items */
  storageKey?: string;
  /** Whether to persist debt items across page reloads */
  persistItems?: boolean;
  /** Default assignee for new debt items */
  defaultAssignee?: string;
  /** Default tags for new debt items */
  defaultTags?: string[];
  /** Callback when debt items are updated */
  onItemsUpdate?: (items: TechnicalDebtItem[]) => void;
  /** Whether to enable automatic detection of potential debt */
  enableAutoDetection?: boolean;
  /** Patterns to look for in code comments that indicate debt */
  debtPatterns?: RegExp[];
}

/**
 * Technical debt statistics
 */
export interface TechnicalDebtStats {
  /** Total number of debt items */
  totalItems: number;
  /** Number of items by status */
  itemsByStatus: Record<TechnicalDebtStatus, number>;
  /** Number of items by type */
  itemsByType: Record<TechnicalDebtType, number>;
  /** Total estimated effort (in person-days) */
  totalEffort: number;
  /** Average impact score */
  averageImpact: number;
  /** Average urgency score */
  averageUrgency: number;
  /** Technical debt score (calculated metric) */
  debtScore: number;
  /** Items by tag */
  itemsByTag: Record<string, number>;
  /** Items by assignee */
  itemsByAssignee: Record<string, number>;
  /** Items by location */
  itemsByLocation: Record<string, number>;
  /** Trend over time */
  trend: {
    labels: string[];
    identified: number[];
    resolved: number[];
    total: number[];
  };
}

/**
 * Technical debt filter options
 */
export interface TechnicalDebtFilter {
  /** Filter by status */
  status?: TechnicalDebtStatus | TechnicalDebtStatus[];
  /** Filter by type */
  type?: TechnicalDebtType | TechnicalDebtType[];
  /** Filter by tags */
  tags?: string[];
  /** Filter by assignee */
  assignee?: string;
  /** Filter by location */
  location?: string;
  /** Filter by minimum impact */
  minImpact?: number;
  /** Filter by maximum impact */
  maxImpact?: number;
  /** Filter by minimum urgency */
  minUrgency?: number;
  /** Filter by maximum urgency */
  maxUrgency?: number;
  /** Filter by creation date range */
  createdRange?: [number, number];
  /** Filter by target date range */
  targetRange?: [number, number];
  /** Filter by text in title or description */
  searchText?: string;
}

/**
 * Technical debt tracker
 */
export class TechnicalDebtTracker {
  private config: Required<TechnicalDebtConfig>;
  private debtItems: TechnicalDebtItem[] = [];
  private listeners: Set<(items: TechnicalDebtItem[]) => void> = new Set();

  /**
   * Creates a new TechnicalDebtTracker instance
   * 
   * @param config - Configuration options
   */
  constructor(config: TechnicalDebtConfig = {}) {
    this.config = {
      storageKey: config.storageKey ?? 'technical-debt-items',
      persistItems: config.persistItems ?? true,
      defaultAssignee: config.defaultAssignee ?? '',
      defaultTags: config.defaultTags ?? [],
      onItemsUpdate: config.onItemsUpdate ?? (() => {}),
      enableAutoDetection: config.enableAutoDetection ?? false,
      debtPatterns: config.debtPatterns ?? [
        /TODO/i,
        /FIXME/i,
        /HACK/i,
        /WORKAROUND/i,
        /TECHNICAL DEBT/i,
        /REFACTOR/i
      ]
    };

    // Load persisted items if enabled
    if (this.config.persistItems) {
      this.loadPersistedItems();
    }
  }

  /**
   * Adds a new technical debt item
   * 
   * @param item - Technical debt item to add (partial, will be completed with defaults)
   * @returns The added debt item
   */
  addDebtItem(item: Partial<TechnicalDebtItem>): TechnicalDebtItem {
    const now = Date.now();
    
    const newItem: TechnicalDebtItem = {
      id: item.id ?? this.generateId(),
      title: item.title ?? 'Untitled Debt Item',
      description: item.description ?? '',
      location: item.location ?? '',
      createdAt: item.createdAt ?? now,
      updatedAt: item.updatedAt ?? now,
      estimatedEffort: item.estimatedEffort ?? 1,
      impact: item.impact ?? 5,
      urgency: item.urgency ?? 5,
      status: item.status ?? TechnicalDebtStatus.IDENTIFIED,
      type: item.type ?? TechnicalDebtType.CODE,
      tags: item.tags ?? [...this.config.defaultTags],
      assignee: item.assignee ?? this.config.defaultAssignee,
      targetDate: item.targetDate,
      notes: item.notes,
      relatedItems: item.relatedItems ?? []
    };
    
    this.debtItems.push(newItem);
    this.notifyItemsUpdate();
    
    return newItem;
  }

  /**
   * Updates an existing technical debt item
   * 
   * @param id - ID of the item to update
   * @param updates - Partial updates to apply
   * @returns The updated debt item or undefined if not found
   */
  updateDebtItem(id: string, updates: Partial<TechnicalDebtItem>): TechnicalDebtItem | undefined {
    const index = this.debtItems.findIndex(item => item.id === id);
    
    if (index === -1) {
      return undefined;
    }
    
    const updatedItem = {
      ...this.debtItems[index],
      ...updates,
      updatedAt: Date.now()
    };
    
    this.debtItems[index] = updatedItem;
    this.notifyItemsUpdate();
    
    return updatedItem;
  }

  /**
   * Removes a technical debt item
   * 
   * @param id - ID of the item to remove
   * @returns Whether the item was removed
   */
  removeDebtItem(id: string): boolean {
    const initialLength = this.debtItems.length;
    this.debtItems = this.debtItems.filter(item => item.id !== id);
    
    if (this.debtItems.length !== initialLength) {
      this.notifyItemsUpdate();
      return true;
    }
    
    return false;
  }

  /**
   * Gets all technical debt items
   * 
   * @param filter - Filter options
   * @returns Array of debt items
   */
  getDebtItems(filter?: TechnicalDebtFilter): TechnicalDebtItem[] {
    if (!filter) {
      return [...this.debtItems];
    }
    
    return this.debtItems.filter(item => this.filterDebtItem(item, filter));
  }

  /**
   * Gets a specific technical debt item by ID
   * 
   * @param id - ID of the item to get
   * @returns The debt item or undefined if not found
   */
  getDebtItem(id: string): TechnicalDebtItem | undefined {
    return this.debtItems.find(item => item.id === id);
  }

  /**
   * Gets technical debt statistics
   * 
   * @returns Technical debt statistics
   */
  getStatistics(): TechnicalDebtStats {
    const items = this.debtItems;
    
    // Count items by status
    const itemsByStatus = Object.values(TechnicalDebtStatus).reduce((acc, status) => {
      acc[status] = items.filter(item => item.status === status).length;
      return acc;
    }, {} as Record<TechnicalDebtStatus, number>);
    
    // Count items by type
    const itemsByType = Object.values(TechnicalDebtType).reduce((acc, type) => {
      acc[type] = items.filter(item => item.type === type).length;
      return acc;
    }, {} as Record<TechnicalDebtType, number>);
    
    // Calculate total effort
    const totalEffort = items.reduce((sum, item) => sum + item.estimatedEffort, 0);
    
    // Calculate average impact and urgency
    const totalImpact = items.reduce((sum, item) => sum + item.impact, 0);
    const totalUrgency = items.reduce((sum, item) => sum + item.urgency, 0);
    const averageImpact = items.length > 0 ? totalImpact / items.length : 0;
    const averageUrgency = items.length > 0 ? totalUrgency / items.length : 0;
    
    // Calculate debt score (impact * urgency * effort)
    const debtScore = items.reduce((sum, item) => sum + (item.impact * item.urgency * item.estimatedEffort), 0);
    
    // Count items by tag
    const itemsByTag: Record<string, number> = {};
    for (const item of items) {
      for (const tag of item.tags) {
        itemsByTag[tag] = (itemsByTag[tag] || 0) + 1;
      }
    }
    
    // Count items by assignee
    const itemsByAssignee: Record<string, number> = {};
    for (const item of items) {
      if (item.assignee) {
        itemsByAssignee[item.assignee] = (itemsByAssignee[item.assignee] || 0) + 1;
      }
    }
    
    // Count items by location
    const itemsByLocation: Record<string, number> = {};
    for (const item of items) {
      if (item.location) {
        itemsByLocation[item.location] = (itemsByLocation[item.location] || 0) + 1;
      }
    }
    
    // Calculate trend over time (last 6 months)
    const now = new Date();
    const months: Date[] = [];
    for (let i = 5; i >= 0; i--) {
      const month = new Date(now.getFullYear(), now.getMonth() - i, 1);
      months.push(month);
    }
    
    const trend = {
      labels: months.map(date => `${date.getMonth() + 1}/${date.getFullYear()}`),
      identified: months.map(date => {
        const startOfMonth = date.getTime();
        const endOfMonth = new Date(date.getFullYear(), date.getMonth() + 1, 0).getTime();
        return items.filter(item => item.createdAt >= startOfMonth && item.createdAt <= endOfMonth).length;
      }),
      resolved: months.map(date => {
        const startOfMonth = date.getTime();
        const endOfMonth = new Date(date.getFullYear(), date.getMonth() + 1, 0).getTime();
        return items.filter(
          item => item.status === TechnicalDebtStatus.RESOLVED && 
                 item.updatedAt >= startOfMonth && 
                 item.updatedAt <= endOfMonth
        ).length;
      }),
      total: months.map(date => {
        const endOfMonth = new Date(date.getFullYear(), date.getMonth() + 1, 0).getTime();
        return items.filter(item => 
          item.createdAt <= endOfMonth && 
          (item.status !== TechnicalDebtStatus.RESOLVED || item.updatedAt > endOfMonth)
        ).length;
      })
    };
    
    return {
      totalItems: items.length,
      itemsByStatus,
      itemsByType,
      totalEffort,
      averageImpact,
      averageUrgency,
      debtScore,
      itemsByTag,
      itemsByAssignee,
      itemsByLocation,
      trend
    };
  }

  /**
   * Adds a listener for debt item updates
   * 
   * @param listener - Listener function
   * @returns Function to remove the listener
   */
  addListener(listener: (items: TechnicalDebtItem[]) => void): () => void {
    this.listeners.add(listener);
    
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Imports technical debt items from a JSON string
   * 
   * @param json - JSON string containing debt items
   * @returns Number of items imported
   */
  importFromJson(json: string): number {
    try {
      const items = JSON.parse(json);
      
      if (!Array.isArray(items)) {
        throw new Error('Invalid format: expected an array of debt items');
      }
      
      // Validate and add each item
      let importedCount = 0;
      for (const item of items) {
        if (this.isValidDebtItem(item)) {
          // Check if item with this ID already exists
          const existingIndex = this.debtItems.findIndex(i => i.id === item.id);
          
          if (existingIndex >= 0) {
            // Update existing item
            this.debtItems[existingIndex] = {
              ...item,
              updatedAt: Date.now()
            };
          } else {
            // Add new item
            this.debtItems.push(item);
          }
          
          importedCount++;
        }
      }
      
      if (importedCount > 0) {
        this.notifyItemsUpdate();
      }
      
      return importedCount;
    } catch (error) {
      console.error('Failed to import technical debt items:', error);
      return 0;
    }
  }

  /**
   * Exports technical debt items to a JSON string
   * 
   * @param filter - Filter to apply before exporting
   * @returns JSON string containing debt items
   */
  exportToJson(filter?: TechnicalDebtFilter): string {
    const items = this.getDebtItems(filter);
    return JSON.stringify(items, null, 2);
  }

  /**
   * Scans code for potential technical debt indicators
   * 
   * @param code - Code to scan
   * @param filename - Filename or path
   * @returns Array of potential debt items
   */
  scanCodeForDebt(code: string, filename: string): Partial<TechnicalDebtItem>[] {
    if (!this.config.enableAutoDetection) {
      return [];
    }
    
    const potentialDebtItems: Partial<TechnicalDebtItem>[] = [];
    const lines = code.split('\n');
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      
      for (const pattern of this.config.debtPatterns) {
        if (pattern.test(line)) {
          // Extract the comment part
          const commentMatch = line.match(/\/\/(.+)$|\/\*(.+)\*\/|#(.+)$/);
          let comment = '';
          
          if (commentMatch) {
            comment = commentMatch[1] || commentMatch[2] || commentMatch[3] || '';
            comment = comment.trim();
          }
          
          // Create a potential debt item
          potentialDebtItems.push({
            title: `Potential debt in ${filename}:${i + 1}`,
            description: comment || `Found pattern "${pattern.source}" in code`,
            location: `${filename}:${i + 1}`,
            type: this.inferDebtType(comment),
            tags: this.inferTags(comment)
          });
          
          break; // Only match one pattern per line
        }
      }
    }
    
    return potentialDebtItems;
  }

  /**
   * Clears all technical debt items
   */
  clearDebtItems(): void {
    this.debtItems = [];
    this.notifyItemsUpdate();
  }

  /**
   * Notifies listeners of debt item updates
   */
  private notifyItemsUpdate(): void {
    // Call the onItemsUpdate callback
    this.config.onItemsUpdate(this.debtItems);
    
    // Notify all listeners
    for (const listener of this.listeners) {
      listener(this.debtItems);
    }
    
    // Persist items if enabled
    if (this.config.persistItems) {
      this.persistItems();
    }
  }

  /**
   * Filters a debt item based on filter options
   * 
   * @param item - Debt item to filter
   * @param filter - Filter options
   * @returns Whether the item matches the filter
   */
  private filterDebtItem(item: TechnicalDebtItem, filter: TechnicalDebtFilter): boolean {
    // Filter by status
    if (filter.status) {
      if (Array.isArray(filter.status)) {
        if (!filter.status.includes(item.status)) {
          return false;
        }
      } else if (item.status !== filter.status) {
        return false;
      }
    }
    
    // Filter by type
    if (filter.type) {
      if (Array.isArray(filter.type)) {
        if (!filter.type.includes(item.type)) {
          return false;
        }
      } else if (item.type !== filter.type) {
        return false;
      }
    }
    
    // Filter by tags
    if (filter.tags && filter.tags.length > 0) {
      if (!filter.tags.some(tag => item.tags.includes(tag))) {
        return false;
      }
    }
    
    // Filter by assignee
    if (filter.assignee && item.assignee !== filter.assignee) {
      return false;
    }
    
    // Filter by location
    if (filter.location && !item.location.includes(filter.location)) {
      return false;
    }
    
    // Filter by impact
    if (filter.minImpact !== undefined && item.impact < filter.minImpact) {
      return false;
    }
    
    if (filter.maxImpact !== undefined && item.impact > filter.maxImpact) {
      return false;
    }
    
    // Filter by urgency
    if (filter.minUrgency !== undefined && item.urgency < filter.minUrgency) {
      return false;
    }
    
    if (filter.maxUrgency !== undefined && item.urgency > filter.maxUrgency) {
      return false;
    }
    
    // Filter by creation date range
    if (filter.createdRange) {
      const [min, max] = filter.createdRange;
      if (item.createdAt < min || item.createdAt > max) {
        return false;
      }
    }
    
    // Filter by target date range
    if (filter.targetRange && item.targetDate) {
      const [min, max] = filter.targetRange;
      if (item.targetDate < min || item.targetDate > max) {
        return false;
      }
    }
    
    // Filter by search text
    if (filter.searchText) {
      const searchText = filter.searchText.toLowerCase();
      const titleMatch = item.title.toLowerCase().includes(searchText);
      const descriptionMatch = item.description.toLowerCase().includes(searchText);
      const notesMatch = item.notes ? item.notes.toLowerCase().includes(searchText) : false;
      
      if (!titleMatch && !descriptionMatch && !notesMatch) {
        return false;
      }
    }
    
    return true;
  }

  /**
   * Persists debt items to storage
   */
  private persistItems(): void {
    if (typeof window === 'undefined' || !window.localStorage) {
      return;
    }
    
    try {
      localStorage.setItem(this.config.storageKey, JSON.stringify(this.debtItems));
    } catch (error) {
      console.error('Failed to persist technical debt items:', error);
    }
  }

  /**
   * Loads persisted debt items from storage
   */
  private loadPersistedItems(): void {
    if (typeof window === 'undefined' || !window.localStorage) {
      return;
    }
    
    try {
      const persistedItemsJson = localStorage.getItem(this.config.storageKey);
      
      if (!persistedItemsJson) {
        return;
      }
      
      const persistedItems = JSON.parse(persistedItemsJson);
      
      if (Array.isArray(persistedItems)) {
        this.debtItems = persistedItems.filter(item => this.isValidDebtItem(item));
      }
    } catch (error) {
      console.error('Failed to load persisted technical debt items:', error);
    }
  }

  /**
   * Validates a technical debt item
   * 
   * @param item - Item to validate
   * @returns Whether the item is valid
   */
  private isValidDebtItem(item: any): item is TechnicalDebtItem {
    return (
      typeof item === 'object' &&
      typeof item.id === 'string' &&
      typeof item.title === 'string' &&
      typeof item.description === 'string' &&
      typeof item.location === 'string' &&
      typeof item.createdAt === 'number' &&
      typeof item.updatedAt === 'number' &&
      typeof item.estimatedEffort === 'number' &&
      typeof item.impact === 'number' &&
      typeof item.urgency === 'number' &&
      typeof item.status === 'string' &&
      typeof item.type === 'string' &&
      Array.isArray(item.tags)
    );
  }

  /**
   * Infers the type of technical debt from a comment
   * 
   * @param comment - Comment to analyze
   * @returns Inferred debt type
   */
  private inferDebtType(comment: string): TechnicalDebtType {
    comment = comment.toLowerCase();
    
    if (comment.includes('refactor') || comment.includes('clean') || comment.includes('rewrite')) {
      return TechnicalDebtType.CODE;
    }
    
    if (comment.includes('architect') || comment.includes('design') || comment.includes('structure')) {
      return TechnicalDebtType.ARCHITECTURE;
    }
    
    if (comment.includes('document') || comment.includes('explain') || comment.includes('clarify')) {
      return TechnicalDebtType.DOCUMENTATION;
    }
    
    if (comment.includes('test') || comment.includes('coverage') || comment.includes('assert')) {
      return TechnicalDebtType.TESTING;
    }
    
    if (comment.includes('build') || comment.includes('deploy') || comment.includes('ci') || comment.includes('cd')) {
      return TechnicalDebtType.INFRASTRUCTURE;
    }
    
    if (comment.includes('dependency') || comment.includes('package') || comment.includes('library') || comment.includes('update')) {
      return TechnicalDebtType.DEPENDENCIES;
    }
    
    if (comment.includes('performance') || comment.includes('slow') || comment.includes('optimize') || comment.includes('speed')) {
      return TechnicalDebtType.PERFORMANCE;
    }
    
    if (comment.includes('security') || comment.includes('vulnerability') || comment.includes('exploit') || comment.includes('hack')) {
      return TechnicalDebtType.SECURITY;
    }
    
    if (comment.includes('accessibility') || comment.includes('a11y') || comment.includes('aria')) {
      return TechnicalDebtType.ACCESSIBILITY;
    }
    
    if (comment.includes('ux') || comment.includes('ui') || comment.includes('user experience') || comment.includes('usability')) {
      return TechnicalDebtType.UX;
    }
    
    return TechnicalDebtType.CODE; // Default to code
  }

  /**
   * Infers tags from a comment
   * 
   * @param comment - Comment to analyze
   * @returns Array of inferred tags
   */
  private inferTags(comment: string): string[] {
    const tags: string[] = [];
    comment = comment.toLowerCase();
    
    // Look for common indicators
    if (comment.includes('todo')) tags.push('todo');
    if (comment.includes('fixme')) tags.push('fixme');
    if (comment.includes('hack')) tags.push('hack');
    if (comment.includes('workaround')) tags.push('workaround');
    if (comment.includes('refactor')) tags.push('refactor');
    if (comment.includes('optimize')) tags.push('optimize');
    if (comment.includes('review')) tags.push('review');
    if (comment.includes('legacy')) tags.push('legacy');
    
    // Look for severity indicators
    if (comment.includes('critical') || comment.includes('severe')) {
      tags.push('critical');
    } else if (comment.includes('important') || comment.includes('major')) {
      tags.push('important');
    } else if (comment.includes('minor') || comment.includes('trivial')) {
      tags.push('minor');
    }
    
    return tags;
  }

  /**
   * Generates a unique ID
   * 
   * @returns Unique ID
   */
  private generateId(): string {
    return 'debt_' + Math.random().toString(36).substring(2, 15) + 
           '_' + Date.now().toString(36);
  }
}

/**
 * Global technical debt tracker instance
 */
export const globalDebtTracker = new TechnicalDebtTracker();

/**
 * Hook for using technical debt tracking in React components
 * 
 * @param config - Technical debt tracking configuration
 * @returns Technical debt tracker instance and utility functions
 */
export function useTechnicalDebtTracking(config?: TechnicalDebtConfig) {
  // Use the global tracker or create a new one
  const tracker = config ? new TechnicalDebtTracker(config) : globalDebtTracker;
  
  // State for component-specific debt items
  const [debtItems, setDebtItems] = useState<TechnicalDebtItem[]>(tracker.getDebtItems());
  const [stats, setStats] = useState<TechnicalDebtStats>(tracker.getStatistics());
  
  // Update debt items when they change
  useEffect(() => {
    const updateDebtItems = () => {
      setDebtItems(tracker.getDebtItems());
      setStats(tracker.getStatistics());
    };
    
    const removeListener = tracker.addListener(() => {
      updateDebtItems();
    });
    
    // Initial update
    updateDebtItems();
    
    return removeListener;
  }, [tracker]);
  
  return {
    tracker,
    debtItems,
    stats,
    
    /**
     * Gets all technical debt items
     * 
     * @param filter - Filter options
     * @returns Array of debt items
     */
    getDebtItems: (filter?: TechnicalDebtFilter) => tracker.getDebtItems(filter),
    
    /**
     * Gets a specific technical debt item by ID
     * 
     * @param id - ID of the item to get
     * @returns The debt item or undefined if not found
     */
    getDebtItem: (id: string) => tracker.getDebtItem(id),
    
    /**
     * Gets technical debt statistics
     * 
     * @returns Technical debt statistics
     */
    getStatistics: () => tracker.getStatistics(),
    
    /**
     * Adds a new technical debt item
     * 
     * @param item - Technical debt item to add (partial, will be completed with defaults)
     * @returns The added debt item
     */
    addDebtItem: (item: Partial<TechnicalDebtItem>) => tracker.addDebtItem(item),
    
    /**
     * Updates an existing technical debt item
     * 
     * @param id - ID of the item to update
     * @param updates - Partial updates to apply
     * @returns The updated debt item or undefined if not found
     */
    updateDebtItem: (id: string, updates: Partial<TechnicalDebtItem>) => 
      tracker.updateDebtItem(id, updates),
    
    /**
     * Removes a technical debt item
     * 
     * @param id - ID of the item to remove
     * @returns Whether the item was removed
     */
    removeDebtItem: (id: string) => tracker.removeDebtItem(id),
    
    /**
     * Scans code for potential technical debt indicators
     * 
     * @param code - Code to scan
     * @param filename - Filename or path
     * @returns Array of potential debt items
     */
    scanCodeForDebt: (code: string, filename: string) => 
      tracker.scanCodeForDebt(code, filename),
    
    /**
     * Imports technical debt items from a JSON string
     * 
     * @param json - JSON string containing debt items
     * @returns Number of items imported
     */
    importFromJson: (json: string) => tracker.importFromJson(json),
    
    /**
     * Exports technical debt items to a JSON string
     * 
     * @param filter - Filter to apply before exporting
     * @returns JSON string containing debt items
     */
    exportToJson: (filter?: TechnicalDebtFilter) => tracker.exportToJson(filter)
  };
}