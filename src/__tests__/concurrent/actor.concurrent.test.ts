import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { ConcurrentTestEnvironment, createControlledRace, delayedResolve, delayedReject } from './testing-utils';

/**
 * Example of a simple actor-like class for testing
 */
class Actor {
  private state: any = {};
  private messageQueue: Array<{ type: string; payload: any }> = [];
  private isProcessing = false;
  private handlers: Record<string, (payload: any) => Promise<any>> = {};

  /**
   * Registers a message handler
   */
  registerHandler(type: string, handler: (payload: any) => Promise<any>) {
    this.handlers[type] = handler;
    return this;
  }

  /**
   * Sends a message to the actor
   */
  async send(type: string, payload: any): Promise<any> {
    if (!this.handlers[type]) {
      throw new Error(`No handler registered for message type: ${type}`);
    }

    this.messageQueue.push({ type, payload });
    
    if (!this.isProcessing) {
      return this.processQueue();
    }
    
    return new Promise((resolve, reject) => {
      const checkQueue = setInterval(() => {
        if (!this.isProcessing) {
          clearInterval(checkQueue);
          this.processQueue().then(resolve).catch(reject);
        }
      }, 10);
    });
  }

  /**
   * Processes the message queue
   */
  private async processQueue(): Promise<any> {
    if (this.messageQueue.length === 0) {
      return null;
    }

    this.isProcessing = true;
    const { type, payload } = this.messageQueue.shift()!;
    
    try {
      const result = await this.handlers[type](payload);
      this.isProcessing = false;
      return result;
    } catch (error) {
      this.isProcessing = false;
      throw error;
    }
  }

  /**
   * Gets the current state
   */
  getState() {
    return { ...this.state };
  }

  /**
   * Updates the state
   */
  setState(newState: any) {
    this.state = { ...this.state, ...newState };
    return this;
  }
}

/**
 * Example tests for concurrent systems using the actor model
 * 
 * These tests demonstrate how to test concurrent systems in a
 * controlled and deterministic way.
 */
describe('Actor Concurrent Tests', () => {
  let testEnv: ConcurrentTestEnvironment;
  let counterActor: Actor;

  beforeEach(() => {
    // Set up a fresh test environment for each test
    testEnv = new ConcurrentTestEnvironment();
    
    // Create an actor for testing
    counterActor = new Actor();
    
    // Register message handlers
    counterActor.registerHandler('increment', async (amount) => {
      const currentValue = counterActor.getState().count || 0;
      counterActor.setState({ count: currentValue + amount });
      return counterActor.getState().count;
    });
    
    counterActor.registerHandler('decrement', async (amount) => {
      const currentValue = counterActor.getState().count || 0;
      counterActor.setState({ count: currentValue - amount });
      return counterActor.getState().count;
    });
    
    counterActor.registerHandler('reset', async () => {
      counterActor.setState({ count: 0 });
      return 0;
    });
    
    counterActor.registerHandler('delayedIncrement', async (params) => {
      const { amount, delay } = params;
      await new Promise(resolve => setTimeout(resolve, delay));
      const currentValue = counterActor.getState().count || 0;
      counterActor.setState({ count: currentValue + amount });
      return counterActor.getState().count;
    });
    
    // Initialize the counter
    counterActor.setState({ count: 0 });
  });

  afterEach(() => {
    // Clean up the test environment
    testEnv.cleanup();
  });

  it('should process messages in order', async () => {
    // Add tasks to the test environment
    testEnv
      .addTask('increment', () => counterActor.send('increment', 5))
      .addTask('decrement', () => counterActor.send('decrement', 2))
      .addTask('increment-again', () => counterActor.send('increment', 10));
    
    // Run tasks in a specific order
    await testEnv.runInOrder(['increment', 'decrement', 'increment-again']);
    
    // Verify the results
    expect(testEnv.getResult('increment')).toBe(5);
    expect(testEnv.getResult('decrement')).toBe(3);
    expect(testEnv.getResult('increment-again')).toBe(13);
    
    // Verify the final state
    expect(counterActor.getState().count).toBe(13);
  });

  it('should handle concurrent messages correctly', async () => {
    // Add tasks with delays to simulate concurrent operations
    testEnv
      .addTask('slow-increment', () => counterActor.send('delayedIncrement', { amount: 5, delay: 100 }))
      .addTask('fast-increment', () => counterActor.send('increment', 10))
      .addTask('reset', () => counterActor.send('reset', null));
    
    // Run all tasks concurrently
    await testEnv.runAll();
    
    // Verify the completion order (should be deterministic due to the controlled environment)
    const completionOrder = testEnv.getCompletionOrder();
    
    // The exact order depends on the implementation, but we can verify that all tasks completed
    expect(completionOrder).toHaveLength(3);
    expect(completionOrder).toContain('slow-increment');
    expect(completionOrder).toContain('fast-increment');
    expect(completionOrder).toContain('reset');
    
    // Verify the final state (should be 0 because reset was the last message processed)
    expect(counterActor.getState().count).toBe(0);
  });

  it('should handle errors in concurrent operations', async () => {
    // Add a handler that throws an error
    counterActor.registerHandler('error', async () => {
      throw new Error('Simulated error');
    });
    
    // Add tasks including one that will error
    testEnv
      .addTask('increment', () => counterActor.send('increment', 5))
      .addTask('error', () => counterActor.send('error', null).catch(e => e))
      .addTask('decrement', () => counterActor.send('decrement', 2));
    
    // Run all tasks
    await testEnv.runAll();
    
    // Verify that the error was captured
    const error = testEnv.getError('error');
    expect(error).toBeDefined();
    expect(error?.message).toBe('Simulated error');
    
    // Verify that other operations completed successfully
    expect(testEnv.getResult('increment')).toBe(5);
    expect(testEnv.getResult('decrement')).toBe(3);
    
    // Verify the final state
    expect(counterActor.getState().count).toBe(3);
  });

  it('should test race conditions with controlled outcomes', async () => {
    // Create promises that will resolve at different times
    const promise1 = delayedResolve('first', 100);
    const promise2 = delayedResolve('second', 50);
    const promise3 = delayedResolve('third', 150);
    
    // Create a controlled race where the second promise wins
    const result = await createControlledRace([promise1, promise2, promise3], 1);
    
    // Verify the result
    expect(result).toBe('second');
  });
});