/**
 * Performance Benchmarking Utilities
 * 
 * This module provides utilities for running, storing, and comparing performance benchmarks.
 * It extends the performance testing utilities to support benchmarking in CI/CD environments.
 */

import fs from 'fs';
import path from 'path';
import { PerformanceResult, measurePerformance, comparePerformance } from './utils';

/**
 * Options for benchmark execution
 */
export interface BenchmarkOptions {
  /** Name of the benchmark */
  name: string;
  /** Description of the benchmark */
  description?: string;
  /** Category of the benchmark (e.g., 'frontend', 'backend', 'database') */
  category?: string;
  /** Tags for the benchmark */
  tags?: string[];
  /** Number of iterations to run */
  iterations?: number;
  /** Warmup iterations (not included in results) */
  warmup?: number;
  /** Whether to log results to console */
  log?: boolean;
  /** Baseline to compare against (in ms) */
  baseline?: number;
  /** Threshold for regression alerts (percentage) */
  regressionThreshold?: number;
}

/**
 * Result of a benchmark run
 */
export interface BenchmarkResult extends PerformanceResult {
  /** Name of the benchmark */
  benchmarkName: string;
  /** Description of the benchmark */
  description?: string;
  /** Category of the benchmark */
  category?: string;
  /** Tags for the benchmark */
  tags?: string[];
  /** Timestamp of the benchmark run */
  timestamp: string;
  /** Git commit hash (if available) */
  commitHash?: string;
  /** Git branch (if available) */
  branch?: string;
  /** Baseline to compare against (in ms) */
  baseline?: number;
  /** Percentage difference from baseline */
  baselineDiff?: number;
  /** Whether the benchmark passed the regression threshold */
  passed?: boolean;
  /** Environment information */
  environment: {
    /** Node.js version */
    nodeVersion: string;
    /** Operating system */
    os: string;
    /** CPU information */
    cpu: string;
    /** Memory information */
    memory: string;
  };
}

/**
 * Benchmark history entry
 */
export interface BenchmarkHistoryEntry {
  /** Timestamp of the benchmark run */
  timestamp: string;
  /** Average execution time in milliseconds */
  averageTime: number;
  /** Minimum execution time in milliseconds */
  minTime: number;
  /** Maximum execution time in milliseconds */
  maxTime: number;
  /** Standard deviation of execution times */
  stdDev: number;
  /** Git commit hash (if available) */
  commitHash?: string;
  /** Git branch (if available) */
  branch?: string;
  /** Whether the benchmark passed the regression threshold */
  passed?: boolean;
}

/**
 * Benchmark history
 */
export interface BenchmarkHistory {
  /** Name of the benchmark */
  name: string;
  /** Description of the benchmark */
  description?: string;
  /** Category of the benchmark */
  category?: string;
  /** Tags for the benchmark */
  tags?: string[];
  /** Baseline to compare against (in ms) */
  baseline?: number;
  /** Regression threshold (percentage) */
  regressionThreshold?: number;
  /** History entries */
  entries: BenchmarkHistoryEntry[];
}

/**
 * Default directory for storing benchmark results
 */
export const DEFAULT_BENCHMARK_DIR = path.resolve(process.cwd(), 'benchmark-results');

/**
 * Gets the current Git commit hash and branch
 * @returns Object with commitHash and branch
 */
export function getGitInfo(): { commitHash?: string; branch?: string } {
  try {
    // Try to get Git commit hash
    const commitHash = require('child_process')
      .execSync('git rev-parse HEAD')
      .toString()
      .trim();
    
    // Try to get Git branch
    const branch = require('child_process')
      .execSync('git rev-parse --abbrev-ref HEAD')
      .toString()
      .trim();
    
    return { commitHash, branch };
  } catch (error) {
    console.warn('Failed to get Git info:', error);
    return {};
  }
}

/**
 * Gets environment information
 * @returns Object with environment information
 */
export function getEnvironmentInfo(): BenchmarkResult['environment'] {
  const os = require('os');
  
  return {
    nodeVersion: process.version,
    os: `${os.type()} ${os.release()}`,
    cpu: os.cpus()[0]?.model || 'Unknown',
    memory: `${Math.round(os.totalmem() / (1024 * 1024 * 1024))} GB`,
  };
}

/**
 * Runs a benchmark and returns the result
 * 
 * @param fn The function to benchmark
 * @param options Benchmark options
 * @returns Benchmark result
 */
export async function runBenchmark<T>(
  fn: () => T | Promise<T>,
  options: BenchmarkOptions
): Promise<BenchmarkResult> {
  const {
    name,
    description,
    category,
    tags,
    iterations,
    warmup,
    log,
    baseline,
    regressionThreshold = 10, // Default 10% regression threshold
  } = options;
  
  // Run the performance measurement
  const result = await measurePerformance(fn, {
    iterations,
    warmup,
    log,
    label: name,
  });
  
  // Get Git and environment information
  const { commitHash, branch } = getGitInfo();
  const environment = getEnvironmentInfo();
  
  // Calculate baseline difference if baseline is provided
  let baselineDiff;
  let passed = true;
  
  if (baseline) {
    baselineDiff = ((result.averageTime - baseline) / baseline) * 100;
    passed = baselineDiff <= regressionThreshold;
  }
  
  // Create the benchmark result
  const benchmarkResult: BenchmarkResult = {
    ...result,
    benchmarkName: name,
    description,
    category,
    tags,
    timestamp: new Date().toISOString(),
    commitHash,
    branch,
    baseline,
    baselineDiff,
    passed,
    environment,
  };
  
  // Log the benchmark result
  if (log) {
    console.log(`Benchmark: ${name}`);
    console.log(`  Average time: ${result.averageTime.toFixed(3)} ms`);
    
    if (baseline) {
      const diffStr = baselineDiff >= 0 ? `+${baselineDiff.toFixed(2)}%` : `${baselineDiff.toFixed(2)}%`;
      const status = passed ? 'PASS' : 'FAIL';
      console.log(`  Baseline: ${baseline.toFixed(3)} ms (${diffStr}) - ${status}`);
    }
    
    console.log(`  Min time: ${result.minTime.toFixed(3)} ms`);
    console.log(`  Max time: ${result.maxTime.toFixed(3)} ms`);
    console.log(`  Std dev: ${result.stdDev.toFixed(3)} ms`);
    console.log(`  Commit: ${commitHash || 'N/A'}`);
    console.log(`  Branch: ${branch || 'N/A'}`);
  }
  
  return benchmarkResult;
}

/**
 * Compares multiple implementations and returns benchmark results
 * 
 * @param fns Object mapping implementation names to functions
 * @param options Benchmark options
 * @returns Object mapping implementation names to benchmark results
 */
export async function compareBenchmarks<T>(
  fns: Record<string, () => T | Promise<T>>,
  options: BenchmarkOptions
): Promise<Record<string, BenchmarkResult>> {
  const results: Record<string, BenchmarkResult> = {};
  
  // Get Git and environment information
  const { commitHash, branch } = getGitInfo();
  const environment = getEnvironmentInfo();
  
  // Run performance comparison
  const perfResults = await comparePerformance(fns, {
    iterations: options.iterations,
    warmup: options.warmup,
    log: options.log,
  });
  
  // Convert performance results to benchmark results
  for (const [name, result] of Object.entries(perfResults)) {
    // Calculate baseline difference if baseline is provided
    let baselineDiff;
    let passed = true;
    
    if (options.baseline) {
      baselineDiff = ((result.averageTime - options.baseline) / options.baseline) * 100;
      passed = baselineDiff <= (options.regressionThreshold || 10);
    }
    
    results[name] = {
      ...result,
      benchmarkName: `${options.name} - ${name}`,
      description: options.description,
      category: options.category,
      tags: options.tags,
      timestamp: new Date().toISOString(),
      commitHash,
      branch,
      baseline: options.baseline,
      baselineDiff,
      passed,
      environment,
    };
  }
  
  return results;
}

/**
 * Saves benchmark results to a file
 * 
 * @param result Benchmark result to save
 * @param dir Directory to save results in
 * @returns Path to the saved file
 */
export function saveBenchmarkResult(
  result: BenchmarkResult,
  dir: string = DEFAULT_BENCHMARK_DIR
): string {
  // Ensure the directory exists
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
  
  // Create a filename based on the benchmark name and timestamp
  const timestamp = new Date(result.timestamp).toISOString().replace(/:/g, '-');
  const filename = `${result.benchmarkName.replace(/\s+/g, '-')}-${timestamp}.json`;
  const filepath = path.join(dir, filename);
  
  // Save the result to a file
  fs.writeFileSync(filepath, JSON.stringify(result, null, 2));
  
  return filepath;
}

/**
 * Updates the benchmark history with a new result
 * 
 * @param result Benchmark result to add to history
 * @param dir Directory where history is stored
 * @returns Path to the history file
 */
export function updateBenchmarkHistory(
  result: BenchmarkResult,
  dir: string = DEFAULT_BENCHMARK_DIR
): string {
  // Ensure the directory exists
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
  
  // Create a history filename based on the benchmark name
  const historyFilename = `${result.benchmarkName.replace(/\s+/g, '-')}-history.json`;
  const historyFilepath = path.join(dir, historyFilename);
  
  // Load existing history or create a new one
  let history: BenchmarkHistory;
  
  if (fs.existsSync(historyFilepath)) {
    history = JSON.parse(fs.readFileSync(historyFilepath, 'utf8'));
  } else {
    history = {
      name: result.benchmarkName,
      description: result.description,
      category: result.category,
      tags: result.tags,
      baseline: result.baseline,
      regressionThreshold: 10, // Default 10% regression threshold
      entries: [],
    };
  }
  
  // Add the new entry
  history.entries.push({
    timestamp: result.timestamp,
    averageTime: result.averageTime,
    minTime: result.minTime,
    maxTime: result.maxTime,
    stdDev: result.stdDev,
    commitHash: result.commitHash,
    branch: result.branch,
    passed: result.passed,
  });
  
  // Sort entries by timestamp (newest first)
  history.entries.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());
  
  // Update the baseline if it's not set
  if (!history.baseline && history.entries.length > 0) {
    history.baseline = history.entries[0].averageTime;
  }
  
  // Save the updated history
  fs.writeFileSync(historyFilepath, JSON.stringify(history, null, 2));
  
  return historyFilepath;
}

/**
 * Loads benchmark history from a file
 * 
 * @param benchmarkName Name of the benchmark
 * @param dir Directory where history is stored
 * @returns Benchmark history or null if not found
 */
export function loadBenchmarkHistory(
  benchmarkName: string,
  dir: string = DEFAULT_BENCHMARK_DIR
): BenchmarkHistory | null {
  const historyFilename = `${benchmarkName.replace(/\s+/g, '-')}-history.json`;
  const historyFilepath = path.join(dir, historyFilename);
  
  if (fs.existsSync(historyFilepath)) {
    return JSON.parse(fs.readFileSync(historyFilepath, 'utf8'));
  }
  
  return null;
}

/**
 * Generates a benchmark report in HTML format
 * 
 * @param histories Array of benchmark histories to include in the report
 * @param outputPath Path to save the HTML report
 * @returns Path to the generated report
 */
export function generateBenchmarkReport(
  histories: BenchmarkHistory[],
  outputPath: string = path.join(DEFAULT_BENCHMARK_DIR, 'benchmark-report.html')
): string {
  // Ensure the directory exists
  const dir = path.dirname(outputPath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
  
  // Generate HTML content
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
      border-bottom: 1px solid #e9ecef;
      padding-bottom: 10px;
    }
    .benchmark-title {
      margin: 0;
      font-size: 20px;
    }
    .benchmark-category {
      background-color: #f8f9fa;
      padding: 3px 8px;
      border-radius: 3px;
      font-size: 14px;
    }
    .benchmark-description {
      margin-bottom: 20px;
      color: #6c757d;
    }
    .benchmark-tags {
      display: flex;
      flex-wrap: wrap;
      gap: 5px;
      margin-bottom: 15px;
    }
    .benchmark-tag {
      background-color: #e9ecef;
      padding: 2px 8px;
      border-radius: 3px;
      font-size: 12px;
    }
    .benchmark-stats {
      display: flex;
      flex-wrap: wrap;
      gap: 20px;
      margin-bottom: 20px;
    }
    .benchmark-stat {
      flex: 1;
      min-width: 150px;
    }
    .benchmark-stat-value {
      font-size: 24px;
      font-weight: bold;
      margin-bottom: 5px;
    }
    .benchmark-stat-label {
      font-size: 14px;
      color: #6c757d;
    }
    .chart-container {
      width: 100%;
      height: 300px;
      margin-bottom: 20px;
    }
    table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    th, td {
      padding: 10px;
      text-align: left;
      border-bottom: 1px solid #e9ecef;
    }
    th {
      background-color: #f8f9fa;
      font-weight: bold;
    }
    tr:nth-child(even) {
      background-color: #f8f9fa;
    }
    .passed {
      color: #28a745;
    }
    .failed {
      color: #dc3545;
    }
    .timestamp {
      color: #6c757d;
      font-size: 14px;
      text-align: right;
      margin-top: 30px;
    }
  </style>
  <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
</head>
<body>
  <h1>Performance Benchmark Report</h1>
  <p class="timestamp">Generated on ${new Date().toLocaleString()}</p>
  
  ${histories.map(history => `
    <div class="benchmark">
      <div class="benchmark-header">
        <h2 class="benchmark-title">${history.name}</h2>
        ${history.category ? `<span class="benchmark-category">${history.category}</span>` : ''}
      </div>
      
      ${history.description ? `<p class="benchmark-description">${history.description}</p>` : ''}
      
      ${history.tags && history.tags.length > 0 ? `
        <div class="benchmark-tags">
          ${history.tags.map(tag => `<span class="benchmark-tag">${tag}</span>`).join('')}
        </div>
      ` : ''}
      
      <div class="benchmark-stats">
        <div class="benchmark-stat">
          <div class="benchmark-stat-value">${history.entries[0]?.averageTime.toFixed(2)} ms</div>
          <div class="benchmark-stat-label">Latest Average Time</div>
        </div>
        
        <div class="benchmark-stat">
          <div class="benchmark-stat-value">${history.baseline?.toFixed(2) || 'N/A'} ms</div>
          <div class="benchmark-stat-label">Baseline</div>
        </div>
        
        <div class="benchmark-stat">
          <div class="benchmark-stat-value ${history.entries[0]?.passed ? 'passed' : 'failed'}">
            ${history.entries[0]?.passed ? 'PASS' : 'FAIL'}
          </div>
          <div class="benchmark-stat-label">Status</div>
        </div>
        
        <div class="benchmark-stat">
          <div class="benchmark-stat-value">
            ${history.entries.length > 1 
              ? ((history.entries[0].averageTime - history.entries[1].averageTime) / history.entries[1].averageTime * 100).toFixed(2) + '%'
              : 'N/A'
            }
          </div>
          <div class="benchmark-stat-label">Change from Previous</div>
        </div>
      </div>
      
      <div class="chart-container">
        <canvas id="chart-${history.name.replace(/\s+/g, '-')}"></canvas>
      </div>
      
      <h3>History</h3>
      <table>
        <thead>
          <tr>
            <th>Timestamp</th>
            <th>Average (ms)</th>
            <th>Min (ms)</th>
            <th>Max (ms)</th>
            <th>Std Dev</th>
            <th>Commit</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          ${history.entries.map(entry => `
            <tr>
              <td>${new Date(entry.timestamp).toLocaleString()}</td>
              <td>${entry.averageTime.toFixed(2)}</td>
              <td>${entry.minTime.toFixed(2)}</td>
              <td>${entry.maxTime.toFixed(2)}</td>
              <td>${entry.stdDev.toFixed(2)}</td>
              <td>${entry.commitHash?.substring(0, 7) || 'N/A'}</td>
              <td class="${entry.passed ? 'passed' : 'failed'}">${entry.passed ? 'PASS' : 'FAIL'}</td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    </div>
  `).join('')}
  
  <script>
    // Initialize charts
    document.addEventListener('DOMContentLoaded', function() {
      ${histories.map(history => `
        const ctx${history.name.replace(/\s+/g, '-')} = document.getElementById('chart-${history.name.replace(/\s+/g, '-')}').getContext('2d');
        new Chart(ctx${history.name.replace(/\s+/g, '-')}, {
          type: 'line',
          data: {
            labels: ${JSON.stringify(history.entries.map(entry => new Date(entry.timestamp).toLocaleDateString()).reverse())},
            datasets: [{
              label: 'Average Time (ms)',
              data: ${JSON.stringify(history.entries.map(entry => entry.averageTime).reverse())},
              borderColor: 'rgb(75, 192, 192)',
              tension: 0.1,
              fill: false
            },
            {
              label: 'Min Time (ms)',
              data: ${JSON.stringify(history.entries.map(entry => entry.minTime).reverse())},
              borderColor: 'rgb(54, 162, 235)',
              tension: 0.1,
              fill: false
            },
            {
              label: 'Max Time (ms)',
              data: ${JSON.stringify(history.entries.map(entry => entry.maxTime).reverse())},
              borderColor: 'rgb(255, 99, 132)',
              tension: 0.1,
              fill: false
            }]
          },
          options: {
            responsive: true,
            scales: {
              y: {
                beginAtZero: true,
                title: {
                  display: true,
                  text: 'Time (ms)'
                }
              },
              x: {
                title: {
                  display: true,
                  text: 'Date'
                }
              }
            }
          }
        });
      `).join('')}
    });
  </script>
</body>
</html>
  `;
  
  // Save the HTML report
  fs.writeFileSync(outputPath, html);
  
  return outputPath;
}

/**
 * Runs a benchmark, saves the result, updates the history, and optionally generates a report
 * 
 * @param fn The function to benchmark
 * @param options Benchmark options
 * @param generateReport Whether to generate an HTML report
 * @returns Benchmark result
 */
export async function benchmark<T>(
  fn: () => T | Promise<T>,
  options: BenchmarkOptions,
  generateReport: boolean = false
): Promise<BenchmarkResult> {
  // Run the benchmark
  const result = await runBenchmark(fn, options);
  
  // Save the result
  saveBenchmarkResult(result);
  
  // Update the history
  updateBenchmarkHistory(result);
  
  // Generate a report if requested
  if (generateReport) {
    const history = loadBenchmarkHistory(options.name);
    if (history) {
      generateBenchmarkReport([history]);
    }
  }
  
  return result;
}