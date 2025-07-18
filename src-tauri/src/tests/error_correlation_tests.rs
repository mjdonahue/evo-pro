//! Tests for error correlation with structured logging
//!
//! This module contains tests for the error correlation functionality,
//! which allows errors to be linked together using correlation IDs.

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;
    use uuid::Uuid;
    
    use crate::error::{AppError, ErrorContext, Result};
    use crate::error::enrichment::{
        with_correlation_id, with_new_correlation_id, with_child_correlation_id,
        with_error_correlation, with_new_error_correlation, with_child_error_correlation,
    };
    use crate::logging::correlation;
    
    // Helper function to create a test error
    fn create_test_error() -> AppError {
        AppError::InternalError("Test error".to_string())
    }
    
    // Helper function to simulate an operation that might fail
    fn fallible_operation(should_fail: bool) -> Result<String> {
        if should_fail {
            Err(create_test_error())
        } else {
            Ok("success".to_string())
        }
    }
    
    #[test]
    fn test_with_correlation_id() {
        let correlation_id = "test-correlation-id";
        let error = with_correlation_id(create_test_error(), correlation_id);
        
        match error {
            AppError::ContextualError { context, .. } => {
                assert_eq!(context.correlation_id, Some(correlation_id.to_string()));
                
                // Verify that the correlation ID was set in thread-local storage
                assert_eq!(correlation::get_correlation_id(), Some(correlation_id.to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }
    }
    
    #[test]
    fn test_with_new_correlation_id() {
        // Clear any existing correlation ID
        correlation::clear_correlation_id();
        
        let error = with_new_correlation_id(create_test_error());
        
        match error {
            AppError::ContextualError { context, .. } => {
                assert!(context.correlation_id.is_some());
                let correlation_id = context.correlation_id.unwrap();
                
                // Verify that the correlation ID was set in thread-local storage
                assert_eq!(correlation::get_correlation_id(), Some(correlation_id));
                
                // Verify that it's a valid UUID
                assert!(Uuid::parse_str(&correlation_id).is_ok());
            }
            _ => panic!("Expected ContextualError"),
        }
    }
    
    #[test]
    fn test_with_child_correlation_id() {
        // Set a parent correlation ID
        let parent_id = "parent-correlation-id";
        correlation::set_correlation_id(parent_id);
        
        let error = with_child_correlation_id(create_test_error());
        
        match error {
            AppError::ContextualError { context, .. } => {
                assert!(context.correlation_id.is_some());
                let child_id = context.correlation_id.unwrap();
                
                // Verify that the child ID starts with the parent ID
                assert!(child_id.starts_with(&format!("{}.", parent_id)));
                
                // Verify that the child ID was set in thread-local storage
                assert_eq!(correlation::get_correlation_id(), Some(child_id));
            }
            _ => panic!("Expected ContextualError"),
        }
    }
    
    #[test]
    fn test_with_error_correlation() {
        // Clear any existing correlation ID
        correlation::clear_correlation_id();
        
        // Capture the correlation ID used in the operation
        let captured_id = Arc::new(Mutex::new(None));
        let captured_id_clone = captured_id.clone();
        
        // Execute a fallible operation with error correlation
        let result = with_error_correlation(|| {
            // Capture the correlation ID
            let current_id = correlation::get_correlation_id();
            *captured_id_clone.lock().unwrap() = current_id.clone();
            
            // Perform the operation
            fallible_operation(true)
        });
        
        // Verify that the operation failed with a correlated error
        assert!(result.is_err());
        match result {
            Err(AppError::ContextualError { context, .. }) => {
                assert!(context.correlation_id.is_some());
                let error_id = context.correlation_id.unwrap();
                
                // Verify that the error has the same correlation ID as the operation
                let captured = captured_id.lock().unwrap().clone().unwrap();
                assert_eq!(error_id, captured);
            }
            _ => panic!("Expected ContextualError"),
        }
    }
    
    #[test]
    fn test_with_new_error_correlation() {
        // Set an existing correlation ID that should be replaced
        correlation::set_correlation_id("existing-correlation-id");
        
        // Capture the correlation ID used in the operation
        let captured_id = Arc::new(Mutex::new(None));
        let captured_id_clone = captured_id.clone();
        
        // Execute a fallible operation with a new error correlation
        let result = with_new_error_correlation(|| {
            // Capture the correlation ID
            let current_id = correlation::get_correlation_id();
            *captured_id_clone.lock().unwrap() = current_id.clone();
            
            // Verify that the correlation ID is not the existing one
            assert_ne!(current_id, Some("existing-correlation-id".to_string()));
            
            // Perform the operation
            fallible_operation(true)
        });
        
        // Verify that the operation failed with a correlated error
        assert!(result.is_err());
        match result {
            Err(AppError::ContextualError { context, .. }) => {
                assert!(context.correlation_id.is_some());
                let error_id = context.correlation_id.unwrap();
                
                // Verify that the error has the same correlation ID as the operation
                let captured = captured_id.lock().unwrap().clone().unwrap();
                assert_eq!(error_id, captured);
                
                // Verify that the correlation ID is not the existing one
                assert_ne!(error_id, "existing-correlation-id");
            }
            _ => panic!("Expected ContextualError"),
        }
        
        // Verify that the original correlation ID is restored
        assert_eq!(correlation::get_correlation_id(), Some("existing-correlation-id".to_string()));
    }
    
    #[test]
    fn test_with_child_error_correlation() {
        // Set a parent correlation ID
        let parent_id = "parent-correlation-id";
        correlation::set_correlation_id(parent_id);
        
        // Capture the correlation ID used in the operation
        let captured_id = Arc::new(Mutex::new(None));
        let captured_id_clone = captured_id.clone();
        
        // Execute a fallible operation with a child error correlation
        let result = with_child_error_correlation(|| {
            // Capture the correlation ID
            let current_id = correlation::get_correlation_id();
            *captured_id_clone.lock().unwrap() = current_id.clone();
            
            // Verify that the correlation ID is a child of the parent
            assert!(current_id.unwrap().starts_with(&format!("{}.", parent_id)));
            
            // Perform the operation
            fallible_operation(true)
        });
        
        // Verify that the operation failed with a correlated error
        assert!(result.is_err());
        match result {
            Err(AppError::ContextualError { context, .. }) => {
                assert!(context.correlation_id.is_some());
                let error_id = context.correlation_id.unwrap();
                
                // Verify that the error has the same correlation ID as the operation
                let captured = captured_id.lock().unwrap().clone().unwrap();
                assert_eq!(error_id, captured);
                
                // Verify that the correlation ID is a child of the parent
                assert!(error_id.starts_with(&format!("{}.", parent_id)));
            }
            _ => panic!("Expected ContextualError"),
        }
        
        // Verify that the parent correlation ID is restored
        assert_eq!(correlation::get_correlation_id(), Some(parent_id.to_string()));
    }
    
    #[test]
    fn test_error_enrichment_with_correlation() {
        // Set a correlation ID
        let correlation_id = "test-correlation-id";
        correlation::set_correlation_id(correlation_id);
        
        // Create an error and enrich it
        let error = create_test_error().enrich().build();
        
        // Verify that the error has the correlation ID
        match error {
            AppError::ContextualError { context, .. } => {
                assert_eq!(context.correlation_id, Some(correlation_id.to_string()));
            }
            _ => panic!("Expected ContextualError"),
        }
    }
    
    #[test]
    fn test_successful_operation_with_correlation() {
        // Clear any existing correlation ID
        correlation::clear_correlation_id();
        
        // Capture the correlation ID used in the operation
        let captured_id = Arc::new(Mutex::new(None));
        let captured_id_clone = captured_id.clone();
        
        // Execute a successful operation with error correlation
        let result = with_error_correlation(|| {
            // Capture the correlation ID
            let current_id = correlation::get_correlation_id();
            *captured_id_clone.lock().unwrap() = current_id.clone();
            
            // Perform the operation
            fallible_operation(false)
        });
        
        // Verify that the operation succeeded
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        
        // Verify that a correlation ID was used
        assert!(captured_id.lock().unwrap().is_some());
    }
    
    #[test]
    fn test_correlation_id_propagation() {
        // Clear any existing correlation ID
        correlation::clear_correlation_id();
        
        // Execute nested operations with correlation IDs
        let result = with_new_error_correlation(|| {
            let parent_id = correlation::get_correlation_id().unwrap();
            
            // Execute a child operation
            let child_result = with_child_error_correlation(|| {
                let child_id = correlation::get_correlation_id().unwrap();
                
                // Verify that the child ID is derived from the parent
                assert!(child_id.starts_with(&format!("{}.", parent_id)));
                
                // Perform the operation
                fallible_operation(true)
            });
            
            // Propagate the error
            child_result
        });
        
        // Verify that the operation failed with a correlated error
        assert!(result.is_err());
        match result {
            Err(AppError::ContextualError { context, .. }) => {
                assert!(context.correlation_id.is_some());
                let error_id = context.correlation_id.unwrap();
                
                // Verify that the correlation ID has the expected format (parent.child)
                let parts: Vec<&str> = error_id.split('.').collect();
                assert_eq!(parts.len(), 2);
            }
            _ => panic!("Expected ContextualError"),
        }
    }
}