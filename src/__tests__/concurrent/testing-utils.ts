/**
 * Utilities for testing concurrent systems
 * 
 * This module provides utilities for testing concurrent systems,
 * particularly those using the actor model with Kameo.
 */

import { vi } from 'vitest';

/**
 * A controlled environment for testing concurrent operations
 * 
 * This class provides utilities for testing concurrent operations
 * in a controlled and deterministic way.
 */
export class ConcurrentTestEnvironment {
  private tasks: Array<{ id: string; fn: () => Promise<any>; delay: number }> = [];
  private completedTasks: string[] = [];
  private results: Map<string, any> = new Map();
  private errors: Map<string, Error> = new Map();
  private clock: ReturnType<typeof vi.useFakeTimers> | null = null;

  /**
   * Creates a new concurrent test environment
   * @param useFakeTimers Whether to use fake timers for deterministic timing
   */
  constructor(useFakeTimers = true) {
    if (useFakeTimers) {
      this.clock = vi.useFakeTimers();
    }
  }

  /**
   * Adds a task to the environment
   * @param id A unique identifier for the task
   * @param fn The function to execute
   * @param delay Optional delay before executing the task (in ms)
   */
  addTask(id: string, fn: () => Promise<any>, delay = 0) {
    this.tasks.push({ id, fn, delay });
    return this;
  }

  /**
   * Runs all tasks in the environment
   * @returns A promise that resolves when all tasks have completed
   */
  async runAll() {
    const promises = this.tasks.map(async ({ id, fn, delay }) => {
      if (delay > 0 && this.clock) {
        await new Promise(resolve => setTimeout(resolve, delay));
        this.clock.advanceTimersByTime(delay);
      }

      try {
        const result = await fn();
        this.results.set(id, result);
        this.completedTasks.push(id);
        return result;
      } catch (error) {
        this.errors.set(id, error as Error);
        this.completedTasks.push(id);
        throw error;
      }
    });

    await Promise.allSettled(promises);
    return this;
  }

  /**
   * Runs tasks in a specific order
   * @param order An array of task IDs specifying the order to run tasks
   * @returns A promise that resolves when all specified tasks have completed
   */
  async runInOrder(order: string[]) {
    for (const id of order) {
      const task = this.tasks.find(t => t.id === id);
      if (!task) {
        throw new Error(`Task with ID ${id} not found`);
      }

      if (task.delay > 0 && this.clock) {
        await new Promise(resolve => setTimeout(resolve, task.delay));
        this.clock.advanceTimersByTime(task.delay);
      }

      try {
        const result = await task.fn();
        this.results.set(id, result);
        this.completedTasks.push(id);
      } catch (error) {
        this.errors.set(id, error as Error);
        this.completedTasks.push(id);
        throw error;
      }
    }

    return this;
  }

  /**
   * Gets the result of a specific task
   * @param id The ID of the task
   * @returns The result of the task
   */
  getResult(id: string) {
    return this.results.get(id);
  }

  /**
   * Gets the error of a specific task
   * @param id The ID of the task
   * @returns The error of the task
   */
  getError(id: string) {
    return this.errors.get(id);
  }

  /**
   * Gets the order in which tasks completed
   * @returns An array of task IDs in the order they completed
   */
  getCompletionOrder() {
    return [...this.completedTasks];
  }

  /**
   * Cleans up the environment
   */
  cleanup() {
    if (this.clock) {
      this.clock.restoreAllMocks();
    }
    this.tasks = [];
    this.completedTasks = [];
    this.results.clear();
    this.errors.clear();
  }
}

/**
 * Creates a controlled race condition for testing
 * @param promises An array of promises to race
 * @param winner The index of the promise that should win the race
 * @returns A promise that resolves with the result of the winning promise
 */
export function createControlledRace<T>(promises: Promise<T>[], winner: number): Promise<T> {
  return new Promise((resolve, reject) => {
    promises.forEach((promise, index) => {
      if (index === winner) {
        promise.then(resolve).catch(reject);
      } else {
        // Ensure other promises don't resolve/reject before the winner
        promise.then(() => {}).catch(() => {});
      }
    });
  });
}

/**
 * Creates a promise that resolves after a specified delay
 * @param value The value to resolve with
 * @param delay The delay in milliseconds
 * @returns A promise that resolves with the specified value after the delay
 */
export function delayedResolve<T>(value: T, delay: number): Promise<T> {
  return new Promise(resolve => {
    setTimeout(() => resolve(value), delay);
  });
}

/**
 * Creates a promise that rejects after a specified delay
 * @param error The error to reject with
 * @param delay The delay in milliseconds
 * @returns A promise that rejects with the specified error after the delay
 */
export function delayedReject(error: Error, delay: number): Promise<never> {
  return new Promise((_, reject) => {
    setTimeout(() => reject(error), delay);
  });
}