# Logging Framework

This document provides guidelines and examples for using the logging framework in the evo-pro project.

## Overview

The logging framework is designed to provide:

1. Structured logging with consistent fields
2. Correlation IDs for request tracing
3. Performance metrics logging
4. Context propagation for logging
5. Automatic logging of service operations

## Log Levels

The framework defines the following log levels:

- **Trace**: Very detailed information, typically only useful for debugging specific issues
- **Debug**: Detailed information that is useful for development and troubleshooting
- **Info**: General operational information about what's happening in the application
- **Warn**: Concerning but non-critical issues that might need attention
- **Error**: Errors that affect functionality and need to be addressed

## Logging Context

The `LogContext` struct provides rich context for logs:

- `log_id`: Unique identifier for the log entry
- `correlation_id`: ID for tracing related logs
- `request_id`: ID of the request that generated the log
- `user_id`: ID of the user associated with the log
- `workspace_id`: ID of the workspace where the log was generated
- `source_location`: Source code location where the log was generated
- `operation`: Operation that generated the log
- `entity_type`: Type of entity involved in the log
- `entity_id`: ID of the entity involved in the log
- `additional_context`: Additional key-value pairs
- `timestamp`: When the log was generated
- `duration`: Duration of the operation (if applicable)

## Logging in Services

### Using the ServiceContext Extension Methods

The `ServiceContextLoggingExt` trait adds logging methods to `ServiceContext`:

```rust
// Log at different levels
ctx.trace("Detailed debug information");
ctx.debug("Debug information");
ctx.info("General information");
ctx.warn("Warning message");
ctx.error("Error message");

// Create an operation logger
let logger = ctx.operation_logger("my_operation");
logger.start();
// ... perform operation ...
logger.info("Operation in progress");
// ... continue operation ...
logger.end();
```

### Using the Operation Logger

The `OperationLogger` provides automatic timing and logging for operations:

```rust
// Create an operation logger
let logger = OperationLogger::from_service_context("my_operation", ctx);
logger.start();

// Log during the operation
logger.info("Operation in progress");
logger.debug("Detailed information about the operation");

// The logger will automatically log completion when dropped
// Or you can explicitly call end()
logger.end();
```

### Using the log_operation Macro

The `log_operation` macro provides a simple way to time and log an operation:

```rust
let result = log_operation!("my_operation", ctx, {
    // Operation code here
    let value = perform_operation();
    value
});
```

### Logging Service Events

Use the `log_service_event` function to log service events:

```rust
let event = ServiceEvent {
    event_type: "user_created".to_string(),
    entity_type: "user".to_string(),
    entity_id: user_id,
    workspace_id: Some(workspace_id),
    data: serde_json::to_value(&user)?,
    metadata: None,
};

log_service_event(ctx, &event)?;
```

### Logging Validation Results

Use the `log_validation_result` function to log validation results:

```rust
let validation_result = validate_user(ctx, &user).await?;
log_validation_result(ctx, "User", &validation_result)?;
```

### Logging Transaction Operations

Use the `log_transaction_operation` function to log transaction operations:

```rust
let attributes = TransactionAttributes {
    isolation_level: Some(IsolationLevel::ReadCommitted),
    propagation: PropagationBehavior::Required,
    read_only: false,
    timeout: Some(30),
    name: Some("create_user".to_string()),
};

log_transaction_operation(ctx, "create_user", &attributes)?;
```

## Logging in Middleware

The framework includes middleware for automatic logging of service operations:

- `LoggingMiddleware`: Basic logging middleware for all operations
- `ServiceLoggingMiddleware`: Advanced logging middleware specifically for service operations
- `MetricsMiddleware`: Middleware for logging performance metrics

These are automatically included in the default middleware stack.

## Best Practices

1. **Be Consistent**: Use the provided logging utilities consistently throughout the codebase.
2. **Add Context**: Always add relevant context to logs to aid debugging and analysis.
3. **Use Appropriate Levels**: Use the appropriate log level for each message:
   - Trace: Very detailed debugging information
   - Debug: Development-time information
   - Info: General operational information
   - Warn: Concerning but non-critical issues
   - Error: Errors that affect functionality
4. **Log Important Events**: Log all important business events, state changes, and errors.
5. **Include Performance Metrics**: Include timing information for performance-sensitive operations.
6. **Use Correlation IDs**: Use correlation IDs to track related logs across services.
7. **Structured Logging**: Use structured logging with consistent fields for easier analysis.

## Examples

### Basic Logging

```rust
// Using the logging macros
info_log!("User logged in", 
    .with_user_id(user_id.to_string())
    .with_context("login_method", "password")
);

// Using the context methods
let context = log_context!(
    .with_user_id(user_id.to_string())
    .with_context("login_method", "password")
);
info_with_context("User logged in", &context);
```

### Operation Logging

```rust
// Using the operation logger
let logger = ctx.operation_logger("create_user");
logger.start();

// Perform the operation
let user = create_user_in_db(ctx, input).await?;

// Log success
logger.info(&format!("Created user with ID: {}", user.id));

// The logger will automatically log completion when dropped
```

### Error Logging

```rust
// Errors are automatically logged by the ErrorHandlingMiddleware
// But you can also log them manually
match operation() {
    Ok(result) => {
        ctx.info(&format!("Operation succeeded with result: {:?}", result));
        Ok(result)
    }
    Err(err) => {
        ctx.error(&format!("Operation failed: {}", err));
        Err(err)
    }
}
```

## Configuration

The logging framework is configured in `src-tauri/src/lib.rs` using the `tracing_subscriber` crate. The default configuration includes:

- An `EnvFilter` that reads log levels from environment variables
- A formatting layer that includes line numbers and file names

You can configure the log level using the `RUST_LOG` environment variable:

```bash
# Set log level for the entire application
RUST_LOG=info

# Set log level for specific modules
RUST_LOG=app::services=debug,app::actors=trace
```

## Viewing Logs

Logs are output to the console during development. In production, logs are also written to log files in the application data directory.

You can view logs using:

1. The console output during development
2. The Tauri developer tools
3. Log files in the application data directory