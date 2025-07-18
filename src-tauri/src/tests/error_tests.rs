//! Tests for the error handling framework

#[cfg(test)]
mod tests {
    use crate::error::{
        AppError, ErrorCategory, ErrorContext, ErrorSeverity,
        reporter::{ErrorReport, ErrorReporter, ErrorReportExt},
    };
    use crate::{contextual_error, enrich_error, log_error};

    #[test]
    fn test_error_factory_methods() {
        // Test not_found
        let error = AppError::not_found("User", "123");
        assert_eq!(error.to_string(), "Not found: User with ID 123 not found");
        assert_eq!(error.category(), ErrorCategory::Other);
        assert_eq!(error.severity(), ErrorSeverity::Warning);
        assert!(!error.is_retriable());

        // Test validation
        let error = AppError::validation("Invalid input");
        assert_eq!(error.to_string(), "Validation error: Invalid input");
        assert_eq!(error.category(), ErrorCategory::Validation);
        assert_eq!(error.severity(), ErrorSeverity::Warning);
        assert!(!error.is_retriable());

        // Test authorization
        let error = AppError::authorization("Missing permission");
        assert_eq!(error.to_string(), "Authorization error: Missing permission");
        assert_eq!(error.category(), ErrorCategory::Authorization);
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(!error.is_retriable());

        // Test database
        let error = AppError::database("Connection failed");
        assert_eq!(error.to_string(), "Database error: Connection failed");
        assert_eq!(error.category(), ErrorCategory::Database);
        assert_eq!(error.severity(), ErrorSeverity::Critical);
        assert!(error.is_retriable());
    }

    #[test]
    fn test_contextual_error_macro() {
        let error = contextual_error!(
            "Test error message",
            .with_category(ErrorCategory::Validation)
            .with_severity(ErrorSeverity::Warning)
            .with_entity_type("User")
            .with_entity_id("123")
            .with_context("test_key", "test_value")
            .with_user_action("Please fix the input")
            .with_developer_action("Check validation logic")
        );

        match error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Test error message");
                assert_eq!(context.category, ErrorCategory::Validation);
                assert_eq!(context.severity, ErrorSeverity::Warning);
                assert_eq!(context.entity_type, Some("User".to_string()));
                assert_eq!(context.entity_id, Some("123".to_string()));
                assert_eq!(context.additional_context.get("test_key"), Some(&"test_value".to_string()));
                assert_eq!(context.user_action, Some("Please fix the input".to_string()));
                assert_eq!(context.developer_action, Some("Check validation logic".to_string()));
                assert!(context.source_location.is_some()); // Should include file and line
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_enrich_error_macro() {
        let original_error = AppError::validation("Invalid input");
        let enriched_error = enrich_error!(
            original_error,
            .with_entity_type("User")
            .with_entity_id("123")
            .with_context("test_key", "test_value")
        );

        match enriched_error {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "Validation error: Invalid input");
                assert_eq!(context.category, ErrorCategory::Validation);
                assert_eq!(context.severity, ErrorSeverity::Warning);
                assert_eq!(context.entity_type, Some("User".to_string()));
                assert_eq!(context.entity_id, Some("123".to_string()));
                assert_eq!(context.additional_context.get("test_key"), Some(&"test_value".to_string()));
                assert!(context.source_location.is_some()); // Should include file and line
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[tokio::test]
    async fn test_error_reporter() {
        // Create a reporter
        let reporter = ErrorReporter::new(10, false);

        // Create and report an error
        let error = AppError::validation("Invalid input");
        let report = reporter.report(&error).await;

        // Check the report
        assert_eq!(report.message, "Validation error: Invalid input");
        assert_eq!(report.context.category, ErrorCategory::Validation);
        assert_eq!(report.context.severity, ErrorSeverity::Warning);
        assert!(!report.context.retriable);

        // Check user message
        assert_eq!(report.user_message, "The provided data is invalid.");

        // Create a contextual error with a custom user action
        let error = contextual_error!(
            "Custom error",
            .with_user_action("This is a custom user action")
        );
        let report = reporter.report(&error).await;

        // Check that the custom user action is used
        assert_eq!(report.user_message, "This is a custom user action");

        // Check that the report is stored
        let recent_reports = reporter.get_recent_reports().await;
        assert_eq!(recent_reports.len(), 2);
        assert_eq!(recent_reports[0].message, "Validation error: Invalid input");
        assert_eq!(recent_reports[1].message, "Custom error");
    }

    #[test]
    fn test_error_categories_and_severities() {
        // Test that all error types have appropriate categories and severities
        let errors = vec![
            (AppError::AuthenticationError("test".to_string()), ErrorCategory::Authentication, ErrorSeverity::Critical),
            (AppError::AuthorizationError("test".to_string()), ErrorCategory::Authorization, ErrorSeverity::Critical),
            (AppError::ValidationError("test".to_string()), ErrorCategory::Validation, ErrorSeverity::Warning),
            (AppError::DatabaseError("test".to_string()), ErrorCategory::Database, ErrorSeverity::Critical),
            (AppError::NotFoundError("test".to_string()), ErrorCategory::Other, ErrorSeverity::Warning),
            (AppError::OperationNotSupported("test".to_string()), ErrorCategory::Other, ErrorSeverity::Error),
            (AppError::ResourceLimitExceeded("test".to_string()), ErrorCategory::ResourceLimit, ErrorSeverity::Error),
            (AppError::ConfigurationError("test".to_string()), ErrorCategory::Configuration, ErrorSeverity::Error),
            (AppError::ExternalServiceError("test".to_string()), ErrorCategory::ExternalService, ErrorSeverity::Error),
        ];

        for (error, expected_category, expected_severity) in errors {
            assert_eq!(error.category(), expected_category, "Wrong category for {:?}", error);
            assert_eq!(error.severity(), expected_severity, "Wrong severity for {:?}", error);
        }
    }

    #[test]
    fn test_error_retriable() {
        // Test that retriable errors are correctly identified
        let retriable_errors = vec![
            AppError::DatabaseError("test".to_string()),
            AppError::ExternalServiceError("test".to_string()),
        ];

        for error in retriable_errors {
            assert!(error.is_retriable(), "Error should be retriable: {:?}", error);
        }

        // Test that non-retriable errors are correctly identified
        let non_retriable_errors = vec![
            AppError::ValidationError("test".to_string()),
            AppError::AuthorizationError("test".to_string()),
            AppError::NotFoundError("test".to_string()),
        ];

        for error in non_retriable_errors {
            assert!(!error.is_retriable(), "Error should not be retriable: {:?}", error);
        }
    }

    #[test]
    fn test_error_report_generation() {
        // Test that error reports are correctly generated
        let errors = vec![
            (AppError::ValidationError("test".to_string()), "The provided data is invalid."),
            (AppError::NotFoundError("test".to_string()), "The requested resource could not be found."),
            (AppError::AuthorizationError("test".to_string()), "You don't have permission to perform this action."),
            (AppError::DatabaseError("test".to_string()), "A database error occurred. Please try again later."),
        ];

        for (error, expected_message) in errors {
            let report = ErrorReport::new(&error);
            assert_eq!(report.user_message, expected_message);
        }

        // Test that custom user actions are respected
        let error = contextual_error!(
            "Custom error",
            .with_user_action("This is a custom user action")
        );
        let report = ErrorReport::new(&error);
        assert_eq!(report.user_message, "This is a custom user action");
    }
}