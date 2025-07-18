use std::collections::HashMap;
use std::time::{Duration, Instant};

use kameo::prelude::*;
use tracing::{debug, error, info, warn, Level};
use uuid::Uuid;

use crate::actors::metrics::{MetricType, MetricValue};
use crate::logging::{self, LogContext, LogLevel, OperationLogger};
use crate::logging::correlation;

/// Actor logging context with additional actor-specific information
#[derive(Debug, Clone)]
pub struct ActorLogContext {
    /// Base log context
    pub base: LogContext,
    /// Actor ID
    pub actor_id: ActorID,
    /// Actor type name
    pub actor_type: String,
    /// Message type being processed (if applicable)
    pub message_type: Option<String>,
    /// Message ID (if applicable)
    pub message_id: Option<Uuid>,
    /// Sender actor ID (if applicable)
    pub sender_id: Option<ActorID>,
    /// Additional actor-specific context
    pub actor_context: HashMap<String, String>,
}

impl ActorLogContext {
    /// Create a new actor log context
    pub fn new(actor_id: ActorID, actor_type: impl Into<String>) -> Self {
        let mut base = LogContext::new();
        
        // Set entity type and ID in the base context
        base = base.with_entity_type("Actor").with_entity_id(actor_id.to_string());
        
        // Create a correlation ID based on the actor ID if none exists
        if base.correlation_id.is_none() {
            let correlation_id = format!("actor-{}", actor_id);
            base = base.with_correlation_id(correlation_id);
        }
        
        Self {
            base,
            actor_id,
            actor_type: actor_type.into(),
            message_type: None,
            message_id: None,
            sender_id: None,
            actor_context: HashMap::new(),
        }
    }
    
    /// Set the message type
    pub fn with_message_type(mut self, message_type: impl Into<String>) -> Self {
        self.message_type = Some(message_type.into());
        self
    }
    
    /// Set the message ID
    pub fn with_message_id(mut self, message_id: Uuid) -> Self {
        self.message_id = Some(message_id);
        self
    }
    
    /// Set the sender actor ID
    pub fn with_sender_id(mut self, sender_id: ActorID) -> Self {
        self.sender_id = Some(sender_id);
        self
    }
    
    /// Add actor-specific context
    pub fn with_actor_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.actor_context.insert(key.into(), value.into());
        self
    }
    
    /// Convert to a base log context with actor information
    pub fn to_log_context(&self) -> LogContext {
        let mut context = self.base.clone();
        
        // Add actor information to the context
        context = context.with_context("actor_type", &self.actor_type);
        
        // Add message information if available
        if let Some(message_type) = &self.message_type {
            context = context.with_context("message_type", message_type);
        }
        
        if let Some(message_id) = &self.message_id {
            context = context.with_context("message_id", &message_id.to_string());
        }
        
        if let Some(sender_id) = &self.sender_id {
            context = context.with_context("sender_id", &sender_id.to_string());
        }
        
        // Add all actor-specific context
        for (key, value) in &self.actor_context {
            context = context.with_context(key, value);
        }
        
        context
    }
}

/// Log a message with actor context
pub fn log_with_actor_context(level: LogLevel, message: &str, context: &ActorLogContext) {
    let log_context = context.to_log_context();
    logging::log_with_context(level, message, &log_context);
}

/// Log a message at trace level with actor context
pub fn trace_with_actor_context(message: &str, context: &ActorLogContext) {
    log_with_actor_context(LogLevel::Trace, message, context);
}

/// Log a message at debug level with actor context
pub fn debug_with_actor_context(message: &str, context: &ActorLogContext) {
    log_with_actor_context(LogLevel::Debug, message, context);
}

/// Log a message at info level with actor context
pub fn info_with_actor_context(message: &str, context: &ActorLogContext) {
    log_with_actor_context(LogLevel::Info, message, context);
}

/// Log a message at warn level with actor context
pub fn warn_with_actor_context(message: &str, context: &ActorLogContext) {
    log_with_actor_context(LogLevel::Warn, message, context);
}

/// Log a message at error level with actor context
pub fn error_with_actor_context(message: &str, context: &ActorLogContext) {
    log_with_actor_context(LogLevel::Error, message, context);
}

/// Actor operation logger for timing and logging actor operations
pub struct ActorOperationLogger {
    /// Name of the operation
    pub name: String,
    /// Start time of the operation
    pub start_time: Instant,
    /// Actor log context
    pub context: ActorLogContext,
    /// Whether to log at start and end
    pub log_start_end: bool,
    /// Whether to record metrics
    pub record_metrics: bool,
}

impl ActorOperationLogger {
    /// Create a new actor operation logger
    pub fn new(name: impl Into<String>, context: ActorLogContext) -> Self {
        let name = name.into();
        let logger = Self {
            name: name.clone(),
            start_time: Instant::now(),
            context,
            log_start_end: true,
            record_metrics: true,
        };
        
        logger
    }
    
    /// Set whether to log at start and end
    pub fn with_log_start_end(mut self, log_start_end: bool) -> Self {
        self.log_start_end = log_start_end;
        self
    }
    
    /// Set whether to record metrics
    pub fn with_record_metrics(mut self, record_metrics: bool) -> Self {
        self.record_metrics = record_metrics;
        self
    }
    
    /// Start the operation and log it
    pub fn start(&self) {
        if self.log_start_end {
            info_with_actor_context(
                &format!("Starting actor operation: {}", self.name),
                &self.context
            );
        }
    }
    
    /// End the operation, log it, and optionally record metrics
    pub fn end(&self) {
        let duration = self.start_time.elapsed();
        
        if self.log_start_end {
            let context = self.context.clone();
            info_with_actor_context(
                &format!(
                    "Completed actor operation: {} in {:?}",
                    self.name,
                    duration
                ),
                &context
            );
        }
        
        // Record metrics if enabled
        if self.record_metrics {
            // This would typically call into the metrics system
            // For now, we'll just log the metric
            debug_with_actor_context(
                &format!(
                    "Recorded metric for actor operation: {} - {:?}",
                    self.name,
                    duration
                ),
                &self.context
            );
        }
    }
    
    /// Log a message during the operation
    pub fn log(&self, level: LogLevel, message: &str) {
        log_with_actor_context(level, message, &self.context);
    }
    
    /// Log a message at trace level
    pub fn trace(&self, message: &str) {
        self.log(LogLevel::Trace, message);
    }
    
    /// Log a message at debug level
    pub fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
    
    /// Log a message at info level
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }
    
    /// Log a message at warn level
    pub fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }
    
    /// Log a message at error level
    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

impl Drop for ActorOperationLogger {
    fn drop(&mut self) {
        self.end();
    }
}

/// Macro to create an actor log context
#[macro_export]
macro_rules! actor_log_context {
    ($actor_id:expr, $actor_type:expr, $($context:tt)*) => {{
        let context = $crate::dev_tools::actor_debug::logging::ActorLogContext::new($actor_id, $actor_type)
            $($context)*;
        context
    }};
}

/// Macro to log a message with actor context
#[macro_export]
macro_rules! actor_log_message {
    ($level:expr, $message:expr, $actor_id:expr, $actor_type:expr, $($context:tt)*) => {{
        let context = $crate::actor_log_context!($actor_id, $actor_type, $($context)*);
        $crate::dev_tools::actor_debug::logging::log_with_actor_context($level, $message, &context);
    }};
}

/// Macro to log an actor operation
#[macro_export]
macro_rules! log_actor_operation {
    ($name:expr, $actor_id:expr, $actor_type:expr, $($context:tt)* => $body:block) => {{
        let context = $crate::actor_log_context!($actor_id, $actor_type, $($context)*);
        let logger = $crate::dev_tools::actor_debug::logging::ActorOperationLogger::new($name, context);
        logger.start();
        let result = (|| $body)();
        logger.end();
        result
    }};
}