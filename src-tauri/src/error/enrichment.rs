//! Contextual error enrichment utilities
//!
//! This module provides utilities for enriching errors with additional context
//! to make them more informative and useful for debugging and error handling.

use std::fmt::Display;
use std::panic::Location;
use uuid::Uuid;
use tracing::{error, warn, info, debug};

use crate::error::{AppError, ErrorCategory, ErrorContext, ErrorSeverity, Result};
use crate::error::taxonomy::{ErrorCode, ErrorDomain, ErrorType, TaxonomyErrorContext, get_error_type_or_default};
use crate::logging::correlation;

/// Error enrichment builder for fluent error enrichment
pub struct ErrorEnricher<'a> {
    /// The error being enriched
    error: &'a AppError,
    /// The context to add to the error
    context: ErrorContext,
    /// Whether to log the error
    should_log: bool,
    /// The log level to use
    log_level: LogLevel,
}

/// Log level for error logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Error level (for serious errors)
    Error,
    /// Warning level (for less serious errors)
    Warning,
    /// Info level (for informational errors)
    Info,
    /// Debug level (for debugging errors)
    Debug,
}

impl<'a> ErrorEnricher<'a> {
    /// Create a new error enricher for the given error
    pub fn new(error: &'a AppError) -> Self {
        // Extract existing context if available
        let mut context = match error {
            AppError::ContextualError { context, .. } => context.clone(),
            _ => ErrorContext::new()
                .with_category(error.category())
                .with_severity(error.severity())
                .with_retriable(error.is_retriable()),
        };

        // Add the current correlation ID if available and not already set
        if context.correlation_id.is_none() {
            if let Some(correlation_id) = correlation::get_correlation_id() {
                context.correlation_id = Some(correlation_id);
            }
        }

        Self {
            error,
            context,
            should_log: false,
            log_level: LogLevel::Error,
        }
    }

    /// Set the source location
    #[track_caller]
    pub fn with_source_location(mut self) -> Self {
        let location = Location::caller();
        self.context = self.context.with_source_location(
            location.file(),
            location.line(),
        );
        self
    }

    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        let correlation_id = correlation_id.into();
        self.context.correlation_id = Some(correlation_id.clone());
        // Also set the correlation ID in the thread-local storage
        correlation::set_correlation_id(correlation_id);
        self
    }

    /// Generate and set a new correlation ID
    pub fn with_new_correlation_id(self) -> Self {
        let correlation_id = correlation::generate_correlation_id();
        self.with_correlation_id(correlation_id)
    }

    /// Create a child correlation ID from the current one
    pub fn with_child_correlation_id(self) -> Self {
        let parent_id = self.context.correlation_id.clone()
            .or_else(|| correlation::get_correlation_id())
            .unwrap_or_else(|| correlation::generate_correlation_id());
        let child_id = correlation::create_child_correlation_id(&parent_id);
        self.with_correlation_id(child_id)
    }

    /// Set the request ID
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.context = self.context.with_request_id(request_id);
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.context = self.context.with_user_id(user_id);
        self
    }

    /// Set the workspace ID
    pub fn with_workspace_id(mut self, workspace_id: Uuid) -> Self {
        self.context = self.context.with_workspace_id(workspace_id);
        self
    }

    /// Set the severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.context = self.context.with_severity(severity);
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: ErrorCategory) -> Self {
        self.context = self.context.with_category(category);
        self
    }

    /// Set the operation
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.context = self.context.with_operation(operation);
        self
    }

    /// Set the entity type
    pub fn with_entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.context = self.context.with_entity_type(entity_type);
        self
    }

    /// Set the entity ID
    pub fn with_entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.context = self.context.with_entity_id(entity_id);
        self
    }

    /// Add additional context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context = self.context.with_context(key, value);
        self
    }

    /// Set whether the error is retriable
    pub fn with_retriable(mut self, retriable: bool) -> Self {
        self.context = self.context.with_retriable(retriable);
        self
    }

    /// Set the suggested user action
    pub fn with_user_action(mut self, user_action: impl Into<String>) -> Self {
        self.context = self.context.with_user_action(user_action);
        self
    }

    /// Set the developer action
    pub fn with_developer_action(mut self, developer_action: impl Into<String>) -> Self {
        self.context = self.context.with_developer_action(developer_action);
        self
    }

    /// Set the error code from the taxonomy
    pub fn with_error_code(mut self, code: ErrorCode) -> Self {
        let error_type = get_error_type_or_default(code);
        let taxonomy_context = TaxonomyErrorContext {
            base: self.context.clone(),
            error_code: code,
            error_type: error_type.name,
            domain: error_type.domain,
            category: error_type.category,
        };
        self.context = taxonomy_context.to_base();
        self
    }

    /// Enable logging for the error
    pub fn with_logging(mut self) -> Self {
        self.should_log = true;
        self
    }

    /// Set the log level
    pub fn with_log_level(mut self, level: LogLevel) -> Self {
        self.log_level = level;
        self.should_log = true;
        self
    }

    /// Build the enriched error
    pub fn build(self) -> AppError {
        // Ensure the context has a correlation ID
        let context = if self.context.correlation_id.is_none() {
            let correlation_id = correlation::get_or_generate_correlation_id();
            let mut context = self.context.clone();
            context.correlation_id = Some(correlation_id.clone());
            correlation::set_correlation_id(correlation_id);
            context
        } else {
            // Make sure the correlation ID is set in thread-local storage
            if let Some(correlation_id) = &self.context.correlation_id {
                correlation::set_correlation_id(correlation_id.clone());
            }
            self.context
        };

        let enriched = match self.error {
            AppError::ContextualError { message, .. } => {
                AppError::with_context(message.clone(), context)
            }
            _ => {
                AppError::with_context(self.error.to_string(), context)
            }
        };

        // Log the error if requested
        if self.should_log {
            // Use the correlation ID from the context for logging
            if let Some(correlation_id) = &context.correlation_id {
                correlation::with_correlation_id(correlation_id.clone(), || {
                    match self.log_level {
                        LogLevel::Error => error!("{}", enriched),
                        LogLevel::Warning => warn!("{}", enriched),
                        LogLevel::Info => info!("{}", enriched),
                        LogLevel::Debug => debug!("{}", enriched),
                    }
                });
            } else {
                match self.log_level {
                    LogLevel::Error => error!("{}", enriched),
                    LogLevel::Warning => warn!("{}", enriched),
                    LogLevel::Info => info!("{}", enriched),
                    LogLevel::Debug => debug!("{}", enriched),
                }
            }
        }

        enriched
    }
}

/// Extension trait for AppError to add enrichment methods
pub trait ErrorEnrichmentExt {
    /// Enrich the error with additional context
    fn enrich(&self) -> ErrorEnricher<'_>;

    /// Enrich the error with source location
    #[track_caller]
    fn with_source_location(&self) -> AppError;

    /// Enrich the error with an error code from the taxonomy
    fn with_error_code(&self, code: ErrorCode) -> AppError;

    /// Enrich the error and log it
    #[track_caller]
    fn log(&self) -> AppError;
}

impl ErrorEnrichmentExt for AppError {
    fn enrich(&self) -> ErrorEnricher<'_> {
        ErrorEnricher::new(self)
    }

    #[track_caller]
    fn with_source_location(&self) -> AppError {
        self.enrich().with_source_location().build()
    }

    fn with_error_code(&self, code: ErrorCode) -> AppError {
        self.enrich().with_error_code(code).build()
    }

    #[track_caller]
    fn log(&self) -> AppError {
        self.enrich().with_source_location().with_logging().build()
    }
}

/// Extension trait for Result to add enrichment methods
pub trait ResultEnrichmentExt<T> {
    /// Enrich the error with additional context if the result is an error
    fn enrich_err<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(ErrorEnricher<'_>) -> ErrorEnricher<'_>;

    /// Enrich the error with source location if the result is an error
    #[track_caller]
    fn with_source_location(self) -> Result<T>;

    /// Enrich the error with an error code from the taxonomy if the result is an error
    fn with_error_code(self, code: ErrorCode) -> Result<T>;

    /// Enrich and log the error if the result is an error
    #[track_caller]
    fn log_err(self) -> Result<T>;
}

impl<T> ResultEnrichmentExt<T> for Result<T> {
    fn enrich_err<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(ErrorEnricher<'_>) -> ErrorEnricher<'_>,
    {
        self.map_err(|err| f(err.enrich()).build())
    }

    #[track_caller]
    fn with_source_location(self) -> Result<T> {
        self.map_err(|err| err.with_source_location())
    }

    fn with_error_code(self, code: ErrorCode) -> Result<T> {
        self.map_err(|err| err.with_error_code(code))
    }

    #[track_caller]
    fn log_err(self) -> Result<T> {
        self.map_err(|err| err.log())
    }
}

/// Create a contextual error with the given message and context builder
#[macro_export]
macro_rules! contextual_error_with {
    ($message:expr, $builder:expr) => {{
        let context = $crate::error::ErrorContext::new();
        let enricher = $crate::error::enrichment::ErrorEnricher {
            error: &$crate::error::AppError::InternalError($message.to_string()),
            context,
            should_log: false,
            log_level: $crate::error::enrichment::LogLevel::Error,
        };
        let enricher = $builder(enricher);
        enricher.build()
    }};
}

/// Create a taxonomic error with the given code and context builder
#[macro_export]
macro_rules! taxonomic_error_with {
    ($code:expr, $builder:expr) => {{
        let code = $crate::error::taxonomy::ErrorCode($code);
        let error_type = $crate::error::taxonomy::get_error_type_or_default(code);
        let message = error_type.description.clone();
        let context = $crate::error::ErrorContext::new();
        let enricher = $crate::error::enrichment::ErrorEnricher {
            error: &$crate::error::AppError::InternalError(message),
            context,
            should_log: false,
            log_level: $crate::error::enrichment::LogLevel::Error,
        };
        let enricher = $builder(enricher.with_error_code(code));
        enricher.build()
    }};
    ($code:expr, $message:expr, $builder:expr) => {{
        let code = $crate::error::taxonomy::ErrorCode($code);
        let context = $crate::error::ErrorContext::new();
        let enricher = $crate::error::enrichment::ErrorEnricher {
            error: &$crate::error::AppError::InternalError($message.to_string()),
            context,
            should_log: false,
            log_level: $crate::error::enrichment::LogLevel::Error,
        };
        let enricher = $builder(enricher.with_error_code(code));
        enricher.build()
    }};
}

/// Enrich an error with the given context builder
#[macro_export]
macro_rules! enrich_error_with {
    ($err:expr, $builder:expr) => {{
        let enricher = $crate::error::enrichment::ErrorEnricher::new(&$err);
        let enricher = $builder(enricher);
        enricher.build()
    }};
}

/// Log an error with the given context builder
#[macro_export]
macro_rules! log_error_with {
    ($err:expr, $builder:expr) => {{
        let enricher = $crate::error::enrichment::ErrorEnricher::new(&$err);
        let enricher = $builder(enricher.with_logging());
        enricher.build()
    }};
}

/// Create a contextual error with source location
#[track_caller]
pub fn with_location<E: Into<AppError>>(error: E) -> AppError {
    let error = error.into();
    error.with_source_location()
}

/// Create a contextual error with the given operation
pub fn with_operation<E: Into<AppError>>(error: E, operation: impl Into<String>) -> AppError {
    let error = error.into();
    error.enrich().with_operation(operation).build()
}

/// Create a contextual error with the given entity
pub fn with_entity<E: Into<AppError>>(
    error: E,
    entity_type: impl Into<String>,
    entity_id: impl Display,
) -> AppError {
    let error = error.into();
    error.enrich()
        .with_entity_type(entity_type)
        .with_entity_id(entity_id.to_string())
        .build()
}

/// Create a contextual error with the given user
pub fn with_user<E: Into<AppError>>(error: E, user_id: impl Into<String>) -> AppError {
    let error = error.into();
    error.enrich().with_user_id(user_id).build()
}

/// Create a contextual error with the given workspace
pub fn with_workspace<E: Into<AppError>>(error: E, workspace_id: Uuid) -> AppError {
    let error = error.into();
    error.enrich().with_workspace_id(workspace_id).build()
}

/// Create a contextual error with the given correlation ID
pub fn with_correlation<E: Into<AppError>>(error: E, correlation_id: impl Into<String>) -> AppError {
    let error = error.into();
    error.enrich().with_correlation_id(correlation_id).build()
}

/// Create a contextual error with the given request ID
pub fn with_request<E: Into<AppError>>(error: E, request_id: impl Into<String>) -> AppError {
    let error = error.into();
    error.enrich().with_request_id(request_id).build()
}

/// Create a contextual error with the given error code
pub fn with_code<E: Into<AppError>>(error: E, code: ErrorCode) -> AppError {
    let error = error.into();
    error.with_error_code(code)
}

/// Log an error
#[track_caller]
pub fn log_error<E: Into<AppError>>(error: E) -> AppError {
    let error = error.into();
    error.log()
}

/// Log an error with the given log level
#[track_caller]
pub fn log_error_with_level<E: Into<AppError>>(error: E, level: LogLevel) -> AppError {
    let error = error.into();
    error.enrich().with_source_location().with_log_level(level).build()
}

/// Create an error with a correlation ID
pub fn with_correlation_id<E: Into<AppError>>(error: E, correlation_id: impl Into<String>) -> AppError {
    let error = error.into();
    error.enrich().with_correlation_id(correlation_id).build()
}

/// Create an error with a new correlation ID
pub fn with_new_correlation_id<E: Into<AppError>>(error: E) -> AppError {
    let error = error.into();
    error.enrich().with_new_correlation_id().build()
}

/// Create an error with a child correlation ID
pub fn with_child_correlation_id<E: Into<AppError>>(error: E) -> AppError {
    let error = error.into();
    error.enrich().with_child_correlation_id().build()
}

/// Execute a function with error correlation
pub fn with_error_correlation<F, R, E>(f: F) -> Result<R, AppError>
where
    F: FnOnce() -> std::result::Result<R, E>,
    E: Into<AppError>,
{
    // Get or generate a correlation ID
    let correlation_id = correlation::get_or_generate_correlation_id();

    // Execute the function with the correlation ID
    correlation::with_correlation_id(correlation_id.clone(), || {
        f().map_err(|e| {
            let error = e.into();
            error.enrich().with_correlation_id(correlation_id).build()
        })
    })
}

/// Execute a function with a new error correlation ID
pub fn with_new_error_correlation<F, R, E>(f: F) -> Result<R, AppError>
where
    F: FnOnce() -> std::result::Result<R, E>,
    E: Into<AppError>,
{
    // Generate a new correlation ID
    let correlation_id = correlation::generate_correlation_id();

    // Execute the function with the new correlation ID
    correlation::with_correlation_id(correlation_id.clone(), || {
        f().map_err(|e| {
            let error = e.into();
            error.enrich().with_correlation_id(correlation_id).build()
        })
    })
}

/// Execute a function with a child error correlation ID
pub fn with_child_error_correlation<F, R, E>(f: F) -> Result<R, AppError>
where
    F: FnOnce() -> std::result::Result<R, E>,
    E: Into<AppError>,
{
    // Get the parent correlation ID or generate a new one
    let parent_id = correlation::get_or_generate_correlation_id();
    let child_id = correlation::create_child_correlation_id(&parent_id);

    // Execute the function with the child correlation ID
    correlation::with_correlation_id(child_id.clone(), || {
        f().map_err(|e| {
            let error = e.into();
            error.enrich().with_correlation_id(child_id).build()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::taxonomy::ErrorCategory as TaxonomyErrorCategory;

    #[test]
    fn test_error_enricher() {
        let error = AppError::InternalError("Test error".to_string());
        let enriched = error.enrich()
            .with_source_location()
            .with_operation("test_operation")
            .with_entity_type("test_entity")
            .with_entity_id("123")
            .with_user_id("user123")
            .with_context("key1", "value1")
            .with_context("key2", "value2")
            .build();

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.operation, Some("test_operation".to_string()));
                assert_eq!(context.entity_type, Some("test_entity".to_string()));
                assert_eq!(context.entity_id, Some("123".to_string()));
                assert_eq!(context.user_id, Some("user123".to_string()));
                assert_eq!(context.additional_context.get("key1"), Some(&"value1".to_string()));
                assert_eq!(context.additional_context.get("key2"), Some(&"value2".to_string()));
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_error_enrichment_ext() {
        let error = AppError::InternalError("Test error".to_string());
        let enriched = error.with_source_location();

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = error.with_error_code(ErrorCode(1001));

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_result_enrichment_ext() {
        let result: Result<()> = Err(AppError::InternalError("Test error".to_string()));
        let enriched = result.with_source_location();

        match enriched {
            Err(AppError::ContextualError { message, context }) => {
                assert_eq!(message, "Test error");
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected Err(ContextualError)"),
        }

        let result: Result<()> = Err(AppError::InternalError("Test error".to_string()));
        let enriched = result.with_error_code(ErrorCode(1001));

        match enriched {
            Err(AppError::ContextualError { message, context }) => {
                assert_eq!(message, "Test error");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected Err(ContextualError)"),
        }

        let result: Result<()> = Err(AppError::InternalError("Test error".to_string()));
        let enriched = result.enrich_err(|e| e.with_operation("test_operation"));

        match enriched {
            Err(AppError::ContextualError { message, context }) => {
                assert_eq!(message, "Test error");
                assert_eq!(context.operation, Some("test_operation".to_string()));
            }
            _ => panic!("Expected Err(ContextualError)"),
        }
    }

    #[test]
    fn test_contextual_error_with_macro() {
        let error = contextual_error_with!("Test error", |e| e.with_operation("test_operation"));

        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.operation, Some("test_operation".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_taxonomic_error_with_macro() {
        let error = taxonomic_error_with!(1001, |e| e.with_operation("test_operation"));

        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "The provided credentials are invalid");
                assert_eq!(context.operation, Some("test_operation".to_string()));
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = taxonomic_error_with!(1001, "Custom message", |e| e.with_operation("test_operation"));

        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Custom message");
                assert_eq!(context.operation, Some("test_operation".to_string()));
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_enrich_error_with_macro() {
        let error = AppError::InternalError("Test error".to_string());
        let enriched = enrich_error_with!(error, |e| e.with_operation("test_operation"));

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.operation, Some("test_operation".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_utility_functions() {
        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_location(error);

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_operation(error, "test_operation");

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.operation, Some("test_operation".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_entity(error, "test_entity", "123");

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.entity_type, Some("test_entity".to_string()));
                assert_eq!(context.entity_id, Some("123".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_user(error, "user123");

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.user_id, Some("user123".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let workspace_id = Uuid::new_v4();
        let enriched = with_workspace(error, workspace_id);

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.workspace_id, Some(workspace_id));
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_correlation(error, "corr123");

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.correlation_id, Some("corr123".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_request(error, "req123");

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.request_id, Some("req123".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }

        let error = AppError::InternalError("Test error".to_string());
        let enriched = with_code(error, ErrorCode(1001));

        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected ContextualError"),
        }
    }
}
