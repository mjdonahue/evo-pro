import { useState, useEffect, useMemo, useCallback } from 'react';
import { 
  UseBaseQueryResult, 
  UseBaseListQueryResult, 
  UseBaseMutationResult,
  useBaseQuery,
  UseBaseQueryOptions
} from './baseHooks';
import { ApiClientError } from './client';

/**
 * Options for the useCombinedQueries hook
 */
export interface UseCombinedQueriesOptions {
  /** Whether all queries should be enabled */
  enabled?: boolean;
}

/**
 * Result of the useCombinedQueries hook
 */
export interface UseCombinedQueriesResult<T extends any[]> {
  /** Combined data from all queries */
  data: { [K in keyof T]: T[K] extends UseBaseQueryResult<infer U> ? U : never } | null;
  /** Whether any query is loading */
  isLoading: boolean;
  /** Whether any query is fetching */
  isFetching: boolean;
  /** Whether any query has errored */
  isError: boolean;
  /** Whether all queries were successful */
  isSuccess: boolean;
  /** The first error encountered, if any */
  error: ApiClientError | null;
  /** Function to refetch all queries */
  refetch: () => Promise<void>;
}

/**
 * Combines multiple queries into a single result
 * 
 * This hook allows you to combine the results of multiple queries into a single result object.
 * It tracks the loading, error, and success states across all queries and provides a unified interface.
 * 
 * @param queries - Array of query results to combine
 * @param options - Options for the combined queries
 * @returns Combined query result
 * 
 * @example
 * ```tsx
 * const user = useUser(userId);
 * const posts = usePosts({ userId });
 * const comments = useComments({ userId });
 * 
 * const result = useCombinedQueries([user, posts, comments]);
 * 
 * if (result.isLoading) {
 *   return <Loading />;
 * }
 * 
 * if (result.isError) {
 *   return <Error error={result.error} />;
 * }
 * 
 * const [userData, postsData, commentsData] = result.data;
 * ```
 */
export function useCombinedQueries<T extends UseBaseQueryResult<any>[]>(
  queries: [...T],
  options: UseCombinedQueriesOptions = {}
): UseCombinedQueriesResult<T> {
  // Check if any query is loading, fetching, or has errored
  const isLoading = queries.some(query => query.isLoading);
  const isFetching = queries.some(query => query.isFetching);
  const isError = queries.some(query => query.isError);
  const isSuccess = queries.every(query => query.isSuccess);
  
  // Get the first error, if any
  const error = useMemo(() => {
    const errorQuery = queries.find(query => query.error);
    return errorQuery ? errorQuery.error : null;
  }, [queries]);
  
  // Combine the data from all queries
  const data = useMemo(() => {
    if (isSuccess) {
      return queries.map(query => query.data) as { [K in keyof T]: T[K] extends UseBaseQueryResult<infer U> ? U : never };
    }
    return null;
  }, [queries, isSuccess]);
  
  // Function to refetch all queries
  const refetch = useCallback(async () => {
    await Promise.all(queries.map(query => query.refetch()));
  }, [queries]);
  
  return {
    data,
    isLoading,
    isFetching,
    isError,
    isSuccess,
    error,
    refetch
  };
}

/**
 * Options for the useDependentQuery hook
 */
export interface UseDependentQueryOptions<TDependency, TData> extends Omit<UseBaseQueryOptions<TData>, 'enabled'> {
  /** Whether the dependent query should be enabled */
  enabled?: boolean;
}

/**
 * Creates a query that depends on data from another query
 * 
 * This hook allows you to create a query that depends on the result of another query.
 * The dependent query will only execute when the dependency query has successfully loaded data.
 * 
 * @param dependency - The query result that this query depends on
 * @param queryFn - Function that returns a promise with the query result, using the dependency data
 * @param options - Options for the dependent query
 * @returns Query result
 * 
 * @example
 * ```tsx
 * const user = useUser(userId);
 * const posts = useDependentQuery(
 *   user,
 *   (userData) => api.posts.getByAuthor(userData.id),
 *   { onSuccess: (posts) => console.log(`Loaded ${posts.length} posts`) }
 * );
 * ```
 */
export function useDependentQuery<TDependency, TData>(
  dependency: UseBaseQueryResult<TDependency>,
  queryFn: (data: TDependency) => Promise<TData>,
  options: UseDependentQueryOptions<TDependency, TData> = {}
): UseBaseQueryResult<TData> {
  // Only enable the query if the dependency has successfully loaded
  const enabled = dependency.isSuccess && dependency.data !== null && options.enabled !== false;
  
  // Create the dependent query
  return useBaseQuery<TData>(
    () => {
      if (!dependency.data) {
        return Promise.reject(new Error('Dependency data is not available'));
      }
      return queryFn(dependency.data);
    },
    [dependency.data],
    {
      ...options,
      enabled
    }
  );
}

/**
 * Options for the useTransformedQuery hook
 */
export interface UseTransformedQueryOptions<TSourceData, TTransformedData> 
  extends Omit<UseBaseQueryOptions<TTransformedData>, 'initialData'> {
  /** Initial data for the transformed result */
  initialData?: TTransformedData | (() => TTransformedData);
  /** Whether to keep the previous transformed data when the source data changes */
  keepPreviousData?: boolean;
}

/**
 * Transforms the result of a query
 * 
 * This hook allows you to transform the result of a query into a different shape.
 * It's useful for data aggregation, filtering, or any other transformation.
 * 
 * @param sourceQuery - The query result to transform
 * @param transformFn - Function that transforms the source data
 * @param options - Options for the transformed query
 * @returns Transformed query result
 * 
 * @example
 * ```tsx
 * const posts = usePosts({ userId });
 * const postStats = useTransformedQuery(
 *   posts,
 *   (postsData) => ({
 *     total: postsData.length,
 *     published: postsData.filter(p => p.status === 'published').length,
 *     draft: postsData.filter(p => p.status === 'draft').length,
 *     averageLength: postsData.reduce((sum, p) => sum + p.content.length, 0) / postsData.length
 *   })
 * );
 * ```
 */
export function useTransformedQuery<TSourceData, TTransformedData>(
  sourceQuery: UseBaseQueryResult<TSourceData>,
  transformFn: (data: TSourceData) => TTransformedData,
  options: UseTransformedQueryOptions<TSourceData, TTransformedData> = {}
): UseBaseQueryResult<TTransformedData> {
  // State for the transformed data
  const [transformedData, setTransformedData] = useState<TTransformedData | null>(() => {
    if (options.initialData) {
      return typeof options.initialData === 'function'
        ? (options.initialData as () => TTransformedData)()
        : options.initialData;
    }
    return null;
  });
  
  // State for tracking previous source data
  const [previousSourceData, setPreviousSourceData] = useState<TSourceData | null>(null);
  
  // Transform the data when the source query data changes
  useEffect(() => {
    if (sourceQuery.data) {
      try {
        const newTransformedData = transformFn(sourceQuery.data);
        setTransformedData(newTransformedData);
        setPreviousSourceData(sourceQuery.data);
      } catch (err) {
        console.error('Error transforming data:', err);
      }
    } else if (!options.keepPreviousData || !previousSourceData) {
      setTransformedData(null);
    }
  }, [sourceQuery.data, transformFn, options.keepPreviousData, previousSourceData]);
  
  // Create a result object that mimics UseBaseQueryResult
  return {
    data: transformedData,
    error: sourceQuery.error,
    isLoading: sourceQuery.isLoading,
    isFetching: sourceQuery.isFetching,
    isError: sourceQuery.isError,
    isSuccess: sourceQuery.isSuccess && transformedData !== null,
    status: transformedData !== null && sourceQuery.isSuccess ? 'success' : sourceQuery.status,
    refetch: sourceQuery.refetch,
    reset: sourceQuery.reset
  };
}

/**
 * Options for the usePaginatedQuery hook
 */
export interface UsePaginatedQueryOptions<TItem, TFilter> {
  /** Initial filter to use */
  initialFilter?: TFilter;
  /** Page size */
  pageSize?: number;
  /** Whether the query should execute automatically */
  enabled?: boolean;
  /** Callback when query succeeds */
  onSuccess?: (data: TItem[]) => void;
  /** Callback when query fails */
  onError?: (error: ApiClientError) => void;
}

/**
 * Result of the usePaginatedQuery hook
 */
export interface UsePaginatedQueryResult<TItem, TFilter> {
  /** The query data */
  data: TItem[];
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
  /** Total number of items available */
  total: number;
  /** Whether there are more items to load */
  hasMore: boolean;
  /** Current page */
  page: number;
  /** Current filter */
  filter: TFilter;
  /** Function to go to a specific page */
  goToPage: (page: number) => Promise<void>;
  /** Function to go to the next page */
  nextPage: () => Promise<void>;
  /** Function to go to the previous page */
  prevPage: () => Promise<void>;
  /** Function to update the filter */
  setFilter: (filter: TFilter) => void;
  /** Function to manually refetch data */
  refetch: () => Promise<void>;
}

/**
 * Creates a paginated query with filter support
 * 
 * This hook provides a higher-level abstraction for paginated queries with filtering.
 * It handles pagination state, filter changes, and provides convenient navigation methods.
 * 
 * @param queryFn - Function that returns a promise with the paginated query result
 * @param options - Options for the paginated query
 * @returns Paginated query result
 * 
 * @example
 * ```tsx
 * const { 
 *   data: users,
 *   isLoading,
 *   page,
 *   nextPage,
 *   prevPage,
 *   setFilter,
 *   filter
 * } = usePaginatedQuery(
 *   (filter, page, pageSize) => api.users.list({ ...filter, offset: page * pageSize, limit: pageSize }),
 *   { initialFilter: { status: 'active' }, pageSize: 10 }
 * );
 * ```
 */
export function usePaginatedQuery<TItem, TFilter = Record<string, any>>(
  queryFn: (filter: TFilter, page: number, pageSize: number) => Promise<{ items: TItem[], total: number }>,
  options: UsePaginatedQueryOptions<TItem, TFilter> = {}
): UsePaginatedQueryResult<TItem, TFilter> {
  // State for pagination and filtering
  const [page, setPage] = useState(0);
  const [filter, setFilter] = useState<TFilter>(options.initialFilter || {} as TFilter);
  const [data, setData] = useState<TItem[]>([]);
  const [total, setTotal] = useState(0);
  const [error, setError] = useState<ApiClientError | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isFetching, setIsFetching] = useState(false);
  const pageSize = options.pageSize || 20;
  
  // Function to fetch data
  const fetchData = useCallback(async (currentPage: number, currentFilter: TFilter): Promise<void> => {
    try {
      setIsFetching(true);
      if (currentPage === 0) {
        setIsLoading(true);
      }
      
      const result = await queryFn(currentFilter, currentPage, pageSize);
      
      setData(result.items);
      setTotal(result.total);
      setError(null);
      options.onSuccess?.(result.items);
    } catch (err) {
      const apiError = err instanceof ApiClientError 
        ? err 
        : new ApiClientError('unknown', 'Unknown error');
      
      setError(apiError);
      options.onError?.(apiError);
    } finally {
      setIsLoading(false);
      setIsFetching(false);
    }
  }, [queryFn, pageSize, options.onSuccess, options.onError]);
  
  // Effect to fetch data when page or filter changes
  useEffect(() => {
    if (options.enabled !== false) {
      fetchData(page, filter).catch(() => {
        // Catch error to prevent unhandled promise rejection
      });
    }
  }, [page, filter, fetchData, options.enabled]);
  
  // Navigation functions
  const goToPage = useCallback(async (newPage: number): Promise<void> => {
    if (newPage >= 0 && newPage * pageSize < total) {
      setPage(newPage);
    }
  }, [pageSize, total]);
  
  const nextPage = useCallback(async (): Promise<void> => {
    if ((page + 1) * pageSize < total) {
      setPage(p => p + 1);
    }
  }, [page, pageSize, total]);
  
  const prevPage = useCallback(async (): Promise<void> => {
    if (page > 0) {
      setPage(p => p - 1);
    }
  }, [page]);
  
  // Function to update the filter and reset to page 0
  const updateFilter = useCallback((newFilter: TFilter): void => {
    setFilter(newFilter);
    setPage(0);
  }, []);
  
  // Function to manually refetch data
  const refetch = useCallback(async (): Promise<void> => {
    return fetchData(page, filter);
  }, [fetchData, page, filter]);
  
  // Calculate derived states
  const isError = error !== null;
  const isSuccess = !isLoading && !isError;
  const hasMore = (page + 1) * pageSize < total;
  
  return {
    data,
    error,
    isLoading,
    isFetching,
    isError,
    isSuccess,
    total,
    hasMore,
    page,
    filter,
    goToPage,
    nextPage,
    prevPage,
    setFilter: updateFilter,
    refetch
  };
}

/**
 * Options for the useOptimisticUpdate hook
 */
export interface UseOptimisticUpdateOptions<TData, TVariables> {
  /** Function to get the current data */
  getCurrentData: () => TData | null;
  /** Function to update the data optimistically */
  optimisticUpdate: (currentData: TData, variables: TVariables) => TData;
  /** Function to update the data after the mutation succeeds */
  onMutationSuccess?: (newData: TData, variables: TVariables) => void;
  /** Function to handle errors */
  onMutationError?: (error: ApiClientError, variables: TVariables) => void;
}

/**
 * Result of the useOptimisticUpdate hook
 */
export interface UseOptimisticUpdateResult<TData, TVariables> {
  /** Function to perform the optimistic update */
  mutate: (variables: TVariables) => Promise<TData>;
  /** Whether the mutation is currently running */
  isLoading: boolean;
  /** Whether the mutation has errored */
  isError: boolean;
  /** The mutation error */
  error: ApiClientError | null;
}

/**
 * Creates an optimistic update for a mutation
 * 
 * This hook provides a higher-level abstraction for optimistic updates.
 * It handles updating the data optimistically before the mutation completes,
 * and then updates it again with the actual result.
 * 
 * @param mutationFn - Function that performs the mutation
 * @param options - Options for the optimistic update
 * @returns Optimistic update result
 * 
 * @example
 * ```tsx
 * const { data: todos, refetch } = useTodos();
 * 
 * const { mutate: toggleTodo } = useOptimisticUpdate(
 *   (id) => api.todos.toggle(id),
 *   {
 *     getCurrentData: () => todos,
 *     optimisticUpdate: (currentTodos, id) => 
 *       currentTodos.map(todo => 
 *         todo.id === id ? { ...todo, completed: !todo.completed } : todo
 *       ),
 *     onMutationSuccess: () => refetch()
 *   }
 * );
 * ```
 */
export function useOptimisticUpdate<TData, TVariables, TMutationResult = any>(
  mutationFn: (variables: TVariables) => Promise<TMutationResult>,
  options: UseOptimisticUpdateOptions<TData, TVariables>
): UseOptimisticUpdateResult<TData, TVariables> {
  // State for the mutation
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<ApiClientError | null>(null);
  
  // Function to perform the optimistic update
  const mutate = useCallback(async (variables: TVariables): Promise<TData> => {
    const currentData = options.getCurrentData();
    
    if (!currentData) {
      throw new Error('Current data is not available for optimistic update');
    }
    
    // Apply optimistic update
    const optimisticData = options.optimisticUpdate(currentData, variables);
    
    try {
      setIsLoading(true);
      setError(null);
      
      // Perform the actual mutation
      await mutationFn(variables);
      
      // Call onMutationSuccess if provided
      options.onMutationSuccess?.(optimisticData, variables);
      
      return optimisticData;
    } catch (err) {
      const apiError = err instanceof ApiClientError 
        ? err 
        : new ApiClientError('unknown', 'Unknown error');
      
      setError(apiError);
      options.onMutationError?.(apiError, variables);
      
      throw apiError;
    } finally {
      setIsLoading(false);
    }
  }, [mutationFn, options]);
  
  return {
    mutate,
    isLoading,
    isError: error !== null,
    error
  };
}

/**
 * Options for the useCachedQuery hook
 */
export interface UseCachedQueryOptions<TData> extends UseBaseQueryOptions<TData> {
  /** Cache key */
  cacheKey: string;
  /** Time to live in milliseconds */
  ttl?: number;
}

/**
 * Creates a query with client-side caching
 * 
 * This hook provides client-side caching for queries.
 * It stores the query result in memory and returns it immediately on subsequent calls,
 * while still fetching fresh data in the background.
 * 
 * @param queryFn - Function that returns a promise with the query result
 * @param options - Options for the cached query
 * @returns Query result
 * 
 * @example
 * ```tsx
 * const { data: user } = useCachedQuery(
 *   () => api.users.get(userId),
 *   { cacheKey: `user-${userId}`, ttl: 60000 }
 * );
 * ```
 */
export function useCachedQuery<TData>(
  queryFn: () => Promise<TData>,
  options: UseCachedQueryOptions<TData>
): UseBaseQueryResult<TData> {
  // Use a static cache object shared across all instances
  const cache = useMemo(() => {
    if (typeof window !== 'undefined' && !window.__QUERY_CACHE__) {
      window.__QUERY_CACHE__ = {};
    }
    return typeof window !== 'undefined' ? window.__QUERY_CACHE__ : {};
  }, []);
  
  // Get cached data if available
  const cachedData = useMemo(() => {
    const cached = cache[options.cacheKey];
    if (cached && (options.ttl === undefined || Date.now() - cached.timestamp < options.ttl)) {
      return cached.data;
    }
    return undefined;
  }, [cache, options.cacheKey, options.ttl]);
  
  // Use the base query hook with the cached data as initial data
  const result = useBaseQuery<TData>(
    queryFn,
    [options.cacheKey],
    {
      ...options,
      initialData: cachedData,
      onSuccess: (data) => {
        // Update the cache
        cache[options.cacheKey] = {
          data,
          timestamp: Date.now()
        };
        options.onSuccess?.(data);
      }
    }
  );
  
  return result;
}

// Add the cache type to the window object
declare global {
  interface Window {
    __QUERY_CACHE__?: Record<string, { data: any; timestamp: number }>;
  }
}