import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { controllers } from '../api/controllers';
import { offlineQueueManager, OperationType } from '../lib/api/offline';
import { syncManager, SyncStatus, SyncEventType } from '../lib/api/sync';
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

describe('Synchronization Mechanisms', () => {
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

  describe('Basic Synchronization', () => {
    it('should synchronize operations when coming back online', async () => {
      // Start offline
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Add operations to the queue
      offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 1', description: 'First task' } },
        entityType: 'task'
      });
      
      offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 2', description: 'Second task' } },
        entityType: 'task'
      });
      
      // Verify operations are in the queue
      expect(controllers.offline.getQueueLength()).toBe(2);
      
      // Mock successful API responses
      (ipc_invoke as any).mockResolvedValue({
        success: true,
        data: { id: 'new-task-id', title: 'Task' }
      });
      
      // Come back online
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      window.dispatchEvent(new Event('online'));
      
      // Synchronize operations
      const result = await controllers.offline.sync.synchronize();
      
      // Verify synchronization was successful
      expect(result.success).toBe(true);
      expect(result.completed).toBe(2);
      expect(result.failed).toBe(0);
      expect(result.skipped).toBe(0);
      
      // Verify the queue is now empty
      expect(controllers.offline.getQueueLength()).toBe(0);
      
      // Verify the API was called for each operation
      expect(ipc_invoke).toHaveBeenCalledTimes(2);
    });

    it('should handle failed operations during synchronization', async () => {
      // Start offline
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Add operations to the queue
      const id1 = offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 1', description: 'First task' } },
        entityType: 'task'
      });
      
      const id2 = offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 2', description: 'Second task' } },
        entityType: 'task'
      });
      
      // Verify operations are in the queue
      expect(controllers.offline.getQueueLength()).toBe(2);
      
      // Mock API responses - first succeeds, second fails
      (ipc_invoke as any)
        .mockResolvedValueOnce({
          success: true,
          data: { id: 'task-1', title: 'Task 1' }
        })
        .mockRejectedValueOnce(new Error('API error'));
      
      // Come back online
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      window.dispatchEvent(new Event('online'));
      
      // Synchronize operations
      const result = await controllers.offline.sync.synchronize();
      
      // Verify synchronization was partially successful
      expect(result.success).toBe(false);
      expect(result.completed).toBe(1);
      expect(result.failed).toBe(1);
      expect(result.skipped).toBe(0);
      
      // Verify only the failed operation remains in the queue
      expect(controllers.offline.getQueueLength()).toBe(1);
      const remainingOps = controllers.offline.getQueuedOperations();
      expect(remainingOps[0].id).toBe(id2);
      
      // Verify the API was called for each operation
      expect(ipc_invoke).toHaveBeenCalledTimes(2);
    });
  });

  describe('Dependency Handling', () => {
    it('should process operations in the correct order based on dependencies', async () => {
      // Start offline
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Add operations to the queue - create, then update the same entity
      const createId = offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'New Task', description: 'Task description' } },
        entityType: 'task'
      });
      
      // Add a small delay to ensure the timestamps are different
      await new Promise(resolve => setTimeout(resolve, 10));
      
      const updateId = offlineQueueManager.enqueue({
        type: OperationType.UPDATE,
        method: 'update_task',
        params: { id: 'local-id', data: { title: 'Updated Task' } },
        entityType: 'task',
        entityId: 'local-id'
      });
      
      // Verify operations are in the queue
      expect(controllers.offline.getQueueLength()).toBe(2);
      
      // Mock successful API responses
      (ipc_invoke as any)
        .mockImplementation((method, params) => {
          if (method === 'create_task') {
            return Promise.resolve({
              success: true,
              data: { id: 'server-id', title: params.input.title }
            });
          } else if (method === 'update_task') {
            return Promise.resolve({
              success: true,
              data: { id: params.id, title: params.data.title }
            });
          }
          return Promise.resolve({ success: true, data: {} });
        });
      
      // Come back online
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      window.dispatchEvent(new Event('online'));
      
      // Track the order of API calls
      const callOrder: string[] = [];
      (ipc_invoke as any).mockImplementation((method, params) => {
        callOrder.push(method);
        if (method === 'create_task') {
          return Promise.resolve({
            success: true,
            data: { id: 'server-id', title: params.input.title }
          });
        } else if (method === 'update_task') {
          return Promise.resolve({
            success: true,
            data: { id: params.id, title: params.data.title }
          });
        }
        return Promise.resolve({ success: true, data: {} });
      });
      
      // Synchronize operations
      const result = await controllers.offline.sync.synchronize();
      
      // Verify synchronization was successful
      expect(result.success).toBe(true);
      expect(result.completed).toBe(2);
      
      // Verify the operations were processed in the correct order
      expect(callOrder[0]).toBe('create_task');
      expect(callOrder[1]).toBe('update_task');
      
      // Verify the queue is now empty
      expect(controllers.offline.getQueueLength()).toBe(0);
    });
  });

  describe('Synchronization Events', () => {
    it('should emit events during synchronization', async () => {
      // Start offline
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Add an operation to the queue
      offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 1', description: 'First task' } },
        entityType: 'task'
      });
      
      // Mock successful API response
      (ipc_invoke as any).mockResolvedValue({
        success: true,
        data: { id: 'new-task-id', title: 'Task 1' }
      });
      
      // Set up event listeners
      const progressEvents: any[] = [];
      const completedEvents: any[] = [];
      const operationSyncedEvents: any[] = [];
      
      controllers.offline.sync.addEventListener(SyncEventType.PROGRESS, (event) => {
        progressEvents.push(event);
      });
      
      controllers.offline.sync.addEventListener(SyncEventType.COMPLETED, (event) => {
        completedEvents.push(event);
      });
      
      controllers.offline.sync.addEventListener(SyncEventType.OPERATION_SYNCED, (event) => {
        operationSyncedEvents.push(event);
      });
      
      // Come back online
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      window.dispatchEvent(new Event('online'));
      
      // Synchronize operations
      await controllers.offline.sync.synchronize();
      
      // Verify events were emitted
      expect(progressEvents.length).toBeGreaterThan(0);
      expect(completedEvents.length).toBe(1);
      expect(operationSyncedEvents.length).toBe(1);
      
      // Verify the completed event has the correct data
      expect(completedEvents[0].result.success).toBe(true);
      expect(completedEvents[0].result.completed).toBe(1);
      
      // Verify the operation synced event has the correct data
      expect(operationSyncedEvents[0].operation.method).toBe('create_task');
      expect(operationSyncedEvents[0].result.data.title).toBe('Task 1');
    });
  });

  describe('Synchronization Status', () => {
    it('should track synchronization status correctly', async () => {
      // Start offline
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(false);
      window.dispatchEvent(new Event('offline'));
      
      // Add an operation to the queue
      offlineQueueManager.enqueue({
        type: OperationType.CREATE,
        method: 'create_task',
        params: { input: { title: 'Task 1', description: 'First task' } },
        entityType: 'task'
      });
      
      // Mock successful API response
      (ipc_invoke as any).mockResolvedValue({
        success: true,
        data: { id: 'new-task-id', title: 'Task 1' }
      });
      
      // Check initial status
      expect(controllers.offline.sync.getStatus()).toBe(SyncStatus.IDLE);
      
      // Come back online
      (navigator.onLine as unknown as vi.Mock).mockReturnValue(true);
      window.dispatchEvent(new Event('online'));
      
      // Start synchronization
      const syncPromise = controllers.offline.sync.synchronize();
      
      // Check status during synchronization
      expect(controllers.offline.sync.isSynchronizing()).toBe(true);
      expect(controllers.offline.sync.getStatus()).toBe(SyncStatus.SYNCING);
      
      // Wait for synchronization to complete
      await syncPromise;
      
      // Check status after synchronization
      expect(controllers.offline.sync.isSynchronizing()).toBe(false);
      expect(controllers.offline.sync.getStatus()).toBe(SyncStatus.COMPLETED);
      
      // Check progress after synchronization
      const progress = controllers.offline.sync.getProgress();
      expect(progress.total).toBe(1);
      expect(progress.completed).toBe(1);
      expect(progress.failed).toBe(0);
      expect(progress.status).toBe(SyncStatus.COMPLETED);
    });
  });
});