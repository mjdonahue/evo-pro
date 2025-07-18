/**
 * Enhanced task scheduler with cooperative multitasking capabilities
 */

import { TaskScheduler, TaskPriority, TaskStatus, Task, TaskOptions, TaskSchedulerConfig } from './taskScheduler';
import { CooperativeTask, CooperativeTaskOptions, createCooperativeTask, executeToCompletion } from './cooperativeTask';

/**
 * Extended task options with cooperative multitasking settings
 */
export interface CooperativeTaskSchedulerOptions extends TaskOptions {
  /** Whether to use cooperative execution (default: true) */
  cooperative?: boolean;
  /** Maximum time to run before yielding (in milliseconds) */
  timeSlice?: number;
  /** Whether to use adaptive time allocation based on system load */
  adaptive?: boolean;
  /** Minimum frame rate to maintain (for adaptive mode) */
  targetFrameRate?: number;
  /** Whether to use a generator function for cooperative execution */
  isGenerator?: boolean;
}

/**
 * Configuration for the cooperative task scheduler
 */
export interface CooperativeTaskSchedulerConfig extends TaskSchedulerConfig {
  /** Default cooperative task options */
  defaultCooperativeOptions?: Partial<CooperativeTaskSchedulerOptions>;
}

/**
 * Enhanced task scheduler that supports cooperative multitasking
 */
export class CooperativeTaskScheduler extends TaskScheduler {
  private cooperativeTasks: Map<string, CooperativeTask<any>> = new Map();
  private defaultCooperativeOptions: Partial<CooperativeTaskSchedulerOptions>;

  /**
   * Creates a new CooperativeTaskScheduler instance
   * 
   * @param config - Scheduler configuration
   */
  constructor(config: CooperativeTaskSchedulerConfig = {}) {
    super(config);
    this.defaultCooperativeOptions = config.defaultCooperativeOptions || {};
  }

  /**
   * Schedules a new task with cooperative multitasking support
   * 
   * @param name - Task name
   * @param fn - Task function that returns a promise or a generator
   * @param options - Task options with cooperative settings
   * @returns Task ID
   */
  override schedule<T>(
    name: string, 
    fn: (() => Promise<T>) | (() => Generator<any, T>) | (() => AsyncGenerator<any, T>), 
    options: CooperativeTaskSchedulerOptions = {}
  ): string {
    const cooperative = options.cooperative ?? true;
    const isGenerator = options.isGenerator ?? false;
    
    // If cooperative execution is disabled or the function is not a generator,
    // fall back to the standard task scheduler
    if (!cooperative || !isGenerator) {
      return super.schedule(name, fn as () => Promise<T>, options);
    }
    
    // Create a wrapper function that uses our cooperative task utilities
    const cooperativeWrapper = async (): Promise<T> => {
      const cooperativeOptions: CooperativeTaskOptions = {
        timeSlice: options.timeSlice ?? this.defaultCooperativeOptions.timeSlice ?? 5,
        priority: options.priority ?? this.defaultCooperativeOptions.priority ?? TaskPriority.NORMAL,
        adaptive: options.adaptive ?? this.defaultCooperativeOptions.adaptive ?? true,
        targetFrameRate: options.targetFrameRate ?? this.defaultCooperativeOptions.targetFrameRate ?? 60,
        onProgress: (progress) => {
          const task = this.getTask(taskId);
          if (task) {
            task.progress = progress;
            options.onProgress?.(progress);
          }
        },
      };
      
      // Create a cooperative task
      const cooperativeTask = createCooperativeTask(fn as () => Generator<any, T>, cooperativeOptions);
      
      // Store the cooperative task for potential cancellation
      this.cooperativeTasks.set(taskId, cooperativeTask);
      
      try {
        // Execute the task to completion, yielding control back to the main thread periodically
        return await executeToCompletion(cooperativeTask);
      } finally {
        // Clean up
        this.cooperativeTasks.delete(taskId);
      }
    };
    
    // Schedule the wrapper function
    const taskId = super.schedule(name, cooperativeWrapper, options);
    return taskId;
  }

  /**
   * Cancels a task, including any associated cooperative task
   * 
   * @param id - Task ID
   * @returns Whether the task was canceled
   */
  override cancel(id: string): boolean {
    // Cancel the cooperative task if it exists
    const cooperativeTask = this.cooperativeTasks.get(id);
    if (cooperativeTask) {
      cooperativeTask.cancel();
      this.cooperativeTasks.delete(id);
    }
    
    // Cancel the task in the base scheduler
    return super.cancel(id);
  }

  /**
   * Creates a cooperative task from a generator function
   * 
   * @param name - Task name
   * @param generatorFn - Generator function that yields progress and returns a result
   * @param options - Task options
   * @returns Task ID
   */
  scheduleGenerator<T>(
    name: string,
    generatorFn: () => Generator<any, T> | AsyncGenerator<any, T>,
    options: CooperativeTaskSchedulerOptions = {}
  ): string {
    return this.schedule(name, generatorFn, { ...options, isGenerator: true, cooperative: true });
  }

  /**
   * Schedules an array processing task that processes items cooperatively
   * 
   * @param name - Task name
   * @param array - Array to process
   * @param processFn - Function to process each item
   * @param options - Task options
   * @returns Task ID
   */
  scheduleArrayProcessing<T, R>(
    name: string,
    array: T[],
    processFn: (item: T, index: number) => Promise<R> | R,
    options: CooperativeTaskSchedulerOptions = {}
  ): string {
    // Create a generator function that processes the array cooperatively
    const generatorFn = async function* () {
      const results: R[] = [];
      const timeSlice = options.timeSlice ?? 5;
      
      let startTime = performance.now();
      let lastYieldTime = startTime;
      
      for (let i = 0; i < array.length; i++) {
        // Process the current item
        results.push(await processFn(array[i], i));
        
        // Calculate progress
        const progress = Math.round((i + 1) / array.length * 100);
        
        // Yield progress
        yield progress;
        
        // Check if we should yield control back to the main thread
        const now = performance.now();
        if (now - lastYieldTime >= timeSlice) {
          // Reset the yield time
          lastYieldTime = performance.now();
          // Explicitly yield to give control back to the main thread
          await new Promise(resolve => setTimeout(resolve, 0));
        }
      }
      
      // Return the processed results
      return results;
    };
    
    // Schedule the generator function
    return this.scheduleGenerator(name, generatorFn, options);
  }

  /**
   * Schedules a task that breaks a large computation into smaller chunks
   * 
   * @param name - Task name
   * @param totalIterations - Total number of iterations
   * @param iterationFn - Function to execute for each iteration
   * @param options - Task options
   * @returns Task ID
   */
  scheduleChunkedComputation<T>(
    name: string,
    totalIterations: number,
    iterationFn: (iteration: number) => Promise<void> | void,
    finalFn: () => Promise<T> | T,
    options: CooperativeTaskSchedulerOptions = {}
  ): string {
    // Create a generator function that executes the computation in chunks
    const generatorFn = async function* () {
      const timeSlice = options.timeSlice ?? 5;
      
      let startTime = performance.now();
      let lastYieldTime = startTime;
      
      for (let i = 0; i < totalIterations; i++) {
        // Execute the current iteration
        await iterationFn(i);
        
        // Calculate progress
        const progress = Math.round((i + 1) / totalIterations * 100);
        
        // Yield progress
        yield progress;
        
        // Check if we should yield control back to the main thread
        const now = performance.now();
        if (now - lastYieldTime >= timeSlice) {
          // Reset the yield time
          lastYieldTime = performance.now();
          // Explicitly yield to give control back to the main thread
          await new Promise(resolve => setTimeout(resolve, 0));
        }
      }
      
      // Execute the final function and return the result
      return await finalFn();
    };
    
    // Schedule the generator function
    return this.scheduleGenerator(name, generatorFn, options);
  }
}

/**
 * Global cooperative task scheduler instance
 */
export const cooperativeTaskScheduler = new CooperativeTaskScheduler();

/**
 * Hook for using the cooperative task scheduler in React components
 * 
 * @param options - Task scheduler configuration
 * @returns Cooperative task scheduler instance and utility functions
 */
export function useCooperativeTaskScheduler(options: CooperativeTaskSchedulerConfig = {}) {
  // Create a new cooperative task scheduler instance or use the global one
  const scheduler = options ? new CooperativeTaskScheduler(options) : cooperativeTaskScheduler;
  
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
    scheduleTask: <T>(name: string, fn: () => Promise<T>, options?: CooperativeTaskSchedulerOptions) => 
      scheduler.schedule(name, fn, options),
    
    /**
     * Schedules a generator task
     * 
     * @param name - Task name
     * @param generatorFn - Generator function
     * @param options - Task options
     * @returns Task ID
     */
    scheduleGenerator: <T>(
      name: string,
      generatorFn: () => Generator<any, T> | AsyncGenerator<any, T>,
      options?: CooperativeTaskSchedulerOptions
    ) => scheduler.scheduleGenerator(name, generatorFn, options),
    
    /**
     * Schedules an array processing task
     * 
     * @param name - Task name
     * @param array - Array to process
     * @param processFn - Function to process each item
     * @param options - Task options
     * @returns Task ID
     */
    scheduleArrayProcessing: <T, R>(
      name: string,
      array: T[],
      processFn: (item: T, index: number) => Promise<R> | R,
      options?: CooperativeTaskSchedulerOptions
    ) => scheduler.scheduleArrayProcessing(name, array, processFn, options),
    
    /**
     * Schedules a chunked computation task
     * 
     * @param name - Task name
     * @param totalIterations - Total number of iterations
     * @param iterationFn - Function to execute for each iteration
     * @param finalFn - Function to execute after all iterations
     * @param options - Task options
     * @returns Task ID
     */
    scheduleChunkedComputation: <T>(
      name: string,
      totalIterations: number,
      iterationFn: (iteration: number) => Promise<void> | void,
      finalFn: () => Promise<T> | T,
      options?: CooperativeTaskSchedulerOptions
    ) => scheduler.scheduleChunkedComputation(name, totalIterations, iterationFn, finalFn, options),
    
    /**
     * Cancels a task
     * 
     * @param id - Task ID
     * @returns Whether the task was canceled
     */
    cancelTask: (id: string) => scheduler.cancel(id),
    
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