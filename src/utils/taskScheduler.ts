/**
 * Priority-based task scheduler for background processing
 */

/**
 * Task priority levels
 */
export enum TaskPriority {
  /** Critical tasks that should be executed immediately */
  CRITICAL = 0,
  /** High priority tasks that should be executed before normal tasks */
  HIGH = 1,
  /** Normal priority tasks */
  NORMAL = 2,
  /** Low priority tasks that can be executed when there are no higher priority tasks */
  LOW = 3,
  /** Background tasks that should only be executed when the system is idle */
  BACKGROUND = 4,
}

/**
 * Task status
 */
export enum TaskStatus {
  /** Task is pending execution */
  PENDING = 'pending',
  /** Task is currently running */
  RUNNING = 'running',
  /** Task has been completed successfully */
  COMPLETED = 'completed',
  /** Task has failed */
  FAILED = 'failed',
  /** Task has been canceled */
  CANCELED = 'canceled',
  /** Task has been paused */
  PAUSED = 'paused',
}

/**
 * Task options
 */
export interface TaskOptions {
  /** Task priority (default: NORMAL) */
  priority?: TaskPriority;
  /** Task timeout in milliseconds (default: no timeout) */
  timeout?: number;
  /** Whether the task can be canceled (default: true) */
  cancelable?: boolean;
  /** Maximum number of retry attempts (default: 0) */
  maxRetries?: number;
  /** Delay between retry attempts in milliseconds (default: 1000) */
  retryDelay?: number;
  /** Dependencies that must be completed before this task can run */
  dependencies?: string[];
  /** Callback when the task status changes */
  onStatusChange?: (status: TaskStatus) => void;
  /** Callback when the task progress changes */
  onProgress?: (progress: number) => void;
  /** Callback when the task completes successfully */
  onSuccess?: (result: any) => void;
  /** Callback when the task fails */
  onError?: (error: Error) => void;
}

/**
 * Task definition
 */
export interface Task<T = any> {
  /** Unique task ID */
  id: string;
  /** Task name */
  name: string;
  /** Task function that returns a promise */
  fn: () => Promise<T>;
  /** Task options */
  options: Required<TaskOptions>;
  /** Task status */
  status: TaskStatus;
  /** Task result */
  result?: T;
  /** Task error */
  error?: Error;
  /** Task progress (0-100) */
  progress: number;
  /** Number of retry attempts */
  retryCount: number;
  /** Task creation timestamp */
  createdAt: number;
  /** Task start timestamp */
  startedAt?: number;
  /** Task completion timestamp */
  completedAt?: number;
  /** Task cancellation token */
  cancelToken: { isCanceled: boolean };
}

/**
 * Task scheduler configuration
 */
export interface TaskSchedulerConfig {
  /** Maximum number of concurrent tasks (default: 4) */
  maxConcurrentTasks?: number;
  /** Whether to automatically start processing tasks (default: true) */
  autoStart?: boolean;
  /** Interval in milliseconds to check for new tasks (default: 100) */
  pollingInterval?: number;
  /** Default task options */
  defaultTaskOptions?: Partial<TaskOptions>;
  /** Whether to persist tasks across page reloads (default: false) */
  persistTasks?: boolean;
  /** Storage key for persisted tasks (default: 'task-scheduler-tasks') */
  storageKey?: string;
}

/**
 * Priority-based task scheduler for background processing
 */
export class TaskScheduler {
  private tasks: Map<string, Task> = new Map();
  private runningTasks: Set<string> = new Set();
  private isProcessing: boolean = false;
  private processingInterval?: number;
  private config: Required<TaskSchedulerConfig>;
  
  /**
   * Creates a new TaskScheduler instance
   * 
   * @param config - Scheduler configuration
   */
  constructor(config: TaskSchedulerConfig = {}) {
    this.config = {
      maxConcurrentTasks: config.maxConcurrentTasks ?? 4,
      autoStart: config.autoStart ?? true,
      pollingInterval: config.pollingInterval ?? 100,
      defaultTaskOptions: config.defaultTaskOptions ?? {},
      persistTasks: config.persistTasks ?? false,
      storageKey: config.storageKey ?? 'task-scheduler-tasks',
    };
    
    // Load persisted tasks if enabled
    if (this.config.persistTasks) {
      this.loadPersistedTasks();
    }
    
    // Start processing tasks if autoStart is enabled
    if (this.config.autoStart) {
      this.start();
    }
  }
  
  /**
   * Schedules a new task
   * 
   * @param name - Task name
   * @param fn - Task function that returns a promise
   * @param options - Task options
   * @returns Task ID
   */
  schedule<T>(name: string, fn: () => Promise<T>, options: TaskOptions = {}): string {
    const id = this.generateTaskId();
    const now = Date.now();
    
    const task: Task<T> = {
      id,
      name,
      fn,
      options: {
        priority: options.priority ?? this.config.defaultTaskOptions.priority ?? TaskPriority.NORMAL,
        timeout: options.timeout ?? this.config.defaultTaskOptions.timeout,
        cancelable: options.cancelable ?? this.config.defaultTaskOptions.cancelable ?? true,
        maxRetries: options.maxRetries ?? this.config.defaultTaskOptions.maxRetries ?? 0,
        retryDelay: options.retryDelay ?? this.config.defaultTaskOptions.retryDelay ?? 1000,
        dependencies: options.dependencies ?? this.config.defaultTaskOptions.dependencies ?? [],
        onStatusChange: options.onStatusChange ?? this.config.defaultTaskOptions.onStatusChange,
        onProgress: options.onProgress ?? this.config.defaultTaskOptions.onProgress,
        onSuccess: options.onSuccess ?? this.config.defaultTaskOptions.onSuccess,
        onError: options.onError ?? this.config.defaultTaskOptions.onError,
      },
      status: TaskStatus.PENDING,
      progress: 0,
      retryCount: 0,
      createdAt: now,
      cancelToken: { isCanceled: false },
    };
    
    this.tasks.set(id, task);
    
    // Persist task if enabled
    if (this.config.persistTasks) {
      this.persistTasks();
    }
    
    return id;
  }
  
  /**
   * Cancels a task
   * 
   * @param id - Task ID
   * @returns Whether the task was canceled
   */
  cancel(id: string): boolean {
    const task = this.tasks.get(id);
    
    if (!task) {
      return false;
    }
    
    if (!task.options.cancelable) {
      return false;
    }
    
    if (task.status === TaskStatus.COMPLETED || task.status === TaskStatus.FAILED || task.status === TaskStatus.CANCELED) {
      return false;
    }
    
    task.cancelToken.isCanceled = true;
    this.updateTaskStatus(task, TaskStatus.CANCELED);
    
    // Remove from running tasks if it was running
    if (this.runningTasks.has(id)) {
      this.runningTasks.delete(id);
    }
    
    return true;
  }
  
  /**
   * Pauses a task
   * 
   * @param id - Task ID
   * @returns Whether the task was paused
   */
  pause(id: string): boolean {
    const task = this.tasks.get(id);
    
    if (!task) {
      return false;
    }
    
    if (task.status !== TaskStatus.PENDING) {
      return false;
    }
    
    this.updateTaskStatus(task, TaskStatus.PAUSED);
    return true;
  }
  
  /**
   * Resumes a paused task
   * 
   * @param id - Task ID
   * @returns Whether the task was resumed
   */
  resume(id: string): boolean {
    const task = this.tasks.get(id);
    
    if (!task) {
      return false;
    }
    
    if (task.status !== TaskStatus.PAUSED) {
      return false;
    }
    
    this.updateTaskStatus(task, TaskStatus.PENDING);
    return true;
  }
  
  /**
   * Gets a task by ID
   * 
   * @param id - Task ID
   * @returns Task or undefined if not found
   */
  getTask(id: string): Task | undefined {
    return this.tasks.get(id);
  }
  
  /**
   * Gets all tasks
   * 
   * @returns Array of all tasks
   */
  getAllTasks(): Task[] {
    return Array.from(this.tasks.values());
  }
  
  /**
   * Gets tasks by status
   * 
   * @param status - Task status
   * @returns Array of tasks with the specified status
   */
  getTasksByStatus(status: TaskStatus): Task[] {
    return Array.from(this.tasks.values()).filter(task => task.status === status);
  }
  
  /**
   * Gets tasks by priority
   * 
   * @param priority - Task priority
   * @returns Array of tasks with the specified priority
   */
  getTasksByPriority(priority: TaskPriority): Task[] {
    return Array.from(this.tasks.values()).filter(task => task.options.priority === priority);
  }
  
  /**
   * Clears all tasks
   */
  clearTasks(): void {
    // Cancel all running tasks
    for (const id of this.runningTasks) {
      this.cancel(id);
    }
    
    this.tasks.clear();
    this.runningTasks.clear();
    
    // Clear persisted tasks if enabled
    if (this.config.persistTasks) {
      this.persistTasks();
    }
  }
  
  /**
   * Starts the task scheduler
   */
  start(): void {
    if (this.isProcessing) {
      return;
    }
    
    this.isProcessing = true;
    
    // Use requestIdleCallback if available, otherwise use setTimeout
    if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
      const processTasksInIdle = () => {
        window.requestIdleCallback(() => {
          this.processTasks();
          if (this.isProcessing) {
            processTasksInIdle();
          }
        }, { timeout: this.config.pollingInterval });
      };
      
      processTasksInIdle();
    } else {
      this.processingInterval = window.setInterval(() => {
        this.processTasks();
      }, this.config.pollingInterval);
    }
  }
  
  /**
   * Stops the task scheduler
   */
  stop(): void {
    this.isProcessing = false;
    
    if (this.processingInterval) {
      clearInterval(this.processingInterval);
      this.processingInterval = undefined;
    }
  }
  
  /**
   * Processes pending tasks
   */
  private processTasks(): void {
    // Skip if we're already at max concurrent tasks
    if (this.runningTasks.size >= this.config.maxConcurrentTasks) {
      return;
    }
    
    // Get all pending tasks
    const pendingTasks = Array.from(this.tasks.values()).filter(task => 
      task.status === TaskStatus.PENDING && 
      this.areDependenciesMet(task)
    );
    
    // Sort by priority (lower number = higher priority)
    pendingTasks.sort((a, b) => a.options.priority - b.options.priority);
    
    // Execute tasks up to max concurrent tasks
    for (const task of pendingTasks) {
      if (this.runningTasks.size >= this.config.maxConcurrentTasks) {
        break;
      }
      
      this.executeTask(task);
    }
  }
  
  /**
   * Executes a task
   * 
   * @param task - Task to execute
   */
  private executeTask(task: Task): void {
    // Skip if task is already running or completed
    if (task.status !== TaskStatus.PENDING) {
      return;
    }
    
    // Add to running tasks
    this.runningTasks.add(task.id);
    
    // Update task status
    task.startedAt = Date.now();
    this.updateTaskStatus(task, TaskStatus.RUNNING);
    
    // Create timeout if specified
    let timeoutId: number | undefined;
    if (task.options.timeout) {
      timeoutId = window.setTimeout(() => {
        if (task.status === TaskStatus.RUNNING) {
          task.error = new Error(`Task timed out after ${task.options.timeout}ms`);
          this.handleTaskFailure(task);
        }
      }, task.options.timeout);
    }
    
    // Execute task
    Promise.resolve().then(async () => {
      try {
        // Check if task was canceled
        if (task.cancelToken.isCanceled) {
          return;
        }
        
        // Execute task function
        const result = await task.fn();
        
        // Check if task was canceled during execution
        if (task.cancelToken.isCanceled) {
          return;
        }
        
        // Handle success
        task.result = result;
        task.completedAt = Date.now();
        task.progress = 100;
        this.updateTaskStatus(task, TaskStatus.COMPLETED);
        
        // Call success callback
        task.options.onSuccess?.(result);
      } catch (error) {
        // Check if task was canceled during execution
        if (task.cancelToken.isCanceled) {
          return;
        }
        
        // Handle error
        task.error = error instanceof Error ? error : new Error(String(error));
        this.handleTaskFailure(task);
      } finally {
        // Clear timeout
        if (timeoutId !== undefined) {
          clearTimeout(timeoutId);
        }
        
        // Remove from running tasks
        this.runningTasks.delete(task.id);
        
        // Persist tasks if enabled
        if (this.config.persistTasks) {
          this.persistTasks();
        }
      }
    });
  }
  
  /**
   * Handles task failure
   * 
   * @param task - Failed task
   */
  private handleTaskFailure(task: Task): void {
    // Check if we should retry
    if (task.retryCount < task.options.maxRetries) {
      task.retryCount++;
      this.updateTaskStatus(task, TaskStatus.PENDING);
      
      // Schedule retry with delay
      setTimeout(() => {
        if (task.status === TaskStatus.PENDING) {
          this.executeTask(task);
        }
      }, task.options.retryDelay);
    } else {
      // Mark as failed
      task.completedAt = Date.now();
      this.updateTaskStatus(task, TaskStatus.FAILED);
      
      // Call error callback
      task.options.onError?.(task.error);
    }
  }
  
  /**
   * Updates task status and calls status change callback
   * 
   * @param task - Task to update
   * @param status - New status
   */
  private updateTaskStatus(task: Task, status: TaskStatus): void {
    task.status = status;
    task.options.onStatusChange?.(status);
  }
  
  /**
   * Checks if all dependencies of a task are met
   * 
   * @param task - Task to check
   * @returns Whether all dependencies are met
   */
  private areDependenciesMet(task: Task): boolean {
    if (!task.options.dependencies || task.options.dependencies.length === 0) {
      return true;
    }
    
    return task.options.dependencies.every(dependencyId => {
      const dependency = this.tasks.get(dependencyId);
      return dependency && dependency.status === TaskStatus.COMPLETED;
    });
  }
  
  /**
   * Generates a unique task ID
   * 
   * @returns Unique task ID
   */
  private generateTaskId(): string {
    return `task-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
  
  /**
   * Persists tasks to storage
   */
  private persistTasks(): void {
    if (typeof window === 'undefined' || !window.localStorage) {
      return;
    }
    
    try {
      // Only persist serializable task data
      const persistableTasks = Array.from(this.tasks.entries()).map(([id, task]) => ({
        id,
        name: task.name,
        status: task.status,
        priority: task.options.priority,
        dependencies: task.options.dependencies,
        createdAt: task.createdAt,
        startedAt: task.startedAt,
        completedAt: task.completedAt,
        progress: task.progress,
        retryCount: task.retryCount,
        result: task.result,
        error: task.error ? task.error.message : undefined,
      }));
      
      localStorage.setItem(this.config.storageKey, JSON.stringify(persistableTasks));
    } catch (error) {
      console.error('Failed to persist tasks:', error);
    }
  }
  
  /**
   * Loads persisted tasks from storage
   */
  private loadPersistedTasks(): void {
    if (typeof window === 'undefined' || !window.localStorage) {
      return;
    }
    
    try {
      const persistedTasksJson = localStorage.getItem(this.config.storageKey);
      
      if (!persistedTasksJson) {
        return;
      }
      
      const persistedTasks = JSON.parse(persistedTasksJson);
      
      // Only restore tasks that were completed, failed, or canceled
      // (we can't restore running or pending tasks since we don't have their functions)
      for (const persistedTask of persistedTasks) {
        if (
          persistedTask.status === TaskStatus.COMPLETED ||
          persistedTask.status === TaskStatus.FAILED ||
          persistedTask.status === TaskStatus.CANCELED
        ) {
          const task: Task = {
            id: persistedTask.id,
            name: persistedTask.name,
            fn: () => Promise.resolve(persistedTask.result),
            options: {
              priority: persistedTask.priority,
              timeout: undefined,
              cancelable: true,
              maxRetries: 0,
              retryDelay: 1000,
              dependencies: persistedTask.dependencies || [],
              onStatusChange: undefined,
              onProgress: undefined,
              onSuccess: undefined,
              onError: undefined,
            },
            status: persistedTask.status,
            progress: persistedTask.progress,
            retryCount: persistedTask.retryCount,
            createdAt: persistedTask.createdAt,
            startedAt: persistedTask.startedAt,
            completedAt: persistedTask.completedAt,
            cancelToken: { isCanceled: persistedTask.status === TaskStatus.CANCELED },
            result: persistedTask.result,
            error: persistedTask.error ? new Error(persistedTask.error) : undefined,
          };
          
          this.tasks.set(task.id, task);
        }
      }
    } catch (error) {
      console.error('Failed to load persisted tasks:', error);
    }
  }
}

/**
 * Global task scheduler instance
 */
export const taskScheduler = new TaskScheduler();

/**
 * Hook for using the task scheduler in React components
 * 
 * @param options - Task scheduler configuration
 * @returns Task scheduler instance and utility functions
 */
export function useTaskScheduler(options: TaskSchedulerConfig = {}) {
  // Create a new task scheduler instance or use the global one
  const scheduler = options ? new TaskScheduler(options) : taskScheduler;
  
  return {
    scheduler,
    
    /**
     * Schedules a new task
     * 
     * @param name - Task name
     * @param fn - Task function that returns a promise
     * @param options - Task options
     * @returns Task ID
     */
    scheduleTask: <T>(name: string, fn: () => Promise<T>, options?: TaskOptions) => 
      scheduler.schedule(name, fn, options),
    
    /**
     * Cancels a task
     * 
     * @param id - Task ID
     * @returns Whether the task was canceled
     */
    cancelTask: (id: string) => scheduler.cancel(id),
    
    /**
     * Pauses a task
     * 
     * @param id - Task ID
     * @returns Whether the task was paused
     */
    pauseTask: (id: string) => scheduler.pause(id),
    
    /**
     * Resumes a paused task
     * 
     * @param id - Task ID
     * @returns Whether the task was resumed
     */
    resumeTask: (id: string) => scheduler.resume(id),
    
    /**
     * Gets a task by ID
     * 
     * @param id - Task ID
     * @returns Task or undefined if not found
     */
    getTask: (id: string) => scheduler.getTask(id),
    
    /**
     * Gets all tasks
     * 
     * @returns Array of all tasks
     */
    getAllTasks: () => scheduler.getAllTasks(),
    
    /**
     * Gets tasks by status
     * 
     * @param status - Task status
     * @returns Array of tasks with the specified status
     */
    getTasksByStatus: (status: TaskStatus) => scheduler.getTasksByStatus(status),
    
    /**
     * Gets tasks by priority
     * 
     * @param priority - Task priority
     * @returns Array of tasks with the specified priority
     */
    getTasksByPriority: (priority: TaskPriority) => scheduler.getTasksByPriority(priority),
  };
}