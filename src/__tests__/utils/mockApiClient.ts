import { vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { ApiClientError, apiClient } from '@/lib/api/client';
import type { ApiResponse, ListResponse } from '@/lib/api/types';

/**
 * Type for mock response configuration
 */
export interface MockResponseConfig<T = any> {
  /** The data to return */
  data?: T;
  /** Whether the response should be successful */
  success?: boolean;
  /** Error message if success is false */
  error?: string;
  /** Delay in milliseconds before resolving */
  delay?: number;
  /** Whether to throw an error instead of returning a response */
  throwError?: boolean;
  /** Custom error to throw */
  customError?: ApiClientError;
}

/**
 * Default mock response configuration
 */
const defaultConfig: MockResponseConfig = {
  success: true,
  delay: 0,
  throwError: false,
};

/**
 * Creates a mock response based on the provided configuration
 */
export function createMockResponse<T>(config: MockResponseConfig<T> = {}): ApiResponse<T> {
  const mergedConfig = { ...defaultConfig, ...config };
  
  return {
    data: mergedConfig.data as T,
    success: mergedConfig.success,
    error: mergedConfig.success ? undefined : mergedConfig.error,
  };
}

/**
 * Creates a mock list response based on the provided configuration
 */
export function createMockListResponse<T>(
  items: T[] = [],
  total: number = items.length,
  config: MockResponseConfig = {}
): ApiResponse<ListResponse<T>> {
  const mergedConfig = { ...defaultConfig, ...config };
  
  return {
    data: {
      items,
      total,
      limit: items.length,
      offset: 0,
    },
    success: mergedConfig.success,
    error: mergedConfig.success ? undefined : mergedConfig.error,
  };
}

/**
 * Mocks the Tauri invoke function to return a specific response
 */
export function mockInvoke<T>(config: MockResponseConfig<T> = {}): void {
  const mergedConfig = { ...defaultConfig, ...config };
  
  if (mergedConfig.throwError) {
    vi.mocked(invoke).mockRejectedValueOnce(
      mergedConfig.customError || new ApiClientError('mock_error', 'Mock error')
    );
    return;
  }
  
  const mockResponse = createMockResponse<T>(mergedConfig);
  
  if (mergedConfig.delay > 0) {
    vi.mocked(invoke).mockImplementationOnce(() => 
      new Promise((resolve) => {
        setTimeout(() => resolve(mockResponse), mergedConfig.delay);
      })
    );
  } else {
    vi.mocked(invoke).mockResolvedValueOnce(mockResponse);
  }
}

/**
 * Mocks the Tauri invoke function to return a list response
 */
export function mockInvokeList<T>(
  items: T[] = [],
  total: number = items.length,
  config: MockResponseConfig = {}
): void {
  const mergedConfig = { ...defaultConfig, ...config };
  
  if (mergedConfig.throwError) {
    vi.mocked(invoke).mockRejectedValueOnce(
      mergedConfig.customError || new ApiClientError('mock_error', 'Mock error')
    );
    return;
  }
  
  const mockResponse = createMockListResponse<T>(items, total, mergedConfig);
  
  if (mergedConfig.delay > 0) {
    vi.mocked(invoke).mockImplementationOnce(() => 
      new Promise((resolve) => {
        setTimeout(() => resolve(mockResponse), mergedConfig.delay);
      })
    );
  } else {
    vi.mocked(invoke).mockResolvedValueOnce(mockResponse);
  }
}

/**
 * Creates a spy on the apiClient to track method calls
 */
export function spyOnApiClient(): void {
  // Spy on conversation methods
  vi.spyOn(apiClient.conversations, 'list');
  vi.spyOn(apiClient.conversations, 'get');
  vi.spyOn(apiClient.conversations, 'create');
  vi.spyOn(apiClient.conversations, 'update');
  vi.spyOn(apiClient.conversations, 'delete');
  
  // Spy on message methods
  vi.spyOn(apiClient.messages, 'list');
  vi.spyOn(apiClient.messages, 'get');
  vi.spyOn(apiClient.messages, 'create');
  vi.spyOn(apiClient.messages, 'update');
  vi.spyOn(apiClient.messages, 'delete');
  vi.spyOn(apiClient.messages, 'markAsRead');
  
  // Spy on task methods
  vi.spyOn(apiClient.tasks, 'list');
  vi.spyOn(apiClient.tasks, 'get');
  vi.spyOn(apiClient.tasks, 'create');
  vi.spyOn(apiClient.tasks, 'update');
  vi.spyOn(apiClient.tasks, 'delete');
  vi.spyOn(apiClient.tasks, 'updateStatus');
  vi.spyOn(apiClient.tasks, 'getStats');
  
  // Spy on plan methods
  vi.spyOn(apiClient.plans, 'list');
  vi.spyOn(apiClient.plans, 'get');
  vi.spyOn(apiClient.plans, 'create');
  vi.spyOn(apiClient.plans, 'update');
  vi.spyOn(apiClient.plans, 'delete');
  vi.spyOn(apiClient.plans, 'updateStatus');
  vi.spyOn(apiClient.plans, 'getStats');
}

/**
 * Resets all mocks and spies
 */
export function resetApiClientMocks(): void {
  vi.clearAllMocks();
}