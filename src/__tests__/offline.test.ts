import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { controllers } from '../api/controllers';
import { offlineQueueManager, OperationType } from '../lib/api/offline';
import { ipc_invoke } from '../api/ipc';

// Mock the ipc_invoke function
vi.mock('../api/ipc', () => ({
  ipc_invoke: vi.fn()
}));

// Mock the navigator.onLine property
Object.defineProperty(navigator, 'onLine', {
  configurable: true,
  get: vi.fn()
});

describe('Offline Queue System', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    offlineQueueManager.clearQueue();
    
    // Mock localStorage
    const localStorageMock = (() => {
      let store: Record<string, string> = {};
      return {
        getItem: (key: string) => store[key] || null,
        setItem: (key: string, value: string) => { store[key] = value.toString(); },
        removeItem: (key: string) => { delete store[key]; },
        clear: () => { store = {}; },
        length: 0,
        key: (_: number) => null
      };
    })();
    
    Object.defineProperty(window, 'localStorage', {
      value: localStorageMock,
      writable: true
    });
  });

  afterEach(() => {
    // Clean up
    offlineQueueManager.clearQueue();
  });

  describe('Online/Offline Detection', () => {
    it('should detect when the application is online', () => {
      // Mock navigator.onLine to return true
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      
      // Trigger the online event
      window.dispatchEvent(new Event('online'));
      
      expect(controllers.offline.isOnline()).toBe(true);
    });

    it('should detect when the application is offline', () => {
      // Mock navigator.onLine to return false
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      
      // Trigger the offline event
      window.dispatchEvent(new Event('offline'));
      
      expect(controllers.offline.isOnline()).toBe(false);
    });
  });

  describe('Operation Queueing', () => {
    it('should queue operations when offline', async () => {
      // Mock navigator.onLine to return false
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Mock ipc_invoke to throw a network error
      (ipc_invoke as any).mockRejectedValue(new Error('Network error'));
      
      // Try to create a task while offline
      try {
        await controllers.tasks.create({ title: 'Test Task', description: 'Created while offline' });
      } catch (error) {
        // We expect this to fail since we're offline
      }
      
      // Check if the operation was queued
      const queuedOperations = controllers.offline.getQueuedOperations();
      expect(queuedOperations.length).toBeGreaterThan(0);
      
      // Verify the queued operation
      const operation = queuedOperations[0];
      expect(operation.type).toBe(OperationType.CREATE);
      expect(operation.entityType).toBe('task');
      expect(operation.method).toContain('create_task');
      expect(operation.params).toHaveProperty('input');
      expect(operation.params.input).toHaveProperty('title', 'Test Task');
    });

    it('should process queued operations when coming back online', async () => {
      // Start offline
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Manually add an operation to the queue
      offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Test Task', description: 'Created while offline' } },
        entityType: 'task'
      });
      
      // Verify the operation is in the queue
      expect(controllers.offline.getQueueLength()).toBe(1);
      
      // Mock successful API response for when we process the queue
      (ipc_invoke as any).mockResolvedValue({
        success: true,
        data: { id: 'new-task-id', title: 'Test Task' }
      });
      
      // Come back online
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      window.dispatchEvent(new Event('online'));
      
      // Process the queue manually (in a real scenario, this would happen automatically)
      await controllers.offline.processQueue();
      
      // Verify the queue is now empty
      expect(controllers.offline.getQueueLength()).toBe(0);
      
      // Verify the API was called with the correct parameters
      expect(ipc_invoke).toHaveBeenCalledWith('create_task', { 
        input: { title: 'Test Task', description: 'Created while offline' } 
      });
    });
  });

  describe('Queue Management', () => {
    it('should allow clearing the queue', () => {
      // Add some operations to the queue
      offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 1' } },
        entityType: 'task'
      });
      
      offlineQueueManager.enqueue({
        type: OperationType.UPDATE,
        method: 'update_task',
        params: { id: 'task-1', data: { title: 'Updated Task' } },
        entityType: 'task',
        entityId: 'task-1'
      });
      
      // Verify operations are in the queue
      expect(controllers.offline.getQueueLength()).toBe(2);
      
      // Clear the queue
      controllers.offline.clearQueue();
      
      // Verify the queue is empty
      expect(controllers.offline.getQueueLength()).toBe(0);
    });

    it('should allow removing specific operations from the queue', () => {
      // Add some operations to the queue
      const id1 = offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 1' } },
        entityType: 'task'
      });
      
      const id2 = offlineQueueManager.enqueue({
        type: OperationType.UPDATE,
        method: 'update_task',
        params: { id: 'task-1', data: { title: 'Updated Task' } },
        entityType: 'task',
        entityId: 'task-1'
      });
      
      // Verify operations are in the queue
      expect(controllers.offline.getQueueLength()).toBe(2);
      
      // Remove one operation
      controllers.offline.removeOperation(id1);
      
      // Verify only one operation remains
      expect(controllers.offline.getQueueLength()).toBe(1);
      
      // Verify the correct operation remains
      const remainingOperations = controllers.offline.getQueuedOperations();
      expect(remainingOperations[0].id).toBe(id2);
    });
  });

  describe('Integration with Controllers', () => {
    it('should handle create operations when offline', async () => {
      // Mock navigator.onLine to return false
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Try to create a task while offline
      const result = await controllers.tasks.create({ 
        title: 'Offline Task', 
        description: 'This task was created while offline' 
      });
      
      // Verify we got a mock response with an ID
      expect(result).toHaveProperty('id');
      
      // Verify the operation was queued
      const queuedOperations = controllers.offline.getQueuedOperations();
      expect(queuedOperations.length).toBe(1);
      expect(queuedOperations[0].type).toBe(OperationType.CREATE);
      expect(queuedOperations[0].entityType).toBe('task');
    });

    it('should handle update operations when offline', async () => {
      // Mock navigator.onLine to return false
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Try to update a task while offline
      const result = await controllers.tasks.update('task-1', { 
        title: 'Updated Offline', 
        status: 'In Progress' 
      });
      
      // Verify we got a mock response
      expect(result).toHaveProperty('id');
      
      // Verify the operation was queued
      const queuedOperations = controllers.offline.getQueuedOperations();
      expect(queuedOperations.length).toBe(1);
      expect(queuedOperations[0].type).toBe(OperationType.UPDATE);
      expect(queuedOperations[0].entityType).toBe('task');
      expect(queuedOperations[0].entityId).toBe('task-1');
    });

    it('should handle delete operations when offline', async () => {
      // Mock navigator.onLine to return false
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Try to delete a task while offline
      await controllers.tasks.delete('task-1');
      
      // Verify the operation was queued
      const queuedOperations = controllers.offline.getQueuedOperations();
      expect(queuedOperations.length).toBe(1);
      expect(queuedOperations[0].type).toBe(OperationType.DELETE);
      expect(queuedOperations[0].entityType).toBe('task');
      expect(queuedOperations[0].entityId).toBe('task-1');
    });
  });
});