use std::{collections::HashMap, fmt::Display, io};

use color_eyre::eyre;
use kameo::error::{BootstrapError, RegistryError, RemoteSendError, SendError};
use libp2p::TransportError;
use rig::completion::CompletionError;
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

// Error modules
pub mod reporter;
pub mod taxonomy;
pub mod enrichment;

macro_rules! from_err {
    ($err:ty, $enum:expr) => {
        impl From<$err> for AppError {
            fn from(e: $err) -> Self {
                $enum(LossyError::Lossless(e.into()))
            }
        }
    };
}

/// Error severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Fatal errors that require immediate attention
    Fatal,
    /// Critical errors that should be addressed soon
    Critical,
    /// Errors that should be fixed but don't require immediate attention
    Error,
    /// Warnings that don't prevent the application from functioning
    Warning,
    /// Informational messages about potential issues
    Info,
}

impl Default for ErrorSeverity {
    fn default() -> Self {
        Self::Error
    }
}

/// Error category for taxonomic classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Errors related to authentication and authorization
    Authentication,
    /// Errors related to authorization
    Authorization,
    /// Errors related to validation
    Validation,
    /// Errors related to database operations
    Database,
    /// Errors related to network operations
    Network,
    /// Errors related to external services
    ExternalService,
    /// Errors related to internal services
    InternalService,
    /// Errors related to configuration
    Configuration,
    /// Errors related to resource limitations
    ResourceLimit,
    /// Errors related to user input
    UserInput,
    /// Errors related to business logic
    BusinessLogic,
    /// Errors related to system operations
    System,
    /// Errors that don't fit into other categories
    Other,
}

impl Default for ErrorCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// Error context for enriching errors with additional information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Unique identifier for the error
    pub error_id: Uuid,
    /// Correlation ID for tracing related errors
    pub correlation_id: Option<String>,
    /// Request ID associated with the error
    pub request_id: Option<String>,
    /// User ID associated with the error
    pub user_id: Option<String>,
    /// Workspace ID associated with the error
    pub workspace_id: Option<Uuid>,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Source of the error (file:line)
    pub source_location: Option<String>,
    /// Operation that caused the error
    pub operation: Option<String>,
    /// Entity type associated with the error
    pub entity_type: Option<String>,
    /// Entity ID associated with the error
    pub entity_id: Option<String>,
    /// Additional context as key-value pairs
    pub additional_context: HashMap<String, String>,
    /// Timestamp when the error occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether the error is retriable
    pub retriable: bool,
    /// Suggested user action
    pub user_action: Option<String>,
    /// Developer action
    pub developer_action: Option<String>,
}

impl ErrorContext {
    /// Create a new error context with default values
    pub fn new() -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Set the correlation ID
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
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

    /// Set the severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the category
    pub fn with_category(mut self, category: ErrorCategory) -> Self {
        self.category = category;
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

    /// Set whether the error is retriable
    pub fn with_retriable(mut self, retriable: bool) -> Self {
        self.retriable = retriable;
        self
    }

    /// Set the suggested user action
    pub fn with_user_action(mut self, user_action: impl Into<String>) -> Self {
        self.user_action = Some(user_action.into());
        self
    }

    /// Set the developer action
    pub fn with_developer_action(mut self, developer_action: impl Into<String>) -> Self {
        self.developer_action = Some(developer_action.into());
        self
    }
}

/// Standardized application error type
#[derive(Debug, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AppError {
    // System-level errors
    #[error("Bootstrap error: {0}")]
    BootstrapError(String),
    #[error("Transport error: {0}")]
    TransportError(LossyError<TransportError<io::Error>>),
    #[error("Registry error: {0}")]
    RegistryError(LossyError<RegistryError>),
    #[error("SQLx error: {0}")]
    SqlxError(LossyError<sqlx::Error>),
    #[error("Send error: {0}")]
    SendError(String),
    #[error("Remote send error: {0}")]
    RemoteSendError(String),
    #[error("Something went wrong: {0}")]
    Generic(LossyError<eyre::Error>),
    #[error("Invalid JSON payload: {0}")]
    JsonError(LossyError<serde_json::Error>),
    #[error("Tool call error: {0}")]
    ToolCallError(LossyError<rig::tool::ToolError>),
    #[error("IO error: {0}")]
    IoError(LossyError<io::Error>),
    #[error("Uuid parse error: {0}")]
    UuidParseError(LossyError<uuid::Error>),
    #[error("Chat completion error: {0}")]
    CompletionError(LossyError<CompletionError>),

    // Application-level errors
    #[error("Not found: {0}")]
    NotFoundError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Transaction error: {0}")]
    TransactionError(String),
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Operation not supported: {0}")]
    OperationNotSupported(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Privacy policy violation: {rule_id} - {details}")]
    PrivacyPolicyViolation {
        rule_id: String,
        details: String,
        remediation: Option<String>,
    },

    // Contextual error with additional information
    #[error("{message}")]
    ContextualError {
        message: String,
        context: ErrorContext,
    },
}

from_err!(eyre::Error, AppError::Generic);
from_err!(TransportError<io::Error>, AppError::TransportError);
from_err!(RegistryError, AppError::RegistryError);
from_err!(sqlx::Error, AppError::SqlxError);
from_err!(serde_json::Error, AppError::JsonError);
from_err!(io::Error, AppError::IoError);
from_err!(uuid::Error, AppError::UuidParseError);
from_err!(rig::tool::ToolError, AppError::ToolCallError);
from_err!(rig::completion::CompletionError, AppError::CompletionError);

// Nececssary error type for Deserialzing errors,
// which is required by kameo to be able to send them over the wire
#[derive(Debug, Error)]
pub enum LossyError<E> {
    Lossless(E),
    Lossy(String),
}

impl<E: Display> Serialize for LossyError<E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            LossyError::Lossless(err) => serializer.collect_str(&err.to_string()),
            LossyError::Lossy(err) => serializer.collect_str(err),
        }
    }
}

impl<'de, E: Display> Deserialize<'de> for LossyError<E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(LossyError::Lossy(s))
    }
}

impl<E: Display> Display for LossyError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LossyError::Lossless(err) => write!(f, "{err}"),
            LossyError::Lossy(err) => write!(f, "{err}"),
        }
    }
}

impl<T, E> From<SendError<T, E>> for AppError
where
    SendError<T, E>: Display,
{
    fn from(err: SendError<T, E>) -> Self {
        Self::SendError(err.to_string())
    }
}

impl<E> From<RemoteSendError<E>> for AppError
where
    RemoteSendError<E>: Display,
{
    fn from(err: RemoteSendError<E>) -> Self {
        Self::RemoteSendError(err.to_string())
    }
}

impl From<BootstrapError> for AppError {
    fn from(err: BootstrapError) -> Self {
        Self::BootstrapError(err.to_string())
    }
}

impl AppError {
    /// Create a new contextual error with the given message and context
    pub fn with_context(message: impl Into<String>, context: ErrorContext) -> Self {
        Self::ContextualError {
            message: message.into(),
            context,
        }
    }

    /// Create a new not found error
    pub fn not_found(entity_type: impl Into<String>, entity_id: impl Display) -> Self {
        Self::NotFoundError(format!("{} with ID {} not found", entity_type.into(), entity_id))
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    /// Create a new authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::AuthorizationError(message.into())
    }

    /// Create a new authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::AuthenticationError(message.into())
    }

    /// Create a new database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::DatabaseError(message.into())
    }

    /// Create a new internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Create a new operation not supported error
    pub fn operation_not_supported(message: impl Into<String>) -> Self {
        Self::OperationNotSupported(message.into())
    }

    /// Create a new resource limit exceeded error
    pub fn resource_limit_exceeded(message: impl Into<String>) -> Self {
        Self::ResourceLimitExceeded(message.into())
    }

    /// Create a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a new external service error
    pub fn external_service(message: impl Into<String>) -> Self {
        Self::ExternalServiceError(message.into())
    }

    /// Get the error category for this error
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::AuthenticationError(_) => ErrorCategory::Authentication,
            Self::AuthorizationError(_) => ErrorCategory::Authorization,
            Self::ValidationError(_) => ErrorCategory::Validation,
            Self::DatabaseError(_) | Self::SqlxError(_) | Self::QueryError(_) => ErrorCategory::Database,
            Self::TransportError(_) | Self::RemoteSendError(_) | Self::SendError(_) => ErrorCategory::Network,
            Self::ExternalServiceError(_) => ErrorCategory::ExternalService,
            Self::ConfigurationError(_) => ErrorCategory::Configuration,
            Self::ResourceLimitExceeded(_) => ErrorCategory::ResourceLimit,
            Self::BootstrapError(_) | Self::RegistryError(_) | Self::IoError(_) => ErrorCategory::System,
            Self::ContextualError { context, .. } => context.category,
            _ => ErrorCategory::Other,
        }
    }

    /// Get the error severity for this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::BootstrapError(_) => ErrorSeverity::Fatal,
            Self::AuthenticationError(_) | Self::AuthorizationError(_) => ErrorSeverity::Critical,
            Self::DatabaseError(_) | Self::SqlxError(_) => ErrorSeverity::Critical,
            Self::ValidationError(_) => ErrorSeverity::Warning,
            Self::NotFoundError(_) => ErrorSeverity::Warning,
            Self::ContextualError { context, .. } => context.severity,
            _ => ErrorSeverity::Error,
        }
    }

    /// Check if this error is retriable
    pub fn is_retriable(&self) -> bool {
        match self {
            Self::TransportError(_) | Self::RemoteSendError(_) | Self::SendError(_) => true,
            Self::DatabaseError(_) | Self::SqlxError(_) => true,
            Self::ExternalServiceError(_) => true,
            Self::ContextualError { context, .. } => context.retriable,
            _ => false,
        }
    }

    /// Enrich this error with context
    pub fn enrich(&self, context_builder: impl FnOnce(ErrorContext) -> ErrorContext) -> Self {
        match self {
            Self::ContextualError { message, context } => {
                let new_context = context_builder(context.clone());
                Self::ContextualError {
                    message: message.clone(),
                    context: new_context,
                }
            }
            _ => {
                let context = context_builder(ErrorContext::new()
                    .with_category(self.category())
                    .with_severity(self.severity())
                    .with_retriable(self.is_retriable()));
                Self::ContextualError {
                    message: self.to_string(),
                    context,
                }
            }
        }
    }

    /// Log this error with structured logging
    pub fn log(&self) {
        match self {
            Self::ContextualError { message, context } => {
                let mut log_builder = tracing::error_span!(
                    "error",
                    error_id = %context.error_id,
                    error_type = %std::any::type_name::<Self>(),
                    severity = ?context.severity,
                    category = ?context.category,
                    correlation_id = %context.correlation_id.as_deref().unwrap_or("none"),
                    request_id = %context.request_id.as_deref().unwrap_or("none"),
                    workspace_id = %context.workspace_id.map(|id| id.to_string()).unwrap_or_else(|| "none".to_string()),
                    source_location = %context.source_location.as_deref().unwrap_or("unknown"),
                    operation = %context.operation.as_deref().unwrap_or("unknown"),
                    entity_type = %context.entity_type.as_deref().unwrap_or("unknown"),
                    entity_id = %context.entity_id.as_deref().unwrap_or("unknown"),
                    timestamp = %context.timestamp,
                    retriable = %context.retriable,
                );

                // Add additional context to the log
                for (key, value) in &context.additional_context {
                    log_builder = log_builder.record(key, value.as_str());
                }

                // Enter the span and log the error
                let _guard = log_builder.enter();
                tracing::error!("{}", message);
            }
            _ => {
                tracing::error!(
                    error_type = %std::any::type_name::<Self>(),
                    severity = ?self.severity(),
                    category = ?self.category(),
                    retriable = %self.is_retriable(),
                    "{}", self
                );
            }
        }
    }
}

/// Macro to create a contextual error with source location
#[macro_export]
macro_rules! contextual_error {
    ($message:expr, $($context:tt)*) => {{
        let context = $crate::error::ErrorContext::new()
            .with_source_location(file!(), line!())
            $($context)*;
        $crate::error::AppError::with_context($message, context)
    }};
}

/// Macro to enrich an error with source location
#[macro_export]
macro_rules! enrich_error {
    ($err:expr, $($context:tt)*) => {{
        $err.enrich(|ctx| {
            ctx.with_source_location(file!(), line!())
            $($context)*
        })
    }};
}

/// Macro to log an error with source location
#[macro_export]
macro_rules! log_error {
    ($err:expr) => {{
        let err = $crate::enrich_error!($err, );
        err.log();
        err
    }};
    ($err:expr, $($context:tt)*) => {{
        let err = $crate::enrich_error!($err, $($context)*);
        err.log();
        err
    }};
}

pub type Result<T, E = AppError> = core::result::Result<T, E>;
