import { useState, useEffect, useCallback, useRef } from 'react';
import { useBaseQuery, UseBaseQueryOptions, UseBaseQueryResult } from './baseHooks';
import { ApiClientError } from './client';
import type { ListResponse } from './types';

/**
 * Options for the useFieldProgressiveQuery hook
 */
export interface UseFieldProgressiveQueryOptions<T> extends UseBaseQueryOptions<T> {
  /** Field sets to fetch in order (essential fields first, then additional details) */
  fieldSets: string[][];
  /** Delay between fetching field sets (in milliseconds) */
  fieldSetDelay?: number;
}

/**
 * Result of the useFieldProgressiveQuery hook
 */
export interface UseFieldProgressiveQueryResult<T> extends UseBaseQueryResult<T> {
  /** Current field set being fetched */
  currentFieldSet: number;
  /** Whether all field sets have been fetched */
  isComplete: boolean;
  /** Function to fetch the next field set */
  fetchNextFieldSet: () => void;
  /** Fields that have been fetched so far */
  fetchedFields: string[];
}

/**
 * Hook that progressively fetches data field by field, starting with essential fields
 * 
 * @param queryFn - Function that returns a promise with the query result for a given set of fields
 * @param dependencies - Dependencies array for the query function
 * @param options - Options for the query
 * @returns Query result with field-level progressive fetching
 */
export function useFieldProgressiveQuery<T>(
  queryFn: (fields: string[]) => Promise<T>,
  dependencies: any[] = [],
  options: UseFieldProgressiveQueryOptions<T>
): UseFieldProgressiveQueryResult<T> {
  const [data, setData] = useState<T | null>(null);
  const [currentFieldSet, setCurrentFieldSet] = useState(0);
  const [fetchedFields, setFetchedFields] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<ApiClientError | null>(null);
  
  const fieldSetDelay = options.fieldSetDelay || 0;
  const timerRef = useRef<NodeJS.Timeout>();
  const mountedRef = useRef(true);

  // Function to fetch a specific field set
  const fetchFieldSet = useCallback(async (fieldSetIndex: number) => {
    if (!mountedRef.current || fieldSetIndex >= options.fieldSets.length) return;
    
    setIsLoading(true);
    
    try {
      // Get all fields up to and including the current field set
      const fieldsToFetch = options.fieldSets.slice(0, fieldSetIndex + 1).flat();
      
      const result = await queryFn(fieldsToFetch);
      
      if (mountedRef.current) {
        setData(result);
        setFetchedFields(fieldsToFetch);
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
        
        // Schedule next field set if not at max
        if (fieldSetIndex < options.fieldSets.length - 1 && fieldSetDelay > 0) {
          timerRef.current = setTimeout(() => {
            if (mountedRef.current) {
              setCurrentFieldSet(fieldSetIndex + 1);
            }
          }, fieldSetDelay);
        }
      }
    }
  }, [queryFn, options.fieldSets, fieldSetDelay, options.onError]);

  // Function to manually fetch the next field set
  const fetchNextFieldSet = useCallback(() => {
    if (currentFieldSet < options.fieldSets.length) {
      fetchFieldSet(currentFieldSet);
    }
  }, [currentFieldSet, options.fieldSets.length, fetchFieldSet]);

  // Effect to fetch the first field set when enabled
  useEffect(() => {
    if (options.enabled !== false) {
      fetchFieldSet(0);
    }
    
    return () => {
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, [...dependencies, options.enabled]);

  // Effect to fetch each field set as currentFieldSet changes
  useEffect(() => {
    if (currentFieldSet > 0 && currentFieldSet < options.fieldSets.length) {
      fetchFieldSet(currentFieldSet);
    }
  }, [currentFieldSet, options.fieldSets.length, fetchFieldSet]);

  // Effect to clean up on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      
      if (timerRef.current) {
        clearTimeout(timerRef.current);
      }
    };
  }, []);

  const isComplete = currentFieldSet >= options.fieldSets.length - 1;
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
      setCurrentFieldSet(0);
      setData(null);
      setFetchedFields([]);
      return fetchFieldSet(0).then(() => data);
    },
    reset: () => {
      setCurrentFieldSet(0);
      setData(null);
      setFetchedFields([]);
      setError(null);
      setIsLoading(false);
    },
    currentFieldSet,
    isComplete,
    fetchNextFieldSet,
    fetchedFields,
  };
}

/**
 * Options for the useCursorPaginationQuery hook
 */
export interface UseCursorPaginationQueryOptions<T, TCursor> extends UseBaseQueryOptions<T[]> {
  /** Initial cursor to start fetching from */
  initialCursor?: TCursor;
  /** Number of items to fetch per page */
  pageSize?: number;
  /** Whether to keep previous data when fetching more */
  keepPreviousData?: boolean;
}

/**
 * Result of the useCursorPaginationQuery hook
 */
export interface UseCursorPaginationQueryResult<T, TCursor> extends UseBaseQueryResult<T[]> {
  /** Whether there are more items to load */
  hasNextPage: boolean;
  /** Function to load the next page of data */
  fetchNextPage: () => Promise<T[] | null>;
  /** Current cursor */
  cursor: TCursor | null;
  /** Next cursor (if available) */
  nextCursor: TCursor | null;
  /** Current page size */
  pageSize: number;
}

/**
 * Hook that implements cursor-based pagination for efficient data loading
 * 
 * @param queryFn - Function that returns a promise with the paginated query result and next cursor
 * @param dependencies - Dependencies array for the query function
 * @param options - Options for the query
 * @returns Query result with cursor-based pagination
 */
export function useCursorPaginationQuery<T, TCursor = string>(
  queryFn: (cursor: TCursor | null, limit: number) => Promise<{ items: T[], nextCursor: TCursor | null }>,
  dependencies: any[] = [],
  options: UseCursorPaginationQueryOptions<T, TCursor> = {}
): UseCursorPaginationQueryResult<T, TCursor> {
  // State for pagination
  const [cursor, setCursor] = useState<TCursor | null>(options.initialCursor || null);
  const [nextCursor, setNextCursor] = useState<TCursor | null>(null);
  const pageSize = options.pageSize || 20;
  
  // Refs for tracking state between renders
  const dataRef = useRef<T[]>([]);
  const isFetchingMoreRef = useRef(false);

  // Function to fetch data with pagination
  const fetchWithPagination = useCallback(async (resetData = true): Promise<T[]> => {
    const currentCursor = resetData ? (options.initialCursor || null) : cursor;
    
    try {
      isFetchingMoreRef.current = !resetData;
      const result = await queryFn(currentCursor, pageSize);
      
      let newData: T[];
      if (resetData) {
        newData = result.items;
        setCursor(result.nextCursor);
      } else {
        newData = [...dataRef.current, ...result.items];
        setCursor(result.nextCursor);
      }
      
      dataRef.current = newData;
      setNextCursor(result.nextCursor);
      
      return newData;
    } finally {
      isFetchingMoreRef.current = false;
    }
  }, [...dependencies, cursor, pageSize, options.initialCursor]);

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
  const fetchNextPage = useCallback(async (): Promise<T[] | null> => {
    if (queryResult.isLoading || isFetchingMoreRef.current || !queryResult.isSuccess || !nextCursor) {
      return null;
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
  }, [fetchWithPagination, queryResult, nextCursor]);

  // Function to refetch data
  const refetch = useCallback(async (): Promise<T[] | null> => {
    setCursor(options.initialCursor || null);
    return queryResult.refetch();
  }, [queryResult, options.initialCursor]);

  // Calculate if there are more items to load
  const hasNextPage = nextCursor !== null;

  return {
    ...queryResult,
    refetch,
    hasNextPage,
    fetchNextPage,
    cursor,
    nextCursor,
    pageSize,
  };
}

/**
 * Options for the useStreamingQuery hook
 */
export interface UseStreamingQueryOptions<T> extends UseBaseQueryOptions<T[]> {
  /** Callback for each chunk of data received */
  onChunk?: (chunk: T) => void;
}

/**
 * Result of the useStreamingQuery hook
 */
export interface UseStreamingQueryResult<T> extends UseBaseQueryResult<T[]> {
  /** Whether the query is currently streaming */
  isStreaming: boolean;
  /** Number of chunks received so far */
  chunkCount: number;
}

/**
 * Hook that implements streaming responses for large datasets
 * 
 * @param streamFn - Function that returns an async iterable of data chunks
 * @param dependencies - Dependencies array for the stream function
 * @param options - Options for the query
 * @returns Query result with streaming support
 */
export function useStreamingQuery<T>(
  streamFn: () => AsyncIterable<T>,
  dependencies: any[] = [],
  options: UseStreamingQueryOptions<T> = {}
): UseStreamingQueryResult<T> {
  const [data, setData] = useState<T[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [chunkCount, setChunkCount] = useState(0);
  const [error, setError] = useState<ApiClientError | null>(null);
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  
  const mountedRef = useRef(true);
  const abortControllerRef = useRef<AbortController | null>(null);

  // Function to start streaming data
  const startStreaming = useCallback(async () => {
    if (!mountedRef.current) return;
    
    setIsStreaming(true);
    setStatus('loading');
    setData([]);
    setChunkCount(0);
    
    // Create a new AbortController for this stream
    abortControllerRef.current = new AbortController();
    
    try {
      const chunks: T[] = [];
      
      // Process the stream
      for await (const chunk of streamFn()) {
        if (!mountedRef.current || abortControllerRef.current?.signal.aborted) {
          break;
        }
        
        chunks.push(chunk);
        setData([...chunks]);
        setChunkCount(prev => prev + 1);
        
        // Call the onChunk callback if provided
        options.onChunk?.(chunk);
      }
      
      if (mountedRef.current) {
        setStatus('success');
        options.onSuccess?.(chunks);
        options.onSettled?.(chunks, null);
      }
    } catch (err) {
      if (!mountedRef.current) return;
      
      const apiError = err instanceof ApiClientError 
        ? err 
        : new ApiClientError('unknown', 'Unknown error');
      
      setError(apiError);
      setStatus('error');
      options.onError?.(apiError);
      options.onSettled?.(null, apiError);
    } finally {
      if (mountedRef.current) {
        setIsStreaming(false);
      }
    }
  }, [streamFn, options]);

  // Effect to start streaming when enabled
  useEffect(() => {
    if (options.enabled !== false) {
      startStreaming();
    }
    
    return () => {
      // Abort any ongoing stream when dependencies change
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [...dependencies, options.enabled]);

  // Effect to clean up on unmount
  useEffect(() => {
    return () => {
      mountedRef.current = false;
      
      // Abort any ongoing stream when unmounting
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, []);

  return {
    data,
    error,
    isLoading: status === 'loading',
    isFetching: isStreaming,
    isError: status === 'error',
    isSuccess: status === 'success',
    status,
    refetch: () => {
      // Abort any ongoing stream
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      
      return startStreaming().then(() => data);
    },
    reset: () => {
      // Abort any ongoing stream
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      
      setData([]);
      setError(null);
      setStatus('idle');
      setIsStreaming(false);
      setChunkCount(0);
    },
    isStreaming,
    chunkCount,
  };
}