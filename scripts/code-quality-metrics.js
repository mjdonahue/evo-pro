/**
 * Code Quality Metrics Collection Script
 * 
 * This script collects various code quality metrics and generates a comprehensive report.
 * It uses existing tools like ESLint, ts-complexity, and test coverage, and combines
 * the results into a single report.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const config = {
  outputDir: path.resolve(__dirname, '../code-quality-reports'),
  metricsFile: 'metrics.json',
  reportFile: 'report.html',
  thresholds: {
    eslintErrors: 0,
    eslintWarnings: 10,
    complexity: 15,
    coverage: {
      lines: 80,
      functions: 80,
      branches: 70,
      statements: 80
    },
    duplications: 3 // percentage
  }
};

// Ensure the output directory exists
if (!fs.existsSync(config.outputDir)) {
  fs.mkdirSync(config.outputDir, { recursive: true });
}

// Collect metrics
const metrics = {
  timestamp: new Date().toISOString(),
  eslint: collectEslintMetrics(),
  complexity: collectComplexityMetrics(),
  coverage: collectCoverageMetrics(),
  duplications: collectDuplicationMetrics(),
  typescript: collectTypeScriptMetrics(),
  rust: collectRustMetrics(),
  summary: {}
};

// Calculate summary metrics
calculateSummary();

// Save metrics to file
fs.writeFileSync(
  path.join(config.outputDir, config.metricsFile),
  JSON.stringify(metrics, null, 2)
);

// Generate HTML report
generateHtmlReport();

console.log(`Code quality metrics collected and saved to ${config.outputDir}`);

/**
 * Collects ESLint metrics
 */
function collectEslintMetrics() {
  console.log('Collecting ESLint metrics...');
  
  try {
    // Run ESLint with JSON output format
    const output = execSync('npx eslint --format json --max-warnings=1000 "src/**/*.{ts,tsx}"', { encoding: 'utf8' });
    const eslintResults = JSON.parse(output);
    
    // Calculate metrics
    let errorCount = 0;
    let warningCount = 0;
    let fileCount = eslintResults.length;
    let filesWithIssues = 0;
    
    eslintResults.forEach(result => {
      errorCount += result.errorCount;
      warningCount += result.warningCount;
      if (result.errorCount > 0 || result.warningCount > 0) {
        filesWithIssues++;
      }
    });
    
    return {
      errorCount,
      warningCount,
      fileCount,
      filesWithIssues,
      issueRatio: fileCount > 0 ? filesWithIssues / fileCount : 0,
      status: errorCount > config.thresholds.eslintErrors || warningCount > config.thresholds.eslintWarnings ? 'fail' : 'pass'
    };
  } catch (error) {
    console.error('Error collecting ESLint metrics:', error.message);
    return {
      errorCount: 0,
      warningCount: 0,
      fileCount: 0,
      filesWithIssues: 0,
      issueRatio: 0,
      status: 'error',
      error: error.message
    };
  }
}

/**
 * Collects code complexity metrics
 */
function collectComplexityMetrics() {
  console.log('Collecting complexity metrics...');
  
  try {
    // Run ts-complexity with JSON output
    const output = execSync(
      'npx ts-complexity --max-complexity 1000 --patterns "src/**/*.{ts,tsx}" --exclude "**/*.test.{ts,tsx}" --output-format json',
      { encoding: 'utf8' }
    );
    
    const complexityResults = JSON.parse(output);
    
    // Calculate metrics
    let totalComplexity = 0;
    let fileCount = complexityResults.length;
    let highComplexityCount = 0;
    
    complexityResults.forEach(result => {
      totalComplexity += result.complexity;
      if (result.complexity > config.thresholds.complexity) {
        highComplexityCount++;
      }
    });
    
    const averageComplexity = fileCount > 0 ? totalComplexity / fileCount : 0;
    
    return {
      averageComplexity,
      highComplexityCount,
      fileCount,
      highComplexityRatio: fileCount > 0 ? highComplexityCount / fileCount : 0,
      status: highComplexityCount > 0 ? 'warning' : 'pass'
    };
  } catch (error) {
    console.error('Error collecting complexity metrics:', error.message);
    return {
      averageComplexity: 0,
      highComplexityCount: 0,
      fileCount: 0,
      highComplexityRatio: 0,
      status: 'error',
      error: error.message
    };
  }
}

/**
 * Collects test coverage metrics
 */
function collectCoverageMetrics() {
  console.log('Collecting coverage metrics...');
  
  try {
    // Check if coverage reports exist
    const coveragePath = path.resolve(__dirname, '../coverage/combined/coverage-summary.json');
    
    if (!fs.existsSync(coveragePath)) {
      console.log('No coverage report found. Running tests with coverage...');
      execSync('npm run coverage', { stdio: 'inherit' });
    }
    
    // Read coverage report
    const coverageData = JSON.parse(fs.readFileSync(coveragePath, 'utf8'));
    const total = coverageData.total;
    
    return {
      lines: total.lines.pct,
      statements: total.statements.pct,
      functions: total.functions.pct,
      branches: total.branches.pct,
      status: 
        total.lines.pct < config.thresholds.coverage.lines ||
        total.statements.pct < config.thresholds.coverage.statements ||
        total.functions.pct < config.thresholds.coverage.functions ||
        total.branches.pct < config.thresholds.coverage.branches
          ? 'warning' : 'pass'
    };
  } catch (error) {
    console.error('Error collecting coverage metrics:', error.message);
    return {
      lines: 0,
      statements: 0,
      functions: 0,
      branches: 0,
      status: 'error',
      error: error.message
    };
  }
}

/**
 * Collects code duplication metrics
 */
function collectDuplicationMetrics() {
  console.log('Collecting duplication metrics...');
  
  try {
    // Run jscpd for duplication detection
    const output = execSync(
      'npx jscpd src --ignore "**/*.test.{ts,tsx}" --format typescript,tsx --output ./code-quality-reports/duplications --reporters json',
      { encoding: 'utf8' }
    );
    
    // Read the jscpd report
    const reportPath = path.resolve(__dirname, '../code-quality-reports/duplications/jscpd-report.json');
    const duplicationData = JSON.parse(fs.readFileSync(reportPath, 'utf8'));
    
    return {
      percentage: duplicationData.statistics.total.percentage,
      clones: duplicationData.statistics.total.clones,
      duplicatedLines: duplicationData.statistics.total.duplicatedLines,
      totalLines: duplicationData.statistics.total.lines,
      files: duplicationData.statistics.total.files,
      status: duplicationData.statistics.total.percentage > config.thresholds.duplications ? 'warning' : 'pass'
    };
  } catch (error) {
    console.error('Error collecting duplication metrics:', error.message);
    return {
      percentage: 0,
      clones: 0,
      duplicatedLines: 0,
      totalLines: 0,
      files: 0,
      status: 'error',
      error: error.message
    };
  }
}

/**
 * Collects TypeScript metrics
 */
function collectTypeScriptMetrics() {
  console.log('Collecting TypeScript metrics...');
  
  try {
    // Run TypeScript compiler with --noEmit to check for errors
    const output = execSync('npx tsc --noEmit', { encoding: 'utf8', stdio: 'pipe' }).toString();
    
    return {
      status: 'pass',
      errorCount: 0
    };
  } catch (error) {
    // TypeScript errors will cause the command to fail, but we can parse the output
    const output = error.stdout.toString();
    const errorCount = (output.match(/error TS\d+/g) || []).length;
    
    return {
      status: 'fail',
      errorCount,
      error: errorCount > 0 ? `${errorCount} TypeScript errors found` : error.message
    };
  }
}

/**
 * Collects Rust metrics
 */
function collectRustMetrics() {
  console.log('Collecting Rust metrics...');
  
  try {
    // Run Rust clippy to check for linting issues
    execSync('cd src-tauri && cargo clippy -- -D warnings', { stdio: 'pipe' });
    
    // Run Rust tests
    const testOutput = execSync('cd src-tauri && cargo test', { encoding: 'utf8' });
    
    // Parse test results
    const testsPassed = testOutput.includes('test result: ok');
    const testCount = (testOutput.match(/test result: ok. \d+ passed/g) || ['0'])[0].match(/\d+/)[0];
    
    return {
      clippy: {
        status: 'pass',
        errorCount: 0
      },
      tests: {
        status: testsPassed ? 'pass' : 'fail',
        count: parseInt(testCount, 10),
        passed: testsPassed
      }
    };
  } catch (error) {
    // Clippy errors will cause the command to fail
    const output = error.stdout ? error.stdout.toString() : '';
    const errorCount = (output.match(/error:/g) || []).length;
    
    return {
      clippy: {
        status: 'fail',
        errorCount,
        error: errorCount > 0 ? `${errorCount} Rust clippy errors found` : error.message
      },
      tests: {
        status: 'error',
        count: 0,
        passed: false,
        error: 'Failed to run Rust tests'
      }
    };
  }
}

/**
 * Calculates summary metrics
 */
function calculateSummary() {
  // Count statuses
  const statuses = [
    metrics.eslint.status,
    metrics.complexity.status,
    metrics.coverage.status,
    metrics.duplications.status,
    metrics.typescript.status,
    metrics.rust.clippy.status,
    metrics.rust.tests.status
  ];
  
  const statusCounts = {
    pass: statuses.filter(s => s === 'pass').length,
    warning: statuses.filter(s => s === 'warning').length,
    fail: statuses.filter(s => s === 'fail').length,
    error: statuses.filter(s => s === 'error').length
  };
  
  // Calculate overall score (0-100)
  const totalChecks = statuses.length;
  const score = Math.round(
    ((statusCounts.pass * 1.0) + (statusCounts.warning * 0.5)) / totalChecks * 100
  );
  
  // Determine overall status
  let overallStatus = 'pass';
  if (statusCounts.error > 0 || statusCounts.fail > 0) {
    overallStatus = 'fail';
  } else if (statusCounts.warning > 0) {
    overallStatus = 'warning';
  }
  
  metrics.summary = {
    score,
    status: overallStatus,
    statusCounts,
    timestamp: new Date().toISOString()
  };
}

/**
 * Generates an HTML report
 */
function generateHtmlReport() {
  console.log('Generating HTML report...');
  
  const reportPath = path.join(config.outputDir, config.reportFile);
  
  // Simple HTML template
  const html = `
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Code Quality Report</title>
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
    .summary {
      display: flex;
      justify-content: space-between;
      background-color: #f8f9fa;
      padding: 20px;
      border-radius: 5px;
      margin-bottom: 30px;
    }
    .score {
      font-size: 48px;
      font-weight: bold;
      text-align: center;
    }
    .status {
      padding: 5px 10px;
      border-radius: 3px;
      font-weight: bold;
    }
    .pass {
      background-color: #d4edda;
      color: #155724;
    }
    .warning {
      background-color: #fff3cd;
      color: #856404;
    }
    .fail {
      background-color: #f8d7da;
      color: #721c24;
    }
    .error {
      background-color: #f8d7da;
      color: #721c24;
    }
    .metric-card {
      background-color: white;
      border: 1px solid #e9ecef;
      border-radius: 5px;
      padding: 20px;
      margin-bottom: 20px;
      box-shadow: 0 1px 3px rgba(0,0,0,0.1);
    }
    .metric-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 15px;
    }
    .metric-title {
      margin: 0;
      font-size: 18px;
    }
    .metric-value {
      font-size: 16px;
      margin: 5px 0;
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
  <h1>Code Quality Report</h1>
  <p class="timestamp">Generated on ${new Date(metrics.timestamp).toLocaleString()}</p>
  
  <div class="summary">
    <div>
      <h2>Overall Quality Score</h2>
      <div class="score">${metrics.summary.score}/100</div>
    </div>
    <div>
      <h2>Status</h2>
      <div class="status ${metrics.summary.status}">${metrics.summary.status.toUpperCase()}</div>
      <div>
        <p>Pass: ${metrics.summary.statusCounts.pass}</p>
        <p>Warning: ${metrics.summary.statusCounts.warning}</p>
        <p>Fail: ${metrics.summary.statusCounts.fail}</p>
        <p>Error: ${metrics.summary.statusCounts.error}</p>
      </div>
    </div>
  </div>
  
  <h2>Metrics</h2>
  
  <div class="metric-card">
    <div class="metric-header">
      <h3 class="metric-title">ESLint</h3>
      <span class="status ${metrics.eslint.status}">${metrics.eslint.status.toUpperCase()}</span>
    </div>
    <p class="metric-value">Errors: ${metrics.eslint.errorCount}</p>
    <p class="metric-value">Warnings: ${metrics.eslint.warningCount}</p>
    <p class="metric-value">Files with issues: ${metrics.eslint.filesWithIssues} / ${metrics.eslint.fileCount}</p>
    ${metrics.eslint.error ? `<p class="metric-value error">Error: ${metrics.eslint.error}</p>` : ''}
  </div>
  
  <div class="metric-card">
    <div class="metric-header">
      <h3 class="metric-title">Code Complexity</h3>
      <span class="status ${metrics.complexity.status}">${metrics.complexity.status.toUpperCase()}</span>
    </div>
    <p class="metric-value">Average complexity: ${metrics.complexity.averageComplexity.toFixed(2)}</p>
    <p class="metric-value">Files with high complexity: ${metrics.complexity.highComplexityCount} / ${metrics.complexity.fileCount}</p>
    ${metrics.complexity.error ? `<p class="metric-value error">Error: ${metrics.complexity.error}</p>` : ''}
  </div>
  
  <div class="metric-card">
    <div class="metric-header">
      <h3 class="metric-title">Test Coverage</h3>
      <span class="status ${metrics.coverage.status}">${metrics.coverage.status.toUpperCase()}</span>
    </div>
    <p class="metric-value">Lines: ${metrics.coverage.lines.toFixed(2)}%</p>
    <p class="metric-value">Statements: ${metrics.coverage.statements.toFixed(2)}%</p>
    <p class="metric-value">Functions: ${metrics.coverage.functions.toFixed(2)}%</p>
    <p class="metric-value">Branches: ${metrics.coverage.branches.toFixed(2)}%</p>
    ${metrics.coverage.error ? `<p class="metric-value error">Error: ${metrics.coverage.error}</p>` : ''}
  </div>
  
  <div class="metric-card">
    <div class="metric-header">
      <h3 class="metric-title">Code Duplication</h3>
      <span class="status ${metrics.duplications.status}">${metrics.duplications.status.toUpperCase()}</span>
    </div>
    <p class="metric-value">Duplication: ${metrics.duplications.percentage.toFixed(2)}%</p>
    <p class="metric-value">Clones: ${metrics.duplications.clones}</p>
    <p class="metric-value">Duplicated lines: ${metrics.duplications.duplicatedLines} / ${metrics.duplications.totalLines}</p>
    ${metrics.duplications.error ? `<p class="metric-value error">Error: ${metrics.duplications.error}</p>` : ''}
  </div>
  
  <div class="metric-card">
    <div class="metric-header">
      <h3 class="metric-title">TypeScript</h3>
      <span class="status ${metrics.typescript.status}">${metrics.typescript.status.toUpperCase()}</span>
    </div>
    <p class="metric-value">Errors: ${metrics.typescript.errorCount}</p>
    ${metrics.typescript.error ? `<p class="metric-value error">Error: ${metrics.typescript.error}</p>` : ''}
  </div>
  
  <div class="metric-card">
    <div class="metric-header">
      <h3 class="metric-title">Rust</h3>
      <span class="status ${metrics.rust.clippy.status === 'pass' && metrics.rust.tests.status === 'pass' ? 'pass' : 'fail'}">${metrics.rust.clippy.status === 'pass' && metrics.rust.tests.status === 'pass' ? 'PASS' : 'FAIL'}</span>
    </div>
    <h4>Clippy</h4>
    <p class="metric-value">Status: <span class="status ${metrics.rust.clippy.status}">${metrics.rust.clippy.status.toUpperCase()}</span></p>
    <p class="metric-value">Errors: ${metrics.rust.clippy.errorCount || 0}</p>
    ${metrics.rust.clippy.error ? `<p class="metric-value error">Error: ${metrics.rust.clippy.error}</p>` : ''}
    
    <h4>Tests</h4>
    <p class="metric-value">Status: <span class="status ${metrics.rust.tests.status}">${metrics.rust.tests.status.toUpperCase()}</span></p>
    <p class="metric-value">Tests: ${metrics.rust.tests.count}</p>
    ${metrics.rust.tests.error ? `<p class="metric-value error">Error: ${metrics.rust.tests.error}</p>` : ''}
  </div>
  
  <div class="timestamp">
    <p>Report generated on ${new Date().toLocaleString()}</p>
  </div>
</body>
</html>
  `;
  
  fs.writeFileSync(reportPath, html);
  console.log(`HTML report generated at ${reportPath}`);
}