# Schema Evolution Guide

## Introduction

This document provides guidelines and best practices for evolving the database schema in the evo-pro project. It covers how to create, test, and apply migrations using our robust migration framework.

## Table of Contents

1. [Migration Framework Overview](#migration-framework-overview)
2. [Creating Migrations](#creating-migrations)
3. [Testing Migrations](#testing-migrations)
4. [Applying Migrations](#applying-migrations)
5. [Rollback Strategies](#rollback-strategies)
6. [Data Transformation](#data-transformation)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

## Migration Framework Overview

The evo-pro project uses a custom migration framework built on top of SQLx that provides:

- **Versioning**: Migrations are versioned and applied in order
- **Validation**: Migrations are validated before being applied
- **Rollback**: Migrations can be rolled back if needed
- **Data Transformation**: Migrations can transform data during schema changes
- **Testing**: A comprehensive testing framework for migrations

The framework is implemented in the following modules:

- `src-tauri/src/storage/migration.rs`: Core migration framework
- `src-tauri/src/storage/migration_test.rs`: Migration testing framework

## Creating Migrations

### Migration File Structure

Migrations are stored in the `src-tauri/migrations` directory. Each migration consists of one or two SQL files:

- `NNNN_name.sql` or `NNNN_name_up.sql`: The migration to apply
- `NNNN_name_down.sql` (optional): The migration to roll back

Where:
- `NNNN` is a 4-digit version number (e.g., `0001`)
- `name` is a descriptive name for the migration (e.g., `create_users_table`)

### Creating a New Migration

To create a new migration:

1. Determine the next version number by looking at the existing migrations
2. Create a new SQL file with the appropriate name
3. Write the SQL statements for the migration

Example:

```sql
-- 0003_add_status_to_users.sql
ALTER TABLE users ADD COLUMN status INTEGER NOT NULL DEFAULT 0;
```

```sql
-- 0003_add_status_to_users_down.sql
-- SQLite doesn't support DROP COLUMN, so we need to recreate the table
CREATE TABLE users_temp AS SELECT id, name, email, created_at FROM users;
DROP TABLE users;
ALTER TABLE users_temp RENAME TO users;
```

### Migration Types

The framework supports different types of migrations:

1. **Schema Migrations**: Changes to the database schema (tables, columns, indexes, etc.)
2. **Data Migrations**: Changes to the data in the database
3. **Combined Migrations**: Both schema and data changes

### Custom Migrations

For complex migrations that can't be expressed in SQL alone, you can create a custom migration by implementing the `Migration` trait:

```rust
use crate::storage::migration::{Migration, MigrationError};
use sqlx::Transaction;

struct CustomMigration;

#[async_trait]
impl Migration for CustomMigration {
    fn version(&self) -> &str {
        "0004"
    }
    
    fn name(&self) -> &str {
        "complex_data_transformation"
    }
    
    fn description(&self) -> Option<&str> {
        Some("A complex data transformation that can't be expressed in SQL alone")
    }
    
    async fn up(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Implement the migration
        // ...
        Ok(())
    }
    
    async fn down(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Implement the rollback
        // ...
        Ok(())
    }
    
    async fn validate(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Validate the migration
        // ...
        Ok(())
    }
    
    async fn transform_data(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Transform data during the migration
        // ...
        Ok(())
    }
}
```

## Testing Migrations

The migration testing framework provides utilities for testing migrations before applying them to the production database.

### Creating a Test

To create a test for a migration:

```rust
use crate::storage::migration_test::{MigrationTestContext, test_utils};

#[tokio::test]
async fn test_add_status_to_users() -> Result<()> {
    // Create a test context
    let mut context = MigrationTestContext::new().await?;
    
    // Add a migration to create the users table
    context.add_migration(test_utils::create_table_migration(
        "0001",
        "create_users_table",
        "users",
        &[
            ("id", "INTEGER PRIMARY KEY"),
            ("name", "TEXT NOT NULL"),
            ("email", "TEXT NOT NULL"),
            ("created_at", "TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP"),
        ],
    ));
    
    // Add a migration to add the status column
    context.add_sql_migration(
        "0002",
        "add_status_to_users",
        "ALTER TABLE users ADD COLUMN status INTEGER NOT NULL DEFAULT 0;",
        Some("-- SQLite doesn't support DROP COLUMN, so we need to recreate the table
CREATE TABLE users_temp AS SELECT id, name, email, created_at FROM users;
DROP TABLE users;
ALTER TABLE users_temp RENAME TO users;".to_string()),
    );
    
    // Run the migrations
    context.run_migrations().await?;
    
    // Verify that the migrations were applied correctly
    context.execute_query("INSERT INTO users (name, email, status) VALUES ('Test User', 'test@example.com', 1);").await?;
    
    let users: Vec<(i64, String, String, i64)> = context.query("SELECT id, name, email, status FROM users").await?;
    
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].1, "Test User");
    assert_eq!(users[0].2, "test@example.com");
    assert_eq!(users[0].3, 1);
    
    // Test rollback
    context.rollback_migration("0002").await?;
    
    // Verify that the status column was removed
    let result = context.query::<(i64, String, String, i64)>("SELECT id, name, email, status FROM users").await;
    assert!(result.is_err());
    
    Ok(())
}
```

### Test Utilities

The testing framework provides utilities for common migration operations:

- `create_table_migration`: Create a migration that creates a table
- `add_column_migration`: Create a migration that adds a column to a table
- `insert_data_migration`: Create a migration that inserts data into a table

## Applying Migrations

Migrations are applied automatically when the application starts. The `DatabaseManager::run_migrations` method:

1. Determines which migrations directory to use (seeding or migrations)
2. Creates and initializes a `MigrationManager`
3. Checks for version conflicts
4. Validates all migrations
5. Runs pending migrations

You can also apply migrations manually using the `MigrationManager`:

```rust
use crate::storage::migration::{MigrationManager, MigrationOptions};

async fn apply_migrations(pool: Pool<Sqlite>) -> Result<()> {
    // Create migration options
    let options = MigrationOptions {
        create_migrations_table: true,
        validate_migrations: true,
        allow_missing_down: true,
        transform_data: true,
        timeout_seconds: 60,
    };
    
    // Create and initialize the migration manager
    let mut manager = MigrationManager::new(pool, "./migrations", Some(options));
    manager.initialize().await?;
    
    // Run pending migrations
    let results = manager.run_migrations().await?;
    
    for record in results {
        println!("Applied migration {}: {} in {}ms", 
            record.version, 
            record.name,
            record.execution_time_ms.unwrap_or(0)
        );
    }
    
    Ok(())
}
```

## Rollback Strategies

The migration framework supports several rollback strategies:

### Rolling Back a Single Migration

```rust
manager.rollback_migration("0003").await?;
```

### Rolling Back All Migrations

```rust
manager.rollback_all().await?;
```

### Rolling Back to a Specific Version

```rust
manager.rollback_to("0002").await?;
```

## Data Transformation

For migrations that require data transformation, you can use the `transform_data` method of the `Migration` trait:

```rust
async fn transform_data(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
    // Transform data during the migration
    sqlx::query("UPDATE users SET status = 1 WHERE email LIKE '%admin%'")
        .execute(tx)
        .await
        .map_err(|e| MigrationError::TransformationFailed(format!("Failed to update user status: {}", e)))?;
    
    Ok(())
}
```

## Best Practices

### General Guidelines

1. **Keep migrations small and focused**: Each migration should do one thing and do it well
2. **Always provide a down migration**: This allows for rollbacks if needed
3. **Test migrations thoroughly**: Use the testing framework to verify that migrations work as expected
4. **Use transactions**: All migrations are run in a transaction to ensure atomicity
5. **Document complex migrations**: Add comments to explain complex migrations
6. **Version control migrations**: Migrations should be committed to version control
7. **Never modify an existing migration**: Create a new migration instead

### Naming Conventions

1. Use a 4-digit version number: `0001`, `0002`, etc.
2. Use a descriptive name: `create_users_table`, `add_status_to_users`, etc.
3. Use snake_case for names

### SQL Guidelines

1. Use uppercase for SQL keywords: `CREATE TABLE`, `ALTER TABLE`, etc.
2. Use lowercase for table and column names
3. Use singular names for tables: `user` instead of `users`
4. Add comments for complex SQL statements

## Troubleshooting

### Common Issues

#### Migration Version Conflict

If you see an error like:

```
Migration version conflict: Checksum mismatch for migration 0001: applied=abc123, current=def456
```

This means that the migration file has been modified after it was applied. You should never modify an existing migration. Instead, create a new migration to make the desired changes.

#### Migration Validation Failed

If you see an error like:

```
Migration validation failed: Invalid up SQL: syntax error at line 3
```

This means that there's a syntax error in your migration. Check the SQL syntax and fix any errors.

#### Migration Execution Failed

If you see an error like:

```
Migration execution failed: Failed to execute up migration: table users already exists
```

This means that the migration is trying to create a table that already exists. Check if the migration has already been applied or if there's a conflict with another migration.

### Getting Help

If you encounter issues with migrations, you can:

1. Check the application logs for detailed error messages
2. Use the `validate_all` method to validate all migrations
3. Use the testing framework to test migrations in isolation
4. Consult the SQLite documentation for SQL syntax and features