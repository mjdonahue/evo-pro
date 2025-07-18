import { describe, it, expect } from 'vitest';
import { 
  measurePerformance, 
  comparePerformance, 
  measureRenderPerformance,
  createPerformanceBudget,
  meetsPerformanceBudget
} from './utils';
import { render, screen } from '@testing-library/react';
import React from 'react';

/**
 * Example performance tests
 * 
 * These tests demonstrate how to use the performance testing utilities
 * to measure and compare the performance of different implementations.
 */
describe('Performance Tests', () => {
  /**
   * Example of measuring the performance of a function
   */
  it('measures function performance', async () => {
    // Function to measure
    const fibonacci = (n: number): number => {
      if (n <= 1) return n;
      return fibonacci(n - 1) + fibonacci(n - 2);
    };
    
    // Measure performance
    const result = await measurePerformance(() => fibonacci(15), {
      iterations: 20,
      warmup: 2,
      log: true,
    });
    
    // Verify that we got valid measurements
    expect(result.averageTime).toBeGreaterThan(0);
    expect(result.iterations).toBe(20);
    expect(result.times.length).toBe(20);
    
    // Check against a performance budget
    const budget = 50; // ms
    expect(meetsPerformanceBudget(result, budget)).toBe(true);
  });
  
  /**
   * Example of comparing the performance of different implementations
   */
  it('compares different implementations', async () => {
    // Different implementations of the same functionality
    const implementations = {
      recursive: (n: number): number => {
        if (n <= 1) return n;
        return implementations.recursive(n - 1) + implementations.recursive(n - 2);
      },
      
      memoized: (() => {
        const cache: Record<number, number> = {};
        return (n: number): number => {
          if (n in cache) return cache[n];
          if (n <= 1) return n;
          cache[n] = implementations.memoized(n - 1) + implementations.memoized(n - 2);
          return cache[n];
        };
      })(),
      
      iterative: (n: number): number => {
        if (n <= 1) return n;
        let a = 0, b = 1;
        for (let i = 2; i <= n; i++) {
          const c = a + b;
          a = b;
          b = c;
        }
        return b;
      },
    };
    
    // Compare performance
    const results = await comparePerformance({
      recursive: () => implementations.recursive(15),
      memoized: () => implementations.memoized(20),
      iterative: () => implementations.iterative(1000),
    }, {
      iterations: 10,
      warmup: 2,
      log: true,
    });
    
    // Verify that we got results for all implementations
    expect(Object.keys(results)).toEqual(['recursive', 'memoized', 'iterative']);
    
    // The iterative implementation should be the fastest for large inputs
    expect(results.iterative.averageTime).toBeLessThan(results.recursive.averageTime);
  });
  
  /**
   * Example of measuring React component rendering performance
   */
  it('measures React component rendering performance', async () => {
    // Simple component to test
    const TestComponent: React.FC<{ items: string[] }> = ({ items }) => (
      <div>
        <h1>Test Component</h1>
        <ul>
          {items.map((item, index) => (
            <li key={index}>{item}</li>
          ))}
        </ul>
      </div>
    );
    
    // Generate test data
    const smallList = Array.from({ length: 10 }, (_, i) => `Item ${i}`);
    const largeList = Array.from({ length: 100 }, (_, i) => `Item ${i}`);
    
    // Measure rendering performance with different list sizes
    const smallListResult = await measureRenderPerformance(
      () => render(<TestComponent items={smallList} />),
      { iterations: 10, log: true }
    );
    
    // Clean up after each render
    screen.unmount();
    
    const largeListResult = await measureRenderPerformance(
      () => render(<TestComponent items={largeList} />),
      { iterations: 10, log: true }
    );
    
    // Verify that rendering a larger list takes more time
    expect(largeListResult.averageTime).toBeGreaterThan(smallListResult.averageTime);
    
    // Create performance budgets
    const smallListBudget = createPerformanceBudget(
      'Small list rendering',
      10, // ms
      smallListResult.averageTime
    );
    
    const largeListBudget = createPerformanceBudget(
      'Large list rendering',
      30, // ms
      largeListResult.averageTime
    );
    
    // Log budget information
    console.log('Performance budgets:');
    console.log(`  ${smallListBudget.name}: ${smallListBudget.status} (${smallListBudget.percentUsed.toFixed(1)}% of budget)`);
    console.log(`  ${largeListBudget.name}: ${largeListBudget.status} (${largeListBudget.percentUsed.toFixed(1)}% of budget)`);
  });
});