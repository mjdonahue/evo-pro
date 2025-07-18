/**
 * Example Benchmark Tests
 * 
 * This file contains benchmark tests for various algorithms and operations.
 * It demonstrates how to use the benchmarking utilities to measure and compare
 * the performance of different implementations.
 */

import { measurePerformance, comparePerformance } from './utils';
import * as fs from 'fs';
import * as path from 'path';

// Define benchmark history interface
interface BenchmarkEntry {
  timestamp: string;
  averageTime: number;
  minTime: number;
  maxTime: number;
  stdDev: number;
  iterations: number;
}

interface BenchmarkHistory {
  name: string;
  baseline: number | null;
  entries: BenchmarkEntry[];
}

// Utility to save benchmark results
async function saveBenchmarkResult(name: string, result: BenchmarkEntry) {
  const resultsDir = path.resolve(process.cwd(), 'benchmark-results');
  
  // Ensure directory exists
  if (!fs.existsSync(resultsDir)) {
    fs.mkdirSync(resultsDir, { recursive: true });
  }
  
  const historyFile = path.join(resultsDir, `${name.replace(/\s+/g, '-').toLowerCase()}-history.json`);
  
  let history: BenchmarkHistory;
  
  // Load existing history or create new
  if (fs.existsSync(historyFile)) {
    history = JSON.parse(fs.readFileSync(historyFile, 'utf8'));
  } else {
    history = {
      name,
      baseline: null,
      entries: []
    };
  }
  
  // Add new entry at the beginning
  history.entries.unshift(result);
  
  // Keep only the last 20 entries
  if (history.entries.length > 20) {
    history.entries = history.entries.slice(0, 20);
  }
  
  // Set baseline if not already set
  if (history.baseline === null && history.entries.length >= 3) {
    // Use average of first 3 entries as baseline
    history.baseline = history.entries.slice(0, 3).reduce((sum, entry) => sum + entry.averageTime, 0) / 3;
  }
  
  // Save history
  fs.writeFileSync(historyFile, JSON.stringify(history, null, 2));
  
  return history;
}

// Generate HTML report for all benchmarks
async function generateReport() {
  const resultsDir = path.resolve(process.cwd(), 'benchmark-results');
  
  if (!fs.existsSync(resultsDir)) {
    return;
  }
  
  const historyFiles = fs.readdirSync(resultsDir).filter(file => file.endsWith('-history.json'));
  
  if (historyFiles.length === 0) {
    return;
  }
  
  const histories: BenchmarkHistory[] = historyFiles.map(file => 
    JSON.parse(fs.readFileSync(path.join(resultsDir, file), 'utf8'))
  );
  
  // Generate HTML report
  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Performance Benchmark Report</title>
  <style>
    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
      line-height: 1.6;
      color: #333;
      max-width: 1200px;
      margin: 0 auto;
      padding: 20px;
    }
    h1, h2, h3 {
      color: #2c3e50;
    }
    .benchmark {
      margin-bottom: 40px;
      border: 1px solid #e9ecef;
      border-radius: 5px;
      padding: 20px;
      box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }
    .benchmark-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 15px;
    }
    .benchmark-title {
      margin: 0;
      font-size: 18px;
    }
    .benchmark-stats {
      display: flex;
      gap: 20px;
      margin-bottom: 20px;
    }
    .stat {
      background-color: #f8f9fa;
      padding: 10px;
      border-radius: 5px;
      min-width: 120px;
    }
    .stat-label {
      font-size: 12px;
      color: #6c757d;
      margin-bottom: 5px;
    }
    .stat-value {
      font-size: 16px;
      font-weight: bold;
    }
    .trend {
      color: #6c757d;
    }
    .trend.better {
      color: #28a745;
    }
    .trend.worse {
      color: #dc3545;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin: 20px 0;
    }
    th, td {
      padding: 12px 15px;
      text-align: left;
      border-bottom: 1px solid #e9ecef;
    }
    th {
      background-color: #f8f9fa;
    }
    .timestamp {
      color: #6c757d;
      font-size: 14px;
      text-align: right;
      margin-top: 30px;
    }
  </style>
</head>
<body>
  <h1>Performance Benchmark Report</h1>
  <p class="timestamp">Generated on ${new Date().toLocaleString()}</p>
  
  ${histories.map(history => {
    const latestEntry = history.entries[0];
    const previousEntry = history.entries[1];
    
    let trendClass = '';
    let trendText = '';
    
    if (previousEntry) {
      const diff = ((latestEntry.averageTime - previousEntry.averageTime) / previousEntry.averageTime) * 100;
      trendClass = diff < 0 ? 'better' : diff > 0 ? 'worse' : '';
      trendText = diff === 0 ? 'no change' : `${Math.abs(diff).toFixed(2)}% ${diff < 0 ? 'faster' : 'slower'}`;
    }
    
    let baselineTrendClass = '';
    let baselineTrendText = '';
    
    if (history.baseline !== null) {
      const diff = ((latestEntry.averageTime - history.baseline) / history.baseline) * 100;
      baselineTrendClass = diff < 0 ? 'better' : diff > 0 ? 'worse' : '';
      baselineTrendText = diff === 0 ? 'no change' : `${Math.abs(diff).toFixed(2)}% ${diff < 0 ? 'faster' : 'slower'} than baseline`;
    }
    
    return `
    <div class="benchmark">
      <div class="benchmark-header">
        <h2 class="benchmark-title">${history.name}</h2>
        <span class="trend ${trendClass}">${trendText}</span>
      </div>
      
      <div class="benchmark-stats">
        <div class="stat">
          <div class="stat-label">Latest</div>
          <div class="stat-value">${latestEntry.averageTime.toFixed(3)} ms</div>
        </div>
        
        <div class="stat">
          <div class="stat-label">Min</div>
          <div class="stat-value">${latestEntry.minTime.toFixed(3)} ms</div>
        </div>
        
        <div class="stat">
          <div class="stat-label">Max</div>
          <div class="stat-value">${latestEntry.maxTime.toFixed(3)} ms</div>
        </div>
        
        <div class="stat">
          <div class="stat-label">Std Dev</div>
          <div class="stat-value">${latestEntry.stdDev.toFixed(3)} ms</div>
        </div>
        
        ${history.baseline !== null ? `
        <div class="stat">
          <div class="stat-label">Baseline</div>
          <div class="stat-value">${history.baseline.toFixed(3)} ms</div>
          <div class="trend ${baselineTrendClass}">${baselineTrendText}</div>
        </div>
        ` : ''}
      </div>
      
      <h3>History</h3>
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Avg Time (ms)</th>
            <th>Min Time (ms)</th>
            <th>Max Time (ms)</th>
            <th>Std Dev</th>
          </tr>
        </thead>
        <tbody>
          ${history.entries.map(entry => `
          <tr>
            <td>${new Date(entry.timestamp).toLocaleString()}</td>
            <td>${entry.averageTime.toFixed(3)}</td>
            <td>${entry.minTime.toFixed(3)}</td>
            <td>${entry.maxTime.toFixed(3)}</td>
            <td>${entry.stdDev.toFixed(3)}</td>
          </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
    `;
  }).join('')}
</body>
</html>
  `;
  
  fs.writeFileSync(path.join(resultsDir, 'benchmark-report.html'), html);
}

// Benchmark implementations

// Fibonacci implementations
const fibonacciImplementations = {
  recursive: (n: number): number => {
    if (n <= 1) return n;
    return fibonacciImplementations.recursive(n - 1) + fibonacciImplementations.recursive(n - 2);
  },
  
  memoized: (() => {
    const cache: Record<number, number> = {};
    return (n: number): number => {
      if (n in cache) return cache[n];
      if (n <= 1) return n;
      cache[n] = fibonacciImplementations.memoized(n - 1) + fibonacciImplementations.memoized(n - 2);
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

// Sorting implementations
const sortImplementations = {
  builtIn: (arr: number[]): number[] => {
    return [...arr].sort((a, b) => a - b);
  },
  
  quickSort: (arr: number[]): number[] => {
    if (arr.length <= 1) return arr;
    
    const pivot = arr[Math.floor(arr.length / 2)];
    const left = arr.filter(x => x < pivot);
    const middle = arr.filter(x => x === pivot);
    const right = arr.filter(x => x > pivot);
    
    return [...sortImplementations.quickSort(left), ...middle, ...sortImplementations.quickSort(right)];
  },
  
  mergeSort: (arr: number[]): number[] => {
    if (arr.length <= 1) return arr;
    
    const mid = Math.floor(arr.length / 2);
    const left = sortImplementations.mergeSort(arr.slice(0, mid));
    const right = sortImplementations.mergeSort(arr.slice(mid));
    
    return merge(left, right);
    
    function merge(left: number[], right: number[]): number[] {
      const result: number[] = [];
      let i = 0, j = 0;
      
      while (i < left.length && j < right.length) {
        if (left[i] < right[j]) {
          result.push(left[i++]);
        } else {
          result.push(right[j++]);
        }
      }
      
      return [...result, ...left.slice(i), ...right.slice(j)];
    }
  },
};

// String operations
const stringOperations = {
  concatenation: (strings: string[]): string => {
    let result = '';
    for (const str of strings) {
      result += str;
    }
    return result;
  },
  
  join: (strings: string[]): string => {
    return strings.join('');
  },
  
  templateLiteral: (strings: string[]): string => {
    return `${strings.join('')}`;
  },
};

// Run benchmarks and save results
async function runBenchmarks() {
  console.log('Running benchmarks...');
  
  // Fibonacci benchmarks
  console.log('\nFibonacci Benchmarks:');
  const fibResults = await comparePerformance({
    'Fibonacci Recursive (n=15)': () => fibonacciImplementations.recursive(15),
    'Fibonacci Memoized (n=35)': () => fibonacciImplementations.memoized(35),
    'Fibonacci Iterative (n=35)': () => fibonacciImplementations.iterative(35),
  }, {
    iterations: 100,
    warmup: 5,
    log: true,
  });
  
  // Sorting benchmarks
  console.log('\nSorting Benchmarks:');
  const randomArray = Array.from({ length: 1000 }, () => Math.floor(Math.random() * 1000));
  const sortResults = await comparePerformance({
    'Array.sort (1000 items)': () => sortImplementations.builtIn(randomArray),
    'Quick Sort (1000 items)': () => sortImplementations.quickSort(randomArray),
    'Merge Sort (1000 items)': () => sortImplementations.mergeSort(randomArray),
  }, {
    iterations: 50,
    warmup: 5,
    log: true,
  });
  
  // String operations benchmarks
  console.log('\nString Operations Benchmarks:');
  const strings = Array.from({ length: 1000 }, () => 'a');
  const stringResults = await comparePerformance({
    'String Concatenation (1000 items)': () => stringOperations.concatenation(strings),
    'Array.join (1000 items)': () => stringOperations.join(strings),
    'Template Literal (1000 items)': () => stringOperations.templateLiteral(strings),
  }, {
    iterations: 100,
    warmup: 5,
    log: true,
  });
  
  // Save results
  console.log('\nSaving benchmark results...');
  
  for (const [name, result] of Object.entries(fibResults)) {
    await saveBenchmarkResult(name, {
      timestamp: new Date().toISOString(),
      averageTime: result.averageTime,
      minTime: result.minTime,
      maxTime: result.maxTime,
      stdDev: result.stdDev,
      iterations: result.iterations,
    });
  }
  
  for (const [name, result] of Object.entries(sortResults)) {
    await saveBenchmarkResult(name, {
      timestamp: new Date().toISOString(),
      averageTime: result.averageTime,
      minTime: result.minTime,
      maxTime: result.maxTime,
      stdDev: result.stdDev,
      iterations: result.iterations,
    });
  }
  
  for (const [name, result] of Object.entries(stringResults)) {
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
  console.log('Generating benchmark report...');
  await generateReport();
  
  console.log('Benchmarks completed successfully.');
}

// Run benchmarks if this file is executed directly
if (require.main === module) {
  runBenchmarks().catch(console.error);
}

// Export for use in other tests
export { runBenchmarks };