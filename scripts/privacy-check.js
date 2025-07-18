#!/usr/bin/env node

/**
 * Privacy Impact Assessment Check Script
 * 
 * This script checks if changes to privacy-sensitive files have an associated
 * Privacy Impact Assessment (PIA) document or reference in the commit message.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const PRIVACY_SENSITIVE_PATHS = [
  // User data related
  'src-tauri/src/entities/users.rs',
  'src-tauri/src/services/user',
  'src/components/user',
  // Message data related
  'src-tauri/src/entities/messages.rs',
  'src-tauri/src/services/message',
  'src/components/message',
  // Authentication related
  'src-tauri/src/auth',
  'src/lib/auth',
  // Data storage related
  'src-tauri/src/storage',
  // Privacy specific features
  'src-tauri/src/services/data_minimization.rs',
  // Add more paths as needed
];

const PIA_REFERENCE_PATTERN = /PIA-\d+|Privacy Impact Assessment/i;
const PIA_TEMPLATE_PATH = 'docs/templates/privacy-impact-assessment.md';
const PIA_DOCS_PATH = 'docs/privacy-impact-assessments';

/**
 * Get the list of files changed in the current commit
 */
function getChangedFiles() {
  try {
    const output = execSync('git diff --cached --name-only').toString();
    return output.split('\n').filter(Boolean);
  } catch (error) {
    console.error('Error getting changed files:', error.message);
    return [];
  }
}

/**
 * Check if any privacy-sensitive files are being modified
 */
function checkPrivacySensitiveChanges(changedFiles) {
  return changedFiles.some(file => 
    PRIVACY_SENSITIVE_PATHS.some(sensitivePath => 
      file.startsWith(sensitivePath)
    )
  );
}

/**
 * Check if the commit message contains a reference to a PIA
 */
function checkCommitMessageForPIA() {
  try {
    const commitMsg = fs.readFileSync('.git/COMMIT_EDITMSG', 'utf8');
    return PIA_REFERENCE_PATTERN.test(commitMsg);
  } catch (error) {
    console.error('Error reading commit message:', error.message);
    return false;
  }
}

/**
 * Check if there's a recent PIA document in the PIA directory
 */
function checkForRecentPIA() {
  if (!fs.existsSync(PIA_DOCS_PATH)) {
    return false;
  }
  
  try {
    const files = fs.readdirSync(PIA_DOCS_PATH);
    const piaFiles = files.filter(file => file.endsWith('.md'));
    
    if (piaFiles.length === 0) {
      return false;
    }
    
    // Check if any PIA was created or modified in the last week
    const oneWeekAgo = new Date();
    oneWeekAgo.setDate(oneWeekAgo.getDate() - 7);
    
    return piaFiles.some(file => {
      const filePath = path.join(PIA_DOCS_PATH, file);
      const stats = fs.statSync(filePath);
      return stats.mtime > oneWeekAgo;
    });
  } catch (error) {
    console.error('Error checking for recent PIA:', error.message);
    return false;
  }
}

/**
 * Main function
 */
function main() {
  const changedFiles = getChangedFiles();
  const touchesPrivacySensitiveFiles = checkPrivacySensitiveChanges(changedFiles);
  
  if (!touchesPrivacySensitiveFiles) {
    // No privacy-sensitive files changed, exit successfully
    process.exit(0);
  }
  
  // Check if there's a PIA reference in the commit message
  const hasPIAReference = checkCommitMessageForPIA();
  
  // Check if there's a recent PIA document
  const hasRecentPIA = checkForRecentPIA();
  
  if (hasPIAReference || hasRecentPIA) {
    // PIA reference found or recent PIA exists, exit successfully
    process.exit(0);
  }
  
  // No PIA reference found, show warning and exit with error
  console.error('\x1b[33m%s\x1b[0m', '⚠️  Privacy Impact Assessment Warning ⚠️');
  console.error('\x1b[33m%s\x1b[0m', 'You are modifying privacy-sensitive files without a reference to a Privacy Impact Assessment.');
  console.error('\x1b[33m%s\x1b[0m', 'Please do one of the following:');
  console.error('\x1b[33m%s\x1b[0m', '1. Add a reference to a PIA in your commit message (e.g., "PIA-123" or "Privacy Impact Assessment")');
  console.error('\x1b[33m%s\x1b[0m', `2. Create a new PIA document in the ${PIA_DOCS_PATH} directory using the template at ${PIA_TEMPLATE_PATH}`);
  console.error('\x1b[33m%s\x1b[0m', '3. Use --no-verify to bypass this check (not recommended)');
  
  process.exit(1);
}

main();