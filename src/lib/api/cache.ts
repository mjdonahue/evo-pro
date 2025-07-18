/**
 * Cache implementation for API client
 * 
 * This module provides a configurable caching layer for the API client with different
 * caching strategies and utilities for cache management.
 */

import { ApiResponse, ListResponse } from './types';

/**
 * Cache entry with data and metadata
 */
interface CacheEntry<T> {
  /** The cached data */
  data: T;
  /** Timestamp when the entry was cached */
  timestamp: number;
  /** Tags associated with this cache entry for invalidation */
  tags?: string[];
}

/**
 * Cache strategy interface
 */
export interface CacheStrategy {
  /** Strategy name */
  name: string;
  /** Get an item from the cache */
  get<T>(key: string): CacheEntry<T> | undefined;
  /** Set an item in the cache */
  set<T>(key: string, entry: CacheEntry<T>): void;
  /** Remove an item from the cache */
  remove(key: string): void;
  /** Clear all items from the cache */
  clear(): void;
  /** Clear items with specific tags */
  clearByTags(tags: string[]): void;
}

/**
 * In-memory cache strategy
 */
export class MemoryCacheStrategy implements CacheStrategy {
  name = 'memory';
  private cache: Record<string, CacheEntry<any>> = {};

  get<T>(key: string): CacheEntry<T> | undefined {
    return this.cache[key];
  }

  set<T>(key: string, entry: CacheEntry<T>): void {
    this.cache[key] = entry;
  }

  remove(key: string): void {
    delete this.cache[key];
  }

  clear(): void {
    this.cache = {};
  }

  clearByTags(tags: string[]): void {
    for (const key in this.cache) {
      const entry = this.cache[key];
      if (entry.tags && entry.tags.some(tag => tags.includes(tag))) {
        delete this.cache[key];
      }
    }
  }
}

/**
 * Local storage cache strategy
 */
export class LocalStorageCacheStrategy implements CacheStrategy {
  name = 'localStorage';
  private prefix = 'api_cache_';

  get<T>(key: string): CacheEntry<T> | undefined {
    if (typeof window === 'undefined') return undefined;
    
    const item = localStorage.getItem(this.prefix + key);
    if (!item) return undefined;
    
    try {
      return JSON.parse(item) as CacheEntry<T>;
    } catch (e) {
      this.remove(key);
      return undefined;
    }
  }

  set<T>(key: string, entry: CacheEntry<T>): void {
    if (typeof window === 'undefined') return;
    
    try {
      localStorage.setItem(this.prefix + key, JSON.stringify(entry));
    } catch (e) {
      // Handle localStorage errors (e.g., quota exceeded)
      console.warn('Failed to store item in localStorage:', e);
    }
  }

  remove(key: string): void {
    if (typeof window === 'undefined') return;
    localStorage.removeItem(this.prefix + key);
  }

  clear(): void {
    if (typeof window === 'undefined') return;
    
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key?.startsWith(this.prefix)) {
        localStorage.removeItem(key);
      }
    }
  }

  clearByTags(tags: string[]): void {
    if (typeof window === 'undefined') return;
    
    const keysToRemove: string[] = [];
    
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key?.startsWith(this.prefix)) {
        try {
          const entry = JSON.parse(localStorage.getItem(key) || '{}') as CacheEntry<any>;
          if (entry.tags && entry.tags.some(tag => tags.includes(tag))) {
            keysToRemove.push(key);
          }
        } catch (e) {
          // Invalid JSON, remove the item
          keysToRemove.push(key);
        }
      }
    }
    
    keysToRemove.forEach(key => localStorage.removeItem(key));
  }
}

/**
 * Cache configuration options
 */
export interface CacheOptions {
  /** Time to live in milliseconds (default: 5 minutes) */
  ttl?: number;
  /** Tags for cache invalidation */
  tags?: string[];
  /** Whether to bypass the cache for this operation */
  bypass?: boolean;
}

/**
 * Default cache options
 */
const DEFAULT_CACHE_OPTIONS: CacheOptions = {
  ttl: 5 * 60 * 1000, // 5 minutes
  tags: [],
  bypass: false
};

/**
 * Cache manager for API client
 */
export class CacheManager {
  private strategy: CacheStrategy;
  private defaultOptions: CacheOptions;

  /**
   * Creates a new cache manager
   * @param strategy - The caching strategy to use
   * @param defaultOptions - Default options for all cache operations
   */
  constructor(
    strategy: CacheStrategy = new MemoryCacheStrategy(),
    defaultOptions: CacheOptions = DEFAULT_CACHE_OPTIONS
  ) {
    this.strategy = strategy;
    this.defaultOptions = { ...DEFAULT_CACHE_OPTIONS, ...defaultOptions };
  }

  /**
   * Sets the caching strategy
   * @param strategy - The new caching strategy
   */
  setStrategy(strategy: CacheStrategy): void {
    this.strategy = strategy;
  }

  /**
   * Sets default options for all cache operations
   * @param options - The new default options
   */
  setDefaultOptions(options: Partial<CacheOptions>): void {
    this.defaultOptions = { ...this.defaultOptions, ...options };
  }

  /**
   * Gets an item from the cache
   * @param key - The cache key
   * @param options - Cache options
   * @returns The cached data or undefined if not found or expired
   */
  get<T>(key: string, options?: Partial<CacheOptions>): T | undefined {
    const opts = { ...this.defaultOptions, ...options };
    
    if (opts.bypass) return undefined;
    
    const entry = this.strategy.get<T>(key);
    if (!entry) return undefined;
    
    // Check if the entry is expired
    if (opts.ttl !== undefined && Date.now() - entry.timestamp > opts.ttl) {
      this.strategy.remove(key);
      return undefined;
    }
    
    return entry.data;
  }

  /**
   * Sets an item in the cache
   * @param key - The cache key
   * @param data - The data to cache
   * @param options - Cache options
   */
  set<T>(key: string, data: T, options?: Partial<CacheOptions>): void {
    const opts = { ...this.defaultOptions, ...options };
    
    if (opts.bypass) return;
    
    const entry: CacheEntry<T> = {
      data,
      timestamp: Date.now(),
      tags: opts.tags
    };
    
    this.strategy.set(key, entry);
  }

  /**
   * Removes an item from the cache
   * @param key - The cache key
   */
  remove(key: string): void {
    this.strategy.remove(key);
  }

  /**
   * Clears all items from the cache
   */
  clear(): void {
    this.strategy.clear();
  }

  /**
   * Clears items with specific tags
   * @param tags - The tags to clear
   */
  clearByTags(tags: string[]): void {
    this.strategy.clearByTags(tags);
  }

  /**
   * Clears items related to a specific entity type
   * @param entityName - The entity name
   */
  clearByEntity(entityName: string): void {
    this.clearByTags([entityName]);
  }
}

// Create a singleton instance of the cache manager
export const cacheManager = new CacheManager();

/**
 * Wraps a function with caching
 * @param fn - The function to wrap
 * @param key - The cache key
 * @param options - Cache options
 * @returns A function that uses the cache
 */
export function withCache<T, Args extends any[]>(
  fn: (...args: Args) => Promise<ApiResponse<T>>,
  key: string,
  options?: Partial<CacheOptions>
): (...args: Args) => Promise<ApiResponse<T>> {
  return async (...args: Args) => {
    const opts = { ...DEFAULT_CACHE_OPTIONS, ...options };
    
    // Skip cache if bypass is true
    if (opts.bypass) {
      return fn(...args);
    }
    
    // Try to get from cache
    const cachedData = cacheManager.get<ApiResponse<T>>(key, opts);
    if (cachedData) {
      return cachedData;
    }
    
    // Call the original function
    const response = await fn(...args);
    
    // Cache the result if successful
    if (response.success) {
      cacheManager.set(key, response, opts);
    }
    
    return response;
  };
}

/**
 * Generates a cache key for an entity
 * @param entityName - The entity name
 * @param id - The entity ID
 * @returns A cache key
 */
export function entityCacheKey(entityName: string, id: string): string {
  return `${entityName}_${id}`;
}

/**
 * Generates a cache key for a list operation
 * @param entityName - The entity name
 * @param filter - The filter object
 * @returns A cache key
 */
export function listCacheKey(entityName: string, filter?: Record<string, any>): string {
  if (!filter || Object.keys(filter).length === 0) {
    return `${entityName}_list`;
  }
  
  // Create a stable representation of the filter
  const filterStr = JSON.stringify(filter, Object.keys(filter).sort());
  return `${entityName}_list_${filterStr}`;
}