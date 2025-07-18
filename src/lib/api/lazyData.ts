import { useState, useEffect, useCallback, useRef } from 'react';
import { useBaseQuery, UseBaseQueryOptions, UseBaseQueryResult } from './baseHooks';
import { ApiClientError } from './client';

/**
 * Options for the useVisibilityQuery hook
 */
export interface UseVisibilityQueryOptions<T> extends UseBaseQueryOptions<T> {
  /** Root element to use for intersection observer */
  root?: Element | null;
  /** Margin around the root element */
  rootMargin?: string;
  /** Threshold at which to trigger the query */
  threshold?: number | number[];
  /** Whether to keep the data after it's loaded */
  keepData?: boolean;
}

/**
 * Hook that fetches data when an element becomes visible in the viewport
 * 
 * @param queryFn - Function that returns a promise with the query result
 * @param elementRef - Ref to the element to observe
 * @param dependencies - Dependencies array for the query function
 * @param options - Options for the query
 * @returns Query result with visibility-based loading
 */
export function useVisibilityQuery<T>(
  queryFn: () => Promise<T>,
  elementRef: React.RefObject<Element>,
  dependencies: any[] = [],
  options: UseVisibilityQueryOptions<T> = {}
): UseBaseQueryResult<T> & { isVisible: boolean } {
  const [isVisible, setIsVisible] = useState(false);
  const [shouldFetch, setShouldFetch] = useState(false);
  const hasLoadedRef = useRef(false);

  // Set up intersection observer to detect when element is visible
  useEffect(() => {
    if (!elementRef.current) return;

    const observer = new IntersectionObserver(
      (entries) => {
        const isElementVisible = entries[0]?.isIntersecting ?? false;
        setIsVisible(isElementVisible);
        
        if (isElementVisible && !hasLoadedRef.current) {
          setShouldFetch(true);
          if (options.keepData !== false) {
            hasLoadedRef.current = true;
          }
        }
      },
      {
        root: options.root || null,
        rootMargin: options.rootMargin || '0px',
        threshold: options.threshold || 0,
      }
    );

    observer.observe(elementRef.current);

    return () => {
      observer.disconnect();
    };
  }, [elementRef, options.root, options.rootMargin, options.threshold, options.keepData]);

  // Reset loaded state if dependencies change
  useEffect(() => {
    if (!options.keepData) {
      hasLoadedRef.current = false;
      setShouldFetch(false);
    }
  }, dependencies);

  // Use the base query hook with visibility-based enabled option
  const queryResult = useBaseQuery<T>(
    queryFn,
    dependencies,
    {
      ...options,
      enabled: shouldFetch && (options.enabled !== false),
    }
  );

  return {
    ...queryResult,
    isVisible,
  };
}

/**
 * Options for the useInteractionQuery hook
 */
export interface UseInteractionQueryOptions<T> extends UseBaseQueryOptions<T> {
  /** Whether to automatically fetch on interaction */
  autoFetch?: boolean;
}

/**
 * Hook that fetches data when the user interacts with the component
 * 
 * @param queryFn - Function that returns a promise with the query result
 * @param dependencies - Dependencies array for the query function
 * @param options - Options for the query
 * @returns Query result with interaction-based loading and trigger function
 */
export function useInteractionQuery<T>(
  queryFn: () => Promise<T>,
  dependencies: any[] = [],
  options: UseInteractionQueryOptions<T> = {}
): UseBaseQueryResult<T> & { trigger: () => void } {
  const [shouldFetch, setShouldFetch] = useState(false);

  // Function to trigger the data fetch
  const trigger = useCallback(() => {
    setShouldFetch(true);
  }, []);

  // Use the base query hook with interaction-based enabled option
  const queryResult = useBaseQuery<T>(
    queryFn,
    dependencies,
    {
      ...options,
      enabled: shouldFetch && (options.enabled !== false),
    }
  );

  return {
    ...queryResult,
    trigger,
  };
}

/**
 * Options for the useProgressiveQuery hook
 */
export interface UseProgressiveQueryOptions<T> extends UseBaseQueryOptions<T> {
  /** Priority levels for data loading (higher number = higher priority) */
  priorityLevels?: number;
  /** Delay between loading different priority levels (in milliseconds) */
  levelDelay?: number;
}

/**
 * Result of the useProgressiveQuery hook
 */
export interface UseProgressiveQueryResult<T> extends UseBaseQueryResult<T> {
  /** Current priority level being loaded */
  currentLevel: number;
  /** Whether all priority levels have been loaded */
  isComplete: boolean;
  /** Function to load the next priority level */
  loadNextLevel: () => void;
}

/**
 * Hook that progressively loads data in priority order
 * 
 * @param queryFnMap - Map of priority levels to query functions
 * @param dependencies - Dependencies array for the query functions
 * @param options - Options for the query
 * @returns Query result with progressive loading
 */
export function useProgressiveQuery<T>(
  queryFnMap: Record<number, () => Promise<Partial<T>>>,
  dependencies: any[] = [],
  options: UseProgressiveQueryOptions<T> = {}
): UseProgressiveQueryResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [currentLevel, setCurrentLevel] = useState(1);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<ApiClientError | null>(null);
  
  const priorityLevels = options.priorityLevels || Object.keys(queryFnMap).length;
  const levelDelay = options.levelDelay || 0;
  const timerRef = useRef<NodeJS.Timeout>();
  const mountedRef = useRef(true);

  // Function to load a specific priority level
  const loadLevel = useCallback(async (level: number) => {
    if (!mountedRef.current || !queryFnMap[level]) return;
    
    setIsLoading(true);
    
    try {
      const levelData = await queryFnMap[level]();
      
      if (mountedRef.current) {
        setData(prevData => ({
          ...(prevData || {} as T),
          ...levelData
        } as T));
        
        setError(null);
      }
    } catch (err) {
      if (mountedRef.current) {
        const apiError = err instanceof ApiClientError 
          ? err 
          : new ApiClientError('unknown', 'Unknown error');
        
        setError(apiError);
        options.onError?.(apiError);
      }
    } finally {
      if (mountedRef.current) {
        setIsLoading(false);
        
        // Schedule next level if not at max level
        if (level < priorityLevels && levelDelay > 0) {
          timerRef.current = setTimeout(() => {
            if (mountedRef.current) {
              setCurrentLevel(level + 1);
            }
          }, levelDelay);
        } else if (level < priorityLevels) {
          setCurrentLevel(level + 1);
        }
      }
    }
  }, [queryFnMap, priorityLevels, levelDelay, options]);

  // Function to manually load the next level
  const loadNextLevel = useCallback(() => {
    if (currentLevel <= priorityLevels) {
      loadLevel(currentLevel);
    }
  }, [currentLevel, priorityLevels, loadLevel]);

  // Effect to load the first level when enabled
  useEffect(() => {
    if (options.enabled !== false) {
      loadLevel(1);
    }
    
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [...dependencies, options.enabled]);

  // Effect to load each level as currentLevel changes
  useEffect(() => {
    if (currentLevel > 1 && currentLevel <= priorityLevels) {
      loadLevel(currentLevel);
    }
  }, [currentLevel, priorityLevels, loadLevel]);

  // Effect to clean up on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  const isComplete = currentLevel > priorityLevels;
  const isSuccess = data !== null && !error;

  return {
    data,
    error,
    isLoading,
    isFetching: isLoading,
    isError: error !== null,
    isSuccess,
    status: isLoading ? 'loading' : error ? 'error' : isSuccess ? 'success' : 'idle',
    refetch: () => {
      setCurrentLevel(1);
      setData(null);
      return loadLevel(1).then(() => data);
    },
    reset: () => {
      setCurrentLevel(1);
      setData(null);
      setError(null);
      setIsLoading(false);
    },
    currentLevel,
    isComplete,
    loadNextLevel,
  };
}