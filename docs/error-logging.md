# Structured Error Logging with Correlation IDs

This document provides an overview of the structured error logging system in the Evo Design application, with a focus on correlation IDs for tracing related errors.

## Overview

The error logging system provides a way to log errors with additional context, including correlation IDs that can be used to link related errors together. This makes it easier to trace the flow of execution through the system and to understand the relationships between different errors.

## Correlation IDs

Correlation IDs are unique identifiers that are used to link related log entries and errors together. They are automatically generated and propagated through the system, making it possible to trace the flow of execution across different parts of the application.

### Key Features

- **Automatic Correlation ID Generation**: Correlation IDs are automatically generated when needed.
- **Thread-Local Storage**: Correlation IDs are stored in thread-local storage, making them accessible from any part of the code.
- **Hierarchical IDs**: Child correlation IDs can be created from parent IDs, allowing for hierarchical tracing.
- **Error Enrichment**: Errors are automatically enriched with correlation IDs.
- **Structured Logging**: Correlation IDs are included in structured log entries.

## Using Correlation IDs

### Basic Usage

The simplest way to use correlation IDs is to let the system handle them automatically. When you create an error using the error enrichment system, it will automatically include the current correlation ID:

```rust
use crate::error::enrichment::log_error;

// This will automatically include the current correlation ID
let error = log_error(AppError::InternalError("Something went wrong".to_string()));
```

### Setting a Correlation ID

You can explicitly set a correlation ID for an error:

```rust
use crate::error::enrichment::with_correlation_id;

// Set a specific correlation ID
let error = with_correlation_id(
    AppError::InternalError("Something went wrong".to_string()),
    "my-correlation-id"
);
```

### Generating a New Correlation ID

You can generate a new correlation ID for an error:

```rust
use crate::error::enrichment::with_new_correlation_id;

// Generate a new correlation ID
let error = with_new_correlation_id(
    AppError::InternalError("Something went wrong".to_string())
);
```

### Creating a Child Correlation ID

You can create a child correlation ID from the current one:

```rust
use crate::error::enrichment::with_child_correlation_id;

// Create a child correlation ID
let error = with_child_correlation_id(
    AppError::InternalError("Something went wrong".to_string())
);
```

### Executing Functions with Correlation IDs

You can execute a function with a correlation ID, which will be used for any errors that occur within the function:

```rust
use crate::error::enrichment::with_error_correlation;

// Execute a function with error correlation
let result = with_error_correlation(|| {
    // This function will have a correlation ID
    // Any errors will be enriched with the correlation ID
    fallible_operation()
});
```

You can also execute a function with a new correlation ID:

```rust
use crate::error::enrichment::with_new_error_correlation;

// Execute a function with a new error correlation ID
let result = with_new_error_correlation(|| {
    // This function will have a new correlation ID
    // Any errors will be enriched with the new correlation ID
    fallible_operation()
});
```

Or with a child correlation ID:

```rust
use crate::error::enrichment::with_child_error_correlation;

// Execute a function with a child error correlation ID
let result = with_child_error_correlation(|| {
    // This function will have a child correlation ID
    // Any errors will be enriched with the child correlation ID
    fallible_operation()
});
```

## Advanced Usage

### Manual Correlation ID Management

If you need more control over correlation IDs, you can use the correlation module directly:

```rust
use crate::logging::correlation;

// Generate a new correlation ID
let id = correlation::generate_correlation_id();

// Set the current correlation ID
correlation::set_correlation_id(id);

// Get the current correlation ID
let current_id = correlation::get_correlation_id();

// Clear the current correlation ID
correlation::clear_correlation_id();
```

### Executing Code with a Correlation ID

You can execute a block of code with a specific correlation ID:

```rust
use crate::logging::correlation;

// Execute a function with a specific correlation ID
let result = correlation::with_correlation_id("my-correlation-id", || {
    // This code will have the specified correlation ID
    // ...
    "result"
});
```

Or with a new correlation ID:

```rust
use crate::logging::correlation;

// Execute a function with a new correlation ID
let result = correlation::with_new_correlation_id(|| {
    // This code will have a new correlation ID
    // ...
    "result"
});
```

Or with a child correlation ID:

```rust
use crate::logging::correlation;

// Execute a function with a child correlation ID
let result = correlation::with_child_correlation_id(|| {
    // This code will have a child correlation ID
    // ...
    "result"
});
```

## Integration with Logging

Correlation IDs are automatically included in log entries when using the structured logging system:

```rust
use crate::logging::{LogContext, info_with_context};

// Create a log context
let context = LogContext::new();

// Log a message with the context
info_with_context("Something happened", &context);
```

The log entry will include the current correlation ID, if available.

## Best Practices

1. **Use High-Level Functions**: Prefer using the high-level functions like `with_error_correlation` instead of manually managing correlation IDs.

2. **Create Child IDs for Subtasks**: When a task consists of multiple subtasks, use child correlation IDs to link them together while maintaining their hierarchy.

3. **Include Correlation IDs in API Responses**: When returning errors to clients, include the correlation ID in the response to help with debugging.

4. **Log at Appropriate Levels**: Use the appropriate log level for different types of events to avoid cluttering the logs.

5. **Add Meaningful Context**: In addition to correlation IDs, add other relevant context to errors and logs to make them more informative.

## Conclusion

The structured error logging system with correlation IDs provides a powerful way to trace the flow of execution through the system and to understand the relationships between different errors. By using correlation IDs consistently throughout the application, you can make debugging and troubleshooting much easier.