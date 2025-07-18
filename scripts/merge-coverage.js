/**
 * Script to merge coverage reports from different test types
 * 
 * This script uses istanbul-merge to combine coverage reports from unit and integration tests
 * into a single report, and then uses istanbul-reports to generate a combined report.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Ensure the coverage directory exists
const coverageDir = path.resolve(__dirname, '../coverage');
if (!fs.existsSync(coverageDir)) {
  fs.mkdirSync(coverageDir);
}

// Ensure the combined directory exists
const combinedDir = path.resolve(coverageDir, 'combined');
if (!fs.existsSync(combinedDir)) {
  fs.mkdirSync(combinedDir);
}

// Define the coverage report paths
const unitCoverage = path.resolve(coverageDir, 'unit/coverage-final.json');
const integrationCoverage = path.resolve(coverageDir, 'integration/coverage-final.json');
const combinedCoverage = path.resolve(combinedDir, 'coverage-final.json');

// Check if the coverage reports exist
const unitExists = fs.existsSync(unitCoverage);
const integrationExists = fs.existsSync(integrationCoverage);

if (!unitExists && !integrationExists) {
  console.error('No coverage reports found. Run tests with coverage first.');
  process.exit(1);
}

// Install required packages if they don't exist
try {
  execSync('npx istanbul-merge --version', { stdio: 'ignore' });
} catch (error) {
  console.log('Installing istanbul-merge...');
  execSync('npm install -g istanbul-merge', { stdio: 'inherit' });
}

try {
  execSync('npx nyc --version', { stdio: 'ignore' });
} catch (error) {
  console.log('Installing nyc...');
  execSync('npm install -g nyc', { stdio: 'inherit' });
}

// Build the command to merge coverage reports
let mergeCommand = 'npx istanbul-merge';

if (unitExists) {
  mergeCommand += ` --out ${combinedCoverage} ${unitCoverage}`;
}

if (integrationExists) {
  if (unitExists) {
    mergeCommand += ` ${integrationCoverage}`;
  } else {
    mergeCommand += ` --out ${combinedCoverage} ${integrationCoverage}`;
  }
}

// Merge the coverage reports
console.log('Merging coverage reports...');
execSync(mergeCommand, { stdio: 'inherit' });

// Generate the combined report
console.log('Generating combined report...');
execSync(`npx nyc report --reporter=html --reporter=text --reporter=lcov --temp-dir=${combinedDir}`, {
  stdio: 'inherit',
});

console.log('Combined coverage report generated in coverage/combined');