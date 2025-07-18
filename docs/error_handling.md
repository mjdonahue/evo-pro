# Error Handling Framework

This document provides guidelines and examples for using the error handling framework in the evo-pro project.

## Overview

The error handling framework is designed to provide:

1. A standardized error taxonomy
2. Contextual error enrichment
3. Structured logging for errors with correlation IDs
4. User-friendly error messages
5. Developer-friendly error information

## Error Types

The `AppError` enum in `src-tauri/src/error.rs` defines all the error types used in the application. These are organized into categories:

### System-level Errors

These errors are related to the underlying system and infrastructure:

- `BootstrapError` - Errors during application startup
- `TransportError` - Errors in network transport
- `RegistryError` - Errors in the actor registry
- `SqlxError` - Errors from the SQLx database library
- `SendError` - Errors when sending messages between actors
- `RemoteSendError` - Errors when sending messages to remote actors
- `Generic` - Generic errors that don't fit other categories
- `JsonError` - Errors in JSON parsing/serialization
- `ToolCallError` - Errors in tool calls
- `IoError` - Input/output errors
- `UuidParseError` - Errors parsing UUIDs
- `CompletionError` - Errors in chat completions

### Application-level Errors

These errors are related to the application's business logic:

- `NotFoundError` - Entity not found
- `DeserializationError` - Error deserializing data
- `DatabaseError` - Database operation errors
- `QueryError` - Database query errors
- `ValidationError` - Data validation errors
- `InternalError` - Internal application errors
- `TransactionError` - Database transaction errors
- `AuthorizationError` - Permission/authorization errors
- `AuthenticationError` - Authentication errors
- `OperationNotSupported` - Operation not supported
- `ResourceLimitExceeded` - Resource limits exceeded
- `ConfigurationError` - Configuration errors
- `ExternalServiceError` - Errors from external services

### Contextual Error

The `ContextualError` variant allows adding rich context to any error:

```rust
AppError::ContextualError {
    message: String,
    context: ErrorContext,
}
```

## Error Context

The `ErrorContext` struct provides rich context for errors:

- `error_id` - Unique identifier for the error
- `correlation_id` - ID for tracing related errors
- `request_id` - ID of the request that caused the error
- `user_id` - ID of the user affected by the error
- `workspace_id` - ID of the workspace where the error occurred
- `severity` - Error severity (Fatal, Critical, Error, Warning, Info)
- `category` - Error category (Authentication, Authorization, Validation, etc.)
- `source_location` - Source code location where the error occurred
- `operation` - Operation that caused the error
- `entity_type` - Type of entity involved in the error
- `entity_id` - ID of the entity involved in the error
- `additional_context` - Additional key-value pairs
- `timestamp` - When the error occurred
- `retriable` - Whether the error can be retried
- `user_action` - Suggested action for the user
- `developer_action` - Suggested action for the developer

## Creating Errors

### Simple Errors

For simple errors, use the factory methods on `AppError`:

```rust
// Create a not found error
let error = AppError::not_found("User", user_id);

// Create a validation error
let error = AppError::validation("Username must be at least 3 characters");

// Create an authorization error
let error = AppError::authorization("Missing required permission: admin");
```

### Contextual Errors

For errors with rich context, use the `contextual_error!` macro:

```rust
let error = contextual_error!(
    "Failed to process payment",
    .with_category(ErrorCategory::ExternalService)
    .with_severity(ErrorSeverity::Critical)
    .with_entity_type("Payment")
    .with_entity_id(payment_id.to_string())
    .with_context("amount", amount.to_string())
    .with_context("currency", "USD")
    .with_user_action("Please try again or use a different payment method")
    .with_developer_action("Check the payment gateway logs for details")
);
```

### Enriching Existing Errors

To add context to an existing error, use the `enrich_error!` macro:

```rust
let result = external_service.process_payment(amount).await;
if let Err(err) = result {
    return Err(enrich_error!(
        err,
        .with_entity_type("Payment")
        .with_entity_id(payment_id.to_string())
        .with_context("amount", amount.to_string())
    ));
}
```

## Logging Errors

To log an error with all its context, use the `log_error!` macro:

```rust
// Log an error and return it
return Err(log_error!(error));

// Log an error with additional context and return it
return Err(log_error!(
    error,
    .with_context("additional_info", "Some additional information")
));
```

## Error Handling in Middleware

The `ErrorHandlingMiddleware` automatically enriches errors with context from the service context and logs them. It also handles retriable errors (though retry logic is not yet implemented).

## Best Practices

1. **Be Specific**: Use the most specific error type that applies to the situation.
2. **Add Context**: Always add relevant context to errors to aid debugging.
3. **User-Friendly Messages**: Include user-friendly messages and actions when appropriate.
4. **Log Errors**: Always log errors with their full context.
5. **Correlation IDs**: Use correlation IDs to track related errors across services.
6. **Categorize Errors**: Properly categorize errors to aid in filtering and analysis.
7. **Source Location**: Include source location information for debugging.

## Examples

### Service Method Example

```rust
pub async fn create_user(&self, ctx: &ServiceContext, input: CreateUserInput) -> Result<User> {
    // Validate input
    if input.username.len() < 3 {
        return Err(contextual_error!(
            "Username must be at least 3 characters",
            .with_category(ErrorCategory::Validation)
            .with_severity(ErrorSeverity::Warning)
            .with_entity_type("User")
            .with_context("username", &input.username)
            .with_user_action("Please choose a username with at least 3 characters")
        ));
    }

    // Check if username is already taken
    let existing_user = self.find_by_username(ctx, &input.username).await?;
    if existing_user.is_some() {
        return Err(contextual_error!(
            "Username is already taken",
            .with_category(ErrorCategory::Validation)
            .with_severity(ErrorSeverity::Warning)
            .with_entity_type("User")
            .with_context("username", &input.username)
            .with_user_action("Please choose a different username")
        ));
    }

    // Create the user
    let result = self.db.create_user(&input).await;
    if let Err(err) = result {
        return Err(enrich_error!(
            err,
            .with_entity_type("User")
            .with_context("username", &input.username)
            .with_developer_action("Check database logs for details")
        ));
    }

    Ok(result.unwrap())
}
```

### Error Handling in Controllers

```rust
#[tauri::command]
pub async fn create_user(
    state: tauri::State<'_, AppState>,
    input: CreateUserInput,
) -> Result<User, String> {
    let ctx = ServiceContext::new(&state);
    
    match state.user_service.create_user(&ctx, input).await {
        Ok(user) => Ok(user),
        Err(err) => {
            // Log the error with its full context
            log_error!(err);
            
            // Convert to a user-friendly message
            Err(err.to_string())
        }
    }
}
```