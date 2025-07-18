import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useBaseQuery, useBaseListQuery, useBaseMutation } from '@/lib/api/baseHooks';
import { mockInvoke, mockInvokeList, resetApiClientMocks } from '../../utils/mockApiClient';
import { 
  waitForLoading, 
  waitForSuccess, 
  waitForError, 
  waitForData,
  waitForListItems,
  waitForMoreItems,
  useMockTimer,
  advanceTimersByTime,
  restoreTimer
} from '../../utils/hookTestUtils';
import { ApiClientError } from '@/lib/api/client';

describe('useBaseQuery', () => {
  beforeEach(() => {
    resetApiClientMocks();
  });

  it('should return data on successful query', async () => {
    // Mock the API response
    mockInvoke({ data: { id: '1', name: 'Test Item' } });
    
    // Render the hook
    const { result } = renderHook(() => 
      useBaseQuery(() => Promise.resolve({ id: '1', name: 'Test Item' }))
    );
    
    // Initially, the hook should be in loading state
    expect(result.current.isLoading).toBe(true);
    
    // Wait for the query to succeed
    await waitForSuccess(result);
    
    // Verify the data
    expect(result.current.data).toEqual({ id: '1', name: 'Test Item' });
    expect(result.current.isLoading).toBe(false);
    expect(result.current.isSuccess).toBe(true);
    expect(result.current.isError).toBe(false);
    expect(result.current.error).toBeNull();
  });
  
  it('should handle errors', async () => {
    // Mock an API error
    mockInvoke({ 
      success: false, 
      error: 'Something went wrong', 
      throwError: true,
      customError: new ApiClientError('test_error', 'Test error message')
    });
    
    // Render the hook with a query that will fail
    const { result } = renderHook(() => 
      useBaseQuery(() => Promise.reject(new ApiClientError('test_error', 'Test error message')))
    );
    
    // Wait for the query to fail
    await waitForError(result);
    
    // Verify the error state
    expect(result.current.isError).toBe(true);
    expect(result.current.isSuccess).toBe(false);
    expect(result.current.data).toBeNull();
    expect(result.current.error).not.toBeNull();
    expect(result.current.error?.code).toBe('test_error');
    expect(result.current.error?.message).toBe('Test error message');
  });
  
  it('should refetch data when dependencies change', async () => {
    // Mock the API responses
    mockInvoke({ data: { id: '1', name: 'First Item' } });
    
    // Set up a dependency
    let dependency = 1;
    
    // Render the hook with the dependency
    const { result, rerender } = renderHook(() => 
      useBaseQuery(() => Promise.resolve({ id: String(dependency), name: `Item ${dependency}` }), [dependency])
    );
    
    // Wait for the first query to succeed
    await waitForSuccess(result);
    
    // Verify the initial data
    expect(result.current.data).toEqual({ id: '1', name: 'Item 1' });
    
    // Mock the second API response
    mockInvoke({ data: { id: '2', name: 'Second Item' } });
    
    // Change the dependency and rerender
    dependency = 2;
    rerender();
    
    // Wait for the second query to succeed
    await waitForSuccess(result);
    
    // Verify the updated data
    expect(result.current.data).toEqual({ id: '2', name: 'Item 2' });
  });
  
  it('should support refetch interval', async () => {
    // Set up mock timers
    useMockTimer();
    
    // Mock the API responses
    mockInvoke({ data: { id: '1', count: 1 } });
    
    // Render the hook with a refetch interval
    const { result } = renderHook(() => 
      useBaseQuery(
        () => Promise.resolve({ id: '1', count: Math.random() }), 
        [], 
        { refetchInterval: 1000 }
      )
    );
    
    // Wait for the first query to succeed
    await waitForSuccess(result);
    
    // Store the initial data
    const initialData = result.current.data;
    
    // Mock the second API response
    mockInvoke({ data: { id: '1', count: 2 } });
    
    // Advance the timer to trigger the refetch
    await advanceTimersByTime(1000);
    
    // Wait for the refetch to complete
    await waitFor(() => {
      expect(result.current.data).not.toEqual(initialData);
    });
    
    // Verify the updated data
    expect(result.current.data).toEqual({ id: '1', count: 2 });
    
    // Clean up
    restoreTimer();
  });
});

describe('useBaseListQuery', () => {
  beforeEach(() => {
    resetApiClientMocks();
  });
  
  it('should handle pagination', async () => {
    // Mock the initial list response
    const initialItems = [{ id: '1', name: 'Item 1' }, { id: '2', name: 'Item 2' }];
    mockInvokeList(initialItems, 4); // 4 total items, 2 per page
    
    // Render the hook
    const { result } = renderHook(() => 
      useBaseListQuery(
        (offset, limit) => Promise.resolve({ 
          items: initialItems, 
          total: 4, 
          limit, 
          offset 
        }),
        [],
        { pageSize: 2 }
      )
    );
    
    // Wait for the initial query to succeed
    await waitForListItems(result, 2);
    
    // Verify the initial state
    expect(result.current.data).toEqual(initialItems);
    expect(result.current.total).toBe(4);
    expect(result.current.hasMore).toBe(true);
    
    // Mock the next page response
    const nextPageItems = [{ id: '3', name: 'Item 3' }, { id: '4', name: 'Item 4' }];
    mockInvokeList(nextPageItems, 4);
    
    // Load more items
    await result.current.loadMore();
    
    // Wait for the next page to load
    await waitForListItems(result, 4);
    
    // Verify the updated state
    expect(result.current.data).toEqual([...initialItems, ...nextPageItems]);
    expect(result.current.hasMore).toBe(false);
  });
});

describe('useBaseMutation', () => {
  beforeEach(() => {
    resetApiClientMocks();
  });
  
  it('should handle successful mutations', async () => {
    // Mock the API response
    mockInvoke({ data: { id: '1', name: 'Updated Item' } });
    
    // Render the hook
    const { result } = renderHook(() => 
      useBaseMutation((variables: { id: string; name: string }) => 
        Promise.resolve({ id: variables.id, name: variables.name })
      )
    );
    
    // Verify initial state
    expect(result.current.isLoading).toBe(false);
    expect(result.current.data).toBeNull();
    
    // Perform the mutation
    let mutationResult;
    await waitFor(async () => {
      mutationResult = await result.current.mutate({ id: '1', name: 'Updated Item' });
    });
    
    // Verify the mutation result
    expect(mutationResult).toEqual({ id: '1', name: 'Updated Item' });
    expect(result.current.data).toEqual({ id: '1', name: 'Updated Item' });
    expect(result.current.isSuccess).toBe(true);
    expect(result.current.isError).toBe(false);
  });
  
  it('should handle optimistic updates', async () => {
    // Mock the API response (with a delay to test optimistic updates)
    mockInvoke({ 
      data: { id: '1', name: 'Server Updated Item' },
      delay: 100
    });
    
    // Render the hook with optimistic update
    const { result } = renderHook(() => 
      useBaseMutation(
        (variables: { id: string; name: string }) => 
          new Promise(resolve => 
            setTimeout(() => resolve({ id: variables.id, name: 'Server Updated Item' }), 100)
          ),
        {
          optimisticUpdate: (variables) => ({ id: variables.id, name: 'Optimistic Updated Item' })
        }
      )
    );
    
    // Perform the mutation
    const mutationPromise = result.current.mutate({ id: '1', name: 'Updated Item' });
    
    // Immediately after mutation starts, we should have the optimistic data
    expect(result.current.data).toEqual({ id: '1', name: 'Optimistic Updated Item' });
    
    // Wait for the mutation to complete
    await mutationPromise;
    
    // After mutation completes, we should have the server data
    expect(result.current.data).toEqual({ id: '1', name: 'Server Updated Item' });
  });
});