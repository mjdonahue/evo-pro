# Performance Benchmarking System

This directory contains the performance benchmarking system for the Evo Design project. The system is designed to measure and track the performance of various parts of the application over time, allowing developers to identify performance regressions and improvements.

## Overview

The benchmarking system consists of:

1. **Benchmark Tests**: TypeScript files that define and run benchmarks for specific parts of the application.
2. **Utilities**: Helper functions for measuring performance, comparing implementations, and generating reports.
3. **CI/CD Integration**: GitHub Actions workflow for running benchmarks automatically and tracking results over time.

## Running Benchmarks

### Locally

To run benchmarks locally, you can use the following command:

```bash
# Run all benchmarks
node -r ts-node/register src/__tests__/performance/example.bench.ts

# Or use the npm script
pnpm test:performance
```

### In CI/CD

Benchmarks are automatically run in the CI/CD pipeline:

1. On a schedule (weekly)
2. On pushes to the main branch that affect source code
3. Manually via GitHub Actions workflow dispatch

## Creating New Benchmarks

To create a new benchmark test:

1. Create a new file in this directory with a `.bench.ts` extension
2. Import the necessary utilities from `./utils.ts`
3. Define the implementations you want to benchmark
4. Use `measurePerformance` or `comparePerformance` to run the benchmarks
5. Save the results using the `saveBenchmarkResult` function
6. Generate a report using the `generateReport` function

Example:

```typescript
import { measurePerformance, comparePerformance } from './utils';
import { saveBenchmarkResult, generateReport } from './example.bench';

// Define implementations
const implementations = {
  implementation1: () => {
    // Your implementation here
  },
  implementation2: () => {
    // Alternative implementation
  },
};

// Run benchmarks
async function runBenchmarks() {
  const results = await comparePerformance({
    'Implementation 1': implementations.implementation1,
    'Implementation 2': implementations.implementation2,
  }, {
    iterations: 100,
    warmup: 5,
    log: true,
  });
  
  // Save results
  for (const [name, result] of Object.entries(results)) {
    await saveBenchmarkResult(name, {
      timestamp: new Date().toISOString(),
      averageTime: result.averageTime,
      minTime: result.minTime,
      maxTime: result.maxTime,
      stdDev: result.stdDev,
      iterations: result.iterations,
    });
  }
  
  // Generate report
  await generateReport();
}

// Run benchmarks if this file is executed directly
if (require.main === module) {
  runBenchmarks().catch(console.error);
}

// Export for use in other tests
export { runBenchmarks };
```

## Benchmark Results

Benchmark results are stored in the `benchmark-results` directory at the root of the project. Each benchmark has its own history file, which contains:

- The name of the benchmark
- A baseline value (average of the first 3 runs)
- A history of benchmark runs, including timestamps and performance metrics

The system also generates an HTML report that visualizes the benchmark results, showing trends over time and comparisons to the baseline.

## Interpreting Results

When interpreting benchmark results, consider the following:

1. **Absolute Values**: The absolute time values (in milliseconds) indicate how long operations take.
2. **Trends**: Changes in performance over time are more important than absolute values.
3. **Comparisons**: Comparing different implementations helps identify the most efficient approach.
4. **Variability**: The standard deviation indicates how consistent the performance is.

A good benchmark should:

- Have a low average time
- Have a low standard deviation (consistent performance)
- Show stable or improving trends over time

## Performance Budgets

Performance budgets can be defined to ensure that performance doesn't regress beyond acceptable limits. These budgets are enforced in the CI/CD pipeline:

1. If performance is within budget, the benchmark passes.
2. If performance exceeds the budget, the benchmark fails and triggers an alert.

To define a performance budget, use the `meetsPerformanceBudget` function from `./utils.ts`:

```typescript
import { meetsPerformanceBudget } from './utils';

// Check if performance meets budget
const result = await measurePerformance(() => myFunction());
const budgetMs = 50; // 50ms budget
const meetsbudget = meetsPerformanceBudget(result, budgetMs);

if (!meetsbudget) {
  console.warn(`Performance budget exceeded: ${result.averageTime}ms > ${budgetMs}ms`);
}
```

## Best Practices

When creating benchmarks:

1. **Isolate What You're Testing**: Ensure you're only measuring the code you intend to benchmark.
2. **Use Realistic Inputs**: Test with data that represents real-world usage.
3. **Warm Up**: Always include warm-up iterations to avoid measuring JIT compilation time.
4. **Run Multiple Iterations**: Single measurements are unreliable; use multiple iterations.
5. **Consider Environment**: Be aware that performance can vary based on hardware, OS, and other factors.
6. **Compare Relative Performance**: Focus on relative changes rather than absolute values.
7. **Document Assumptions**: Document any assumptions or constraints in your benchmark code.