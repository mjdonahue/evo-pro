import { useState, useEffect, useCallback, useRef } from 'react';
import { ApiClientError } from './client';
import type { ListResponse } from './types';

/**
 * Status of a query or mutation
 */
export type QueryStatus = 'idle' | 'loading' | 'success' | 'error';

/**
 * Options for the useBaseQuery hook
 */
export interface UseBaseQueryOptions<T> {
  /** Whether the query should execute automatically */
  enabled?: boolean;
  /** Interval in milliseconds to refetch data */
  refetchInterval?: number;
  /** Initial data to use before the query executes */
  initialData?: T | (() => T);
  /** Number of retry attempts for failed queries */
  retry?: number | boolean;
  /** Delay between retry attempts in milliseconds */
  retryDelay?: number | ((attempt: number) => number);
  /** Callback when query succeeds */
  onSuccess?: (data: T) => void;
  /** Callback when query fails */
  onError?: (error: ApiClientError) => void;
  /** Callback when query settles (success or error) */
  onSettled?: (data: T | null, error: ApiClientError | null) => void;
}

/**
 * Result of the useBaseQuery hook
 */
export interface UseBaseQueryResult<T> {
  /** The query data */
  data: T | null;
  /** The query error */
  error: ApiClientError | null;
  /** Whether the query is currently loading */
  isLoading: boolean;
  /** Whether the query is currently fetching (initial load or refetch) */
  isFetching: boolean;
  /** Whether the query has errored */
  isError: boolean;
  /** Whether the query was successful */
  isSuccess: boolean;
  /** The current status of the query */
  status: QueryStatus;
  /** Function to manually refetch data */
  refetch: () => Promise<T | null>;
  /** Function to reset the query state */
  reset: () => void;
}

/**
 * Enhanced base query hook with improved loading/error states and additional features
 * 
 * @param queryFn - Function that returns a promise with the query result
 * @param dependencies - Dependencies array for the query function
 * @param options - Options for the query
 * @returns Query result with enhanced state management
 */
export function useBaseQuery<T>(
  queryFn: () => Promise<T>,
  dependencies: any[] = [],
  options: UseBaseQueryOptions<T> = {}
): UseBaseQueryResult<T> {
  // Get initial data if provided
  const initialData = typeof options.initialData === 'function'
    ? (options.initialData as () => T)()
    : options.initialData;

  // State for the query
  const [data, setData] = useState<T | null>(initialData || null);
  const [error, setError] = useState<ApiClientError | null>(null);
  const [status, setStatus] = useState<QueryStatus>(initialData ? 'success' : 'idle');
  const [isFetching, setIsFetching] = useState(false);
  
  // Refs for tracking state between renders
  const intervalRef = useRef<NodeJS.Timeout>();
  const retryCountRef = useRef(0);
  const retryTimerRef = useRef<NodeJS.Timeout>();
  const enabledRef = useRef(options.enabled !== false);
  const mountedRef = useRef(true);

  // Calculate retry settings
  const shouldRetry = options.retry === true || (typeof options.retry === 'number' && options.retry > 0);
  const maxRetries = typeof options.retry === 'number' ? options.retry : 3;
  
  // Calculate retry delay
  const getRetryDelay = useCallback((attempt: number): number => {
    if (typeof options.retryDelay === 'function') {
      return options.retryDelay(attempt);
    }
    if (typeof options.retryDelay === 'number') {
      return options.retryDelay;
    }
    // Default exponential backoff: 1000ms, 2000ms, 4000ms, etc.
    return Math.min(1000 * 2 ** attempt, 30000);
  }, [options.retryDelay]);

  // Function to fetch data
  const fetchData = useCallback(async (isRetry = false): Promise<T | null> => {
    if (!mountedRef.current) return null;
    
    try {
      setIsFetching(true);
      if (!isRetry) {
        setStatus('loading');
      }
      
      const result = await queryFn();
      
      if (mountedRef.current) {
        setData(result);
        setError(null);
        setStatus('success');
        retryCountRef.current = 0;
        options.onSuccess?.(result);
        options.onSettled?.(result, null);
      }
      
      return result;
    } catch (err) {
      if (!mountedRef.current) return null;
      
      const apiError = err instanceof ApiClientError 
        ? err 
        : new ApiClientError('unknown', 'Unknown error');
      
      // Handle retry logic
      if (shouldRetry && retryCountRef.current < maxRetries) {
        retryCountRef.current++;
        
        // Clear any existing retry timer
        if (retryTimerRef.current) {
          clearTimeout(retryTimerRef.current);
        }
        
        // Set up retry with delay
        const delay = getRetryDelay(retryCountRef.current);
        retryTimerRef.current = setTimeout(() => {
          if (mountedRef.current) {
            fetchData(true).catch(() => {
              // Catch error to prevent unhandled promise rejection
            });
          }
        }, delay);
        
        return null;
      }
      
      // If we've exhausted retries or shouldn't retry, set error state
      setError(apiError);
      setStatus('error');
      options.onError?.(apiError);
      options.onSettled?.(null, apiError);
      
      return null;
    } finally {
      if (mountedRef.current) {
        setIsFetching(false);
      }
    }
  }, [queryFn, shouldRetry, maxRetries, getRetryDelay, options]);

  // Function to manually refetch data
  const refetch = useCallback(async (): Promise<T | null> => {
    return fetchData();
  }, [fetchData]);

  // Function to reset the query state
  const reset = useCallback(() => {
    setData(initialData || null);
    setError(null);
    setStatus(initialData ? 'success' : 'idle');
    setIsFetching(false);
    retryCountRef.current = 0;
    
    // Clear any timers
    if (retryTimerRef.current) {
      clearTimeout(retryTimerRef.current);
    }
  }, [initialData]);

  // Effect to fetch data when dependencies change or when enabled
  useEffect(() => {
    enabledRef.current = options.enabled !== false;
    
    if (enabledRef.current) {
      fetchData().catch(() => {
        // Catch error to prevent unhandled promise rejection
      });
    }
  }, [...dependencies, options.enabled]);

  // Effect to set up refetch interval
  useEffect(() => {
    if (options.refetchInterval && options.refetchInterval > 0) {
      intervalRef.current = setInterval(() => {
        if (enabledRef.current) {
          fetchData().catch(() => {
            // Catch error to prevent unhandled promise rejection
          });
        }
      }, options.refetchInterval);
      
      return () => {
        if (intervalRef.current) {
          clearInterval(intervalRef.current);
        }
      };
    }
  }, [fetchData, options.refetchInterval]);

  // Effect to clean up on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
      
      if (retryTimerRef.current) {
        clearTimeout(retryTimerRef.current);
      }
    };
  }, []);

  // Derived states
  const isLoading = status === 'loading';
  const isError = status === 'error';
  const isSuccess = status === 'success';

  return {
    data,
    error,
    isLoading,
    isFetching,
    isError,
    isSuccess,
    status,
    refetch,
    reset
  };
}

/**
 * Options for the useBaseListQuery hook
 */
export interface UseBaseListQueryOptions<T> extends UseBaseQueryOptions<T[]> {
  /** Number of items to fetch per page */
  pageSize?: number;
  /** Whether to keep previous data when fetching more */
  keepPreviousData?: boolean;
}

/**
 * Result of the useBaseListQuery hook
 */
export interface UseBaseListQueryResult<T> extends UseBaseQueryResult<T[]> {
  /** Total number of items available */
  total: number;
  /** Whether there are more items to load */
  hasMore: boolean;
  /** Function to load the next page of data */
  loadMore: () => Promise<T[] | null>;
  /** Current page size */
  pageSize: number;
  /** Current offset */
  offset: number;
}

/**
 * Enhanced base list query hook with pagination and improved loading/error states
 * 
 * @param queryFn - Function that returns a promise with the paginated query result
 * @param dependencies - Dependencies array for the query function
 * @param options - Options for the query
 * @returns List query result with enhanced state management and pagination
 */
export function useBaseListQuery<T>(
  queryFn: (offset: number, limit: number) => Promise<ListResponse<T>>,
  dependencies: any[] = [],
  options: UseBaseListQueryOptions<T> = {}
): UseBaseListQueryResult<T> {
  // State for pagination
  const [total, setTotal] = useState(0);
  const [offset, setOffset] = useState(0);
  const pageSize = options.pageSize || 20;
  
  // Refs for tracking state between renders
  const dataRef = useRef<T[]>([]);
  const isFetchingMoreRef = useRef(false);

  // Function to fetch data with pagination
  const fetchWithPagination = useCallback(async (resetData = true): Promise<T[]> => {
    const currentOffset = resetData ? 0 : offset;
    
    try {
      isFetchingMoreRef.current = !resetData;
      const result = await queryFn(currentOffset, pageSize);
      
      let newData: T[];
      if (resetData) {
        newData = result.items;
        setOffset(result.items.length);
      } else {
        newData = [...dataRef.current, ...result.items];
        setOffset(currentOffset + result.items.length);
      }
      
      dataRef.current = newData;
      setTotal(result.total);
      
      return newData;
    } finally {
      isFetchingMoreRef.current = false;
    }
  }, [...dependencies, offset, pageSize]);

  // Use the base query hook with our pagination function
  const queryResult = useBaseQuery<T[]>(
    () => fetchWithPagination(true),
    dependencies,
    {
      ...options,
      // Override onSuccess to update our dataRef
      onSuccess: (data) => {
        dataRef.current = data;
        options.onSuccess?.(data);
      }
    }
  );

  // Function to load more data
  const loadMore = useCallback(async (): Promise<T[] | null> => {
    if (queryResult.isLoading || isFetchingMoreRef.current || !queryResult.isSuccess) {
      return null;
    }
    
    if (dataRef.current.length >= total) {
      return dataRef.current;
    }
    
    try {
      const newData = await fetchWithPagination(false);
      queryResult.onSuccess?.(newData);
      return newData;
    } catch (error) {
      if (error instanceof ApiClientError) {
        queryResult.onError?.(error);
      }
      return null;
    }
  }, [fetchWithPagination, queryResult, total]);

  // Function to refetch data
  const refetch = useCallback(async (): Promise<T[] | null> => {
    setOffset(0);
    return queryResult.refetch();
  }, [queryResult]);

  // Calculate if there are more items to load
  const hasMore = (queryResult.data?.length || 0) < total;

  return {
    ...queryResult,
    refetch,
    total,
    hasMore,
    loadMore,
    pageSize,
    offset
  };
}

/**
 * Options for the useBaseMutation hook
 */
export interface UseBaseMutationOptions<T, TVariables> {
  /** Callback when mutation succeeds */
  onSuccess?: (data: T, variables: TVariables) => void;
  /** Callback when mutation fails */
  onError?: (error: ApiClientError, variables: TVariables) => void;
  /** Callback when mutation settles (success or error) */
  onSettled?: (data: T | null, error: ApiClientError | null, variables: TVariables) => void;
  /** Function to get optimistic data based on variables */
  optimisticUpdate?: (variables: TVariables) => T;
  /** Number of retry attempts for failed mutations */
  retry?: number | boolean;
  /** Delay between retry attempts in milliseconds */
  retryDelay?: number | ((attempt: number) => number);
}

/**
 * Result of the useBaseMutation hook
 */
export interface UseBaseMutationResult<T, TVariables> {
  /** Function to trigger the mutation */
  mutate: (variables: TVariables) => Promise<T>;
  /** The mutation data */
  data: T | null;
  /** The mutation error */
  error: ApiClientError | null;
  /** Whether the mutation is currently running */
  isLoading: boolean;
  /** Whether the mutation has errored */
  isError: boolean;
  /** Whether the mutation was successful */
  isSuccess: boolean;
  /** The current status of the mutation */
  status: QueryStatus;
  /** Function to reset the mutation state */
  reset: () => void;
  /** Variables from the last mutation */
  variables: TVariables | null;
}

/**
 * Enhanced base mutation hook with improved loading/error states and additional features
 * 
 * @param mutationFn - Function that returns a promise with the mutation result
 * @param options - Options for the mutation
 * @returns Mutation result with enhanced state management
 */
export function useBaseMutation<T, TVariables = unknown>(
  mutationFn: (variables: TVariables) => Promise<T>,
  options: UseBaseMutationOptions<T, TVariables> = {}
): UseBaseMutationResult<T, TVariables> {
  // State for the mutation
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<ApiClientError | null>(null);
  const [status, setStatus] = useState<QueryStatus>('idle');
  const [variables, setVariables] = useState<TVariables | null>(null);
  
  // Refs for tracking state between renders
  const retryCountRef = useRef(0);
  const retryTimerRef = useRef<NodeJS.Timeout>();
  const mountedRef = useRef(true);

  // Calculate retry settings
  const shouldRetry = options.retry === true || (typeof options.retry === 'number' && options.retry > 0);
  const maxRetries = typeof options.retry === 'number' ? options.retry : 3;
  
  // Calculate retry delay
  const getRetryDelay = useCallback((attempt: number): number => {
    if (typeof options.retryDelay === 'function') {
      return options.retryDelay(attempt);
    }
    if (typeof options.retryDelay === 'number') {
      return options.retryDelay;
    }
    // Default exponential backoff: 1000ms, 2000ms, 4000ms, etc.
    return Math.min(1000 * 2 ** attempt, 30000);
  }, [options.retryDelay]);

  // Function to execute the mutation
  const mutate = useCallback(async (mutationVariables: TVariables): Promise<T> => {
    if (!mountedRef.current) {
      throw new Error('Cannot mutate when component is unmounted');
    }
    
    // Store variables for potential retries and callbacks
    setVariables(mutationVariables);
    
    // Apply optimistic update if provided
    if (options.optimisticUpdate) {
      try {
        const optimisticData = options.optimisticUpdate(mutationVariables);
        setData(optimisticData);
      } catch (err) {
        console.error('Error applying optimistic update:', err);
      }
    }
    
    setStatus('loading');
    
    try {
      const result = await mutationFn(mutationVariables);
      
      if (mountedRef.current) {
        setData(result);
        setError(null);
        setStatus('success');
        retryCountRef.current = 0;
        options.onSuccess?.(result, mutationVariables);
        options.onSettled?.(result, null, mutationVariables);
      }
      
      return result;
    } catch (err) {
      if (!mountedRef.current) {
        throw err;
      }
      
      const apiError = err instanceof ApiClientError 
        ? err 
        : new ApiClientError('unknown', 'Unknown error');
      
      // Handle retry logic
      if (shouldRetry && retryCountRef.current < maxRetries) {
        retryCountRef.current++;
        
        // Clear any existing retry timer
        if (retryTimerRef.current) {
          clearTimeout(retryTimerRef.current);
        }
        
        // Set up retry with delay
        const delay = getRetryDelay(retryCountRef.current);
        
        return new Promise<T>((resolve, reject) => {
          retryTimerRef.current = setTimeout(() => {
            if (mountedRef.current) {
              mutate(mutationVariables).then(resolve).catch(reject);
            } else {
              reject(apiError);
            }
          }, delay);
        });
      }
      
      // If we've exhausted retries or shouldn't retry, set error state
      setError(apiError);
      setStatus('error');
      options.onError?.(apiError, mutationVariables);
      options.onSettled?.(null, apiError, mutationVariables);
      
      throw apiError;
    }
  }, [mutationFn, options, shouldRetry, maxRetries, getRetryDelay]);

  // Function to reset the mutation state
  const reset = useCallback(() => {
    setData(null);
    setError(null);
    setStatus('idle');
    setVariables(null);
    retryCountRef.current = 0;
    
    // Clear any timers
    if (retryTimerRef.current) {
      clearTimeout(retryTimerRef.current);
    }
  }, []);

  // Effect to clean up on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      
      if (retryTimerRef.current) {
        clearTimeout(retryTimerRef.current);
      }
    };
  }, []);

  // Derived states
  const isLoading = status === 'loading';
  const isError = status === 'error';
  const isSuccess = status === 'success';

  return {
    mutate,
    data,
    error,
    isLoading,
    isError,
    isSuccess,
    status,
    reset,
    variables
  };
}