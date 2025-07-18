//! Comprehensive logging framework for the application
//!
//! This module provides a structured logging approach with consistent fields,
//! correlation IDs for request tracing, and performance metrics logging.

use crate::services::traits::ServiceContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, trace, warn, Level, Span};
use uuid::Uuid;

pub mod correlation;

/// Log levels for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level for very detailed debugging
    Trace,
    /// Debug level for development information
    Debug,
    /// Info level for general operational information
    Info,
    /// Warn level for concerning but non-critical issues
    Warn,
    /// Error level for errors that affect functionality
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Logging context for structured logging
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogContext {
    /// Unique identifier for the log entry
    pub log_id: Uuid,
    /// Correlation ID for tracing related logs
    pub correlation_id: Option<String>,
    /// Request ID associated with the log
    pub request_id: Option<String>,
    /// User ID associated with the log
    pub user_id: Option<String>,
    /// Workspace ID associated with the log
    pub workspace_id: Option<Uuid>,
    /// Source of the log (file:line)
    pub source_location: Option<String>,
    /// Operation that generated the log
    pub operation: Option<String>,
    /// Entity type associated with the log
    pub entity_type: Option<String>,
    /// Entity ID associated with the log
    pub entity_id: Option<String>,
    /// Additional context as key-value pairs
    pub additional_context: HashMap<String, String>,
    /// Timestamp when the log was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Duration of the operation (if applicable)
    pub duration: Option<Duration>,
}

impl LogContext {
    /// Create a new log context with default values
    pub fn new() -> Self {
        let mut context = Self {
            log_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            ..Default::default()
        };
        
        // Add the current correlation ID if available
        if let Some(correlation_id) = correlation::get_correlation_id() {
            context.correlation_id = Some(correlation_id);
        }
        
        context
    }

    /// Create a log context from a service context
    pub fn from_service_context(ctx: &ServiceContext) -> Self {
        let mut log_context = Self::new();
        
        // Add request ID if available
        if let Some(request_id) = ctx.request_id {
            log_context = log_context.with_request_id(request_id);
            
            // Try to extract correlation ID from request ID
            // Format: "service_name.method_name:uuid"
            if request_id.contains(":") {
                let parts: Vec<&str> = request_id.split(":").collect();
                if parts.len() > 1 {
                    log_context = log_context.with_correlation_id(parts[1]);
                    // Also set the correlation ID in the thread-local storage
                    correlation::set_correlation_id(parts[1]);
                }
            }
        }
        
        // Add workspace ID if available
        if let Some(workspace_id) = ctx.workspace_id {
            log_context = log_context.with_workspace_id(workspace_id);
        }
        
        // Add user ID if available
        if let Some(auth_context) = &ctx.auth_context {
            log_context = log_context.with_user_id(auth_context.participant_id.to_string());
        }
        
        log_context
    }

    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        let correlation_id = correlation_id.into();
        self.correlation_id = Some(correlation_id.clone());
        // Also set the correlation ID in the thread-local storage
        correlation::set_correlation_id(correlation_id);
        self
    }

    /// Set the request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the workspace ID
    pub fn with_workspace_id(mut self, workspace_id: Uuid) -> Self {
        self.workspace_id = Some(workspace_id);
        self
    }

    /// Set the source location
    pub fn with_source_location(mut self, file: &str, line: u32) -> Self {
        self.source_location = Some(format!("{}:{}", file, line));
        self
    }

    /// Set the operation
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Set the entity type
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self
    }

    /// Set the entity ID
    pub fn with_entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }

    /// Add additional context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_context.insert(key.into(), value.into());
        self
    }

    /// Set the duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }
}

/// Create a tracing span from a log context
pub fn create_span(name: &str, context: &LogContext) -> Span {
    let span = tracing::span!(
        Level::INFO,
        name,
        log_id = %context.log_id,
        correlation_id = %context.correlation_id.as_deref().unwrap_or_else(|| correlation::get_correlation_id().as_deref().unwrap_or("none")),
        request_id = %context.request_id.as_deref().unwrap_or("none"),
        user_id = %context.user_id.as_deref().unwrap_or("none"),
        workspace_id = %context.workspace_id.map(|id| id.to_string()).unwrap_or_else(|| "none".to_string()),
        source_location = %context.source_location.as_deref().unwrap_or("unknown"),
        operation = %context.operation.as_deref().unwrap_or("unknown"),
        entity_type = %context.entity_type.as_deref().unwrap_or("unknown"),
        entity_id = %context.entity_id.as_deref().unwrap_or("unknown"),
        timestamp = %context.timestamp,
    );

    // Add additional context to the span
    for (key, value) in &context.additional_context {
        span.record(key, value.as_str());
    }

    // Add duration if available
    if let Some(duration) = context.duration {
        span.record("duration_ms", duration.as_millis() as u64);
    }

    span
}

/// Log a message with the given level and context
pub fn log_with_context(level: LogLevel, message: &str, context: &LogContext) {
    let span = create_span("log", context);
    let _guard = span.enter();

    match level {
        LogLevel::Trace => trace!("{}", message),
        LogLevel::Debug => debug!("{}", message),
        LogLevel::Info => info!("{}", message),
        LogLevel::Warn => warn!("{}", message),
        LogLevel::Error => error!("{}", message),
    }
}

/// Log a message at trace level with context
pub fn trace_with_context(message: &str, context: &LogContext) {
    log_with_context(LogLevel::Trace, message, context);
}

/// Log a message at debug level with context
pub fn debug_with_context(message: &str, context: &LogContext) {
    log_with_context(LogLevel::Debug, message, context);
}

/// Log a message at info level with context
pub fn info_with_context(message: &str, context: &LogContext) {
    log_with_context(LogLevel::Info, message, context);
}

/// Log a message at warn level with context
pub fn warn_with_context(message: &str, context: &LogContext) {
    log_with_context(LogLevel::Warn, message, context);
}

/// Log a message at error level with context
pub fn error_with_context(message: &str, context: &LogContext) {
    log_with_context(LogLevel::Error, message, context);
}

/// Operation logger for timing and logging operations
pub struct OperationLogger {
    /// Name of the operation
    pub name: String,
    /// Start time of the operation
    pub start_time: Instant,
    /// Log context for the operation
    pub context: LogContext,
    /// Whether to log at start and end
    pub log_start_end: bool,
}

impl OperationLogger {
    /// Create a new operation logger
    pub fn new(name: impl Into<String>, context: LogContext) -> Self {
        let name = name.into();
        let logger = Self {
            name: name.clone(),
            start_time: Instant::now(),
            context: context.with_operation(name),
            log_start_end: true,
        };
        
        // Ensure the context has a correlation ID
        if logger.context.correlation_id.is_none() {
            let correlation_id = correlation::get_or_generate_correlation_id();
            logger.context.correlation_id = Some(correlation_id.clone());
            correlation::set_correlation_id(correlation_id);
        }
        
        logger
    }

    /// Create a new operation logger from a service context
    pub fn from_service_context(name: impl Into<String>, ctx: &ServiceContext) -> Self {
        Self::new(name, LogContext::from_service_context(ctx))
    }

    /// Set whether to log at start and end
    pub fn with_log_start_end(mut self, log_start_end: bool) -> Self {
        self.log_start_end = log_start_end;
        self
    }

    /// Start the operation and log it
    pub fn start(&self) {
        if self.log_start_end {
            info_with_context(&format!("Starting operation: {}", self.name), &self.context);
        }
    }

    /// End the operation and log it
    pub fn end(&self) {
        let duration = self.start_time.elapsed();
        let context = self.context.clone().with_duration(duration);
        
        if self.log_start_end {
            info_with_context(
                &format!(
                    "Completed operation: {} in {:?}",
                    self.name,
                    duration
                ),
                &context,
            );
        }
    }

    /// Log a message during the operation
    pub fn log(&self, level: LogLevel, message: &str) {
        let duration = self.start_time.elapsed();
        let context = self.context.clone().with_duration(duration);
        log_with_context(level, message, &context);
    }

    /// Log a message at trace level
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }

    /// Log a message at debug level
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }

    /// Log a message at info level
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    /// Log a message at warn level
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }

    /// Log a message at error level
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

impl Drop for OperationLogger {
    fn drop(&mut self) {
        self.end();
    }
}

/// Macro to create a log context with source location
#[macro_export]
macro_rules! log_context {
    ($($context:tt)*) => {{
        let context = $crate::logging::LogContext::new()
            .with_source_location(file!(), line!())
            $($context)*;
        context
    }};
}

/// Macro to log a message with context
#[macro_export]
macro_rules! log_message {
    ($level:expr, $message:expr, $($context:tt)*) => {{
        let context = $crate::log_context!($($context)*);
        $crate::logging::log_with_context($level, $message, &context);
    }};
}

/// Macro to log a trace message with context
#[macro_export]
macro_rules! trace_log {
    ($message:expr, $($context:tt)*) => {{
        $crate::log_message!($crate::logging::LogLevel::Trace, $message, $($context)*);
    }};
}

/// Macro to log a debug message with context
#[macro_export]
macro_rules! debug_log {
    ($message:expr, $($context:tt)*) => {{
        $crate::log_message!($crate::logging::LogLevel::Debug, $message, $($context)*);
    }};
}

/// Macro to log an info message with context
#[macro_export]
macro_rules! info_log {
    ($message:expr, $($context:tt)*) => {{
        $crate::log_message!($crate::logging::LogLevel::Info, $message, $($context)*);
    }};
}

/// Macro to log a warn message with context
#[macro_export]
macro_rules! warn_log {
    ($message:expr, $($context:tt)*) => {{
        $crate::log_message!($crate::logging::LogLevel::Warn, $message, $($context)*);
    }};
}

/// Macro to log an error message with context
#[macro_export]
macro_rules! error_log {
    ($message:expr, $($context:tt)*) => {{
        $crate::log_message!($crate::logging::LogLevel::Error, $message, $($context)*);
    }};
}

/// Macro to time and log an operation
#[macro_export]
macro_rules! log_operation {
    ($name:expr, $ctx:expr, $body:block) => {{
        let logger = $crate::logging::OperationLogger::from_service_context($name, $ctx);
        logger.start();
        let result = (|| $body)();
        logger.end();
        result
    }};
}

/// Macro to time and log an operation with a new correlation ID
#[macro_export]
macro_rules! log_operation_with_correlation {
    ($name:expr, $ctx:expr, $body:block) => {{
        $crate::logging::correlation::with_new_correlation_id(|| {
            let logger = $crate::logging::OperationLogger::from_service_context($name, $ctx);
            logger.start();
            let result = (|| $body)();
            logger.end();
            result
        })
    }};
}

/// Macro to time and log an operation with a child correlation ID
#[macro_export]
macro_rules! log_operation_with_child_correlation {
    ($name:expr, $ctx:expr, $body:block) => {{
        $crate::logging::correlation::with_child_correlation_id(|| {
            let logger = $crate::logging::OperationLogger::from_service_context($name, $ctx);
            logger.start();
            let result = (|| $body)();
            logger.end();
            result
        })
    }};
}