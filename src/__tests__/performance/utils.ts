/**
 * Performance Testing Utilities
 * 
 * This module provides utilities for measuring and analyzing performance metrics
 * in both frontend and backend code.
 */

/**
 * Options for performance measurement
 */
export interface PerformanceMeasureOptions {
  /** Number of iterations to run */
  iterations?: number;
  /** Warmup iterations (not included in results) */
  warmup?: number;
  /** Whether to log results to console */
  log?: boolean;
  /** Custom label for the measurement */
  label?: string;
}

/**
 * Result of a performance measurement
 */
export interface PerformanceResult {
  /** Name of the measured function */
  name: string;
  /** Average execution time in milliseconds */
  averageTime: number;
  /** Minimum execution time in milliseconds */
  minTime: number;
  /** Maximum execution time in milliseconds */
  maxTime: number;
  /** Standard deviation of execution times */
  stdDev: number;
  /** All measured times */
  times: number[];
  /** Number of iterations */
  iterations: number;
  /** Memory usage in bytes (if available) */
  memoryUsage?: number;
}

/**
 * Measures the performance of a function
 * 
 * @param fn The function to measure
 * @param options Measurement options
 * @returns Performance measurement results
 */
export async function measurePerformance<T>(
  fn: () => T | Promise<T>,
  options: PerformanceMeasureOptions = {}
): Promise<PerformanceResult> {
  const {
    iterations = 100,
    warmup = 5,
    log = false,
    label,
  } = options;

  const name = label || fn.name || 'anonymous';
  
  // Warmup phase
  for (let i = 0; i < warmup; i++) {
    await fn();
  }
  
  // Measurement phase
  const times: number[] = [];
  let memoryBefore: number | undefined;
  let memoryAfter: number | undefined;
  
  // Check if we can measure memory
  if (typeof process !== 'undefined' && process.memoryUsage) {
    memoryBefore = process.memoryUsage().heapUsed;
  }
  
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await fn();
    const end = performance.now();
    times.push(end - start);
  }
  
  if (typeof process !== 'undefined' && process.memoryUsage) {
    memoryAfter = process.memoryUsage().heapUsed;
  }
  
  // Calculate statistics
  const averageTime = times.reduce((sum, time) => sum + time, 0) / times.length;
  const minTime = Math.min(...times);
  const maxTime = Math.max(...times);
  
  // Calculate standard deviation
  const squareDiffs = times.map(time => {
    const diff = time - averageTime;
    return diff * diff;
  });
  const avgSquareDiff = squareDiffs.reduce((sum, diff) => sum + diff, 0) / squareDiffs.length;
  const stdDev = Math.sqrt(avgSquareDiff);
  
  const result: PerformanceResult = {
    name,
    averageTime,
    minTime,
    maxTime,
    stdDev,
    times,
    iterations,
    memoryUsage: memoryAfter !== undefined && memoryBefore !== undefined 
      ? memoryAfter - memoryBefore 
      : undefined,
  };
  
  if (log) {
    console.log(`Performance test: ${name}`);
    console.log(`  Iterations: ${iterations}`);
    console.log(`  Average time: ${averageTime.toFixed(3)} ms`);
    console.log(`  Min time: ${minTime.toFixed(3)} ms`);
    console.log(`  Max time: ${maxTime.toFixed(3)} ms`);
    console.log(`  Std dev: ${stdDev.toFixed(3)} ms`);
    if (result.memoryUsage !== undefined) {
      console.log(`  Memory usage: ${(result.memoryUsage / 1024).toFixed(2)} KB`);
    }
  }
  
  return result;
}

/**
 * Compares the performance of multiple functions
 * 
 * @param fns Object mapping function names to functions
 * @param options Measurement options
 * @returns Object mapping function names to performance results
 */
export async function comparePerformance<T>(
  fns: Record<string, () => T | Promise<T>>,
  options: PerformanceMeasureOptions = {}
): Promise<Record<string, PerformanceResult>> {
  const results: Record<string, PerformanceResult> = {};
  
  for (const [name, fn] of Object.entries(fns)) {
    results[name] = await measurePerformance(fn, { ...options, label: name });
  }
  
  if (options.log) {
    console.log('Performance comparison:');
    const sortedResults = Object.entries(results).sort(
      ([, a], [, b]) => a.averageTime - b.averageTime
    );
    
    const fastest = sortedResults[0][1].averageTime;
    
    sortedResults.forEach(([name, result], index) => {
      const ratio = result.averageTime / fastest;
      const comparison = index === 0 
        ? 'fastest' 
        : `${ratio.toFixed(2)}x slower than fastest`;
      
      console.log(`  ${name}: ${result.averageTime.toFixed(3)} ms (${comparison})`);
    });
  }
  
  return results;
}

/**
 * Measures the rendering performance of a React component
 * 
 * @param renderFn Function that renders the component
 * @param options Measurement options
 * @returns Performance measurement results
 */
export async function measureRenderPerformance(
  renderFn: () => void,
  options: PerformanceMeasureOptions = {}
): Promise<PerformanceResult> {
  return measurePerformance(renderFn, {
    iterations: 50,
    warmup: 5,
    ...options,
    label: options.label || 'render',
  });
}

/**
 * Creates a performance budget for a specific metric
 * 
 * @param name Name of the metric
 * @param budget Maximum allowed value
 * @param current Current value
 * @returns Object with budget information and status
 */
export function createPerformanceBudget(
  name: string,
  budget: number,
  current: number
) {
  const percentUsed = (current / budget) * 100;
  const status = percentUsed <= 80 
    ? 'good' 
    : percentUsed <= 100 
      ? 'warning' 
      : 'exceeded';
  
  return {
    name,
    budget,
    current,
    percentUsed,
    remaining: budget - current,
    status,
  };
}

/**
 * Checks if the performance meets the budget
 * 
 * @param result Performance measurement result
 * @param budgetMs Maximum allowed average time in milliseconds
 * @returns Whether the performance meets the budget
 */
export function meetsPerformanceBudget(
  result: PerformanceResult,
  budgetMs: number
): boolean {
  return result.averageTime <= budgetMs;
}