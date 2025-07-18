//! Tests for the error enrichment functionality
//!
//! This module contains tests for the error enrichment functionality.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::error::{AppError, ErrorCategory, ErrorContext, ErrorSeverity, Result};
    use crate::error::taxonomy::ErrorCode;
    use crate::error::enrichment::{
        ErrorEnricher, LogLevel, ErrorEnrichmentExt, ResultEnrichmentExt,
        with_location, with_operation, with_entity, with_user, with_workspace,
        with_correlation, with_request, with_code, log_error, log_error_with_level
    };
    use crate::{
        contextual_error_with, taxonomic_error_with, enrich_error_with, log_error_with
    };

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

        // Test with Ok result
        let result: Result<i32> = Ok(42);
        let enriched = result.with_source_location();
        assert_eq!(enriched, Ok(42));

        let result: Result<i32> = Ok(42);
        let enriched = result.with_error_code(ErrorCode(1001));
        assert_eq!(enriched, Ok(42));

        let result: Result<i32> = Ok(42);
        let enriched = result.enrich_err(|e| e.with_operation("test_operation"));
        assert_eq!(enriched, Ok(42));
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
    fn test_log_error_with_macro() {
        let error = AppError::InternalError("Test error".to_string());
        let enriched = log_error_with!(error, |e| e.with_operation("test_operation"));

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

    #[test]
    fn test_integration_with_taxonomy() {
        // Create a taxonomic error
        let error = taxonomic_error_with!(1001, |e| e);
        
        // Enrich it with additional context
        let enriched = error.enrich()
            .with_source_location()
            .with_operation("login")
            .with_user_id("user123")
            .build();
        
        match enriched {
            AppError::ContextualError { message, context } => {
                assert_eq!(message, "The provided credentials are invalid");
                assert_eq!(context.category, ErrorCategory::Authentication);
                assert_eq!(context.severity, ErrorSeverity::Critical);
                assert_eq!(context.retriable, true);
                assert_eq!(context.operation, Some("login".to_string()));
                assert_eq!(context.user_id, Some("user123".to_string()));
                assert!(context.source_location.is_some());
            }
            _ => panic!("Expected ContextualError"),
        }
    }

    #[test]
    fn test_real_world_usage() {
        // Simulate a function that returns a Result
        fn authenticate_user(username: &str, password: &str) -> Result<String> {
            // Simulate authentication failure
            if username != "admin" || password != "password" {
                return Err(AppError::AuthenticationError("Invalid credentials".to_string()));
            }
            
            // Simulate success
            Ok("auth_token_123".to_string())
        }
        
        // Simulate a handler function that uses the authenticate_user function
        fn login_handler(username: &str, password: &str) -> Result<String> {
            // Call the authenticate_user function and enrich any errors
            authenticate_user(username, password)
                .enrich_err(|e| e
                    .with_source_location()
                    .with_operation("login")
                    .with_user_id(username)
                    .with_context("attempt_time", chrono::Utc::now().to_string())
                )
        }
        
        // Test with invalid credentials
        let result = login_handler("user123", "wrong_password");
        match result {
            Err(AppError::ContextualError { message, context }) => {
                assert_eq!(message, "Invalid credentials");
                assert_eq!(context.operation, Some("login".to_string()));
                assert_eq!(context.user_id, Some("user123".to_string()));
                assert!(context.source_location.is_some());
                assert!(context.additional_context.contains_key("attempt_time"));
            }
            _ => panic!("Expected Err(ContextualError)"),
        }
        
        // Test with valid credentials
        let result = login_handler("admin", "password");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "auth_token_123");
    }
}