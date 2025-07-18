//! Tests for the error taxonomy
//!
//! This module contains tests for the error taxonomy functionality.

#[cfg(test)]
mod tests {
    use crate::error::{AppError, ErrorCategory, ErrorContext, ErrorSeverity};
    use crate::error::taxonomy::{
        ErrorCode, ErrorDomain, ErrorType, TaxonomyErrorContext,
        create_error, create_error_with_context, default_taxonomy,
        get_error_type, get_error_type_or_default
    };
    use crate::taxonomic_error;

    #[test]
    fn test_error_domain_display() {
        assert_eq!(ErrorDomain::Security.to_string(), "Security");
        assert_eq!(ErrorDomain::Data.to_string(), "Data");
        assert_eq!(ErrorDomain::Network.to_string(), "Network");
        assert_eq!(ErrorDomain::Business.to_string(), "Business");
        assert_eq!(ErrorDomain::System.to_string(), "System");
        assert_eq!(ErrorDomain::Validation.to_string(), "Validation");
        assert_eq!(ErrorDomain::Integration.to_string(), "Integration");
        assert_eq!(ErrorDomain::Other.to_string(), "Other");
    }

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode(1001).to_string(), "E1001");
        assert_eq!(ErrorCode(9999).to_string(), "E9999");
    }

    #[test]
    fn test_error_type_creation() {
        let error_type = ErrorType::new(
            1001,
            crate::error::taxonomy::ErrorCategory::Authentication,
            "InvalidCredentials",
            "The provided credentials are invalid",
            true,
        );

        assert_eq!(error_type.code, ErrorCode(1001));
        assert_eq!(error_type.domain, ErrorDomain::Security);
        assert_eq!(error_type.category, crate::error::taxonomy::ErrorCategory::Authentication);
        assert_eq!(error_type.name, "InvalidCredentials");
        assert_eq!(error_type.description, "The provided credentials are invalid");
        assert_eq!(error_type.retriable, true);
        assert_eq!(error_type.user_action, None);
        assert_eq!(error_type.developer_action, None);
    }

    #[test]
    fn test_error_type_with_actions() {
        let error_type = ErrorType::new(
            1001,
            crate::error::taxonomy::ErrorCategory::Authentication,
            "InvalidCredentials",
            "The provided credentials are invalid",
            true,
        )
        .with_user_action("Please check your username and password and try again")
        .with_developer_action("Verify the authentication logic and credential validation");

        assert_eq!(error_type.user_action, Some("Please check your username and password and try again".to_string()));
        assert_eq!(error_type.developer_action, Some("Verify the authentication logic and credential validation".to_string()));
    }

    #[test]
    fn test_error_type_display() {
        let error_type = ErrorType::new(
            1001,
            crate::error::taxonomy::ErrorCategory::Authentication,
            "InvalidCredentials",
            "The provided credentials are invalid",
            true,
        );

        assert_eq!(error_type.to_string(), "E1001: InvalidCredentials (Authentication)");
    }

    #[test]
    fn test_default_taxonomy() {
        let taxonomy = default_taxonomy();
        
        // Check that the taxonomy contains the expected error types
        assert!(taxonomy.get(ErrorCode(1001)).is_some());
        assert!(taxonomy.get(ErrorCode(2001)).is_some());
        assert!(taxonomy.get(ErrorCode(3001)).is_some());
        assert!(taxonomy.get(ErrorCode(9001)).is_some());
        
        // Check that the taxonomy doesn't contain non-existent error types
        assert!(taxonomy.get(ErrorCode(1234)).is_none());
        
        // Check that we can get error types by domain
        let security_errors = taxonomy.get_by_domain(ErrorDomain::Security);
        assert!(!security_errors.is_empty());
        assert!(security_errors.iter().all(|e| e.domain == ErrorDomain::Security));
        
        // Check that we can get error types by category
        let auth_errors = taxonomy.get_by_category(crate::error::taxonomy::ErrorCategory::Authentication);
        assert!(!auth_errors.is_empty());
        assert!(auth_errors.iter().all(|e| e.category == crate::error::taxonomy::ErrorCategory::Authentication));
    }

    #[test]
    fn test_get_error_type() {
        // Get an existing error type
        let error_type = get_error_type(ErrorCode(1001));
        assert!(error_type.is_some());
        let error_type = error_type.unwrap();
        assert_eq!(error_type.code, ErrorCode(1001));
        assert_eq!(error_type.name, "InvalidCredentials");
        
        // Get a non-existent error type
        let error_type = get_error_type(ErrorCode(1234));
        assert!(error_type.is_none());
    }

    #[test]
    fn test_get_error_type_or_default() {
        // Get an existing error type
        let error_type = get_error_type_or_default(ErrorCode(1001));
        assert_eq!(error_type.code, ErrorCode(1001));
        assert_eq!(error_type.name, "InvalidCredentials");
        
        // Get a non-existent error type (should return default)
        let error_type = get_error_type_or_default(ErrorCode(1234));
        assert_eq!(error_type.code, ErrorCode(9999));
        assert_eq!(error_type.name, "UnknownError");
    }

    #[test]
    fn test_taxonomy_error_context() {
        let context = TaxonomyErrorContext::new(ErrorCode(1001));
        
        // Check that the base context has the expected values
        assert_eq!(context.base.category, ErrorCategory::Authentication);
        assert_eq!(context.base.severity, ErrorSeverity::Critical);
        assert_eq!(context.base.retriable, true);
        
        // Check that the taxonomy context has the expected values
        assert_eq!(context.error_code, ErrorCode(1001));
        assert_eq!(context.error_type, "InvalidCredentials");
        assert_eq!(context.domain, ErrorDomain::Security);
        assert_eq!(context.category, crate::error::taxonomy::ErrorCategory::Authentication);
    }

    #[test]
    fn test_create_error() {
        // Create an error with default message
        let error = create_error(ErrorCode(1001), None);
        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "The provided credentials are invalid");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected ContextualError"),
        }
        
        // Create an error with custom message
        let error = create_error(ErrorCode(1001), Some("Invalid username or password".to_string()));
        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Invalid username or password");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_create_error_with_context() {
        // Create an error with additional context
        let error = create_error_with_context(
            ErrorCode(1001),
            Some("Invalid username or password".to_string()),
            |ctx| ctx.with_user_id("user123").with_operation("login"),
        );
        
        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Invalid username or password");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
                assert_eq!(context.user_id, Some("user123".to_string()));
                assert_eq!(context.operation, Some("login".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_taxonomic_error_macro() {
        // Basic usage
        let error = taxonomic_error!(1001);
        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "The provided credentials are invalid");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected ContextualError"),
        }
        
        // With custom message
        let error = taxonomic_error!(1001, "Invalid username or password");
        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Invalid username or password");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected ContextualError"),
        }
        
        // With custom message and additional context
        let error = taxonomic_error!(1001, "Invalid username or password", .with_user_id("user123").with_operation("login"));
        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Invalid username or password");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
                assert!(context.source_location.is_some());
                assert_eq!(context.user_id, Some("user123".to_string()));
                assert_eq!(context.operation, Some("login".to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }
    }
}