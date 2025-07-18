# Security Testing Framework

This document describes the security testing framework implemented in the evo-pro project. The framework is designed to identify and mitigate security vulnerabilities in both the frontend (JavaScript/TypeScript) and backend (Rust) code.

## Overview

The security testing framework is integrated into the CI/CD pipeline and runs automatically on:
- Every push to the main branch
- Every pull request to the main branch
- Weekly schedule (Sundays at midnight)

The framework includes:
- Static code analysis
- Dependency scanning
- Security-focused linting
- Vulnerability scanning
- Unsafe code detection (Rust)

## Components

### JavaScript/TypeScript Security Checks

1. **ESLint with Security Plugins**
   - Identifies potential security vulnerabilities in JavaScript/TypeScript code
   - Enforces secure coding practices

2. **pnpm audit**
   - Scans dependencies for known vulnerabilities
   - Alerts on moderate or higher severity issues

3. **OWASP Dependency-Check**
   - Comprehensive dependency scanning
   - Identifies vulnerabilities in third-party libraries
   - Generates detailed HTML reports

4. **Snyk**
   - Scans for vulnerabilities in dependencies
   - Provides remediation advice
   - Monitors for new vulnerabilities

### Rust Security Checks

1. **Clippy with Security Lints**
   - Identifies potential security issues in Rust code
   - Enforces secure coding practices

2. **cargo-audit**
   - Scans Rust dependencies for known vulnerabilities
   - Alerts on security advisories from the RustSec Advisory Database

3. **cargo-deny**
   - Enforces license compliance
   - Prevents usage of forbidden crates
   - Checks for duplicate dependencies

4. **cargo-geiger**
   - Detects usage of unsafe Rust code
   - Generates reports on unsafe code usage
   - Helps identify potential memory safety issues

### SonarCloud Analysis

- Comprehensive static code analysis
- Identifies code quality issues
- Detects security vulnerabilities
- Tracks code coverage

## Reports

The security testing framework generates several reports:

1. **Dependency-Check Report**
   - HTML report with details on vulnerable dependencies
   - Available as an artifact in GitHub Actions

2. **Geiger Report**
   - JSON report with details on unsafe Rust code usage
   - Available as an artifact in GitHub Actions

3. **Security Summary Report**
   - Markdown report summarizing all security checks
   - Available as an artifact in GitHub Actions

4. **SonarCloud Dashboard**
   - Web-based dashboard with detailed analysis
   - Tracks security issues over time

## Suppressing False Positives

Sometimes security tools may report false positives or vulnerabilities that cannot be fixed immediately. The framework includes a suppression mechanism:

1. **OWASP Dependency-Check Suppressions**
   - Edit `.github/workflows/suppressions.xml` to suppress specific vulnerabilities
   - Include detailed notes explaining why the suppression is necessary
   - Review suppressions regularly to ensure they're still valid

## Interpreting Results

When reviewing security testing results:

1. **Critical and High Severity Issues**
   - Address immediately
   - May block merges to main branch

2. **Medium Severity Issues**
   - Address in a timely manner
   - May require risk assessment

3. **Low Severity Issues**
   - Address when convenient
   - Document if not addressing

## Adding Custom Security Checks

To add custom security checks to the framework:

1. Edit `.github/workflows/security.yml`
2. Add new steps to the appropriate job
3. Configure reporting for the new checks
4. Update this documentation

## Best Practices

1. **Regular Reviews**
   - Review security reports weekly
   - Address high-priority issues promptly

2. **Dependency Management**
   - Keep dependencies up to date
   - Remove unused dependencies

3. **Code Reviews**
   - Include security considerations in code reviews
   - Use the security testing results as a guide

4. **Documentation**
   - Document security decisions
   - Update suppressions with clear explanations

## Related Documents

- [Privacy Impact Assessment Workflow](privacy-impact-assessment-workflow.md)
- [Threat Modeling System](security/threat-modeling.md)