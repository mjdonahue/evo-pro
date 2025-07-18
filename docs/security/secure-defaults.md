# Secure Defaults for Application Components

This document describes the secure defaults implemented for various components of the Evo Design application. These defaults ensure that security best practices are applied consistently throughout the codebase.

## Overview

The secure defaults framework provides a centralized mechanism for defining and applying secure configurations to various components of the application. This ensures that security is built-in by default, rather than being an afterthought.

The framework includes:

- A `SecureDefaultsService` for applying secure defaults to all components
- Specialized structs for different types of secure defaults
- Integration with the application initialization process

## Secure Default Categories

### Database Secure Defaults

Database secure defaults ensure that SQLite databases are configured with secure settings:

- **WAL Mode**: Uses Write-Ahead Logging for better concurrency and crash recovery
- **Synchronous Mode**: Sets synchronous mode to NORMAL for a balance of safety and performance
- **Foreign Key Constraints**: Enforces referential integrity
- **Busy Timeout**: Sets a reasonable timeout for busy connections
- **Secure Connection String**: Provides a connection string with all secure settings

Example usage:

```rust
// Get a secure connection string
let connection_string = DatabaseSecureDefaults::get_connection_string("path/to/db.sqlite");

// Get secure SQLite pragmas
let pragmas = DatabaseSecureDefaults::get_pragmas();
```

### Network Secure Defaults

Network secure defaults ensure that network communications are secure:

- **Timeouts**: Sets reasonable timeouts to prevent hanging connections
- **Keepalive Intervals**: Configures keepalive intervals to maintain connections
- **TLS Configuration**: Enforces TLS for secure communications

Example usage:

```rust
// Get secure timeout duration
let timeout = NetworkSecureDefaults::get_timeout();

// Check if TLS should be used
let use_tls = NetworkSecureDefaults::get_tls_config();
```

### Authentication Secure Defaults

Authentication secure defaults ensure that authentication mechanisms are secure:

- **Password Hashing Iterations**: Sets a high number of iterations for password hashing
- **Token Expiration**: Sets reasonable expiration times for authentication tokens
- **Key Length**: Ensures cryptographic keys are of sufficient length

Example usage:

```rust
// Get secure password hashing iterations
let iterations = AuthenticationSecureDefaults::get_password_hashing_iterations();

// Get secure token expiration
let expiration = AuthenticationSecureDefaults::get_token_expiration();
```

### Input Validation Secure Defaults

Input validation secure defaults ensure that user input is properly validated:

- **Maximum Input Length**: Sets reasonable limits on input length
- **Username Validation**: Provides a secure regex for validating usernames
- **Email Validation**: Provides a secure regex for validating email addresses

Example usage:

```rust
// Get secure maximum input length
let max_length = InputValidationSecureDefaults::get_max_input_length();

// Get secure username validation regex
let username_regex = InputValidationSecureDefaults::get_username_regex();
```

### File Operation Secure Defaults

File operation secure defaults ensure that file operations are secure:

- **Temporary Directory**: Provides a secure application-specific temporary directory
- **File Permissions**: Sets secure permissions for files (Unix only)
- **Directory Permissions**: Sets secure permissions for directories (Unix only)

Example usage:

```rust
// Get secure temporary directory
let temp_dir = FileOperationSecureDefaults::get_temp_dir();

// Get secure file permissions (Unix only)
#[cfg(unix)]
let file_permissions = FileOperationSecureDefaults::get_file_permissions();
```

## Application Integration

The secure defaults are applied during application initialization:

1. The database is initialized with secure connection parameters
2. The `SecurityService` is initialized, which includes the `SecureDefaultsService`
3. Secure defaults are applied to all components
4. Default threat models are initialized

This ensures that all components have secure defaults applied before they are used.

## Configuration

The secure defaults framework can be configured through the `SecureDefaultsConfig` struct:

- **enabled**: Whether secure defaults are enabled
- **enforce**: Whether to enforce secure defaults (fail if they can't be applied)
- **log_application**: Whether to log when secure defaults are applied

Example configuration:

```rust
let config = SecureDefaultsConfig {
    enabled: true,
    enforce: true,
    log_application: true,
};

let service = SecureDefaultsService::new(db, Some(config)).await?;
```

## Testing

The secure defaults framework includes comprehensive tests to ensure that secure defaults are applied correctly:

- Tests for the `SecureDefaultsService`
- Tests for each type of secure default
- Integration tests with the `SecurityService`

## Best Practices

When developing new components or modifying existing ones:

1. Use the secure defaults provided by the framework
2. Don't override secure defaults without a good reason
3. If you need to override a secure default, document why
4. Add new secure defaults to the framework if needed