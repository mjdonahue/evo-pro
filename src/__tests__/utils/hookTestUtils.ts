import { renderHook, act, waitFor } from '@testing-library/react';
import { vi } from 'vitest';
import type { UseBaseQueryResult, UseBaseListQueryResult, UseBaseMutationResult } from '@/lib/api/baseHooks';
import React from 'react';

/**
 * Options for testing hooks
 */
export interface HookTestOptions {
  /** Timeout for waitFor in milliseconds */
  timeout?: number;
  /** Interval for waitFor in milliseconds */
  interval?: number;
}

/**
 * Default hook test options
 */
const defaultOptions: HookTestOptions = {
  timeout: 1000,
  interval: 50,
};

/**
 * Waits for a hook to be in a loading state
 */
export async function waitForLoading<T>(
  result: { current: UseBaseQueryResult<T> | UseBaseListQueryResult<any> | UseBaseMutationResult<any, any> },
  options: HookTestOptions = {}
): Promise<void> {
  const mergedOptions = { ...defaultOptions, ...options };
  
  await waitFor(
    () => expect(result.current.isLoading).toBe(true),
    { timeout: mergedOptions.timeout, interval: mergedOptions.interval }
  );
}

/**
 * Waits for a hook to be in a success state
 */
export async function waitForSuccess<T>(
  result: { current: UseBaseQueryResult<T> | UseBaseListQueryResult<any> | UseBaseMutationResult<any, any> },
  options: HookTestOptions = {}
): Promise<void> {
  const mergedOptions = { ...defaultOptions, ...options };
  
  await waitFor(
    () => expect(result.current.isSuccess).toBe(true),
    { timeout: mergedOptions.timeout, interval: mergedOptions.interval }
  );
}

/**
 * Waits for a hook to be in an error state
 */
export async function waitForError<T>(
  result: { current: UseBaseQueryResult<T> | UseBaseListQueryResult<any> | UseBaseMutationResult<any, any> },
  options: HookTestOptions = {}
): Promise<void> {
  const mergedOptions = { ...defaultOptions, ...options };
  
  await waitFor(
    () => expect(result.current.isError).toBe(true),
    { timeout: mergedOptions.timeout, interval: mergedOptions.interval }
  );
}

/**
 * Waits for a hook to have data
 */
export async function waitForData<T>(
  result: { current: UseBaseQueryResult<T> | UseBaseListQueryResult<any> | UseBaseMutationResult<any, any> },
  options: HookTestOptions = {}
): Promise<void> {
  const mergedOptions = { ...defaultOptions, ...options };
  
  await waitFor(
    () => expect(result.current.data).not.toBeNull(),
    { timeout: mergedOptions.timeout, interval: mergedOptions.interval }
  );
}

/**
 * Waits for a list query hook to have a specific number of items
 */
export async function waitForListItems<T>(
  result: { current: UseBaseListQueryResult<T> },
  expectedLength: number,
  options: HookTestOptions = {}
): Promise<void> {
  const mergedOptions = { ...defaultOptions, ...options };
  
  await waitFor(
    () => {
      expect(result.current.data).not.toBeNull();
      expect(result.current.data?.length).toBe(expectedLength);
    },
    { timeout: mergedOptions.timeout, interval: mergedOptions.interval }
  );
}

/**
 * Waits for a list query hook to have more items after loading more
 */
export async function waitForMoreItems<T>(
  result: { current: UseBaseListQueryResult<T> },
  initialLength: number,
  options: HookTestOptions = {}
): Promise<void> {
  const mergedOptions = { ...defaultOptions, ...options };
  
  await waitFor(
    () => {
      expect(result.current.data).not.toBeNull();
      expect(result.current.data?.length).toBeGreaterThan(initialLength);
    },
    { timeout: mergedOptions.timeout, interval: mergedOptions.interval }
  );
}

/**
 * Creates a mock timer for testing hooks with intervals or delays
 */
export function useMockTimer(): void {
  vi.useFakeTimers();
}

/**
 * Advances the mock timer by a specified amount of time
 */
export async function advanceTimersByTime(ms: number): Promise<void> {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
  
  // Allow any pending promises to resolve
  await Promise.resolve();
}

/**
 * Restores the real timer
 */
export function restoreTimer(): void {
  vi.useRealTimers();
}

/**
 * Creates a wrapper for testing hooks with context providers
 */
export function createWrapper(providers: React.FC<{ children: React.ReactNode }>[]) {
  return ({ children }: { children: React.ReactNode }) => {
    return providers.reduceRight(
      (acc, Provider) => React.createElement(Provider, {}, acc),
      React.createElement(React.Fragment, {}, children)
    );
  };
}

/**
 * Utility for testing optimistic updates
 */
export interface OptimisticUpdateTestUtils<T, TVariables> {
  /** The original data before the update */
  originalData: T;
  /** The optimistic data after the update */
  optimisticData: T;
  /** Function to perform the optimistic update */
  performUpdate: (variables: TVariables) => Promise<void>;
  /** Function to verify the optimistic update was applied */
  verifyOptimisticUpdate: () => void;
  /** Function to verify the final update was applied */
  verifyFinalUpdate: () => void;
}

/**
 * Creates utilities for testing optimistic updates
 */
export function createOptimisticUpdateTest<T, TVariables>(
  result: { current: UseBaseMutationResult<T, TVariables> },
  getCurrentData: () => T,
  optimisticUpdateFn: (data: T, variables: TVariables) => T,
  variables: TVariables
): OptimisticUpdateTestUtils<T, TVariables> {
  const originalData = getCurrentData();
  const optimisticData = optimisticUpdateFn(originalData, variables);
  
  return {
    originalData,
    optimisticData,
    performUpdate: async (vars) => {
      act(() => {
        result.current.mutate(vars);
      });
    },
    verifyOptimisticUpdate: () => {
      expect(getCurrentData()).toEqual(optimisticData);
    },
    verifyFinalUpdate: () => {
      expect(getCurrentData()).toEqual(result.current.data);
    }
  };
}