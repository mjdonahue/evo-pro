import '@testing-library/jest-dom';
import { vi } from 'vitest';
import { cleanup } from '@testing-library/react';

// Mock Tauri APIs with more realistic behavior for integration tests
// This allows testing the interaction between components and services
// without requiring a running Tauri instance

// Create a store for mock data that can be shared across tests
export const mockStore = {
  data: new Map(),
  events: new Map(),
  listeners: new Map(),
};

// Mock invoke with the ability to return different responses based on command
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((command, args) => {
    // Integration tests can set up expected responses for specific commands
    if (mockStore.data.has(command)) {
      const handler = mockStore.data.get(command);
      if (typeof handler === 'function') {
        return handler(args);
      }
      return handler;
    }
    console.warn(`No mock response set for command: ${command}`);
    return null;
  }),
}));

// Mock event system with the ability to emit events
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event, callback) => {
    const listeners = mockStore.listeners.get(event) || [];
    listeners.push(callback);
    mockStore.listeners.set(event, listeners);
    
    // Return unsubscribe function
    return Promise.resolve(() => {
      const currentListeners = mockStore.listeners.get(event) || [];
      const index = currentListeners.indexOf(callback);
      if (index !== -1) {
        currentListeners.splice(index, 1);
        mockStore.listeners.set(event, currentListeners);
      }
    });
  }),
  emit: vi.fn((event, payload) => {
    const listeners = mockStore.listeners.get(event) || [];
    listeners.forEach(callback => callback({ payload }));
    return Promise.resolve();
  }),
}));

// Helper to set up mock responses
export const mockCommand = (command, response) => {
  mockStore.data.set(command, response);
};

// Helper to emit mock events
export const emitMockEvent = (event, payload) => {
  const listeners = mockStore.listeners.get(event) || [];
  listeners.forEach(callback => callback({ payload }));
};

// Clean up after each test
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  mockStore.data.clear();
  mockStore.events.clear();
  // Don't clear listeners between tests to allow for persistent subscriptions
});

// Reset everything after all tests
afterAll(() => {
  mockStore.listeners.clear();
});