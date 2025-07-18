//! Service layer logging utilities
//!
//! This module provides logging utilities specifically for the service layer,
//! including middleware for automatic logging of service operations.

use crate::error::Result;
use crate::logging::{self, LogContext, LogLevel, OperationLogger};
use crate::services::traits::*;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

/// Service logging middleware that automatically logs service operations
pub struct ServiceLoggingMiddleware {
    /// Log level for service operations
    pub log_level: LogLevel,
    /// Whether to log detailed information about service operations
    pub log_details: bool,
}

impl Default for ServiceLoggingMiddleware {
    fn default() -> Self {
        Self {
            log_level: LogLevel::Info,
            log_details: true,
        }
    }
}

#[async_trait]
impl Middleware for ServiceLoggingMiddleware {
    async fn process<'a, T>(
        &self,
        ctx: &'a ServiceContext,
        next: Next<'a, T>,
    ) -> Result<ServiceResult<T>> {
        let operation_name = std::any::type_name::<T>();
        let service_name = if operation_name.contains("::") {
            operation_name.split("::").next().unwrap_or("unknown")
        } else {
            "unknown"
        };
        
        // Create a logger for this operation
        let mut log_context = LogContext::from_service_context(ctx)
            .with_operation(operation_name)
            .with_context("service", service_name);
            
        // Add entity information if available from the operation name
        if let Some(entity_type) = extract_entity_type(operation_name) {
            log_context = log_context.with_entity_type(entity_type);
        }
        
        let logger = OperationLogger::new(operation_name, log_context);
        
        // Log operation start with parameters if enabled
        if self.log_details {
            // In a real implementation, we would log the parameters
            // For now, we just log that the operation is starting
            logger.info(&format!("Service operation starting: {}", operation_name));
        }
        
        // Execute the operation
        let start = Instant::now();
        let result = next.run(ctx).await;
        let duration = start.elapsed();
        
        // Log operation result
        match &result {
            Ok(service_result) => {
                // Log success with metrics if available
                if let Some(metadata) = &service_result.metadata {
                    logger.info(&format!(
                        "Service operation completed successfully in {}ms with {} actor interactions",
                        metadata.execution_time_ms,
                        metadata.actor_interactions.len()
                    ));
                    
                    // Log detailed metrics if enabled
                    if self.log_details {
                        // Log actor interactions
                        for interaction in &metadata.actor_interactions {
                            logger.debug(&format!(
                                "Actor interaction: {} - {} ({}ms, success: {})",
                                interaction.actor_type,
                                interaction.operation,
                                interaction.duration_ms,
                                interaction.success
                            ));
                        }
                        
                        // Log if this was a cache hit
                        if metadata.cache_hit {
                            logger.debug("Result was served from cache");
                        }
                    }
                } else {
                    logger.info(&format!(
                        "Service operation completed successfully in {:?}",
                        duration
                    ));
                }
                
                // Log result details if enabled
                if self.log_details {
                    // In a real implementation, we would log the result details
                    // For now, we just log that the operation completed
                    logger.debug("Operation result details omitted for brevity");
                }
            }
            Err(err) => {
                // Log error (the error will be logged in detail by the ErrorHandlingMiddleware)
                logger.error(&format!("Service operation failed: {}", err));
            }
        }
        
        // Return the result
        result
    }
}

/// Extract the entity type from an operation name
/// 
/// For example, "app::services::user::UserService::create" would return "User"
fn extract_entity_type(operation_name: &str) -> Option<String> {
    // Split by :: and look for patterns like "UserService" or "User"
    let parts: Vec<&str> = operation_name.split("::").collect();
    
    for part in parts {
        // Look for "XxxService"
        if part.ends_with("Service") {
            return Some(part.trim_end_matches("Service").to_string());
        }
        
        // Look for entity names (typically PascalCase)
        if !part.is_empty() && part.chars().next().unwrap().is_uppercase() {
            return Some(part.to_string());
        }
    }
    
    None
}

/// Log a service event
pub fn log_service_event(ctx: &ServiceContext, event: &ServiceEvent) -> Result<()> {
    let log_context = LogContext::from_service_context(ctx)
        .with_operation("service_event")
        .with_entity_type(&event.entity_type)
        .with_entity_id(event.entity_id.to_string())
        .with_context("event_type", &event.event_type);
        
    if let Some(workspace_id) = event.workspace_id {
        let log_context = log_context.with_workspace_id(workspace_id);
        logging::info_with_context(&format!("Service event: {}", event.event_type), &log_context);
    } else {
        logging::info_with_context(&format!("Service event: {}", event.event_type), &log_context);
    }
    
    Ok(())
}

/// Log a validation result
pub fn log_validation_result(ctx: &ServiceContext, entity_type: &str, result: &ValidationResult) -> Result<()> {
    let log_context = LogContext::from_service_context(ctx)
        .with_operation("validate")
        .with_entity_type(entity_type)
        .with_context("valid", result.valid.to_string())
        .with_context("error_count", result.errors.len().to_string())
        .with_context("warning_count", result.warnings.len().to_string());
        
    if result.valid {
        logging::info_with_context(&format!("Validation passed for {}", entity_type), &log_context);
    } else {
        let error_messages: Vec<String> = result.errors.iter()
            .map(|e| format!("{}: {}", e.field, e.message))
            .collect();
            
        logging::warn_with_context(
            &format!("Validation failed for {}: {}", entity_type, error_messages.join(", ")),
            &log_context
        );
    }
    
    // Log warnings if any
    if !result.warnings.is_empty() {
        let warning_messages: Vec<String> = result.warnings.iter()
            .map(|w| format!("{}: {}", w.field, w.message))
            .collect();
            
        logging::warn_with_context(
            &format!("Validation warnings for {}: {}", entity_type, warning_messages.join(", ")),
            &log_context
        );
    }
    
    Ok(())
}

/// Log a transaction operation
pub fn log_transaction_operation(
    ctx: &ServiceContext,
    operation: &str,
    attributes: &TransactionAttributes,
) -> Result<()> {
    let mut log_context = LogContext::from_service_context(ctx)
        .with_operation(operation)
        .with_context("transaction", "true")
        .with_context("propagation", format!("{:?}", attributes.propagation))
        .with_context("read_only", attributes.read_only.to_string());
        
    if let Some(isolation) = attributes.isolation_level {
        log_context = log_context.with_context("isolation_level", format!("{:?}", isolation));
    }
    
    if let Some(timeout) = attributes.timeout {
        log_context = log_context.with_context("timeout_seconds", timeout.to_string());
    }
    
    if let Some(name) = &attributes.name {
        log_context = log_context.with_context("transaction_name", name);
    }
    
    logging::debug_with_context(
        &format!("Transaction operation: {}", operation),
        &log_context
    );
    
    Ok(())
}

/// Extension trait for ServiceContext to add logging methods
pub trait ServiceContextLoggingExt {
    /// Log a message at the specified level
    fn log(&self, level: LogLevel, message: &str) -> Result<()>;
    
    /// Log a message at trace level
    fn trace(&self, message: &str) -> Result<()>;
    
    /// Log a message at debug level
    fn debug(&self, message: &str) -> Result<()>;
    
    /// Log a message at info level
    fn info(&self, message: &str) -> Result<()>;
    
    /// Log a message at warn level
    fn warn(&self, message: &str) -> Result<()>;
    
    /// Log a message at error level
    fn error(&self, message: &str) -> Result<()>;
    
    /// Create an operation logger for this context
    fn operation_logger(&self, operation: &str) -> OperationLogger;
}

impl ServiceContextLoggingExt for ServiceContext {
    fn log(&self, level: LogLevel, message: &str) -> Result<()> {
        let context = LogContext::from_service_context(self);
        logging::log_with_context(level, message, &context);
        Ok(())
    }
    
    fn trace(&self, message: &str) -> Result<()> {
        self.log(LogLevel::Trace, message)
    }
    
    fn debug(&self, message: &str) -> Result<()> {
        self.log(LogLevel::Debug, message)
    }
    
    fn info(&self, message: &str) -> Result<()> {
        self.log(LogLevel::Info, message)
    }
    
    fn warn(&self, message: &str) -> Result<()> {
        self.log(LogLevel::Warn, message)
    }
    
    fn error(&self, message: &str) -> Result<()> {
        self.log(LogLevel::Error, message)
    }
    
    fn operation_logger(&self, operation: &str) -> OperationLogger {
        OperationLogger::from_service_context(operation, self)
    }
}