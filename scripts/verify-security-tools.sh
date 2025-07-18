#!/bin/bash
# Script to verify that security testing tools are installed and working correctly

echo "Verifying security testing tools..."

# Check for Node.js and pnpm
echo "Checking Node.js and pnpm..."
if ! command -v node &> /dev/null; then
    echo "Node.js is not installed. Please install Node.js."
    exit 1
fi

if ! command -v pnpm &> /dev/null; then
    echo "pnpm is not installed. Please install pnpm."
    exit 1
fi

# Check for ESLint
echo "Checking ESLint..."
if ! pnpm list | grep -q eslint; then
    echo "ESLint is not installed. Please run 'pnpm install'."
    exit 1
fi

# Check for Rust and Cargo
echo "Checking Rust and Cargo..."
if ! command -v cargo &> /dev/null; then
    echo "Rust/Cargo is not installed. Please install Rust."
    exit 1
fi

# Check for cargo-audit
echo "Checking cargo-audit..."
if ! command -v cargo-audit &> /dev/null; then
    echo "cargo-audit is not installed. Please run 'cargo install cargo-audit'."
    exit 1
fi

# Check for cargo-deny
echo "Checking cargo-deny..."
if ! command -v cargo-deny &> /dev/null; then
    echo "cargo-deny is not installed. Please run 'cargo install cargo-deny'."
    exit 1
fi

# Check for cargo-geiger
echo "Checking cargo-geiger..."
if ! command -v cargo-geiger &> /dev/null; then
    echo "cargo-geiger is not installed. Please run 'cargo install cargo-geiger'."
    exit 1
fi

# Create a test file with a known security issue
echo "Creating test file with a known security issue..."
mkdir -p /tmp/security-test
cat > /tmp/security-test/test.js << EOL
// This file contains a deliberate security issue for testing
const exec = require('child_process').exec;
function runCommand(cmd) {
    // SECURITY ISSUE: Command injection vulnerability
    exec(cmd);
}
runCommand(process.argv[1]);
EOL

# Run ESLint on the test file
echo "Running ESLint on test file..."
if ! npx eslint /tmp/security-test/test.js &> /dev/null; then
    echo "ESLint detected issues in the test file as expected."
else
    echo "WARNING: ESLint did not detect the security issue in the test file."
fi

# Clean up
echo "Cleaning up..."
rm -rf /tmp/security-test

echo "Verification complete. All security testing tools are installed."
echo "Note: This script only verifies that the tools are installed, not that they are configured correctly."
echo "For a complete test, run the GitHub Actions workflow locally using 'act'."

exit 0