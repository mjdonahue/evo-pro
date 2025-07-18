//! Error taxonomy for the application
//!
//! This module defines a comprehensive taxonomy of errors that can occur in the application.
//! The taxonomy is organized hierarchically, with top-level categories and subcategories.
//! Each error type has a unique code, a description, and information about severity and retriability.

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Error domain for high-level categorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorDomain {
    /// Errors related to user authentication and authorization
    Security,
    /// Errors related to data storage and retrieval
    Data,
    /// Errors related to network operations
    Network,
    /// Errors related to the application's business logic
    Business,
    /// Errors related to the system and infrastructure
    System,
    /// Errors related to user input and validation
    Validation,
    /// Errors related to external services and integrations
    Integration,
    /// Errors that don't fit into other domains
    Other,
}

impl Display for ErrorDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Security => write!(f, "Security"),
            Self::Data => write!(f, "Data"),
            Self::Network => write!(f, "Network"),
            Self::Business => write!(f, "Business"),
            Self::System => write!(f, "System"),
            Self::Validation => write!(f, "Validation"),
            Self::Integration => write!(f, "Integration"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// Error category for more specific classification within a domain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    // Security domain
    /// Errors related to authentication
    Authentication,
    /// Errors related to authorization
    Authorization,
    /// Errors related to credential management
    Credentials,
    /// Errors related to session management
    Session,
    /// Errors related to access control
    AccessControl,

    // Data domain
    /// Errors related to database operations
    Database,
    /// Errors related to data validation
    DataValidation,
    /// Errors related to data integrity
    DataIntegrity,
    /// Errors related to data migration
    DataMigration,
    /// Errors related to data serialization/deserialization
    Serialization,
    /// Errors related to data not found
    NotFound,
    /// Errors related to data conflicts
    Conflict,

    // Network domain
    /// Errors related to network connectivity
    Connectivity,
    /// Errors related to network timeouts
    Timeout,
    /// Errors related to network protocols
    Protocol,
    /// Errors related to peer-to-peer communication
    P2P,
    /// Errors related to message delivery
    Messaging,

    // Business domain
    /// Errors related to business rules
    BusinessRule,
    /// Errors related to workflow
    Workflow,
    /// Errors related to state transitions
    StateTransition,
    /// Errors related to resource limits
    ResourceLimit,
    /// Errors related to feature flags
    FeatureFlag,

    // System domain
    /// Errors related to configuration
    Configuration,
    /// Errors related to initialization
    Initialization,
    /// Errors related to resource allocation
    ResourceAllocation,
    /// Errors related to file system operations
    FileSystem,
    /// Errors related to concurrency
    Concurrency,
    /// Errors related to memory management
    Memory,
    /// Errors related to actor system
    Actor,

    // Validation domain
    /// Errors related to input validation
    InputValidation,
    /// Errors related to format validation
    FormatValidation,
    /// Errors related to constraint validation
    ConstraintValidation,
    /// Errors related to type validation
    TypeValidation,

    // Integration domain
    /// Errors related to external services
    ExternalService,
    /// Errors related to API calls
    API,
    /// Errors related to webhooks
    Webhook,
    /// Errors related to third-party libraries
    ThirdParty,
    /// Errors related to plugin system
    Plugin,

    // Other domain
    /// Errors that don't fit into other categories
    Uncategorized,
    /// Errors that are unexpected or unknown
    Unexpected,
    /// Errors that are internal to the application
    Internal,
}

impl Display for ErrorCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            // Security domain
            Self::Authentication => write!(f, "Authentication"),
            Self::Authorization => write!(f, "Authorization"),
            Self::Credentials => write!(f, "Credentials"),
            Self::Session => write!(f, "Session"),
            Self::AccessControl => write!(f, "AccessControl"),

            // Data domain
            Self::Database => write!(f, "Database"),
            Self::DataValidation => write!(f, "DataValidation"),
            Self::DataIntegrity => write!(f, "DataIntegrity"),
            Self::DataMigration => write!(f, "DataMigration"),
            Self::Serialization => write!(f, "Serialization"),
            Self::NotFound => write!(f, "NotFound"),
            Self::Conflict => write!(f, "Conflict"),

            // Network domain
            Self::Connectivity => write!(f, "Connectivity"),
            Self::Timeout => write!(f, "Timeout"),
            Self::Protocol => write!(f, "Protocol"),
            Self::P2P => write!(f, "P2P"),
            Self::Messaging => write!(f, "Messaging"),

            // Business domain
            Self::BusinessRule => write!(f, "BusinessRule"),
            Self::Workflow => write!(f, "Workflow"),
            Self::StateTransition => write!(f, "StateTransition"),
            Self::ResourceLimit => write!(f, "ResourceLimit"),
            Self::FeatureFlag => write!(f, "FeatureFlag"),

            // System domain
            Self::Configuration => write!(f, "Configuration"),
            Self::Initialization => write!(f, "Initialization"),
            Self::ResourceAllocation => write!(f, "ResourceAllocation"),
            Self::FileSystem => write!(f, "FileSystem"),
            Self::Concurrency => write!(f, "Concurrency"),
            Self::Memory => write!(f, "Memory"),
            Self::Actor => write!(f, "Actor"),

            // Validation domain
            Self::InputValidation => write!(f, "InputValidation"),
            Self::FormatValidation => write!(f, "FormatValidation"),
            Self::ConstraintValidation => write!(f, "ConstraintValidation"),
            Self::TypeValidation => write!(f, "TypeValidation"),

            // Integration domain
            Self::ExternalService => write!(f, "ExternalService"),
            Self::API => write!(f, "API"),
            Self::Webhook => write!(f, "Webhook"),
            Self::ThirdParty => write!(f, "ThirdParty"),
            Self::Plugin => write!(f, "Plugin"),

            // Other domain
            Self::Uncategorized => write!(f, "Uncategorized"),
            Self::Unexpected => write!(f, "Unexpected"),
            Self::Internal => write!(f, "Internal"),
        }
    }
}

/// Get the domain for a category
pub fn domain_for_category(category: ErrorCategory) -> ErrorDomain {
    match category {
        // Security domain
        ErrorCategory::Authentication => ErrorDomain::Security,
        ErrorCategory::Authorization => ErrorDomain::Security,
        ErrorCategory::Credentials => ErrorDomain::Security,
        ErrorCategory::Session => ErrorDomain::Security,
        ErrorCategory::AccessControl => ErrorDomain::Security,

        // Data domain
        ErrorCategory::Database => ErrorDomain::Data,
        ErrorCategory::DataValidation => ErrorDomain::Data,
        ErrorCategory::DataIntegrity => ErrorDomain::Data,
        ErrorCategory::DataMigration => ErrorDomain::Data,
        ErrorCategory::Serialization => ErrorDomain::Data,
        ErrorCategory::NotFound => ErrorDomain::Data,
        ErrorCategory::Conflict => ErrorDomain::Data,

        // Network domain
        ErrorCategory::Connectivity => ErrorDomain::Network,
        ErrorCategory::Timeout => ErrorDomain::Network,
        ErrorCategory::Protocol => ErrorDomain::Network,
        ErrorCategory::P2P => ErrorDomain::Network,
        ErrorCategory::Messaging => ErrorDomain::Network,

        // Business domain
        ErrorCategory::BusinessRule => ErrorDomain::Business,
        ErrorCategory::Workflow => ErrorDomain::Business,
        ErrorCategory::StateTransition => ErrorDomain::Business,
        ErrorCategory::ResourceLimit => ErrorDomain::Business,
        ErrorCategory::FeatureFlag => ErrorDomain::Business,

        // System domain
        ErrorCategory::Configuration => ErrorDomain::System,
        ErrorCategory::Initialization => ErrorDomain::System,
        ErrorCategory::ResourceAllocation => ErrorDomain::System,
        ErrorCategory::FileSystem => ErrorDomain::System,
        ErrorCategory::Concurrency => ErrorDomain::System,
        ErrorCategory::Memory => ErrorDomain::System,
        ErrorCategory::Actor => ErrorDomain::System,

        // Validation domain
        ErrorCategory::InputValidation => ErrorDomain::Validation,
        ErrorCategory::FormatValidation => ErrorDomain::Validation,
        ErrorCategory::ConstraintValidation => ErrorDomain::Validation,
        ErrorCategory::TypeValidation => ErrorDomain::Validation,

        // Integration domain
        ErrorCategory::ExternalService => ErrorDomain::Integration,
        ErrorCategory::API => ErrorDomain::Integration,
        ErrorCategory::Webhook => ErrorDomain::Integration,
        ErrorCategory::ThirdParty => ErrorDomain::Integration,
        ErrorCategory::Plugin => ErrorDomain::Integration,

        // Other domain
        ErrorCategory::Uncategorized => ErrorDomain::Other,
        ErrorCategory::Unexpected => ErrorDomain::Other,
        ErrorCategory::Internal => ErrorDomain::Other,
    }
}

/// Error code for unique identification of errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCode(pub u32);

impl Display for ErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{:04}", self.0)
    }
}

/// Error type for detailed classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorType {
    /// Unique code for this error type
    pub code: ErrorCode,
    /// Domain of the error
    pub domain: ErrorDomain,
    /// Category of the error
    pub category: ErrorCategory,
    /// Name of the error
    pub name: String,
    /// Description of the error
    pub description: String,
    /// Whether the error is retriable
    pub retriable: bool,
    /// Suggested user action
    pub user_action: Option<String>,
    /// Suggested developer action
    pub developer_action: Option<String>,
}

impl ErrorType {
    /// Create a new error type
    pub fn new(
        code: u32,
        category: ErrorCategory,
        name: impl Into<String>,
        description: impl Into<String>,
        retriable: bool,
    ) -> Self {
        Self {
            code: ErrorCode(code),
            domain: domain_for_category(category),
            category,
            name: name.into(),
            description: description.into(),
            retriable,
            user_action: None,
            developer_action: None,
        }
    }

    /// Set the suggested user action
    pub fn with_user_action(mut self, action: impl Into<String>) -> Self {
        self.user_action = Some(action.into());
        self
    }

    /// Set the suggested developer action
    pub fn with_developer_action(mut self, action: impl Into<String>) -> Self {
        self.developer_action = Some(action.into());
        self
    }
}

impl Display for ErrorType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.code, self.name, self.category)
    }
}

/// Error taxonomy containing all defined error types
#[derive(Debug, Clone)]
pub struct ErrorTaxonomy {
    /// All defined error types, indexed by code
    pub types: std::collections::HashMap<ErrorCode, ErrorType>,
}

impl ErrorTaxonomy {
    /// Create a new empty taxonomy
    pub fn new() -> Self {
        Self {
            types: std::collections::HashMap::new(),
        }
    }

    /// Register an error type in the taxonomy
    pub fn register(&mut self, error_type: ErrorType) {
        self.types.insert(error_type.code, error_type);
    }

    /// Get an error type by code
    pub fn get(&self, code: ErrorCode) -> Option<&ErrorType> {
        self.types.get(&code)
    }

    /// Get all error types in a specific domain
    pub fn get_by_domain(&self, domain: ErrorDomain) -> Vec<&ErrorType> {
        self.types
            .values()
            .filter(|t| t.domain == domain)
            .collect()
    }

    /// Get all error types in a specific category
    pub fn get_by_category(&self, category: ErrorCategory) -> Vec<&ErrorType> {
        self.types
            .values()
            .filter(|t| t.category == category)
            .collect()
    }
}

/// Default error taxonomy with predefined error types
pub fn default_taxonomy() -> ErrorTaxonomy {
    let mut taxonomy = ErrorTaxonomy::new();

    // Security domain errors (1000-1999)
    taxonomy.register(ErrorType::new(
        1001,
        ErrorCategory::Authentication,
        "InvalidCredentials",
        "The provided credentials are invalid",
        true,
    ).with_user_action("Please check your username and password and try again")
      .with_developer_action("Verify the authentication logic and credential validation"));

    taxonomy.register(ErrorType::new(
        1002,
        ErrorCategory::Authentication,
        "AccountLocked",
        "The account has been locked due to too many failed login attempts",
        false,
    ).with_user_action("Please contact support to unlock your account")
      .with_developer_action("Check the account locking policy and consider implementing account recovery"));

    taxonomy.register(ErrorType::new(
        1003,
        ErrorCategory::Authentication,
        "SessionExpired",
        "The user session has expired",
        true,
    ).with_user_action("Please log in again to continue")
      .with_developer_action("Verify session timeout settings and consider implementing session refresh"));

    taxonomy.register(ErrorType::new(
        1101,
        ErrorCategory::Authorization,
        "InsufficientPermissions",
        "The user does not have sufficient permissions for this operation",
        false,
    ).with_user_action("Please contact your administrator to request access")
      .with_developer_action("Check the permission requirements for this operation"));

    taxonomy.register(ErrorType::new(
        1102,
        ErrorCategory::Authorization,
        "ResourceForbidden",
        "Access to the requested resource is forbidden",
        false,
    ).with_user_action("You do not have permission to access this resource")
      .with_developer_action("Verify access control rules for this resource"));

    // Data domain errors (2000-2999)
    taxonomy.register(ErrorType::new(
        2001,
        ErrorCategory::Database,
        "ConnectionFailed",
        "Failed to connect to the database",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Check database connection settings and availability"));

    taxonomy.register(ErrorType::new(
        2002,
        ErrorCategory::Database,
        "QueryFailed",
        "A database query failed to execute",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Check the query syntax and parameters"));

    taxonomy.register(ErrorType::new(
        2003,
        ErrorCategory::Database,
        "TransactionFailed",
        "A database transaction failed to complete",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Check transaction logic and consider retry mechanisms"));

    taxonomy.register(ErrorType::new(
        2101,
        ErrorCategory::DataIntegrity,
        "ConstraintViolation",
        "A database constraint was violated",
        false,
    ).with_user_action("The data you provided conflicts with existing data")
      .with_developer_action("Check data validation and constraint handling"));

    taxonomy.register(ErrorType::new(
        2102,
        ErrorCategory::DataIntegrity,
        "DataCorruption",
        "The data in the database is corrupted",
        false,
    ).with_user_action("Please contact support")
      .with_developer_action("Implement data integrity checks and recovery mechanisms"));

    taxonomy.register(ErrorType::new(
        2201,
        ErrorCategory::NotFound,
        "ResourceNotFound",
        "The requested resource was not found",
        false,
    ).with_user_action("The item you're looking for doesn't exist or has been deleted")
      .with_developer_action("Implement proper error handling for not found cases"));

    taxonomy.register(ErrorType::new(
        2301,
        ErrorCategory::Serialization,
        "DeserializationFailed",
        "Failed to deserialize data",
        false,
    ).with_user_action("Please try again or contact support")
      .with_developer_action("Check data format and deserialization logic"));

    // Network domain errors (3000-3999)
    taxonomy.register(ErrorType::new(
        3001,
        ErrorCategory::Connectivity,
        "ConnectionLost",
        "The network connection was lost",
        true,
    ).with_user_action("Please check your internet connection and try again")
      .with_developer_action("Implement connection retry and offline mode"));

    taxonomy.register(ErrorType::new(
        3002,
        ErrorCategory::Timeout,
        "RequestTimeout",
        "The request timed out",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Check timeout settings and consider implementing circuit breakers"));

    taxonomy.register(ErrorType::new(
        3101,
        ErrorCategory::P2P,
        "PeerNotFound",
        "The requested peer was not found",
        true,
    ).with_user_action("The peer may be offline or unavailable")
      .with_developer_action("Implement peer discovery and connection management"));

    taxonomy.register(ErrorType::new(
        3102,
        ErrorCategory::P2P,
        "PeerRejected",
        "The connection was rejected by the peer",
        true,
    ).with_user_action("The peer rejected the connection request")
      .with_developer_action("Check authentication and authorization for peer connections"));

    // Business domain errors (4000-4999)
    taxonomy.register(ErrorType::new(
        4001,
        ErrorCategory::BusinessRule,
        "RuleViolation",
        "A business rule was violated",
        false,
    ).with_user_action("The operation cannot be completed due to business rules")
      .with_developer_action("Check business rule implementation and validation"));

    taxonomy.register(ErrorType::new(
        4002,
        ErrorCategory::Workflow,
        "InvalidStateTransition",
        "The requested state transition is invalid",
        false,
    ).with_user_action("This operation is not allowed in the current state")
      .with_developer_action("Check state machine implementation and allowed transitions"));

    taxonomy.register(ErrorType::new(
        4101,
        ErrorCategory::ResourceLimit,
        "QuotaExceeded",
        "The resource quota has been exceeded",
        false,
    ).with_user_action("You have reached the limit for this resource")
      .with_developer_action("Implement quota management and user notifications"));

    // System domain errors (5000-5999)
    taxonomy.register(ErrorType::new(
        5001,
        ErrorCategory::Configuration,
        "MissingConfiguration",
        "A required configuration value is missing",
        false,
    ).with_user_action("Please contact support")
      .with_developer_action("Check configuration loading and validation"));

    taxonomy.register(ErrorType::new(
        5002,
        ErrorCategory::Configuration,
        "InvalidConfiguration",
        "A configuration value is invalid",
        false,
    ).with_user_action("Please contact support")
      .with_developer_action("Implement configuration validation and provide better error messages"));

    taxonomy.register(ErrorType::new(
        5101,
        ErrorCategory::Initialization,
        "InitializationFailed",
        "Failed to initialize a component",
        false,
    ).with_user_action("Please restart the application")
      .with_developer_action("Check initialization sequence and dependencies"));

    taxonomy.register(ErrorType::new(
        5201,
        ErrorCategory::FileSystem,
        "FileNotFound",
        "The requested file was not found",
        false,
    ).with_user_action("The file you're looking for doesn't exist or has been moved")
      .with_developer_action("Implement proper error handling for file operations"));

    taxonomy.register(ErrorType::new(
        5202,
        ErrorCategory::FileSystem,
        "FileAccessDenied",
        "Access to the file was denied",
        false,
    ).with_user_action("You don't have permission to access this file")
      .with_developer_action("Check file permissions and access control"));

    taxonomy.register(ErrorType::new(
        5301,
        ErrorCategory::Actor,
        "ActorNotFound",
        "The requested actor was not found",
        false,
    ).with_user_action("Please try again or contact support")
      .with_developer_action("Check actor lifecycle management"));

    taxonomy.register(ErrorType::new(
        5302,
        ErrorCategory::Actor,
        "ActorInitializationFailed",
        "Failed to initialize an actor",
        false,
    ).with_user_action("Please restart the application")
      .with_developer_action("Check actor initialization and dependencies"));

    // Validation domain errors (6000-6999)
    taxonomy.register(ErrorType::new(
        6001,
        ErrorCategory::InputValidation,
        "RequiredFieldMissing",
        "A required field is missing",
        false,
    ).with_user_action("Please fill in all required fields")
      .with_developer_action("Implement client-side validation for required fields"));

    taxonomy.register(ErrorType::new(
        6002,
        ErrorCategory::InputValidation,
        "InvalidInput",
        "The input is invalid",
        false,
    ).with_user_action("Please check your input and try again")
      .with_developer_action("Provide more specific validation error messages"));

    taxonomy.register(ErrorType::new(
        6101,
        ErrorCategory::FormatValidation,
        "InvalidFormat",
        "The input format is invalid",
        false,
    ).with_user_action("Please check the format of your input")
      .with_developer_action("Implement format validation and provide examples"));

    taxonomy.register(ErrorType::new(
        6201,
        ErrorCategory::ConstraintValidation,
        "ValueOutOfRange",
        "The value is out of the allowed range",
        false,
    ).with_user_action("Please enter a value within the allowed range")
      .with_developer_action("Implement range validation and provide clear constraints"));

    // Integration domain errors (7000-7999)
    taxonomy.register(ErrorType::new(
        7001,
        ErrorCategory::ExternalService,
        "ServiceUnavailable",
        "An external service is unavailable",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Implement circuit breakers and fallback mechanisms"));

    taxonomy.register(ErrorType::new(
        7002,
        ErrorCategory::ExternalService,
        "ServiceError",
        "An external service returned an error",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Check service integration and error handling"));

    taxonomy.register(ErrorType::new(
        7101,
        ErrorCategory::API,
        "ApiRateLimitExceeded",
        "The API rate limit has been exceeded",
        true,
    ).with_user_action("Please try again later")
      .with_developer_action("Implement rate limiting and throttling"));

    taxonomy.register(ErrorType::new(
        7201,
        ErrorCategory::Plugin,
        "PluginNotFound",
        "The requested plugin was not found",
        false,
    ).with_user_action("The plugin may not be installed")
      .with_developer_action("Implement plugin discovery and management"));

    taxonomy.register(ErrorType::new(
        7202,
        ErrorCategory::Plugin,
        "PluginError",
        "A plugin encountered an error",
        false,
    ).with_user_action("Please try again or contact the plugin developer")
      .with_developer_action("Implement plugin sandboxing and error handling"));

    // Other domain errors (9000-9999)
    taxonomy.register(ErrorType::new(
        9001,
        ErrorCategory::Unexpected,
        "UnexpectedError",
        "An unexpected error occurred",
        false,
    ).with_user_action("Please try again or contact support")
      .with_developer_action("Implement comprehensive error logging and monitoring"));

    taxonomy.register(ErrorType::new(
        9002,
        ErrorCategory::Internal,
        "InternalError",
        "An internal error occurred",
        false,
    ).with_user_action("Please try again or contact support")
      .with_developer_action("Check internal error handling and logging"));

    taxonomy
}

/// Get an error type by code from the default taxonomy
pub fn get_error_type(code: ErrorCode) -> Option<ErrorType> {
    default_taxonomy().get(code).cloned()
}

/// Get an error type by code, or a default error type if not found
pub fn get_error_type_or_default(code: ErrorCode) -> ErrorType {
    get_error_type(code).unwrap_or_else(|| ErrorType::new(
        9999,
        ErrorCategory::Unexpected,
        "UnknownError",
        "An unknown error occurred",
        false,
    ))
}

/// Extended error context with taxonomy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyErrorContext {
    /// Base error context
    pub base: crate::error::ErrorContext,
    /// Error code
    pub error_code: ErrorCode,
    /// Error type
    pub error_type: String,
    /// Error domain
    pub domain: ErrorDomain,
    /// Error category
    pub category: ErrorCategory,
}

impl TaxonomyErrorContext {
    /// Create a new taxonomy error context
    pub fn new(error_code: ErrorCode) -> Self {
        let error_type = get_error_type_or_default(error_code);
        Self {
            base: crate::error::ErrorContext::new()
                .with_category(map_category(error_type.category))
                .with_severity(map_severity(&error_type))
                .with_retriable(error_type.retriable)
                .with_user_action(error_type.user_action.unwrap_or_default())
                .with_developer_action(error_type.developer_action.unwrap_or_default()),
            error_code,
            error_type: error_type.name,
            domain: error_type.domain,
            category: error_type.category,
        }
    }

    /// Convert to base error context
    pub fn to_base(self) -> crate::error::ErrorContext {
        self.base
    }
}

/// Map taxonomy category to legacy category
fn map_category(category: ErrorCategory) -> crate::error::ErrorCategory {
    match category {
        ErrorCategory::Authentication => crate::error::ErrorCategory::Authentication,
        ErrorCategory::Authorization => crate::error::ErrorCategory::Authorization,
        ErrorCategory::Credentials => crate::error::ErrorCategory::Authentication,
        ErrorCategory::Session => crate::error::ErrorCategory::Authentication,
        ErrorCategory::AccessControl => crate::error::ErrorCategory::Authorization,

        ErrorCategory::Database => crate::error::ErrorCategory::Database,
        ErrorCategory::DataValidation => crate::error::ErrorCategory::Validation,
        ErrorCategory::DataIntegrity => crate::error::ErrorCategory::Database,
        ErrorCategory::DataMigration => crate::error::ErrorCategory::Database,
        ErrorCategory::Serialization => crate::error::ErrorCategory::Validation,
        ErrorCategory::NotFound => crate::error::ErrorCategory::Other,
        ErrorCategory::Conflict => crate::error::ErrorCategory::Database,

        ErrorCategory::Connectivity => crate::error::ErrorCategory::Network,
        ErrorCategory::Timeout => crate::error::ErrorCategory::Network,
        ErrorCategory::Protocol => crate::error::ErrorCategory::Network,
        ErrorCategory::P2P => crate::error::ErrorCategory::Network,
        ErrorCategory::Messaging => crate::error::ErrorCategory::Network,

        ErrorCategory::BusinessRule => crate::error::ErrorCategory::BusinessLogic,
        ErrorCategory::Workflow => crate::error::ErrorCategory::BusinessLogic,
        ErrorCategory::StateTransition => crate::error::ErrorCategory::BusinessLogic,
        ErrorCategory::ResourceLimit => crate::error::ErrorCategory::ResourceLimit,
        ErrorCategory::FeatureFlag => crate::error::ErrorCategory::Configuration,

        ErrorCategory::Configuration => crate::error::ErrorCategory::Configuration,
        ErrorCategory::Initialization => crate::error::ErrorCategory::System,
        ErrorCategory::ResourceAllocation => crate::error::ErrorCategory::System,
        ErrorCategory::FileSystem => crate::error::ErrorCategory::System,
        ErrorCategory::Concurrency => crate::error::ErrorCategory::System,
        ErrorCategory::Memory => crate::error::ErrorCategory::System,
        ErrorCategory::Actor => crate::error::ErrorCategory::System,

        ErrorCategory::InputValidation => crate::error::ErrorCategory::Validation,
        ErrorCategory::FormatValidation => crate::error::ErrorCategory::Validation,
        ErrorCategory::ConstraintValidation => crate::error::ErrorCategory::Validation,
        ErrorCategory::TypeValidation => crate::error::ErrorCategory::Validation,

        ErrorCategory::ExternalService => crate::error::ErrorCategory::ExternalService,
        ErrorCategory::API => crate::error::ErrorCategory::ExternalService,
        ErrorCategory::Webhook => crate::error::ErrorCategory::ExternalService,
        ErrorCategory::ThirdParty => crate::error::ErrorCategory::ExternalService,
        ErrorCategory::Plugin => crate::error::ErrorCategory::ExternalService,

        ErrorCategory::Uncategorized => crate::error::ErrorCategory::Other,
        ErrorCategory::Unexpected => crate::error::ErrorCategory::Other,
        ErrorCategory::Internal => crate::error::ErrorCategory::InternalService,
    }
}

/// Map error type to severity
fn map_severity(error_type: &ErrorType) -> crate::error::ErrorSeverity {
    match error_type.domain {
        ErrorDomain::Security => {
            match error_type.category {
                ErrorCategory::Authentication => crate::error::ErrorSeverity::Critical,
                ErrorCategory::Authorization => crate::error::ErrorSeverity::Critical,
                _ => crate::error::ErrorSeverity::Error,
            }
        }
        ErrorDomain::Data => {
            match error_type.category {
                ErrorCategory::Database => crate::error::ErrorSeverity::Critical,
                ErrorCategory::DataIntegrity => crate::error::ErrorSeverity::Critical,
                ErrorCategory::NotFound => crate::error::ErrorSeverity::Warning,
                _ => crate::error::ErrorSeverity::Error,
            }
        }
        ErrorDomain::Network => {
            if error_type.retriable {
                crate::error::ErrorSeverity::Warning
            } else {
                crate::error::ErrorSeverity::Error
            }
        }
        ErrorDomain::System => {
            match error_type.category {
                ErrorCategory::Initialization => crate::error::ErrorSeverity::Fatal,
                ErrorCategory::Memory => crate::error::ErrorSeverity::Critical,
                _ => crate::error::ErrorSeverity::Error,
            }
        }
        ErrorDomain::Validation => crate::error::ErrorSeverity::Warning,
        _ => {
            if error_type.retriable {
                crate::error::ErrorSeverity::Warning
            } else {
                crate::error::ErrorSeverity::Error
            }
        }
    }
}

/// Create a taxonomic error
pub fn create_error(code: ErrorCode, message: Option<String>) -> crate::error::AppError {
    let error_type = get_error_type_or_default(code);
    let message = message.unwrap_or_else(|| error_type.description.clone());
    let context = TaxonomyErrorContext::new(code).to_base();
    crate::error::AppError::with_context(message, context)
}

/// Create a taxonomic error with additional context
pub fn create_error_with_context<F>(code: ErrorCode, message: Option<String>, context_builder: F) -> crate::error::AppError
where
    F: FnOnce(crate::error::ErrorContext) -> crate::error::ErrorContext,
{
    let error_type = get_error_type_or_default(code);
    let message = message.unwrap_or_else(|| error_type.description.clone());
    let base_context = TaxonomyErrorContext::new(code).to_base();
    let context = context_builder(base_context);
    crate::error::AppError::with_context(message, context)
}

/// Macro to create a taxonomic error with source location
#[macro_export]
macro_rules! taxonomic_error {
    ($code:expr) => {{
        let code = $crate::error::taxonomy::ErrorCode($code);
        $crate::error::taxonomy::create_error(code, None)
            .enrich(|ctx| ctx.with_source_location(file!(), line!()))
    }};
    ($code:expr, $message:expr) => {{
        let code = $crate::error::taxonomy::ErrorCode($code);
        $crate::error::taxonomy::create_error(code, Some($message.to_string()))
            .enrich(|ctx| ctx.with_source_location(file!(), line!()))
    }};
    ($code:expr, $message:expr, $($context:tt)*) => {{
        let code = $crate::error::taxonomy::ErrorCode($code);
        $crate::error::taxonomy::create_error_with_context(
            code,
            Some($message.to_string()),
            |ctx| ctx.with_source_location(file!(), line!()) $($context)*
        )
    }};
}