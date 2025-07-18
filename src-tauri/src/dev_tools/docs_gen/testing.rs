//! Documentation Testing
//!
//! This module provides tools for testing documentation to ensure it stays up-to-date
//! with the codebase. It includes functionality for validating code examples, checking
//! for broken links, and verifying that API documentation matches the actual code.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use regex::Regex;
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

use super::DocsGenConfig;

/// Documentation testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsTestingConfig {
    /// Whether documentation testing is enabled
    pub enabled: bool,
    /// Whether to validate code examples
    pub validate_code_examples: bool,
    /// Whether to check for broken links
    pub check_links: bool,
    /// Whether to verify API documentation
    pub verify_api_docs: bool,
    /// Whether to check for outdated screenshots
    pub check_screenshots: bool,
    /// Maximum age for documentation before it's considered outdated (in days)
    pub max_doc_age_days: u32,
    /// Whether to treat outdated documentation as errors (fails tests) instead of warnings
    pub outdated_docs_as_errors: bool,
    /// Whether to check for code references in documentation
    pub check_code_references: bool,
    /// Directories to exclude from testing
    pub exclude_dirs: Vec<String>,
    /// File patterns to exclude from testing
    pub exclude_patterns: Vec<String>,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for DocsTestingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            validate_code_examples: true,
            check_links: true,
            verify_api_docs: true,
            check_screenshots: true,
            max_doc_age_days: 90, // 3 months
            outdated_docs_as_errors: false, // By default, treat as warnings
            check_code_references: true,
            exclude_dirs: vec![
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
            ],
            exclude_patterns: vec![
                r"\.git".to_string(),
                r"\.DS_Store".to_string(),
            ],
            options: HashMap::new(),
        }
    }
}

/// Documentation test result severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestSeverity {
    /// Informational message
    Info,
    /// Warning (doesn't fail the test)
    Warning,
    /// Error (fails the test)
    Error,
}

impl std::fmt::Display for TestSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestSeverity::Info => write!(f, "Info"),
            TestSeverity::Warning => write!(f, "Warning"),
            TestSeverity::Error => write!(f, "Error"),
        }
    }
}

/// Documentation test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test ID
    pub id: String,
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Test severity
    pub severity: TestSeverity,
    /// Test message
    pub message: String,
    /// File path (if applicable)
    pub file_path: Option<String>,
    /// Line number (if applicable)
    pub line_number: Option<u32>,
    /// Test timestamp
    pub timestamp: String,
    /// Test duration
    pub duration: Duration,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl TestResult {
    /// Create a new test result
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        severity: TestSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            severity,
            message: message.into(),
            file_path: None,
            line_number: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration: Duration::from_secs(0),
            context: HashMap::new(),
        }
    }

    /// Set the file path
    pub fn with_file_path(mut self, file_path: impl Into<String>) -> Self {
        self.file_path = Some(file_path.into());
        self
    }

    /// Set the line number
    pub fn with_line_number(mut self, line_number: u32) -> Self {
        self.line_number = Some(line_number);
        self
    }

    /// Set the test duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add context to the test result
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Check if the test passed
    pub fn passed(&self) -> bool {
        self.severity != TestSeverity::Error
    }
}

/// Documentation test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Test suite ID
    pub id: String,
    /// Test suite name
    pub name: String,
    /// Test suite description
    pub description: String,
    /// Test results
    pub results: Vec<TestResult>,
    /// Test suite timestamp
    pub timestamp: String,
    /// Test suite duration
    pub duration: Duration,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            results: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration: Duration::from_secs(0),
            context: HashMap::new(),
        }
    }

    /// Add a test result to the suite
    pub fn add_result(&mut self, result: TestResult) {
        self.results.push(result);
    }

    /// Set the test suite duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add context to the test suite
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Check if all tests passed
    pub fn passed(&self) -> bool {
        self.results.iter().all(|r| r.passed())
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.results.iter().filter(|r| r.severity == TestSeverity::Error).count()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.results.iter().filter(|r| r.severity == TestSeverity::Warning).count()
    }

    /// Get the number of info messages
    pub fn info_count(&self) -> usize {
        self.results.iter().filter(|r| r.severity == TestSeverity::Info).count()
    }
}

/// Documentation test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestReport {
    /// Test report ID
    pub id: String,
    /// Test report name
    pub name: String,
    /// Test report description
    pub description: String,
    /// Test suites
    pub suites: Vec<TestSuite>,
    /// Test report timestamp
    pub timestamp: String,
    /// Test report duration
    pub duration: Duration,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl TestReport {
    /// Create a new test report
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            suites: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration: Duration::from_secs(0),
            context: HashMap::new(),
        }
    }

    /// Add a test suite to the report
    pub fn add_suite(&mut self, suite: TestSuite) {
        self.suites.push(suite);
    }

    /// Set the test report duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add context to the test report
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// Check if all tests passed
    pub fn passed(&self) -> bool {
        self.suites.iter().all(|s| s.passed())
    }

    /// Get the total number of errors
    pub fn error_count(&self) -> usize {
        self.suites.iter().map(|s| s.error_count()).sum()
    }

    /// Get the total number of warnings
    pub fn warning_count(&self) -> usize {
        self.suites.iter().map(|s| s.warning_count()).sum()
    }

    /// Get the total number of info messages
    pub fn info_count(&self) -> usize {
        self.suites.iter().map(|s| s.info_count()).sum()
    }

    /// Get the total number of test results
    pub fn total_count(&self) -> usize {
        self.suites.iter().map(|s| s.results.len()).sum()
    }
}

/// Documentation tester
#[derive(Debug)]
pub struct DocsTester {
    /// Configuration
    pub config: DocsTestingConfig,
    /// Base documentation generator configuration
    pub base_config: DocsGenConfig,
    /// Test report
    pub report: TestReport,
}

impl DocsTester {
    /// Create a new documentation tester
    pub fn new(config: DocsTestingConfig, base_config: DocsGenConfig) -> Self {
        let report = TestReport::new(
            "docs-test-report",
            "Documentation Test Report",
            "Results of testing documentation for accuracy and freshness",
        );

        Self {
            config,
            base_config,
            report,
        }
    }

    /// Run all documentation tests
    pub fn run_tests(&mut self, docs_dir: &Path) -> io::Result<TestReport> {
        if !self.config.enabled {
            info!("Documentation testing is disabled");
            return Ok(self.report.clone());
        }

        info!("Running documentation tests");
        let start_time = Instant::now();

        // Create test suites for each type of test
        let mut code_examples_suite = TestSuite::new(
            "code-examples",
            "Code Examples",
            "Tests that verify code examples in documentation are valid",
        );

        let mut links_suite = TestSuite::new(
            "links",
            "Documentation Links",
            "Tests that verify links in documentation are valid",
        );

        let mut api_docs_suite = TestSuite::new(
            "api-docs",
            "API Documentation",
            "Tests that verify API documentation matches the actual code",
        );

        let mut freshness_suite = TestSuite::new(
            "freshness",
            "Documentation Freshness",
            "Tests that verify documentation is up-to-date",
        );

        // Run tests if enabled
        if self.config.validate_code_examples {
            self.test_code_examples(docs_dir, &mut code_examples_suite)?;
        }

        if self.config.check_links {
            self.test_links(docs_dir, &mut links_suite)?;
        }

        if self.config.verify_api_docs {
            self.test_api_docs(docs_dir, &mut api_docs_suite)?;
        }

        // Always run freshness tests
        self.test_freshness(docs_dir, &mut freshness_suite)?;

        // Add test suites to the report
        self.report.add_suite(code_examples_suite);
        self.report.add_suite(links_suite);
        self.report.add_suite(api_docs_suite);
        self.report.add_suite(freshness_suite);

        // Set report duration
        let duration = start_time.elapsed();
        self.report = self.report.clone().with_duration(duration);

        // Log test results
        let passed = self.report.passed();
        let status = if passed { "PASSED" } else { "FAILED" };
        info!(
            "Documentation tests {}: {} errors, {} warnings, {} total tests in {:?}",
            status,
            self.report.error_count(),
            self.report.warning_count(),
            self.report.total_count(),
            duration,
        );

        Ok(self.report.clone())
    }

    /// Test code examples in documentation
    fn test_code_examples(&self, docs_dir: &Path, suite: &mut TestSuite) -> io::Result<()> {
        info!("Testing code examples in documentation");
        let start_time = Instant::now();

        // Find all Markdown files
        let markdown_files = self.find_files(docs_dir, &["md"])?;

        for file_path in markdown_files {
            // Read file content
            let content = fs::read_to_string(&file_path)?;

            // Extract code blocks
            let code_blocks = self.extract_code_blocks(&content);

            for (block_index, (language, code, line_number)) in code_blocks.iter().enumerate() {
                let block_id = format!("code-block-{}", block_index + 1);

                // Skip blocks without a language specifier
                if language.is_empty() {
                    continue;
                }

                // Validate code based on language
                match language.as_str() {
                    "rust" => {
                        self.validate_rust_code(suite, &file_path, block_id, code, *line_number)?;
                    }
                    "typescript" | "ts" => {
                        self.validate_typescript_code(suite, &file_path, block_id, code, *line_number)?;
                    }
                    "javascript" | "js" => {
                        self.validate_javascript_code(suite, &file_path, block_id, code, *line_number)?;
                    }
                    "json" => {
                        self.validate_json(suite, &file_path, block_id, code, *line_number)?;
                    }
                    "toml" => {
                        self.validate_toml(suite, &file_path, block_id, code, *line_number)?;
                    }
                    "sql" => {
                        self.validate_sql(suite, &file_path, block_id, code, *line_number)?;
                    }
                    "bash" | "sh" => {
                        // Skip bash code validation for now
                    }
                    _ => {
                        // Skip other languages
                    }
                }
            }
        }

        // Set suite duration
        *suite = suite.clone().with_duration(start_time.elapsed());

        Ok(())
    }

    /// Extract code blocks from Markdown content
    fn extract_code_blocks(&self, content: &str) -> Vec<(String, String, u32)> {
        let mut blocks = Vec::new();
        let mut in_code_block = false;
        let mut current_language = String::new();
        let mut current_code = String::new();
        let mut start_line = 0;

        for (i, line) in content.lines().enumerate() {
            let line_number = i as u32 + 1;

            if line.starts_with("```") && !in_code_block {
                in_code_block = true;
                current_language = line.trim_start_matches('`').trim().to_string();
                start_line = line_number + 1;
                current_code.clear();
            } else if line.starts_with("```") && in_code_block {
                in_code_block = false;
                blocks.push((current_language.clone(), current_code.clone(), start_line));
                current_language.clear();
                current_code.clear();
            } else if in_code_block {
                current_code.push_str(line);
                current_code.push('\n');
            }
        }

        blocks
    }

    /// Validate Rust code
    fn validate_rust_code(&self, suite: &mut TestSuite, file_path: &Path, block_id: String, code: &str, line_number: u32) -> io::Result<()> {
        // Create a temporary file with the code
        let temp_dir = tempfile::tempdir()?;
        let temp_file_path = temp_dir.path().join("test.rs");
        let mut temp_file = File::create(&temp_file_path)?;
        temp_file.write_all(code.as_bytes())?;

        // Run rustc to check for syntax errors
        let output = std::process::Command::new("rustc")
            .arg("--edition=2021")
            .arg("--check")
            .arg(&temp_file_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    // Code is valid
                    suite.add_result(TestResult::new(
                        format!("{}-rust", block_id),
                        "Rust Code Example",
                        "Validates that Rust code examples are syntactically correct",
                        TestSeverity::Info,
                        "Rust code is valid",
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                } else {
                    // Code has errors
                    let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                    suite.add_result(TestResult::new(
                        format!("{}-rust", block_id),
                        "Rust Code Example",
                        "Validates that Rust code examples are syntactically correct",
                        TestSeverity::Error,
                        format!("Rust code has syntax errors: {}", error_message),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                }
            }
            Err(e) => {
                // Failed to run rustc
                suite.add_result(TestResult::new(
                    format!("{}-rust", block_id),
                    "Rust Code Example",
                    "Validates that Rust code examples are syntactically correct",
                    TestSeverity::Warning,
                    format!("Failed to validate Rust code: {}", e),
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
        }

        Ok(())
    }

    /// Validate TypeScript code
    fn validate_typescript_code(&self, suite: &mut TestSuite, file_path: &Path, block_id: String, code: &str, line_number: u32) -> io::Result<()> {
        // Create a temporary file with the code
        let temp_dir = tempfile::tempdir()?;
        let temp_file_path = temp_dir.path().join("test.ts");
        let mut temp_file = File::create(&temp_file_path)?;
        temp_file.write_all(code.as_bytes())?;

        // Run tsc to check for syntax errors
        let output = std::process::Command::new("npx")
            .arg("tsc")
            .arg("--noEmit")
            .arg(&temp_file_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    // Code is valid
                    suite.add_result(TestResult::new(
                        format!("{}-typescript", block_id),
                        "TypeScript Code Example",
                        "Validates that TypeScript code examples are syntactically correct",
                        TestSeverity::Info,
                        "TypeScript code is valid",
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                } else {
                    // Code has errors
                    let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                    suite.add_result(TestResult::new(
                        format!("{}-typescript", block_id),
                        "TypeScript Code Example",
                        "Validates that TypeScript code examples are syntactically correct",
                        TestSeverity::Error,
                        format!("TypeScript code has syntax errors: {}", error_message),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                }
            }
            Err(e) => {
                // Failed to run tsc
                suite.add_result(TestResult::new(
                    format!("{}-typescript", block_id),
                    "TypeScript Code Example",
                    "Validates that TypeScript code examples are syntactically correct",
                    TestSeverity::Warning,
                    format!("Failed to validate TypeScript code: {}", e),
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
        }

        Ok(())
    }

    /// Validate JavaScript code
    fn validate_javascript_code(&self, suite: &mut TestSuite, file_path: &Path, block_id: String, code: &str, line_number: u32) -> io::Result<()> {
        // Create a temporary file with the code
        let temp_dir = tempfile::tempdir()?;
        let temp_file_path = temp_dir.path().join("test.js");
        let mut temp_file = File::create(&temp_file_path)?;
        temp_file.write_all(code.as_bytes())?;

        // Run eslint to check for syntax errors
        let output = std::process::Command::new("npx")
            .arg("eslint")
            .arg("--no-eslintrc")
            .arg("--parser-options=ecmaVersion:latest")
            .arg(&temp_file_path)
            .output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    // Code is valid
                    suite.add_result(TestResult::new(
                        format!("{}-javascript", block_id),
                        "JavaScript Code Example",
                        "Validates that JavaScript code examples are syntactically correct",
                        TestSeverity::Info,
                        "JavaScript code is valid",
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                } else {
                    // Code has errors
                    let error_message = String::from_utf8_lossy(&output.stderr).to_string();
                    suite.add_result(TestResult::new(
                        format!("{}-javascript", block_id),
                        "JavaScript Code Example",
                        "Validates that JavaScript code examples are syntactically correct",
                        TestSeverity::Error,
                        format!("JavaScript code has syntax errors: {}", error_message),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                }
            }
            Err(e) => {
                // Failed to run eslint
                suite.add_result(TestResult::new(
                    format!("{}-javascript", block_id),
                    "JavaScript Code Example",
                    "Validates that JavaScript code examples are syntactically correct",
                    TestSeverity::Warning,
                    format!("Failed to validate JavaScript code: {}", e),
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
        }

        Ok(())
    }

    /// Validate JSON
    fn validate_json(&self, suite: &mut TestSuite, file_path: &Path, block_id: String, code: &str, line_number: u32) -> io::Result<()> {
        match serde_json::from_str::<serde_json::Value>(code) {
            Ok(_) => {
                // JSON is valid
                suite.add_result(TestResult::new(
                    format!("{}-json", block_id),
                    "JSON Example",
                    "Validates that JSON examples are syntactically correct",
                    TestSeverity::Info,
                    "JSON is valid",
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
            Err(e) => {
                // JSON has errors
                suite.add_result(TestResult::new(
                    format!("{}-json", block_id),
                    "JSON Example",
                    "Validates that JSON examples are syntactically correct",
                    TestSeverity::Error,
                    format!("JSON has syntax errors: {}", e),
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
        }

        Ok(())
    }

    /// Validate TOML
    fn validate_toml(&self, suite: &mut TestSuite, file_path: &Path, block_id: String, code: &str, line_number: u32) -> io::Result<()> {
        match toml::from_str::<toml::Value>(code) {
            Ok(_) => {
                // TOML is valid
                suite.add_result(TestResult::new(
                    format!("{}-toml", block_id),
                    "TOML Example",
                    "Validates that TOML examples are syntactically correct",
                    TestSeverity::Info,
                    "TOML is valid",
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
            Err(e) => {
                // TOML has errors
                suite.add_result(TestResult::new(
                    format!("{}-toml", block_id),
                    "TOML Example",
                    "Validates that TOML examples are syntactically correct",
                    TestSeverity::Error,
                    format!("TOML has syntax errors: {}", e),
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            }
        }

        Ok(())
    }

    /// Validate SQL
    fn validate_sql(&self, suite: &mut TestSuite, file_path: &Path, block_id: String, code: &str, line_number: u32) -> io::Result<()> {
        // For SQL, we'll just do a basic syntax check
        // A more thorough check would require a SQL parser

        // Check for basic SQL syntax issues
        let sql_errors = self.check_basic_sql_syntax(code);

        if sql_errors.is_empty() {
            // SQL looks valid
            suite.add_result(TestResult::new(
                format!("{}-sql", block_id),
                "SQL Example",
                "Validates that SQL examples have basic syntax correctness",
                TestSeverity::Info,
                "SQL appears to be valid",
            )
            .with_file_path(file_path.to_string_lossy().to_string())
            .with_line_number(line_number));
        } else {
            // SQL has potential issues
            suite.add_result(TestResult::new(
                format!("{}-sql", block_id),
                "SQL Example",
                "Validates that SQL examples have basic syntax correctness",
                TestSeverity::Warning,
                format!("SQL may have syntax issues: {}", sql_errors.join(", ")),
            )
            .with_file_path(file_path.to_string_lossy().to_string())
            .with_line_number(line_number));
        }

        Ok(())
    }

    /// Check basic SQL syntax
    fn check_basic_sql_syntax(&self, sql: &str) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for unbalanced parentheses
        let open_count = sql.chars().filter(|c| *c == '(').count();
        let close_count = sql.chars().filter(|c| *c == ')').count();

        if open_count != close_count {
            errors.push(format!("Unbalanced parentheses: {} opening vs {} closing", open_count, close_count));
        }

        // Check for missing semicolons at the end of statements
        let statements: Vec<&str> = sql.split(';').collect();
        if statements.len() > 1 && !sql.trim().ends_with(';') {
            errors.push("Missing semicolon at the end of SQL statement".to_string());
        }

        // Check for common SQL keywords
        let sql_lower = sql.to_lowercase();
        let has_select = sql_lower.contains("select");
        let has_from = sql_lower.contains("from");
        let has_insert = sql_lower.contains("insert");
        let has_update = sql_lower.contains("update");
        let has_delete = sql_lower.contains("delete");
        let has_create = sql_lower.contains("create");
        let has_alter = sql_lower.contains("alter");
        let has_drop = sql_lower.contains("drop");

        // Check for common SQL patterns
        if has_select && !has_from {
            errors.push("SELECT statement without FROM clause".to_string());
        }

        if has_insert && !sql_lower.contains("into") {
            errors.push("INSERT statement without INTO keyword".to_string());
        }

        if has_update && !sql_lower.contains("set") {
            errors.push("UPDATE statement without SET clause".to_string());
        }

        errors
    }

    /// Test links in documentation
    fn test_links(&self, docs_dir: &Path, suite: &mut TestSuite) -> io::Result<()> {
        info!("Testing links in documentation");
        let start_time = Instant::now();

        // Find all Markdown files
        let markdown_files = self.find_files(docs_dir, &["md"])?;

        // Track all internal links for cross-referencing
        let mut all_files = HashSet::new();
        for file_path in &markdown_files {
            let relative_path = file_path.strip_prefix(docs_dir).unwrap_or(file_path);
            all_files.insert(relative_path.to_string_lossy().to_string());
        }

        // Find all HTML files
        let html_files = self.find_files(docs_dir, &["html"])?;
        for file_path in &html_files {
            let relative_path = file_path.strip_prefix(docs_dir).unwrap_or(file_path);
            all_files.insert(relative_path.to_string_lossy().to_string());
        }

        // Check links in each file
        for file_path in markdown_files {
            // Read file content
            let content = fs::read_to_string(&file_path)?;

            // Extract links
            let links = self.extract_links(&content);

            for (link, line_number) in links {
                let link_id = format!("link-{}-{}", file_path.to_string_lossy().to_string(), line_number);

                if link.starts_with("http://") || link.starts_with("https://") {
                    // External link - we won't actually check these to avoid network requests
                    suite.add_result(TestResult::new(
                        format!("{}-external", link_id),
                        "External Link",
                        "Identifies external links in documentation",
                        TestSeverity::Info,
                        format!("External link: {}", link),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                } else if link.starts_with('#') {
                    // Anchor link - check if the anchor exists in the file
                    let anchor = link.trim_start_matches('#');
                    if !content.contains(&format!("id=\"{}\"", anchor)) && !content.contains(&format!("name=\"{}\"", anchor)) {
                        // Check for Markdown headings that would create anchors
                        let heading_pattern = format!("# {}", anchor);
                        if !content.contains(&heading_pattern) {
                            suite.add_result(TestResult::new(
                                format!("{}-anchor", link_id),
                                "Anchor Link",
                                "Validates that anchor links point to existing anchors",
                                TestSeverity::Error,
                                format!("Anchor not found: {}", link),
                            )
                            .with_file_path(file_path.to_string_lossy().to_string())
                            .with_line_number(line_number));
                        }
                    }
                } else {
                    // Internal link - check if the file exists
                    let target_path = if link.contains('#') {
                        link.split('#').next().unwrap()
                    } else {
                        &link
                    };

                    let file_dir = file_path.parent().unwrap_or(Path::new(""));
                    let target_file = file_dir.join(target_path);
                    let relative_target = target_file.strip_prefix(docs_dir).unwrap_or(&target_file);

                    if !all_files.contains(&relative_target.to_string_lossy().to_string()) && !target_file.exists() {
                        suite.add_result(TestResult::new(
                            format!("{}-internal", link_id),
                            "Internal Link",
                            "Validates that internal links point to existing files",
                            TestSeverity::Error,
                            format!("File not found: {}", link),
                        )
                        .with_file_path(file_path.to_string_lossy().to_string())
                        .with_line_number(line_number));
                    }
                }
            }
        }

        // Set suite duration
        *suite = suite.clone().with_duration(start_time.elapsed());

        Ok(())
    }

    /// Extract links from Markdown content
    fn extract_links(&self, content: &str) -> Vec<(String, u32)> {
        let mut links = Vec::new();

        // Match Markdown links: [text](url)
        let md_link_regex = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            for capture in md_link_regex.captures_iter(line) {
                if let Some(url_match) = capture.get(2) {
                    links.push((url_match.as_str().to_string(), line_number));
                }
            }
        }

        // Match HTML links: <a href="url">
        let html_link_regex = Regex::new(r#"<a\s+(?:[^>]*?\s+)?href="([^"]*)"#).unwrap();
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            for capture in html_link_regex.captures_iter(line) {
                if let Some(url_match) = capture.get(1) {
                    links.push((url_match.as_str().to_string(), line_number));
                }
            }
        }

        links
    }

    /// Test API documentation
    fn test_api_docs(&self, docs_dir: &Path, suite: &mut TestSuite) -> io::Result<()> {
        info!("Testing API documentation");
        let start_time = Instant::now();

        // Find API documentation files
        let api_docs_dir = docs_dir.join("api");
        if !api_docs_dir.exists() {
            // No API docs directory, skip this test
            return Ok(());
        }

        // Find all API documentation files
        let api_docs_files = self.find_files(&api_docs_dir, &["md", "html"])?;

        // Find all source code files
        let src_dir = Path::new("src");
        let src_tauri_dir = Path::new("src-tauri/src");

        let mut source_files = Vec::new();
        if src_dir.exists() {
            source_files.extend(self.find_files(src_dir, &["ts", "tsx", "js", "jsx"])?);
        }
        if src_tauri_dir.exists() {
            source_files.extend(self.find_files(src_tauri_dir, &["rs"])?);
        }

        // Extract API definitions from source code
        let api_definitions = self.extract_api_definitions(&source_files)?;

        // Check API documentation for each file
        for file_path in api_docs_files {
            // Read file content
            let content = fs::read_to_string(&file_path)?;

            // Extract API references from documentation
            let api_references = self.extract_api_references(&content);

            for (api_name, line_number) in api_references {
                let api_id = format!("api-{}-{}", file_path.to_string_lossy().to_string(), line_number);

                // Check if the API exists in the source code
                if api_definitions.contains(&api_name) {
                    suite.add_result(TestResult::new(
                        format!("{}-exists", api_id),
                        "API Reference",
                        "Validates that API references in documentation match the actual code",
                        TestSeverity::Info,
                        format!("API exists: {}", api_name),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                } else {
                    // Check for similar APIs (possible typos or renamed APIs)
                    let similar_apis: Vec<&String> = api_definitions.iter()
                        .filter(|name| name.contains(&api_name) || api_name.contains(name))
                        .collect();

                    if !similar_apis.is_empty() {
                        suite.add_result(TestResult::new(
                            format!("{}-similar", api_id),
                            "API Reference",
                            "Validates that API references in documentation match the actual code",
                            TestSeverity::Warning,
                            format!("API not found: {}, but similar APIs exist: {}", api_name, similar_apis.join(", ")),
                        )
                        .with_file_path(file_path.to_string_lossy().to_string())
                        .with_line_number(line_number));
                    } else {
                        suite.add_result(TestResult::new(
                            format!("{}-missing", api_id),
                            "API Reference",
                            "Validates that API references in documentation match the actual code",
                            TestSeverity::Error,
                            format!("API not found: {}", api_name),
                        )
                        .with_file_path(file_path.to_string_lossy().to_string())
                        .with_line_number(line_number));
                    }
                }
            }
        }

        // Set suite duration
        *suite = suite.clone().with_duration(start_time.elapsed());

        Ok(())
    }

    /// Extract API definitions from source code
    fn extract_api_definitions(&self, source_files: &[PathBuf]) -> io::Result<HashSet<String>> {
        let mut api_definitions = HashSet::new();

        for file_path in source_files {
            // Read file content
            let content = fs::read_to_string(file_path)?;

            // Extract API definitions based on file extension
            if let Some(extension) = file_path.extension() {
                match extension.to_string_lossy().as_ref() {
                    "rs" => {
                        self.extract_rust_api_definitions(&content, &mut api_definitions);
                    }
                    "ts" | "tsx" => {
                        self.extract_typescript_api_definitions(&content, &mut api_definitions);
                    }
                    "js" | "jsx" => {
                        self.extract_javascript_api_definitions(&content, &mut api_definitions);
                    }
                    _ => {
                        // Unsupported file type
                    }
                }
            }
        }

        Ok(api_definitions)
    }

    /// Extract API definitions from Rust code
    fn extract_rust_api_definitions(&self, content: &str, api_definitions: &mut HashSet<String>) {
        // Extract struct definitions
        let struct_regex = Regex::new(r"(?m)^(?:pub\s+)?struct\s+([A-Za-z0-9_]+)").unwrap();
        for capture in struct_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract enum definitions
        let enum_regex = Regex::new(r"(?m)^(?:pub\s+)?enum\s+([A-Za-z0-9_]+)").unwrap();
        for capture in enum_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract trait definitions
        let trait_regex = Regex::new(r"(?m)^(?:pub\s+)?trait\s+([A-Za-z0-9_]+)").unwrap();
        for capture in trait_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract function definitions
        let fn_regex = Regex::new(r"(?m)^(?:pub\s+)?fn\s+([A-Za-z0-9_]+)").unwrap();
        for capture in fn_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract impl blocks
        let impl_regex = Regex::new(r"(?m)^(?:pub\s+)?impl(?:<[^>]*>)?\s+([A-Za-z0-9_]+)").unwrap();
        for capture in impl_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract const definitions
        let const_regex = Regex::new(r"(?m)^(?:pub\s+)?const\s+([A-Za-z0-9_]+)").unwrap();
        for capture in const_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract static definitions
        let static_regex = Regex::new(r"(?m)^(?:pub\s+)?static\s+([A-Za-z0-9_]+)").unwrap();
        for capture in static_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract type definitions
        let type_regex = Regex::new(r"(?m)^(?:pub\s+)?type\s+([A-Za-z0-9_]+)").unwrap();
        for capture in type_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }
    }

    /// Extract API definitions from TypeScript code
    fn extract_typescript_api_definitions(&self, content: &str, api_definitions: &mut HashSet<String>) {
        // Extract class definitions
        let class_regex = Regex::new(r"(?m)^(?:export\s+)?(?:abstract\s+)?class\s+([A-Za-z0-9_]+)").unwrap();
        for capture in class_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract interface definitions
        let interface_regex = Regex::new(r"(?m)^(?:export\s+)?interface\s+([A-Za-z0-9_]+)").unwrap();
        for capture in interface_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract type definitions
        let type_regex = Regex::new(r"(?m)^(?:export\s+)?type\s+([A-Za-z0-9_]+)").unwrap();
        for capture in type_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract enum definitions
        let enum_regex = Regex::new(r"(?m)^(?:export\s+)?enum\s+([A-Za-z0-9_]+)").unwrap();
        for capture in enum_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract function definitions
        let fn_regex = Regex::new(r"(?m)^(?:export\s+)?function\s+([A-Za-z0-9_]+)").unwrap();
        for capture in fn_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract const definitions
        let const_regex = Regex::new(r"(?m)^(?:export\s+)?const\s+([A-Za-z0-9_]+)").unwrap();
        for capture in const_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract namespace definitions
        let namespace_regex = Regex::new(r"(?m)^(?:export\s+)?namespace\s+([A-Za-z0-9_]+)").unwrap();
        for capture in namespace_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }
    }

    /// Extract API definitions from JavaScript code
    fn extract_javascript_api_definitions(&self, content: &str, api_definitions: &mut HashSet<String>) {
        // Extract class definitions
        let class_regex = Regex::new(r"(?m)^(?:export\s+)?class\s+([A-Za-z0-9_]+)").unwrap();
        for capture in class_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract function definitions
        let fn_regex = Regex::new(r"(?m)^(?:export\s+)?function\s+([A-Za-z0-9_]+)").unwrap();
        for capture in fn_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract const definitions
        let const_regex = Regex::new(r"(?m)^(?:export\s+)?const\s+([A-Za-z0-9_]+)").unwrap();
        for capture in const_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract let definitions
        let let_regex = Regex::new(r"(?m)^(?:export\s+)?let\s+([A-Za-z0-9_]+)").unwrap();
        for capture in let_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }

        // Extract var definitions
        let var_regex = Regex::new(r"(?m)^(?:export\s+)?var\s+([A-Za-z0-9_]+)").unwrap();
        for capture in var_regex.captures_iter(content) {
            if let Some(name_match) = capture.get(1) {
                api_definitions.insert(name_match.as_str().to_string());
            }
        }
    }

    /// Extract API references from documentation
    fn extract_api_references(&self, content: &str) -> Vec<(String, u32)> {
        let mut references = Vec::new();

        // Match code references in backticks: `ApiName`
        let code_regex = Regex::new(r"`([A-Za-z0-9_]+)`").unwrap();
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            for capture in code_regex.captures_iter(line) {
                if let Some(name_match) = capture.get(1) {
                    references.push((name_match.as_str().to_string(), line_number));
                }
            }
        }

        // Match code references in code blocks
        let mut in_code_block = false;
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            if line.starts_with("```") {
                in_code_block = !in_code_block;
            } else if in_code_block {
                // Extract potential API names from code blocks
                let word_regex = Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b").unwrap();
                for capture in word_regex.captures_iter(line) {
                    if let Some(name_match) = capture.get(1) {
                        references.push((name_match.as_str().to_string(), line_number));
                    }
                }
            }
        }

        references
    }

    /// Test documentation freshness
    fn test_freshness(&self, docs_dir: &Path, suite: &mut TestSuite) -> io::Result<()> {
        info!("Testing documentation freshness");
        let start_time = Instant::now();

        // Find all documentation files
        let doc_files = self.find_files(docs_dir, &["md", "html", "txt"])?;

        // Get the current time
        let now = chrono::Utc::now();

        // Check each file's modification time
        for file_path in doc_files {
            // Check if the file has a "reviewed" metadata marker
            let is_reviewed = self.check_reviewed_metadata(&file_path)?;

            let metadata = fs::metadata(&file_path)?;
            let modified = metadata.modified()?;

            // Convert to DateTime<Utc>
            let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);

            // Calculate age in days
            let age = now.signed_duration_since(modified_time).num_days();

            // Skip age check if the file has been explicitly reviewed
            if is_reviewed {
                suite.add_result(TestResult::new(
                    format!("freshness-{}", file_path.to_string_lossy().to_string()),
                    "Documentation Freshness",
                    "Checks if documentation has been updated recently",
                    TestSeverity::Info,
                    format!("Document has been explicitly reviewed and marked as up-to-date"),
                )
                .with_file_path(file_path.to_string_lossy().to_string()));
            } else if age > self.config.max_doc_age_days as i64 {
                // Document is older than the maximum age
                // Use the configured severity (error or warning)
                let severity = if self.config.outdated_docs_as_errors {
                    TestSeverity::Error
                } else {
                    TestSeverity::Warning
                };

                suite.add_result(TestResult::new(
                    format!("freshness-{}", file_path.to_string_lossy().to_string()),
                    "Documentation Freshness",
                    "Checks if documentation has been updated recently",
                    severity,
                    format!("Document is {} days old (older than the maximum age of {} days)", age, self.config.max_doc_age_days),
                )
                .with_file_path(file_path.to_string_lossy().to_string()));
            } else {
                // Document is fresh
                suite.add_result(TestResult::new(
                    format!("freshness-{}", file_path.to_string_lossy().to_string()),
                    "Documentation Freshness",
                    "Checks if documentation has been updated recently",
                    TestSeverity::Info,
                    format!("Document is {} days old (within the maximum age of {} days)", age, self.config.max_doc_age_days),
                )
                .with_file_path(file_path.to_string_lossy().to_string()));
            }

            // Check for outdated screenshots if enabled
            if self.config.check_screenshots {
                self.check_screenshots_freshness(&file_path, suite)?;
            }

            // Check for code references if enabled
            if self.config.check_code_references {
                self.check_code_references(&file_path, suite)?;
            }
        }

        // Set suite duration
        *suite = suite.clone().with_duration(start_time.elapsed());

        Ok(())
    }

    /// Check if a document has been explicitly reviewed
    fn check_reviewed_metadata(&self, file_path: &Path) -> io::Result<bool> {
        // Read file content
        let content = fs::read_to_string(file_path)?;

        // Look for review metadata in the file
        // Format: <!-- REVIEWED: YYYY-MM-DD -->
        let review_regex = Regex::new(r"<!--\s*REVIEWED:\s*(\d{4}-\d{2}-\d{2})\s*-->").unwrap();

        if let Some(captures) = review_regex.captures(&content) {
            if let Some(date_match) = captures.get(1) {
                let review_date_str = date_match.as_str();

                // Parse the review date
                if let Ok(review_date) = chrono::NaiveDate::parse_from_str(review_date_str, "%Y-%m-%d") {
                    // Convert to DateTime<Utc>
                    let review_datetime = chrono::DateTime::<chrono::Utc>::from_utc(
                        chrono::NaiveDateTime::new(review_date, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()),
                        chrono::Utc,
                    );

                    // Get the current time
                    let now = chrono::Utc::now();

                    // Calculate age in days
                    let age = now.signed_duration_since(review_datetime).num_days();

                    // If the review is newer than max_doc_age_days, consider it reviewed
                    return Ok(age <= self.config.max_doc_age_days as i64);
                }
            }
        }

        Ok(false)
    }

    /// Check code references in documentation
    fn check_code_references(&self, file_path: &Path, suite: &mut TestSuite) -> io::Result<()> {
        // Only check Markdown files
        if let Some(ext) = file_path.extension() {
            if ext.to_string_lossy() != "md" {
                return Ok(());
            }
        } else {
            return Ok(());
        }

        // Read file content
        let content = fs::read_to_string(file_path)?;

        // Extract code references
        let code_references = self.extract_code_references(&content);

        // Find all source code files
        let src_dir = Path::new("src");
        let src_tauri_dir = Path::new("src-tauri/src");

        let mut source_files = Vec::new();
        if src_dir.exists() {
            source_files.extend(self.find_files(src_dir, &["ts", "tsx", "js", "jsx"])?);
        }
        if src_tauri_dir.exists() {
            source_files.extend(self.find_files(src_tauri_dir, &["rs"])?);
        }

        // Extract code definitions from source code
        let code_definitions = self.extract_code_definitions(&source_files)?;

        // Check each code reference
        for (code_ref, line_number) in code_references {
            let ref_id = format!("code-ref-{}-{}", file_path.to_string_lossy().to_string(), line_number);

            // Skip short references (likely not code)
            if code_ref.len() < 3 {
                continue;
            }

            // Check if the code reference exists in the source code
            if code_definitions.contains(&code_ref) {
                suite.add_result(TestResult::new(
                    format!("{}-exists", ref_id),
                    "Code Reference",
                    "Validates that code references in documentation match the actual code",
                    TestSeverity::Info,
                    format!("Code reference exists: {}", code_ref),
                )
                .with_file_path(file_path.to_string_lossy().to_string())
                .with_line_number(line_number));
            } else {
                // Check for similar code references (possible typos or renamed code)
                let similar_refs: Vec<&String> = code_definitions.iter()
                    .filter(|name| name.contains(&code_ref) || code_ref.contains(name))
                    .collect();

                if !similar_refs.is_empty() {
                    suite.add_result(TestResult::new(
                        format!("{}-similar", ref_id),
                        "Code Reference",
                        "Validates that code references in documentation match the actual code",
                        TestSeverity::Warning,
                        format!("Code reference not found: {}, but similar references exist: {}", code_ref, similar_refs.join(", ")),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                } else {
                    // Only report errors for references that look like code (CamelCase or snake_case)
                    let is_camel_case = code_ref.chars().next().unwrap_or('_').is_uppercase() 
                        && code_ref.contains(char::is_uppercase);
                    let is_snake_case = code_ref.contains('_');

                    if is_camel_case || is_snake_case {
                        suite.add_result(TestResult::new(
                            format!("{}-missing", ref_id),
                            "Code Reference",
                            "Validates that code references in documentation match the actual code",
                            TestSeverity::Warning,
                            format!("Code reference not found: {}", code_ref),
                        )
                        .with_file_path(file_path.to_string_lossy().to_string())
                        .with_line_number(line_number));
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract code references from documentation
    fn extract_code_references(&self, content: &str) -> Vec<(String, u32)> {
        let mut references = Vec::new();

        // Match inline code references: `CodeReference`
        let code_regex = Regex::new(r"`([A-Za-z0-9_]+)`").unwrap();
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            for capture in code_regex.captures_iter(line) {
                if let Some(name_match) = capture.get(1) {
                    references.push((name_match.as_str().to_string(), line_number));
                }
            }
        }

        // Match code references in code blocks
        let mut in_code_block = false;
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            if line.starts_with("```") {
                in_code_block = !in_code_block;
            } else if in_code_block {
                // Extract potential code references from code blocks
                // Look for CamelCase or snake_case identifiers
                let word_regex = Regex::new(r"\b([A-Z][A-Za-z0-9_]*|[a-z][a-z0-9_]*_[a-z0-9_]+)\b").unwrap();
                for capture in word_regex.captures_iter(line) {
                    if let Some(name_match) = capture.get(1) {
                        references.push((name_match.as_str().to_string(), line_number));
                    }
                }
            }
        }

        references
    }

    /// Extract code definitions from source code
    fn extract_code_definitions(&self, source_files: &[PathBuf]) -> io::Result<HashSet<String>> {
        let mut code_definitions = HashSet::new();

        for file_path in source_files {
            // Read file content
            let content = fs::read_to_string(file_path)?;

            // Extract code definitions based on file extension
            if let Some(extension) = file_path.extension() {
                match extension.to_string_lossy().as_ref() {
                    "rs" => {
                        self.extract_rust_api_definitions(&content, &mut code_definitions);
                    }
                    "ts" | "tsx" => {
                        self.extract_typescript_api_definitions(&content, &mut code_definitions);
                    }
                    "js" | "jsx" => {
                        self.extract_javascript_api_definitions(&content, &mut code_definitions);
                    }
                    _ => {
                        // Unsupported file type
                    }
                }
            }
        }

        Ok(code_definitions)
    }

    /// Check screenshots freshness
    fn check_screenshots_freshness(&self, file_path: &Path, suite: &mut TestSuite) -> io::Result<()> {
        // Read file content
        let content = fs::read_to_string(file_path)?;

        // Extract image references
        let image_references = self.extract_image_references(&content);

        for (image_path, line_number) in image_references {
            // Skip external images
            if image_path.starts_with("http://") || image_path.starts_with("https://") {
                continue;
            }

            // Resolve image path relative to the document
            let file_dir = file_path.parent().unwrap_or(Path::new(""));
            let image_file_path = file_dir.join(&image_path);

            if image_file_path.exists() {
                let metadata = fs::metadata(&image_file_path)?;
                let modified = metadata.modified()?;

                // Convert to DateTime<Utc>
                let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);

                // Calculate age in days
                let now = chrono::Utc::now();
                let age = now.signed_duration_since(modified_time).num_days();

                if age > self.config.max_doc_age_days as i64 {
                    // Screenshot is older than the maximum age
                    suite.add_result(TestResult::new(
                        format!("screenshot-{}-{}", file_path.to_string_lossy().to_string(), line_number),
                        "Screenshot Freshness",
                        "Checks if screenshots in documentation are up-to-date",
                        TestSeverity::Warning,
                        format!("Screenshot {} is {} days old (older than the maximum age of {} days)", image_path, age, self.config.max_doc_age_days),
                    )
                    .with_file_path(file_path.to_string_lossy().to_string())
                    .with_line_number(line_number));
                }
            }
        }

        Ok(())
    }

    /// Extract image references from content
    fn extract_image_references(&self, content: &str) -> Vec<(String, u32)> {
        let mut references = Vec::new();

        // Match Markdown image syntax: ![alt](url)
        let md_image_regex = Regex::new(r"!\[([^\]]*)\]\(([^)]+)\)").unwrap();
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            for capture in md_image_regex.captures_iter(line) {
                if let Some(url_match) = capture.get(2) {
                    references.push((url_match.as_str().to_string(), line_number));
                }
            }
        }

        // Match HTML image syntax: <img src="url">
        let html_image_regex = Regex::new(r#"<img\s+(?:[^>]*?\s+)?src="([^"]*)"#).unwrap();
        for (line_index, line) in content.lines().enumerate() {
            let line_number = line_index as u32 + 1;

            for capture in html_image_regex.captures_iter(line) {
                if let Some(url_match) = capture.get(1) {
                    references.push((url_match.as_str().to_string(), line_number));
                }
            }
        }

        references
    }

    /// Find files with specific extensions
    fn find_files(&self, dir: &Path, extensions: &[&str]) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        if !dir.exists() || !dir.is_dir() {
            return Ok(files);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip excluded directories
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                if self.config.exclude_dirs.contains(&dir_name) {
                    continue;
                }

                // Skip directories matching exclude patterns
                let should_skip = self.config.exclude_patterns.iter()
                    .any(|pattern| {
                        let regex = Regex::new(pattern).unwrap();
                        regex.is_match(&dir_name)
                    });

                if should_skip {
                    continue;
                }

                // Recursively search subdirectories
                let mut subdir_files = self.find_files(&path, extensions)?;
                files.append(&mut subdir_files);
            } else if path.is_file() {
                // Check file extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if extensions.iter().any(|&e| e == ext_str) {
                        files.push(path);
                    }
                }
            }
        }

        Ok(files)
    }

    /// Generate HTML report
    pub fn generate_html_report(&self, output_dir: &Path) -> io::Result<PathBuf> {
        let report_dir = output_dir.join("docs-test-report");
        fs::create_dir_all(&report_dir)?;

        let report_path = report_dir.join("index.html");
        let mut report_file = File::create(&report_path)?;

        writeln!(report_file, "<!DOCTYPE html>")?;
        writeln!(report_file, "<html>")?;
        writeln!(report_file, "<head>")?;
        writeln!(report_file, "    <title>Documentation Test Report</title>")?;
        writeln!(report_file, "    <meta charset=\"UTF-8\">")?;
        writeln!(report_file, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">")?;
        writeln!(report_file, "    <style>")?;
        writeln!(report_file, "        body {{ font-family: Arial, sans-serif; margin: 0; padding: 20px; }}")?;
        writeln!(report_file, "        h1, h2, h3 {{ color: #333; }}")?;
        writeln!(report_file, "        .summary {{ margin-bottom: 20px; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}")?;
        writeln!(report_file, "        .summary.passed {{ background-color: #e6ffe6; }}")?;
        writeln!(report_file, "        .summary.failed {{ background-color: #ffe6e6; }}")?;
        writeln!(report_file, "        .suite {{ margin-bottom: 20px; }}")?;
        writeln!(report_file, "        .suite-header {{ padding: 10px; background-color: #f0f0f0; border-radius: 5px; }}")?;
        writeln!(report_file, "        .suite-body {{ padding: 10px; }}")?;
        writeln!(report_file, "        .result {{ margin-bottom: 10px; padding: 10px; border-radius: 5px; }}")?;
        writeln!(report_file, "        .result.info {{ background-color: #e6f9ff; }}")?;
        writeln!(report_file, "        .result.warning {{ background-color: #fff3e6; }}")?;
        writeln!(report_file, "        .result.error {{ background-color: #ffe6e6; }}")?;
        writeln!(report_file, "        .badge {{ display: inline-block; padding: 3px 8px; border-radius: 3px; font-size: 0.8em; color: white; }}")?;
        writeln!(report_file, "        .badge-info {{ background-color: #17a2b8; }}")?;
        writeln!(report_file, "        .badge-warning {{ background-color: #fd7e14; }}")?;
        writeln!(report_file, "        .badge-error {{ background-color: #dc3545; }}")?;
        writeln!(report_file, "    </style>")?;
        writeln!(report_file, "</head>")?;
        writeln!(report_file, "<body>")?;

        // Report header
        writeln!(report_file, "    <h1>Documentation Test Report</h1>")?;

        // Summary
        let summary_class = if self.report.passed() { "passed" } else { "failed" };
        writeln!(report_file, "    <div class=\"summary {}\">", summary_class)?;
        writeln!(report_file, "        <h2>Summary</h2>")?;
        writeln!(report_file, "        <p><strong>Status:</strong> {}</p>", if self.report.passed() { "PASSED" } else { "FAILED" })?;
        writeln!(report_file, "        <p><strong>Total Tests:</strong> {}</p>", self.report.total_count())?;
        writeln!(report_file, "        <p><strong>Errors:</strong> {}</p>", self.report.error_count())?;
        writeln!(report_file, "        <p><strong>Warnings:</strong> {}</p>", self.report.warning_count())?;
        writeln!(report_file, "        <p><strong>Info:</strong> {}</p>", self.report.info_count())?;
        writeln!(report_file, "        <p><strong>Duration:</strong> {:?}</p>", self.report.duration)?;
        writeln!(report_file, "        <p><strong>Timestamp:</strong> {}</p>", self.report.timestamp)?;
        writeln!(report_file, "    </div>")?;

        // Test suites
        for suite in &self.report.suites {
            writeln!(report_file, "    <div class=\"suite\">")?;
            writeln!(report_file, "        <div class=\"suite-header\">")?;
            writeln!(report_file, "            <h2>{}</h2>", suite.name)?;
            writeln!(report_file, "            <p>{}</p>", suite.description)?;
            writeln!(report_file, "            <p><strong>Status:</strong> {}</p>", if suite.passed() { "PASSED" } else { "FAILED" })?;
            writeln!(report_file, "            <p><strong>Tests:</strong> {}</p>", suite.results.len())?;
            writeln!(report_file, "            <p><strong>Errors:</strong> {}</p>", suite.error_count())?;
            writeln!(report_file, "            <p><strong>Warnings:</strong> {}</p>", suite.warning_count())?;
            writeln!(report_file, "            <p><strong>Info:</strong> {}</p>", suite.info_count())?;
            writeln!(report_file, "            <p><strong>Duration:</strong> {:?}</p>", suite.duration)?;
            writeln!(report_file, "        </div>")?;

            writeln!(report_file, "        <div class=\"suite-body\">")?;

            // Group results by file
            let mut results_by_file: HashMap<String, Vec<&TestResult>> = HashMap::new();

            for result in &suite.results {
                let file_path = result.file_path.clone().unwrap_or_else(|| "Unknown".to_string());
                results_by_file.entry(file_path).or_default().push(result);
            }

            // Write results by file
            for (file_path, results) in results_by_file {
                writeln!(report_file, "            <h3>{}</h3>", file_path)?;

                for result in results {
                    let result_class = match result.severity {
                        TestSeverity::Info => "info",
                        TestSeverity::Warning => "warning",
                        TestSeverity::Error => "error",
                    };

                    let badge_class = match result.severity {
                        TestSeverity::Info => "badge-info",
                        TestSeverity::Warning => "badge-warning",
                        TestSeverity::Error => "badge-error",
                    };

                    writeln!(report_file, "            <div class=\"result {}\">", result_class)?;
                    writeln!(report_file, "                <p><span class=\"badge {}\">{}</span> <strong>{}</strong></p>", badge_class, result.severity, result.name)?;
                    writeln!(report_file, "                <p>{}</p>", result.message)?;

                    if let Some(line_number) = result.line_number {
                        writeln!(report_file, "                <p><strong>Line:</strong> {}</p>", line_number)?;
                    }

                    writeln!(report_file, "            </div>")?;
                }
            }

            writeln!(report_file, "        </div>")?;
            writeln!(report_file, "    </div>")?;
        }

        writeln!(report_file, "</body>")?;
        writeln!(report_file, "</html>")?;

        Ok(report_path)
    }
}

/// Global documentation tester
lazy_static::lazy_static! {
    static ref DOCS_TESTER: Arc<Mutex<DocsTester>> = Arc::new(Mutex::new(
        DocsTester::new(DocsTestingConfig::default(), DocsGenConfig::default())
    ));
}

/// Get the global documentation tester
pub fn get_docs_tester() -> Arc<Mutex<DocsTester>> {
    DOCS_TESTER.clone()
}

/// Configure documentation testing
pub fn configure(config: DocsTestingConfig, base_config: DocsGenConfig) {
    let mut tester = DOCS_TESTER.lock().unwrap();
    *tester = DocsTester::new(config, base_config);
}

/// Run documentation tests
pub fn run_docs_tests(docs_dir: &Path) -> io::Result<TestReport> {
    let mut tester = DOCS_TESTER.lock().unwrap();
    tester.run_tests(docs_dir)
}

/// Generate HTML report
pub fn generate_html_report(output_dir: &Path) -> io::Result<PathBuf> {
    let tester = DOCS_TESTER.lock().unwrap();
    tester.generate_html_report(output_dir)
}

/// Initialize the documentation testing system
pub fn init() {
    info!("Initializing documentation testing system");
}
