/**
 * Utilities for cooperative multitasking
 */

/**
 * Options for cooperative task execution
 */
export interface CooperativeTaskOptions {
  /** Maximum time to run before yielding (in milliseconds) */
  timeSlice?: number;
  /** Priority of the task (lower number = higher priority) */
  priority?: number;
  /** Whether to use adaptive time allocation based on system load */
  adaptive?: boolean;
  /** Minimum frame rate to maintain (for adaptive mode) */
  targetFrameRate?: number;
  /** Callback for progress updates */
  onProgress?: (progress: number) => void;
}

/**
 * Result of a cooperative task execution
 */
export interface CooperativeTaskResult<T> {
  /** Whether the task is complete */
  done: boolean;
  /** Current progress (0-100) */
  progress: number;
  /** Result of the task (if complete) */
  result?: T;
  /** Error that occurred during execution (if any) */
  error?: Error;
}

/**
 * A task that can be executed cooperatively, yielding control back to the main thread periodically
 */
export class CooperativeTask<T = any> {
  private iterator: Iterator<any, T> | AsyncIterator<any, T>;
  private options: Required<CooperativeTaskOptions>;
  private lastYieldTime: number = 0;
  private progress: number = 0;
  private frameTimeHistory: number[] = [];
  private isRunning: boolean = false;
  private abortController: AbortController = new AbortController();

  /**
   * Creates a new cooperative task
   * 
   * @param taskFn - Generator or async generator function that implements the task
   * @param options - Options for task execution
   */
  constructor(
    private taskFn: () => Generator<any, T> | AsyncGenerator<any, T>,
    options: CooperativeTaskOptions = {}
  ) {
    this.options = {
      timeSlice: options.timeSlice ?? 5,
      priority: options.priority ?? 0,
      adaptive: options.adaptive ?? true,
      targetFrameRate: options.targetFrameRate ?? 60,
      onProgress: options.onProgress ?? (() => {}),
    };
    
    this.iterator = this.taskFn();
  }

  /**
   * Executes the task until completion or until the time slice is exceeded
   * 
   * @returns Promise that resolves to the task result
   */
  async execute(): Promise<CooperativeTaskResult<T>> {
    if (this.isRunning) {
      throw new Error('Task is already running');
    }
    
    this.isRunning = true;
    
    try {
      const startTime = performance.now();
      this.lastYieldTime = startTime;
      
      // Calculate the deadline based on the time slice
      const timeSlice = this.getAdaptiveTimeSlice();
      const deadline = startTime + timeSlice;
      
      // Execute the task until the deadline is reached or the task is complete
      while (performance.now() < deadline) {
        if (this.abortController.signal.aborted) {
          return { done: false, progress: this.progress };
        }
        
        const result = await this.iterator.next();
        
        // Update progress if the yielded value is a number
        if (typeof result.value === 'number' && result.value >= 0 && result.value <= 100) {
          this.progress = result.value;
          this.options.onProgress(this.progress);
        }
        
        // If the task is complete, return the result
        if (result.done) {
          this.isRunning = false;
          return { done: true, progress: 100, result: result.value };
        }
        
        // Check if we should yield control back to the main thread
        if (this.shouldYield()) {
          break;
        }
      }
      
      // Record the frame time for adaptive time slicing
      if (this.options.adaptive) {
        const frameTime = performance.now() - startTime;
        this.updateFrameTimeHistory(frameTime);
      }
      
      // Task is not complete yet, return progress
      this.isRunning = false;
      return { done: false, progress: this.progress };
    } catch (error) {
      this.isRunning = false;
      return { 
        done: false, 
        progress: this.progress, 
        error: error instanceof Error ? error : new Error(String(error)) 
      };
    }
  }

  /**
   * Cancels the task
   */
  cancel(): void {
    this.abortController.abort();
  }

  /**
   * Resets the task to its initial state
   */
  reset(): void {
    this.iterator = this.taskFn();
    this.progress = 0;
    this.lastYieldTime = 0;
    this.frameTimeHistory = [];
    this.isRunning = false;
    this.abortController = new AbortController();
  }

  /**
   * Checks if the task should yield control back to the main thread
   * 
   * @returns Whether the task should yield
   */
  private shouldYield(): boolean {
    const now = performance.now();
    const timeSinceLastYield = now - this.lastYieldTime;
    
    // Yield if we've been executing for longer than the time slice
    if (timeSinceLastYield >= this.options.timeSlice) {
      this.lastYieldTime = now;
      return true;
    }
    
    return false;
  }

  /**
   * Updates the frame time history for adaptive time slicing
   * 
   * @param frameTime - Time taken to execute the current frame
   */
  private updateFrameTimeHistory(frameTime: number): void {
    // Keep a history of the last 10 frame times
    this.frameTimeHistory.push(frameTime);
    if (this.frameTimeHistory.length > 10) {
      this.frameTimeHistory.shift();
    }
  }

  /**
   * Gets the adaptive time slice based on system load
   * 
   * @returns Time slice in milliseconds
   */
  private getAdaptiveTimeSlice(): number {
    if (!this.options.adaptive || this.frameTimeHistory.length < 5) {
      return this.options.timeSlice;
    }
    
    // Calculate the average frame time
    const avgFrameTime = this.frameTimeHistory.reduce((sum, time) => sum + time, 0) / this.frameTimeHistory.length;
    
    // Target frame time based on the desired frame rate
    const targetFrameTime = 1000 / this.options.targetFrameRate;
    
    // If we're taking too long, reduce the time slice
    if (avgFrameTime > targetFrameTime) {
      return Math.max(1, this.options.timeSlice * (targetFrameTime / avgFrameTime));
    }
    
    // If we have room to spare, increase the time slice
    return Math.min(16, this.options.timeSlice * 1.2);
  }
}

/**
 * Creates a cooperative task that can be executed in small time slices
 * 
 * @param taskFn - Generator or async generator function that implements the task
 * @param options - Options for task execution
 * @returns Cooperative task instance
 */
export function createCooperativeTask<T>(
  taskFn: () => Generator<any, T> | AsyncGenerator<any, T>,
  options?: CooperativeTaskOptions
): CooperativeTask<T> {
  return new CooperativeTask<T>(taskFn, options);
}

/**
 * Executes a cooperative task to completion, yielding control back to the main thread periodically
 * 
 * @param task - Cooperative task to execute
 * @returns Promise that resolves to the task result
 */
export async function executeToCompletion<T>(task: CooperativeTask<T>): Promise<T> {
  while (true) {
    const result = await task.execute();
    
    if (result.done) {
      return result.result;
    }
    
    if (result.error) {
      throw result.error;
    }
    
    // Yield control back to the main thread
    await new Promise(resolve => setTimeout(resolve, 0));
  }
}

/**
 * Breaks an array into chunks that can be processed cooperatively
 * 
 * @param array - Array to process
 * @param processFn - Function to process each item
 * @param options - Options for cooperative execution
 * @returns Generator that yields progress and returns the processed results
 */
export async function* processArrayCooperatively<T, R>(
  array: T[],
  processFn: (item: T, index: number) => Promise<R> | R,
  options: CooperativeTaskOptions = {}
): AsyncGenerator<number, R[], void> {
  const results: R[] = [];
  const timeSlice = options.timeSlice ?? 5;
  
  let startTime = performance.now();
  let lastYieldTime = startTime;
  
  for (let i = 0; i < array.length; i++) {
    // Process the current item
    results.push(await processFn(array[i], i));
    
    // Calculate progress
    const progress = Math.round((i + 1) / array.length * 100);
    
    // Check if we should yield control back to the main thread
    const now = performance.now();
    if (now - lastYieldTime >= timeSlice) {
      // Yield progress
      yield progress;
      
      // Reset the yield time
      lastYieldTime = performance.now();
    }
  }
  
  // Return the processed results
  return results;
}

/**
 * Breaks a task into time slices using the browser's requestAnimationFrame API
 * 
 * @param callback - Function to call on each animation frame
 * @param options - Options for time slicing
 * @returns Function to cancel the time slicing
 */
export function requestTimeSlice(
  callback: (timeRemaining: number) => boolean | Promise<boolean>,
  options: { priority?: number } = {}
): () => void {
  let cancelled = false;
  
  const executeSlice = async (deadline?: IdleDeadline) => {
    if (cancelled) return;
    
    // Calculate time remaining (use 16ms as default if no deadline is provided)
    const timeRemaining = deadline ? deadline.timeRemaining() : 16;
    
    // Call the callback with the time remaining
    const shouldContinue = await callback(timeRemaining);
    
    // If the callback returns true, schedule another slice
    if (shouldContinue && !cancelled) {
      if ('requestIdleCallback' in window) {
        window.requestIdleCallback(executeSlice);
      } else {
        requestAnimationFrame(() => executeSlice());
      }
    }
  };
  
  // Start the time slicing
  if ('requestIdleCallback' in window) {
    window.requestIdleCallback(executeSlice);
  } else {
    requestAnimationFrame(() => executeSlice());
  }
  
  // Return a function to cancel the time slicing
  return () => {
    cancelled = true;
  };
}