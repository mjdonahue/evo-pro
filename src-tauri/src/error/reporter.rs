//! Error reporting mechanisms for tracking errors and generating user-friendly messages

use crate::error::{AppError, ErrorContext, ErrorSeverity};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use async_trait::async_trait;
use std::collections::HashMap;

/// UI-friendly error object for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiError {
    /// Unique identifier for the error
    pub id: String,
    /// User-friendly error message
    pub message: String,
    /// Detailed explanation of the error
    pub explanation: Option<String>,
    /// Suggested actions for the user to recover
    pub actions: Vec<String>,
    /// Whether this is a critical error that requires immediate attention
    pub is_critical: bool,
    /// Whether the user can continue despite this error
    pub can_continue: bool,
    /// Whether to show a support contact option
    pub show_support_contact: bool,
    /// Timestamp when the error occurred
    pub timestamp: String,
    /// Error code for reference
    pub error_code: String,
}

/// External error reporting service trait
#[async_trait]
pub trait ExternalErrorReporter: Send + Sync {
    /// Initialize the reporter with configuration
    async fn initialize(&mut self, config: HashMap<String, String>) -> Result<(), String>;

    /// Report an error to the external service
    async fn report_error(&self, report: &ErrorReport) -> Result<(), String>;

    /// Get the name of the reporter
    fn name(&self) -> &str;
}

/// Sentry error reporter implementation
pub struct SentryReporter {
    /// Sentry DSN
    dsn: Option<String>,
    /// Environment (production, staging, development)
    environment: String,
    /// Whether the reporter is initialized
    initialized: bool,
}

impl SentryReporter {
    /// Create a new Sentry reporter
    pub fn new() -> Self {
        Self {
            dsn: None,
            environment: "development".to_string(),
            initialized: false,
        }
    }
}

#[async_trait]
impl ExternalErrorReporter for SentryReporter {
    async fn initialize(&mut self, config: HashMap<String, String>) -> Result<(), String> {
        // Get DSN from config
        self.dsn = config.get("dsn").map(|s| s.to_string());

        // Get environment from config or use default
        self.environment = config.get("environment")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "development".to_string());

        // In a real implementation, this would initialize the Sentry SDK
        // For now, just log the initialization
        if let Some(dsn) = &self.dsn {
            tracing::info!(
                dsn = %dsn,
                environment = %self.environment,
                "Initialized Sentry reporter"
            );
            self.initialized = true;
            Ok(())
        } else {
            Err("Sentry DSN not provided".to_string())
        }
    }

    async fn report_error(&self, report: &ErrorReport) -> Result<(), String> {
        if !self.initialized {
            return Err("Sentry reporter not initialized".to_string());
        }

        // In a real implementation, this would send the report to Sentry
        // For now, just log the report
        tracing::info!(
            report_id = %report.report_id,
            error_id = %report.context.error_id,
            message = %report.message,
            severity = ?report.context.severity,
            category = ?report.context.category,
            environment = %self.environment,
            "Sent error report to Sentry"
        );

        Ok(())
    }

    fn name(&self) -> &str {
        "sentry"
    }
}

/// Console error reporter implementation (logs to console)
pub struct ConsoleReporter {
    /// Log level
    log_level: String,
}

impl ConsoleReporter {
    /// Create a new console reporter
    pub fn new() -> Self {
        Self {
            log_level: "info".to_string(),
        }
    }
}

#[async_trait]
impl ExternalErrorReporter for ConsoleReporter {
    async fn initialize(&mut self, config: HashMap<String, String>) -> Result<(), String> {
        // Get log level from config or use default
        self.log_level = config.get("log_level")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "info".to_string());

        tracing::info!(
            log_level = %self.log_level,
            "Initialized Console reporter"
        );

        Ok(())
    }

    async fn report_error(&self, report: &ErrorReport) -> Result<(), String> {
        // Log the error based on the configured log level
        match self.log_level.as_str() {
            "error" => {
                tracing::error!(
                    report_id = %report.report_id,
                    error_id = %report.context.error_id,
                    message = %report.message,
                    severity = ?report.context.severity,
                    category = ?report.context.category,
                    "Error report"
                );
            },
            "warn" => {
                tracing::warn!(
                    report_id = %report.report_id,
                    error_id = %report.context.error_id,
                    message = %report.message,
                    severity = ?report.context.severity,
                    category = ?report.context.category,
                    "Error report"
                );
            },
            _ => {
                tracing::info!(
                    report_id = %report.report_id,
                    error_id = %report.context.error_id,
                    message = %report.message,
                    severity = ?report.context.severity,
                    category = ?report.context.category,
                    "Error report"
                );
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "console"
    }
}

/// Error report for telemetry and user feedback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Unique identifier for the error report
    pub report_id: Uuid,
    /// Error message
    pub message: String,
    /// User-friendly message
    pub user_message: String,
    /// Error context
    pub context: ErrorContext,
    /// Timestamp when the report was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Developer action suggestion
    pub developer_action: Option<String>,
    /// Stack trace if available
    pub stack_trace: Option<String>,
    /// User recovery suggestion
    pub recovery_suggestion: UserRecoverySuggestion,
}

impl ErrorReport {
    /// Create a new error report from an error
    pub fn new(error: &AppError) -> Self {
        let (message, context) = match error {
            AppError::ContextualError { message, context } => (message.clone(), context.clone()),
            _ => {
                let message = error.to_string();
                let context = ErrorContext::new()
                    .with_category(error.category())
                    .with_severity(error.severity())
                    .with_retriable(error.is_retriable());
                (message, context)
            }
        };

        // Generate recovery suggestion for the error
        let recovery_suggestion = Self::generate_recovery_suggestion(error, &context);

        // Generate a user-friendly message based on the error
        let user_message = Self::generate_user_message(error, &context);

        // Get developer action from context or generate one
        let developer_action = context.developer_action.clone().or_else(|| {
            Self::generate_developer_action(error, &context)
        });

        // Get stack trace if available (in a real implementation, this would capture the actual stack trace)
        let stack_trace = if cfg!(debug_assertions) {
            Some(format!("Error occurred in: {}", context.source_location.as_deref().unwrap_or("unknown location")))
        } else {
            None
        };

        Self {
            report_id: Uuid::new_v4(),
            message,
            user_message,
            context,
            created_at: chrono::Utc::now(),
            developer_action,
            stack_trace,
            recovery_suggestion,
        }
    }

    /// User recovery suggestion for an error
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct UserRecoverySuggestion {
        /// Primary message to display to the user
        pub message: String,
        /// Detailed explanation of the error
        pub explanation: Option<String>,
        /// Suggested actions for the user to recover
        pub actions: Vec<String>,
        /// Whether this is a critical error that requires immediate attention
        pub is_critical: bool,
        /// Whether the user can continue despite this error
        pub can_continue: bool,
        /// Whether to show a support contact option
        pub show_support_contact: bool,
    }

    /// Generate a user-friendly message based on the error
    fn generate_user_message(error: &AppError, context: &ErrorContext) -> String {
        // If the error context has a user action, use it
        if let Some(user_action) = &context.user_action {
            return user_action.clone();
        }

        // Get the full recovery suggestion
        let suggestion = Self::generate_recovery_suggestion(error, context);

        // Format the message with the first action if available
        if let Some(action) = suggestion.actions.first() {
            format!("{}. {}", suggestion.message, action)
        } else {
            suggestion.message
        }
    }

    /// Generate a comprehensive recovery suggestion for the user
    pub fn generate_recovery_suggestion(error: &AppError, context: &ErrorContext) -> UserRecoverySuggestion {
        match error {
            AppError::NotFoundError(msg) => {
                let entity_type = if let Some(entity_type) = &context.entity_type {
                    entity_type
                } else if msg.contains("with ID") {
                    // Try to extract entity type from the message
                    let parts: Vec<&str> = msg.split(" with ID ").collect();
                    if !parts.is_empty() {
                        parts[0]
                    } else {
                        "resource"
                    }
                } else {
                    "resource"
                };

                UserRecoverySuggestion {
                    message: format!("The {} you're looking for could not be found", entity_type),
                    explanation: Some(format!("The system couldn't locate the {} you requested. It may have been deleted, moved, or never existed.", entity_type)),
                    actions: vec![
                        "Check if you entered the correct identifier".to_string(),
                        "Try searching for the item instead of accessing it directly".to_string(),
                        "Return to the previous page and try again".to_string(),
                    ],
                    is_critical: false,
                    can_continue: true,
                    show_support_contact: false,
                }
            },
            AppError::ValidationError(msg) => {
                let field_hint = if msg.to_lowercase().contains("field") || msg.to_lowercase().contains("value") {
                    msg.clone()
                } else {
                    "One or more fields have invalid values".to_string()
                };

                UserRecoverySuggestion {
                    message: "The information you provided contains errors".to_string(),
                    explanation: Some(field_hint),
                    actions: vec![
                        "Review the highlighted fields and correct any errors".to_string(),
                        "Make sure all required fields are filled out".to_string(),
                        "Check that the information is in the correct format".to_string(),
                    ],
                    is_critical: false,
                    can_continue: true,
                    show_support_contact: false,
                }
            },
            AppError::AuthorizationError(_) => UserRecoverySuggestion {
                message: "You don't have permission to perform this action".to_string(),
                explanation: Some("Your account doesn't have the necessary permissions for this operation.".to_string()),
                actions: vec![
                    "Contact your administrator to request access".to_string(),
                    "Try logging out and logging back in".to_string(),
                    "Check if you're using the correct account".to_string(),
                ],
                is_critical: false,
                can_continue: true,
                show_support_contact: false,
            },
            AppError::AuthenticationError(_) => UserRecoverySuggestion {
                message: "Authentication is required".to_string(),
                explanation: Some("You need to be logged in to perform this action.".to_string()),
                actions: vec![
                    "Please log in to continue".to_string(),
                    "If you were already logged in, your session may have expired".to_string(),
                ],
                is_critical: false,
                can_continue: false,
                show_support_contact: false,
            },
            AppError::DatabaseError(_) | AppError::SqlxError(_) => UserRecoverySuggestion {
                message: "A database error occurred".to_string(),
                explanation: Some("The system encountered an issue while accessing the database.".to_string()),
                actions: vec![
                    "Please try again in a few moments".to_string(),
                    "If the problem persists, restart the application".to_string(),
                ],
                is_critical: true,
                can_continue: false,
                show_support_contact: true,
            },
            AppError::ExternalServiceError(_) => UserRecoverySuggestion {
                message: "An external service is currently unavailable".to_string(),
                explanation: Some("The system couldn't connect to a required external service.".to_string()),
                actions: vec![
                    "Please try again later".to_string(),
                    "Check your internet connection".to_string(),
                    "Verify that the service is operational".to_string(),
                ],
                is_critical: context.severity == ErrorSeverity::Critical || context.severity == ErrorSeverity::Fatal,
                can_continue: context.severity != ErrorSeverity::Fatal,
                show_support_contact: context.severity == ErrorSeverity::Critical || context.severity == ErrorSeverity::Fatal,
            },
            AppError::ResourceLimitExceeded(_) => UserRecoverySuggestion {
                message: "Resource limit exceeded".to_string(),
                explanation: Some("The operation couldn't be completed because a resource limit was reached.".to_string()),
                actions: vec![
                    "Try again with a smaller request".to_string(),
                    "Close other applications to free up resources".to_string(),
                    "Contact support if you need a higher resource limit".to_string(),
                ],
                is_critical: false,
                can_continue: true,
                show_support_contact: true,
            },
            AppError::TransportError(_) | AppError::RemoteSendError(_) | AppError::SendError(_) => UserRecoverySuggestion {
                message: "Network connection issue".to_string(),
                explanation: Some("The system encountered a problem with the network connection.".to_string()),
                actions: vec![
                    "Check your internet connection".to_string(),
                    "Try again in a few moments".to_string(),
                    "If the problem persists, restart the application".to_string(),
                ],
                is_critical: false,
                can_continue: true,
                show_support_contact: false,
            },
            AppError::ConfigurationError(_) => UserRecoverySuggestion {
                message: "Configuration error".to_string(),
                explanation: Some("The application is not configured correctly.".to_string()),
                actions: vec![
                    "Contact your administrator".to_string(),
                    "Check the application settings".to_string(),
                    "Restart the application".to_string(),
                ],
                is_critical: true,
                can_continue: false,
                show_support_contact: true,
            },
            _ => match context.severity {
                ErrorSeverity::Fatal => UserRecoverySuggestion {
                    message: "A critical error occurred".to_string(),
                    explanation: Some("The application encountered a serious problem that prevents it from functioning.".to_string()),
                    actions: vec![
                        "Please restart the application".to_string(),
                        "If the problem persists, contact support".to_string(),
                    ],
                    is_critical: true,
                    can_continue: false,
                    show_support_contact: true,
                },
                ErrorSeverity::Critical => UserRecoverySuggestion {
                    message: "A critical error occurred".to_string(),
                    explanation: Some("The application encountered a serious problem.".to_string()),
                    actions: vec![
                        "Please try again".to_string(),
                        "If the problem persists, restart the application".to_string(),
                        "Contact support if the issue continues".to_string(),
                    ],
                    is_critical: true,
                    can_continue: false,
                    show_support_contact: true,
                },
                ErrorSeverity::Error => UserRecoverySuggestion {
                    message: "An error occurred".to_string(),
                    explanation: Some("The operation couldn't be completed due to an error.".to_string()),
                    actions: vec![
                        "Please try again".to_string(),
                        "If the problem persists, restart the application".to_string(),
                    ],
                    is_critical: false,
                    can_continue: true,
                    show_support_contact: false,
                },
                ErrorSeverity::Warning => UserRecoverySuggestion {
                    message: "Warning: There may be issues with your request".to_string(),
                    explanation: Some("The operation completed, but there might be some issues.".to_string()),
                    actions: vec![
                        "Review the results carefully".to_string(),
                        "Consider adjusting your input and trying again".to_string(),
                    ],
                    is_critical: false,
                    can_continue: true,
                    show_support_contact: false,
                },
                ErrorSeverity::Info => UserRecoverySuggestion {
                    message: "Information: Your request completed with notes".to_string(),
                    explanation: Some("The operation completed successfully, but there are some notes you should be aware of.".to_string()),
                    actions: vec![
                        "Review the additional information provided".to_string(),
                    ],
                    is_critical: false,
                    can_continue: true,
                    show_support_contact: false,
                },
            },
        }
    }

    /// Generate developer action suggestions based on the error
    fn generate_developer_action(error: &AppError, context: &ErrorContext) -> Option<String> {
        match error {
            AppError::NotFoundError(_) => Some(
                "Check that the entity exists before attempting to access it. Consider implementing a more robust existence check.".to_string()
            ),
            AppError::ValidationError(_) => Some(
                "Improve input validation. Consider adding client-side validation or enhancing server-side validation rules.".to_string()
            ),
            AppError::AuthorizationError(_) => Some(
                "Review permission checks. Ensure proper authorization is implemented at all access points.".to_string()
            ),
            AppError::AuthenticationError(_) => Some(
                "Verify authentication flow. Consider implementing more robust session management.".to_string()
            ),
            AppError::DatabaseError(_) | AppError::SqlxError(_) => Some(
                "Check database connection and query syntax. Consider implementing retry logic for transient failures.".to_string()
            ),
            AppError::TransportError(_) | AppError::RemoteSendError(_) | AppError::SendError(_) => Some(
                "Implement more robust network error handling. Consider adding retry mechanisms with exponential backoff.".to_string()
            ),
            AppError::ExternalServiceError(_) => Some(
                "Implement circuit breaker pattern for external service calls. Add fallback mechanisms for critical functionality.".to_string()
            ),
            AppError::ResourceLimitExceeded(_) => Some(
                "Optimize resource usage. Consider implementing resource pooling or throttling mechanisms.".to_string()
            ),
            AppError::ConfigurationError(_) => Some(
                "Review configuration settings. Add validation for configuration values at startup.".to_string()
            ),
            _ => match context.category {
                ErrorCategory::Database => Some(
                    "Review database operations. Consider optimizing queries and implementing proper transaction management.".to_string()
                ),
                ErrorCategory::Network => Some(
                    "Enhance network resilience. Implement proper timeout handling and connection pooling.".to_string()
                ),
                ErrorCategory::ExternalService => Some(
                    "Improve external service integration. Consider implementing service mocks for testing.".to_string()
                ),
                ErrorCategory::Validation => Some(
                    "Enhance validation logic. Consider implementing a comprehensive validation framework.".to_string()
                ),
                ErrorCategory::BusinessLogic => Some(
                    "Review business logic implementation. Consider adding more comprehensive unit tests.".to_string()
                ),
                ErrorCategory::System => Some(
                    "Check system resources and configuration. Consider implementing health checks.".to_string()
                ),
                _ => None,
            },
        }
    }
}

/// Error reporter for tracking errors and generating user-friendly messages
#[derive(Clone)]
pub struct ErrorReporter {
    /// Recent error reports (limited to a maximum number)
    recent_reports: Arc<Mutex<Vec<ErrorReport>>>,
    /// Maximum number of recent reports to keep
    max_recent_reports: usize,
    /// Whether to send telemetry data
    send_telemetry: bool,
    /// External error reporters
    external_reporters: Arc<Mutex<Vec<Box<dyn ExternalErrorReporter>>>>,
    /// Error aggregation data
    error_aggregation: Arc<Mutex<HashMap<String, ErrorAggregation>>>,
}

/// Error aggregation data for analyzing error patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAggregation {
    /// Error category
    pub category: ErrorCategory,
    /// First occurrence timestamp
    pub first_occurrence: chrono::DateTime<chrono::Utc>,
    /// Last occurrence timestamp
    pub last_occurrence: chrono::DateTime<chrono::Utc>,
    /// Count of occurrences
    pub count: usize,
    /// Whether this error is resolved
    pub resolved: bool,
    /// Resolution timestamp if resolved
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Resolution notes
    pub resolution_notes: Option<String>,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new(max_recent_reports: usize, send_telemetry: bool) -> Self {
        Self {
            recent_reports: Arc::new(Mutex::new(Vec::with_capacity(max_recent_reports))),
            max_recent_reports,
            send_telemetry,
            external_reporters: Arc::new(Mutex::new(Vec::new())),
            error_aggregation: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add an external error reporter
    pub async fn add_external_reporter(&self, reporter: Box<dyn ExternalErrorReporter>) {
        let mut reporters = self.external_reporters.lock().await;
        reporters.push(reporter);
    }

    /// Initialize an external reporter with configuration
    pub async fn init_external_reporter(&self, name: &str, config: HashMap<String, String>) -> Result<(), String> {
        let mut reporters = self.external_reporters.lock().await;

        // Find the reporter with the given name
        for reporter in reporters.iter_mut() {
            if reporter.name() == name {
                return reporter.initialize(config).await;
            }
        }

        Err(format!("External reporter '{}' not found", name))
    }

    /// Report an error
    pub async fn report(&self, error: &AppError) -> ErrorReport {
        // Create an error report
        let report = ErrorReport::new(error);

        // Store the report in recent reports
        let mut recent_reports = self.recent_reports.lock().await;
        if recent_reports.len() >= self.max_recent_reports {
            recent_reports.remove(0); // Remove the oldest report
        }
        recent_reports.push(report.clone());

        // Update error aggregation data
        self.update_aggregation(&report).await;

        // Send to external reporters
        self.send_to_external_reporters(&report).await;

        // Send telemetry if enabled (legacy method, kept for backward compatibility)
        if self.send_telemetry {
            self.send_telemetry(&report).await;
        }

        report
    }

    /// Get recent error reports
    pub async fn get_recent_reports(&self) -> Vec<ErrorReport> {
        let recent_reports = self.recent_reports.lock().await;
        recent_reports.clone()
    }

    /// Get error aggregation data
    pub async fn get_error_aggregation(&self) -> HashMap<String, ErrorAggregation> {
        let aggregation = self.error_aggregation.lock().await;
        aggregation.clone()
    }

    /// Mark an error as resolved
    pub async fn mark_error_resolved(&self, error_key: &str, notes: Option<String>) -> Result<(), String> {
        let mut aggregation = self.error_aggregation.lock().await;

        if let Some(agg) = aggregation.get_mut(error_key) {
            agg.resolved = true;
            agg.resolved_at = Some(chrono::Utc::now());
            agg.resolution_notes = notes;
            Ok(())
        } else {
            Err(format!("Error with key '{}' not found", error_key))
        }
    }

    /// Update error aggregation data
    async fn update_aggregation(&self, report: &ErrorReport) {
        let mut aggregation = self.error_aggregation.lock().await;

        // Create a key for this error type (category + message pattern)
        let key = format!("{}:{}", report.context.category as u8, report.message);

        if let Some(agg) = aggregation.get_mut(&key) {
            // Update existing aggregation
            agg.count += 1;
            agg.last_occurrence = report.created_at;
            agg.resolved = false;
            agg.resolved_at = None;
        } else {
            // Create new aggregation
            aggregation.insert(key, ErrorAggregation {
                category: report.context.category,
                first_occurrence: report.created_at,
                last_occurrence: report.created_at,
                count: 1,
                resolved: false,
                resolved_at: None,
                resolution_notes: None,
            });
        }
    }

    /// Send error report to all external reporters
    async fn send_to_external_reporters(&self, report: &ErrorReport) {
        let reporters = self.external_reporters.lock().await;

        for reporter in reporters.iter() {
            if let Err(e) = reporter.report_error(report).await {
                // Log the error but don't fail the reporting process
                tracing::warn!(
                    reporter = %reporter.name(),
                    error = %e,
                    "Failed to send error report to external reporter"
                );
            }
        }
    }

    /// Send telemetry data (legacy method)
    async fn send_telemetry(&self, report: &ErrorReport) {
        // In a real implementation, this would send the report to a telemetry service
        // For now, just log it
        tracing::info!(
            report_id = %report.report_id,
            error_id = %report.context.error_id,
            message = %report.message,
            severity = ?report.context.severity,
            category = ?report.context.category,
            "Error telemetry report"
        );
    }

    /// Get a user-friendly message for an error
    pub fn get_user_message(&self, error: &AppError) -> String {
        let report = ErrorReport::new(error);
        report.user_message
    }

    /// Get developer action suggestion for an error
    pub fn get_developer_action(&self, error: &AppError) -> Option<String> {
        let report = ErrorReport::new(error);
        report.developer_action
    }

    /// Get recovery suggestion for an error
    pub fn get_recovery_suggestion(&self, error: &AppError) -> UserRecoverySuggestion {
        let report = ErrorReport::new(error);
        report.recovery_suggestion
    }

    /// Create a UI-friendly error object for frontend display
    pub fn create_ui_error(&self, error: &AppError) -> UiError {
        let report = ErrorReport::new(error);
        UiError {
            id: report.report_id.to_string(),
            message: report.user_message,
            explanation: report.recovery_suggestion.explanation,
            actions: report.recovery_suggestion.actions,
            is_critical: report.recovery_suggestion.is_critical,
            can_continue: report.recovery_suggestion.can_continue,
            show_support_contact: report.recovery_suggestion.show_support_contact,
            timestamp: report.created_at.to_rfc3339(),
            error_code: format!("{:?}", report.context.category),
        }
    }
}

/// Default error reporter instance
static mut DEFAULT_REPORTER: Option<ErrorReporter> = None;

/// Initialize the default error reporter
pub fn init_default_reporter(max_recent_reports: usize, send_telemetry: bool) {
    unsafe {
        DEFAULT_REPORTER = Some(ErrorReporter::new(max_recent_reports, send_telemetry));
    }
}

/// Initialize the default error reporter with external reporters
pub async fn init_default_reporter_with_externals(
    max_recent_reports: usize, 
    send_telemetry: bool,
    init_console: bool,
    init_sentry: bool,
    sentry_dsn: Option<String>,
    environment: String,
) -> Result<(), String> {
    // Create the reporter
    let reporter = ErrorReporter::new(max_recent_reports, send_telemetry);

    // Add console reporter if requested
    if init_console {
        let console_reporter = Box::new(ConsoleReporter::new());
        reporter.add_external_reporter(console_reporter).await;

        let mut config = HashMap::new();
        config.insert("log_level".to_string(), "info".to_string());
        reporter.init_external_reporter("console", config).await?;
    }

    // Add Sentry reporter if requested
    if init_sentry {
        let sentry_reporter = Box::new(SentryReporter::new());
        reporter.add_external_reporter(sentry_reporter).await;

        if let Some(dsn) = sentry_dsn {
            let mut config = HashMap::new();
            config.insert("dsn".to_string(), dsn);
            config.insert("environment".to_string(), environment);
            reporter.init_external_reporter("sentry", config).await?;
        } else {
            return Err("Sentry DSN not provided".to_string());
        }
    }

    // Set as default reporter
    unsafe {
        DEFAULT_REPORTER = Some(reporter);
    }

    Ok(())
}

/// Get the default error reporter
pub fn default_reporter() -> Option<ErrorReporter> {
    unsafe { DEFAULT_REPORTER.clone() }
}

/// Report an error using the default reporter
pub async fn report_error(error: &AppError) -> Option<ErrorReport> {
    if let Some(reporter) = default_reporter() {
        Some(reporter.report(error).await)
    } else {
        None
    }
}

/// Get a user-friendly message for an error using the default reporter
pub fn get_user_message(error: &AppError) -> String {
    if let Some(reporter) = default_reporter() {
        reporter.get_user_message(error)
    } else {
        // Fallback if no reporter is initialized
        ErrorReport::new(error).user_message
    }
}

/// Extension trait for AppError to add reporting methods
pub trait ErrorReportExt {
    /// Report this error using the default reporter
    async fn report(&self) -> Option<ErrorReport>;

    /// Get a user-friendly message for this error
    fn user_message(&self) -> String;

    /// Get recovery suggestion for this error
    fn recovery_suggestion(&self) -> Option<UserRecoverySuggestion>;

    /// Get a UI-friendly error object for frontend display
    fn to_ui_error(&self) -> Option<UiError>;
}

impl ErrorReportExt for AppError {
    async fn report(&self) -> Option<ErrorReport> {
        report_error(self).await
    }

    fn user_message(&self) -> String {
        get_user_message(self)
    }

    fn recovery_suggestion(&self) -> Option<UserRecoverySuggestion> {
        if let Some(reporter) = default_reporter() {
            Some(reporter.get_recovery_suggestion(self))
        } else {
            // Fallback if no reporter is initialized
            Some(ErrorReport::generate_recovery_suggestion(self, &ErrorContext::new()
                .with_category(self.category())
                .with_severity(self.severity())
                .with_retriable(self.is_retriable())))
        }
    }

    fn to_ui_error(&self) -> Option<UiError> {
        if let Some(reporter) = default_reporter() {
            Some(reporter.create_ui_error(self))
        } else {
            // Fallback if no reporter is initialized
            let report = ErrorReport::new(self);
            Some(UiError {
                id: report.report_id.to_string(),
                message: report.user_message,
                explanation: report.recovery_suggestion.explanation,
                actions: report.recovery_suggestion.actions,
                is_critical: report.recovery_suggestion.is_critical,
                can_continue: report.recovery_suggestion.can_continue,
                show_support_contact: report.recovery_suggestion.show_support_contact,
                timestamp: report.created_at.to_rfc3339(),
                error_code: format!("{:?}", report.context.category),
            })
        }
    }
}
