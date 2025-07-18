//! Development-Specific Logging Utilities
//!
//! This module provides specialized logging utilities for development environments.
//! It builds on the application's core logging framework but adds features that
//! are particularly useful during development, such as enhanced formatting,
//! interactive filtering, log replay, and visualization.

use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error, Level, Span};
use uuid::Uuid;

use crate::logging::{self, LogContext, LogLevel, OperationLogger};

/// Maximum number of log entries to keep in memory
const MAX_LOG_ENTRIES: usize = 10000;

/// Development logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevLoggingConfig {
    /// Whether development logging is enabled
    pub enabled: bool,
    /// Whether to use colorized output
    pub colorized: bool,
    /// Whether to capture logs in memory
    pub capture_in_memory: bool,
    /// Maximum number of log entries to keep in memory
    pub max_log_entries: usize,
    /// Whether to log to console
    pub log_to_console: bool,
    /// Whether to log to file
    pub log_to_file: bool,
    /// Path to log file
    pub log_file_path: Option<String>,
    /// Minimum log level to capture
    pub min_log_level: LogLevel,
    /// Filters for log capture
    pub filters: Vec<LogFilter>,
    /// Additional options
    pub options: HashMap<String, String>,
}

impl Default for DevLoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            colorized: true,
            capture_in_memory: true,
            max_log_entries: MAX_LOG_ENTRIES,
            log_to_console: true,
            log_to_file: false,
            log_file_path: None,
            min_log_level: LogLevel::Debug,
            filters: Vec::new(),
            options: HashMap::new(),
        }
    }
}

/// Log filter for development logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    /// Field to filter on
    pub field: String,
    /// Value to match
    pub value: String,
    /// Whether to include or exclude matching logs
    pub include: bool,
}

/// Development log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevLogEntry {
    /// Unique ID for the log entry
    pub id: Uuid,
    /// Timestamp when the log was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
    /// Log context
    pub context: LogContext,
    /// Source file
    pub file: Option<String>,
    /// Source line
    pub line: Option<u32>,
    /// Thread ID
    pub thread_id: Option<String>,
    /// Whether this log is highlighted
    pub highlighted: bool,
    /// User-added notes
    pub notes: Option<String>,
}

impl DevLogEntry {
    /// Create a new development log entry
    pub fn new(level: LogLevel, message: String, context: LogContext) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            level,
            message,
            context,
            file: None,
            line: None,
            thread_id: None,
            highlighted: false,
            notes: None,
        }
    }

    /// Set the source location
    pub fn with_source_location(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    /// Set the thread ID
    pub fn with_thread_id(mut self, thread_id: impl Into<String>) -> Self {
        self.thread_id = Some(thread_id.into());
        self
    }

    /// Highlight this log entry
    pub fn highlight(mut self) -> Self {
        self.highlighted = true;
        self
    }

    /// Add notes to this log entry
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Format the log entry as a string
    pub fn format(&self, colorized: bool) -> String {
        let level_str = match self.level {
            LogLevel::Trace => if colorized { "\x1b[90mTRACE\x1b[0m" } else { "TRACE" },
            LogLevel::Debug => if colorized { "\x1b[36mDEBUG\x1b[0m" } else { "DEBUG" },
            LogLevel::Info => if colorized { "\x1b[32mINFO\x1b[0m" } else { "INFO" },
            LogLevel::Warn => if colorized { "\x1b[33mWARN\x1b[0m" } else { "WARN" },
            LogLevel::Error => if colorized { "\x1b[31mERROR\x1b[0m" } else { "ERROR" },
        };

        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let source = match (&self.file, &self.line) {
            (Some(file), Some(line)) => format!("{}:{}", file, line),
            _ => "unknown".to_string(),
        };

        let correlation_id = self.context.correlation_id.as_deref().unwrap_or("-");
        let operation = self.context.operation.as_deref().unwrap_or("-");

        let message = if colorized && self.highlighted {
            format!("\x1b[1m{}\x1b[0m", self.message)
        } else {
            self.message.clone()
        };

        format!(
            "{} {} [{}] [{}] [{}] {}",
            timestamp, level_str, correlation_id, operation, source, message
        )
    }
}

/// Development log manager
#[derive(Debug)]
pub struct DevLogManager {
    /// Configuration
    pub config: DevLoggingConfig,
    /// Log entries
    pub entries: VecDeque<DevLogEntry>,
    /// Log file handle (if enabled)
    pub log_file: Option<std::fs::File>,
}

impl DevLogManager {
    /// Create a new development log manager
    pub fn new(config: DevLoggingConfig) -> Self {
        let log_file = if config.log_to_file {
            if let Some(path) = &config.log_file_path {
                match std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                {
                    Ok(file) => Some(file),
                    Err(e) => {
                        eprintln!("Failed to open log file: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            config,
            entries: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            log_file,
        }
    }

    /// Log a message
    pub fn log(&mut self, level: LogLevel, message: String, context: LogContext) {
        // Check if logging is enabled
        if !self.config.enabled {
            return;
        }

        // Check log level
        if level as u8 < self.config.min_log_level as u8 {
            return;
        }

        // Apply filters
        for filter in &self.config.filters {
            let field_value = match filter.field.as_str() {
                "message" => Some(message.as_str()),
                "level" => Some(format!("{:?}", level).as_str()),
                "correlation_id" => context.correlation_id.as_deref(),
                "request_id" => context.request_id.as_deref(),
                "user_id" => context.user_id.as_deref(),
                "operation" => context.operation.as_deref(),
                "entity_type" => context.entity_type.as_deref(),
                "entity_id" => context.entity_id.as_deref(),
                _ => context.additional_context.get(&filter.field).map(|s| s.as_str()),
            };

            if let Some(value) = field_value {
                let matches = value.contains(&filter.value);
                if matches != filter.include {
                    return;
                }
            }
        }

        // Create log entry
        let entry = DevLogEntry::new(level, message.clone(), context.clone());

        // Capture in memory if enabled
        if self.config.capture_in_memory {
            self.entries.push_back(entry.clone());

            // Trim if needed
            while self.entries.len() > self.config.max_log_entries {
                self.entries.pop_front();
            }
        }

        // Log to console if enabled
        if self.config.log_to_console {
            let formatted = entry.format(self.config.colorized);
            println!("{}", formatted);
        }

        // Log to file if enabled
        if self.config.log_to_file {
            if let Some(file) = &mut self.log_file {
                let formatted = entry.format(false);
                if let Err(e) = writeln!(file, "{}", formatted) {
                    eprintln!("Failed to write to log file: {}", e);
                }
            }
        }
    }

    /// Get all log entries
    pub fn get_entries(&self) -> Vec<&DevLogEntry> {
        self.entries.iter().collect()
    }

    /// Get log entries matching a filter
    pub fn get_filtered_entries(&self, filter: &LogFilter) -> Vec<&DevLogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                let field_value = match filter.field.as_str() {
                    "message" => Some(entry.message.as_str()),
                    "level" => Some(format!("{:?}", entry.level).as_str()),
                    "correlation_id" => entry.context.correlation_id.as_deref(),
                    "request_id" => entry.context.request_id.as_deref(),
                    "user_id" => entry.context.user_id.as_deref(),
                    "operation" => entry.context.operation.as_deref(),
                    "entity_type" => entry.context.entity_type.as_deref(),
                    "entity_id" => entry.context.entity_id.as_deref(),
                    _ => entry.context.additional_context.get(&filter.field).map(|s| s.as_str()),
                };

                if let Some(value) = field_value {
                    let matches = value.contains(&filter.value);
                    matches == filter.include
                } else {
                    !filter.include
                }
            })
            .collect()
    }

    /// Get log entries for a specific correlation ID
    pub fn get_entries_by_correlation_id(&self, correlation_id: &str) -> Vec<&DevLogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.context.correlation_id.as_deref() == Some(correlation_id)
            })
            .collect()
    }

    /// Get log entries for a specific operation
    pub fn get_entries_by_operation(&self, operation: &str) -> Vec<&DevLogEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.context.operation.as_deref() == Some(operation)
            })
            .collect()
    }

    /// Clear all log entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Export logs to a file
    pub fn export_to_file(&self, path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;

        for entry in &self.entries {
            let formatted = entry.format(false);
            writeln!(file, "{}", formatted)?;
        }

        Ok(())
    }

    /// Export logs as JSON
    pub fn export_as_json(&self) -> Result<String, serde_json::Error> {
        let entries: Vec<&DevLogEntry> = self.entries.iter().collect();
        serde_json::to_string_pretty(&entries)
    }

    /// Highlight log entries matching a filter
    pub fn highlight_entries(&mut self, filter: &LogFilter) {
        for entry in self.entries.iter_mut() {
            let field_value = match filter.field.as_str() {
                "message" => Some(entry.message.as_str()),
                "level" => Some(format!("{:?}", entry.level).as_str()),
                "correlation_id" => entry.context.correlation_id.as_deref(),
                "request_id" => entry.context.request_id.as_deref(),
                "user_id" => entry.context.user_id.as_deref(),
                "operation" => entry.context.operation.as_deref(),
                "entity_type" => entry.context.entity_type.as_deref(),
                "entity_id" => entry.context.entity_id.as_deref(),
                _ => entry.context.additional_context.get(&filter.field).map(|s| s.as_str()),
            };

            if let Some(value) = field_value {
                let matches = value.contains(&filter.value);
                if matches == filter.include {
                    entry.highlighted = true;
                }
            }
        }
    }

    /// Add notes to a log entry
    pub fn add_notes(&mut self, entry_id: Uuid, notes: impl Into<String>) -> bool {
        for entry in self.entries.iter_mut() {
            if entry.id == entry_id {
                entry.notes = Some(notes.into());
                return true;
            }
        }

        false
    }
}

/// Global development log manager
lazy_static::lazy_static! {
    static ref DEV_LOG_MANAGER: Arc<Mutex<DevLogManager>> = Arc::new(Mutex::new(
        DevLogManager::new(DevLoggingConfig::default())
    ));
}

/// Get the global development log manager
pub fn get_dev_log_manager() -> Arc<Mutex<DevLogManager>> {
    DEV_LOG_MANAGER.clone()
}

/// Configure development logging
pub fn configure(config: DevLoggingConfig) {
    let mut manager = DEV_LOG_MANAGER.lock().unwrap();
    *manager = DevLogManager::new(config);
}

/// Log a message with development-specific logging
pub fn dev_log(level: LogLevel, message: impl Into<String>, context: LogContext) {
    let message = message.into();

    // Also log through the regular logging system
    logging::log_with_context(level, &message, &context);

    // Log through the development logging system
    let mut manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.log(level, message, context);
}

/// Log a message at trace level
pub fn trace(message: impl Into<String>, context: LogContext) {
    dev_log(LogLevel::Trace, message, context);
}

/// Log a message at debug level
pub fn debug(message: impl Into<String>, context: LogContext) {
    dev_log(LogLevel::Debug, message, context);
}

/// Log a message at info level
pub fn info(message: impl Into<String>, context: LogContext) {
    dev_log(LogLevel::Info, message, context);
}

/// Log a message at warn level
pub fn warn(message: impl Into<String>, context: LogContext) {
    dev_log(LogLevel::Warn, message, context);
}

/// Log a message at error level
pub fn error(message: impl Into<String>, context: LogContext) {
    dev_log(LogLevel::Error, message, context);
}

/// Get all log entries
pub fn get_all_entries() -> Vec<DevLogEntry> {
    let manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.get_entries().into_iter().cloned().collect()
}

/// Get log entries matching a filter
pub fn get_filtered_entries(filter: &LogFilter) -> Vec<DevLogEntry> {
    let manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.get_filtered_entries(filter).into_iter().cloned().collect()
}

/// Get log entries for a specific correlation ID
pub fn get_entries_by_correlation_id(correlation_id: &str) -> Vec<DevLogEntry> {
    let manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.get_entries_by_correlation_id(correlation_id).into_iter().cloned().collect()
}

/// Clear all log entries
pub fn clear_logs() {
    let mut manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.clear();
}

/// Export logs to a file
pub fn export_to_file(path: &str) -> std::io::Result<()> {
    let manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.export_to_file(path)
}

/// Export logs as JSON
pub fn export_as_json() -> Result<String, serde_json::Error> {
    let manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.export_as_json()
}

/// Highlight log entries matching a filter
pub fn highlight_entries(filter: &LogFilter) {
    let mut manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.highlight_entries(filter);
}

/// Add notes to a log entry
pub fn add_notes(entry_id: Uuid, notes: impl Into<String>) -> bool {
    let mut manager = DEV_LOG_MANAGER.lock().unwrap();
    manager.add_notes(entry_id, notes)
}

/// Initialize the development logging system
pub fn init() {
    info!("Initializing development logging system");
}

/// Macro to log a message with development-specific logging
#[macro_export]
macro_rules! dev_log {
    ($level:expr, $message:expr, $($context:tt)*) => {{
        let context = $crate::log_context!($($context)*);
        $crate::dev_tools::logging::dev_log($level, $message, context);
    }};
}

/// Macro to log a trace message with development-specific logging
#[macro_export]
macro_rules! dev_trace {
    ($message:expr, $($context:tt)*) => {{
        $crate::dev_log!($crate::logging::LogLevel::Trace, $message, $($context)*);
    }};
}

/// Macro to log a debug message with development-specific logging
#[macro_export]
macro_rules! dev_debug {
    ($message:expr, $($context:tt)*) => {{
        $crate::dev_log!($crate::logging::LogLevel::Debug, $message, $($context)*);
    }};
}

/// Macro to log an info message with development-specific logging
#[macro_export]
macro_rules! dev_info {
    ($message:expr, $($context:tt)*) => {{
        $crate::dev_log!($crate::logging::LogLevel::Info, $message, $($context)*);
    }};
}

/// Macro to log a warn message with development-specific logging
#[macro_export]
macro_rules! dev_warn {
    ($message:expr, $($context:tt)*) => {{
        $crate::dev_log!($crate::logging::LogLevel::Warn, $message, $($context)*);
    }};
}

/// Macro to log an error message with development-specific logging
#[macro_export]
macro_rules! dev_error {
    ($message:expr, $($context:tt)*) => {{
        $crate::dev_log!($crate::logging::LogLevel::Error, $message, $($context)*);
    }};
}
