//! Migration testing framework
//!
//! This module provides utilities for testing database migrations.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{Pool, Sqlite, SqlitePool, Transaction};
use tempfile::tempdir;
use tokio::fs;

use crate::error::Result;
use crate::storage::migration::{Migration, MigrationError, MigrationManager, MigrationOptions, SqlMigration};

/// Test migration that can be used for testing
pub struct TestMigration {
    /// Migration version
    version: String,
    /// Migration name
    name: String,
    /// Migration description
    description: Option<String>,
    /// Migration up function
    up_fn: Box<dyn Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync>,
    /// Migration down function
    down_fn: Box<dyn Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync>,
    /// Migration validate function
    validate_fn: Box<dyn Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync>,
    /// Migration transform data function
    transform_data_fn: Box<dyn Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync>,
}

impl TestMigration {
    /// Create a new test migration
    pub fn new(
        version: impl Into<String>,
        name: impl Into<String>,
        description: Option<String>,
        up_fn: impl Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync + 'static,
        down_fn: impl Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync + 'static,
        validate_fn: Option<impl Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync + 'static>,
        transform_data_fn: Option<impl Fn(&mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> + Send + Sync + 'static>,
    ) -> Self {
        Self {
            version: version.into(),
            name: name.into(),
            description,
            up_fn: Box::new(up_fn),
            down_fn: Box::new(down_fn),
            validate_fn: Box::new(validate_fn.unwrap_or(|_| Ok(()))),
            transform_data_fn: Box::new(transform_data_fn.unwrap_or(|_| Ok(()))),
        }
    }
}

#[async_trait]
impl Migration for TestMigration {
    fn version(&self) -> &str {
        &self.version
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    
    fn checksum(&self) -> Option<&str> {
        None
    }
    
    fn metadata(&self) -> Option<&serde_json::Value> {
        None
    }
    
    async fn up(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        (self.up_fn)(tx)
    }
    
    async fn down(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        (self.down_fn)(tx)
    }
    
    async fn validate(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        (self.validate_fn)(tx)
    }
    
    async fn transform_data(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        (self.transform_data_fn)(tx)
    }
}

/// Migration test context
pub struct MigrationTestContext {
    /// Migration manager
    pub manager: MigrationManager,
    /// Database pool
    pub pool: Pool<Sqlite>,
    /// Temporary directory for test migrations
    _temp_dir: tempfile::TempDir,
}

impl MigrationTestContext {
    /// Create a new migration test context
    pub async fn new() -> Result<Self> {
        // Create a temporary directory for test migrations
        let temp_dir = tempdir().map_err(|e| MigrationError::FileNotFound(format!("Failed to create temporary directory: {}", e)))?;
        let migrations_dir = temp_dir.path().join("migrations");
        
        // Create the migrations directory
        fs::create_dir_all(&migrations_dir)
            .await
            .map_err(|e| MigrationError::FileNotFound(format!("Failed to create migrations directory: {}", e)))?;
        
        // Create an in-memory database
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to create in-memory database: {}", e)))?;
        
        // Create migration options
        let options = MigrationOptions {
            create_migrations_table: true,
            validate_migrations: true,
            allow_missing_down: true,
            transform_data: true,
            timeout_seconds: 60,
        };
        
        // Create and initialize the migration manager
        let mut manager = MigrationManager::new(pool.clone(), migrations_dir.to_string_lossy().to_string(), Some(options));
        manager.initialize().await?;
        
        Ok(Self {
            manager,
            pool,
            _temp_dir: temp_dir,
        })
    }
    
    /// Add a test migration
    pub fn add_migration(&mut self, migration: impl Migration + 'static) {
        self.manager.register_migration(Arc::new(migration));
    }
    
    /// Add a SQL migration
    pub fn add_sql_migration(
        &mut self,
        version: impl Into<String>,
        name: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: Option<String>,
    ) {
        let migration = SqlMigration::new(
            version,
            name,
            up_sql,
            down_sql,
            None,
            None,
        );
        
        self.manager.register_migration(Arc::new(migration));
    }
    
    /// Run migrations
    pub async fn run_migrations(&self) -> Result<()> {
        let results = self.manager.run_migrations().await?;
        
        if !results.is_empty() {
            println!("Applied {} migrations", results.len());
            
            for record in results {
                println!("Applied migration {}: {} in {}ms", 
                    record.version, 
                    record.name,
                    record.execution_time_ms.unwrap_or(0)
                );
            }
        } else {
            println!("No migrations to apply");
        }
        
        Ok(())
    }
    
    /// Rollback a migration
    pub async fn rollback_migration(&self, version: &str) -> Result<()> {
        self.manager.rollback_migration(version).await?;
        println!("Rolled back migration {}", version);
        
        Ok(())
    }
    
    /// Rollback all migrations
    pub async fn rollback_all(&self) -> Result<()> {
        let results = self.manager.rollback_all().await?;
        
        if !results.is_empty() {
            println!("Rolled back {} migrations", results.len());
            
            for record in results {
                println!("Rolled back migration {}: {}", 
                    record.version, 
                    record.name
                );
            }
        } else {
            println!("No migrations to roll back");
        }
        
        Ok(())
    }
    
    /// Validate migrations
    pub async fn validate_migrations(&self) -> Result<()> {
        self.manager.validate_all().await?;
        println!("All migrations validated successfully");
        
        Ok(())
    }
    
    /// Execute a query
    pub async fn execute_query(&self, query: &str) -> Result<()> {
        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to execute query: {}", e)))?;
        
        Ok(())
    }
    
    /// Execute a query and return the results
    pub async fn query<T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>>(&self, query: &str) -> Result<Vec<T>> {
        let results = sqlx::query_as::<_, T>(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to execute query: {}", e)))?;
        
        Ok(results)
    }
}

/// Migration test utilities
pub mod test_utils {
    use super::*;
    
    /// Create a test migration that creates a table
    pub fn create_table_migration(
        version: &str,
        name: &str,
        table_name: &str,
        columns: &[(&str, &str)],
    ) -> TestMigration {
        let table_name = table_name.to_string();
        let columns = columns.iter()
            .map(|(name, type_)| (name.to_string(), type_.to_string()))
            .collect::<Vec<_>>();
        
        // Create the up function
        let up_fn = move |tx: &mut Transaction<'_, Sqlite>| {
            let table_name = table_name.clone();
            let columns = columns.clone();
            
            let columns_sql = columns.iter()
                .map(|(name, type_)| format!("{} {}", name, type_))
                .collect::<Vec<_>>()
                .join(", ");
            
            let sql = format!("CREATE TABLE {} ({})", table_name, columns_sql);
            
            Box::pin(async move {
                sqlx::query(&sql)
                    .execute(tx)
                    .await
                    .map_err(|e| MigrationError::ExecutionFailed(format!("Failed to create table: {}", e)))?;
                
                Ok(())
            })
        };
        
        // Create the down function
        let down_fn = move |tx: &mut Transaction<'_, Sqlite>| {
            let table_name = table_name.clone();
            
            let sql = format!("DROP TABLE IF EXISTS {}", table_name);
            
            Box::pin(async move {
                sqlx::query(&sql)
                    .execute(tx)
                    .await
                    .map_err(|e| MigrationError::RollbackFailed(format!("Failed to drop table: {}", e)))?;
                
                Ok(())
            })
        };
        
        TestMigration::new(
            version,
            name,
            None,
            up_fn,
            down_fn,
            None,
            None,
        )
    }
    
    /// Create a test migration that adds a column to a table
    pub fn add_column_migration(
        version: &str,
        name: &str,
        table_name: &str,
        column_name: &str,
        column_type: &str,
    ) -> TestMigration {
        let table_name = table_name.to_string();
        let column_name = column_name.to_string();
        let column_type = column_type.to_string();
        
        // Create the up function
        let up_fn = move |tx: &mut Transaction<'_, Sqlite>| {
            let table_name = table_name.clone();
            let column_name = column_name.clone();
            let column_type = column_type.clone();
            
            let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table_name, column_name, column_type);
            
            Box::pin(async move {
                sqlx::query(&sql)
                    .execute(tx)
                    .await
                    .map_err(|e| MigrationError::ExecutionFailed(format!("Failed to add column: {}", e)))?;
                
                Ok(())
            })
        };
        
        // Create the down function - SQLite doesn't support dropping columns, so we need to recreate the table
        let down_fn = move |tx: &mut Transaction<'_, Sqlite>| {
            let table_name = table_name.clone();
            let column_name = column_name.clone();
            
            // This is a simplified version that assumes we know the original table schema
            // In a real implementation, we would need to query the table schema and recreate it without the column
            let sql = format!(
                "
                BEGIN TRANSACTION;
                
                -- Create a temporary table with the original schema
                CREATE TABLE {}_temp AS SELECT * FROM {} WHERE 0;
                
                -- Copy data from the original table to the temporary table, excluding the new column
                INSERT INTO {}_temp SELECT * FROM {};
                
                -- Drop the original table
                DROP TABLE {};
                
                -- Rename the temporary table to the original table name
                ALTER TABLE {}_temp RENAME TO {};
                
                COMMIT;
                ",
                table_name, table_name, table_name, table_name, table_name, table_name, table_name
            );
            
            Box::pin(async move {
                sqlx::query(&sql)
                    .execute(tx)
                    .await
                    .map_err(|e| MigrationError::RollbackFailed(format!("Failed to drop column: {}", e)))?;
                
                Ok(())
            })
        };
        
        TestMigration::new(
            version,
            name,
            None,
            up_fn,
            down_fn,
            None,
            None,
        )
    }
    
    /// Create a test migration that inserts data into a table
    pub fn insert_data_migration(
        version: &str,
        name: &str,
        table_name: &str,
        columns: &[&str],
        values: &[Vec<String>],
    ) -> TestMigration {
        let table_name = table_name.to_string();
        let columns = columns.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let values = values.to_vec();
        
        // Create the up function
        let up_fn = move |tx: &mut Transaction<'_, Sqlite>| {
            let table_name = table_name.clone();
            let columns = columns.clone();
            let values = values.clone();
            
            let columns_sql = columns.join(", ");
            let placeholders = (0..columns.len()).map(|_| "?").collect::<Vec<_>>().join(", ");
            
            let sql = format!("INSERT INTO {} ({}) VALUES ({})", table_name, columns_sql, placeholders);
            
            Box::pin(async move {
                for row in values {
                    // Convert the row values to a Vec<&str> for the query
                    let row_values: Vec<&str> = row.iter().map(|s| s.as_str()).collect();
                    
                    // Build the query with the right number of parameters
                    let mut query = sqlx::query(&sql);
                    for value in row_values {
                        query = query.bind(value);
                    }
                    
                    query.execute(tx)
                        .await
                        .map_err(|e| MigrationError::ExecutionFailed(format!("Failed to insert data: {}", e)))?;
                }
                
                Ok(())
            })
        };
        
        // Create the down function
        let down_fn = move |tx: &mut Transaction<'_, Sqlite>| {
            let table_name = table_name.clone();
            
            let sql = format!("DELETE FROM {}", table_name);
            
            Box::pin(async move {
                sqlx::query(&sql)
                    .execute(tx)
                    .await
                    .map_err(|e| MigrationError::RollbackFailed(format!("Failed to delete data: {}", e)))?;
                
                Ok(())
            })
        };
        
        TestMigration::new(
            version,
            name,
            None,
            up_fn,
            down_fn,
            None,
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_migration_test_context() -> Result<()> {
        // Create a test context
        let mut context = MigrationTestContext::new().await?;
        
        // Add a test migration that creates a users table
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
        
        // Add a test migration that adds a column to the users table
        context.add_migration(test_utils::add_column_migration(
            "0002",
            "add_status_to_users",
            "users",
            "status",
            "INTEGER NOT NULL DEFAULT 0",
        ));
        
        // Add a test migration that inserts data into the users table
        context.add_migration(test_utils::insert_data_migration(
            "0003",
            "insert_test_users",
            "users",
            &["name", "email"],
            &[
                vec!["User 1".to_string(), "user1@example.com".to_string()],
                vec!["User 2".to_string(), "user2@example.com".to_string()],
            ],
        ));
        
        // Run the migrations
        context.run_migrations().await?;
        
        // Verify that the migrations were applied
        let users: Vec<(i64, String, String)> = context.query("SELECT id, name, email FROM users").await?;
        
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].1, "User 1");
        assert_eq!(users[0].2, "user1@example.com");
        assert_eq!(users[1].1, "User 2");
        assert_eq!(users[1].2, "user2@example.com");
        
        // Rollback the last migration
        context.rollback_migration("0003").await?;
        
        // Verify that the data was removed
        let users: Vec<(i64, String, String)> = context.query("SELECT id, name, email FROM users").await?;
        assert_eq!(users.len(), 0);
        
        // Rollback all migrations
        context.rollback_all().await?;
        
        // Verify that the table was dropped
        let result = context.query::<(i64,)>("SELECT 1 FROM users").await;
        assert!(result.is_err());
        
        Ok(())
    }
}