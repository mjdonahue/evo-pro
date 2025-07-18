//! Tests for the logging framework

#[cfg(test)]
mod tests {
    use crate::error::Result;
    use crate::logging::{LogContext, LogLevel, OperationLogger};
    use crate::services::logging::ServiceContextLoggingExt;
    use crate::services::traits::{AuthContext, ServiceContext};
    use sqlx::sqlite::SqlitePoolOptions;
    use std::sync::Arc;
    use uuid::Uuid;

    // Helper function to create a test service context
    async fn create_test_context() -> Result<ServiceContext> {
        // Create an in-memory SQLite database for testing
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await?;
            
        // Create a mock actor system
        let actor_system = Arc::new(crate::services::traits::ActorSystem::new());
        
        // Create a test auth context
        let auth_context = Some(AuthContext {
            participant_id: Uuid::new_v4(),
            permissions: vec!["read".to_string(), "write".to_string()],
        });
        
        // Create a test service context
        let ctx = ServiceContext {
            db,
            actor_system,
            auth_context,
            request_id: Some(format!("test_service.test_method:{}", Uuid::new_v4())),
            workspace_id: Some(Uuid::new_v4()),
        };
        
        Ok(ctx)
    }

    #[tokio::test]
    async fn test_log_context_from_service_context() -> Result<()> {
        // Create a test service context
        let ctx = create_test_context().await?;
        
        // Create a log context from the service context
        let log_context = LogContext::from_service_context(&ctx);
        
        // Verify that the log context has the expected values
        assert!(log_context.log_id != Uuid::nil());
        assert_eq!(log_context.request_id, ctx.request_id);
        assert_eq!(log_context.workspace_id, ctx.workspace_id);
        assert_eq!(
            log_context.user_id,
            ctx.auth_context.as_ref().map(|auth| auth.participant_id.to_string())
        );
        
        // Verify that the correlation ID was extracted from the request ID
        if let Some(request_id) = &ctx.request_id {
            if request_id.contains(":") {
                let parts: Vec<&str> = request_id.split(":").collect();
                if parts.len() > 1 {
                    assert_eq!(log_context.correlation_id, Some(parts[1].to_string()));
                }
            }
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_operation_logger() -> Result<()> {
        // Create a test service context
        let ctx = create_test_context().await?;
        
        // Create an operation logger
        let logger = OperationLogger::from_service_context("test_operation", &ctx);
        
        // Start the operation
        logger.start();
        
        // Log some messages
        logger.info("Operation in progress");
        logger.debug("Detailed information about the operation");
        
        // End the operation
        logger.end();
        
        // We can't easily verify the log output in a test, but we can verify that the code runs without errors
        Ok(())
    }

    #[tokio::test]
    async fn test_service_context_logging_ext() -> Result<()> {
        // Create a test service context
        let ctx = create_test_context().await?;
        
        // Use the extension methods to log messages
        ctx.trace("Trace message")?;
        ctx.debug("Debug message")?;
        ctx.info("Info message")?;
        ctx.warn("Warning message")?;
        ctx.error("Error message")?;
        
        // Create an operation logger
        let logger = ctx.operation_logger("test_operation");
        logger.start();
        logger.info("Operation in progress");
        logger.end();
        
        // We can't easily verify the log output in a test, but we can verify that the code runs without errors
        Ok(())
    }

    #[tokio::test]
    async fn test_log_operation_macro() -> Result<()> {
        // Create a test service context
        let ctx = create_test_context().await?;
        
        // Use the log_operation macro
        let result = crate::log_operation!("test_operation", &ctx, {
            // Simulate some work
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            "operation result"
        });
        
        // Verify the result
        assert_eq!(result, "operation result");
        
        Ok(())
    }
}