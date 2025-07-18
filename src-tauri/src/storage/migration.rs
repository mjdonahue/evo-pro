//! Migration framework for database schema evolution
//!
//! This module provides a robust migration framework with versioning,
//! data transformation, validation, and rollback capabilities.

use std::{collections::HashMap, fmt::Display, path::Path, sync::Arc, time::Duration};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateError, Connection, Executor, Pool, Sqlite, Transaction};
use thiserror::Error;
use tokio::fs;
use tracing::{debug, error, info, instrument, warn};

use crate::error::{AppError, Result};

/// Migration error types
#[derive(Debug, Error)]
pub enum MigrationError {
    /// Migration file not found
    #[error("Migration file not found: {0}")]
    FileNotFound(String),
    
    /// Migration version conflict
    #[error("Migration version conflict: {0}")]
    VersionConflict(String),
    
    /// Migration validation failed
    #[error("Migration validation failed: {0}")]
    ValidationFailed(String),
    
    /// Migration execution failed
    #[error("Migration execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Migration rollback failed
    #[error("Migration rollback failed: {0}")]
    RollbackFailed(String),
    
    /// Migration data transformation failed
    #[error("Migration data transformation failed: {0}")]
    TransformationFailed(String),
    
    /// SQLx migration error
    #[error("SQLx migration error: {0}")]
    SqlxError(#[from] MigrateError),
    
    /// Database error
    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl From<MigrationError> for AppError {
    fn from(err: MigrationError) -> Self {
        AppError::database(err.to_string())
    }
}

/// Migration status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// Migration is pending
    Pending,
    /// Migration is in progress
    InProgress,
    /// Migration completed successfully
    Completed,
    /// Migration failed
    Failed,
    /// Migration was rolled back
    RolledBack,
}

impl Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStatus::Pending => write!(f, "Pending"),
            MigrationStatus::InProgress => write!(f, "InProgress"),
            MigrationStatus::Completed => write!(f, "Completed"),
            MigrationStatus::Failed => write!(f, "Failed"),
            MigrationStatus::RolledBack => write!(f, "RolledBack"),
        }
    }
}

/// Migration record stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration version
    pub version: String,
    /// Migration name
    pub name: String,
    /// Migration description
    pub description: Option<String>,
    /// Migration status
    pub status: MigrationStatus,
    /// Migration applied at timestamp
    pub applied_at: Option<DateTime<Utc>>,
    /// Migration execution duration in milliseconds
    pub execution_time_ms: Option<i64>,
    /// Migration checksum
    pub checksum: Option<String>,
    /// Migration metadata
    pub metadata: Option<serde_json::Value>,
}

/// Migration trait for implementing custom migrations
#[async_trait]
pub trait Migration: Send + Sync {
    /// Get the migration version
    fn version(&self) -> &str;
    
    /// Get the migration name
    fn name(&self) -> &str;
    
    /// Get the migration description
    fn description(&self) -> Option<&str>;
    
    /// Get the migration checksum
    fn checksum(&self) -> Option<&str>;
    
    /// Get the migration metadata
    fn metadata(&self) -> Option<&serde_json::Value>;
    
    /// Run the migration
    async fn up(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError>;
    
    /// Rollback the migration
    async fn down(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError>;
    
    /// Validate the migration
    async fn validate(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError>;
    
    /// Transform data during migration
    async fn transform_data(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError>;
}

/// SQL migration that reads from a file
pub struct SqlMigration {
    /// Migration version
    version: String,
    /// Migration name
    name: String,
    /// Migration description
    description: Option<String>,
    /// Migration up SQL
    up_sql: String,
    /// Migration down SQL
    down_sql: Option<String>,
    /// Migration checksum
    checksum: Option<String>,
    /// Migration metadata
    metadata: Option<serde_json::Value>,
}

impl SqlMigration {
    /// Create a new SQL migration
    pub fn new(
        version: impl Into<String>,
        name: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: Option<String>,
        description: Option<String>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        let version = version.into();
        let name = name.into();
        let up_sql = up_sql.into();
        
        // Calculate checksum of the up SQL
        let checksum = Some(format!("{:x}", md5::compute(&up_sql)));
        
        Self {
            version,
            name,
            description,
            up_sql,
            down_sql,
            checksum,
            metadata,
        }
    }
    
    /// Load a SQL migration from a file
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self, MigrationError> {
        let path = path.as_ref();
        
        // Read the file
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| MigrationError::FileNotFound(format!("Failed to read migration file: {}", e)))?;
        
        // Parse the filename to get version and name
        let filename = path.file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| MigrationError::FileNotFound("Invalid migration filename".to_string()))?;
        
        // Expected format: NNNN_name.sql or NNNN_name_up.sql/NNNN_name_down.sql
        let parts: Vec<&str> = filename.split('_').collect();
        if parts.len() < 2 {
            return Err(MigrationError::FileNotFound(format!("Invalid migration filename format: {}", filename)));
        }
        
        let version = parts[0].to_string();
        
        // Check if this is an up or down migration
        let is_up = filename.ends_with("_up.sql") || (!filename.ends_with("_down.sql") && !filename.ends_with("_up.sql"));
        let is_down = filename.ends_with("_down.sql");
        
        // Get the name without the up/down suffix
        let name = if is_up && filename.ends_with("_up.sql") {
            filename[..filename.len() - 7].split('_').skip(1).collect::<Vec<&str>>().join("_")
        } else if is_down {
            filename[..filename.len() - 9].split('_').skip(1).collect::<Vec<&str>>().join("_")
        } else {
            filename[..filename.len() - 4].split('_').skip(1).collect::<Vec<&str>>().join("_")
        };
        
        // If this is a down migration, we need to find the corresponding up migration
        if is_down {
            // This is a down migration, so we need to find the corresponding up migration
            let up_path = path.with_file_name(format!("{}_{}_up.sql", version, name));
            if !up_path.exists() {
                return Err(MigrationError::FileNotFound(format!("Up migration not found for down migration: {}", filename)));
            }
            
            // Read the up migration
            let up_sql = fs::read_to_string(&up_path)
                .await
                .map_err(|e| MigrationError::FileNotFound(format!("Failed to read up migration file: {}", e)))?;
            
            // Create the migration
            Ok(Self::new(
                version,
                name,
                up_sql,
                Some(content),
                None,
                None,
            ))
        } else {
            // This is an up migration or a combined migration
            
            // Check if there's a corresponding down migration
            let down_path = path.with_file_name(format!("{}_{}_down.sql", version, name));
            let down_sql = if down_path.exists() {
                Some(fs::read_to_string(&down_path)
                    .await
                    .map_err(|e| MigrationError::FileNotFound(format!("Failed to read down migration file: {}", e)))?)
            } else {
                None
            };
            
            // Create the migration
            Ok(Self::new(
                version,
                name,
                content,
                down_sql,
                None,
                None,
            ))
        }
    }
}

#[async_trait]
impl Migration for SqlMigration {
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
        self.checksum.as_deref()
    }
    
    fn metadata(&self) -> Option<&serde_json::Value> {
        self.metadata.as_ref()
    }
    
    async fn up(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Execute the up SQL
        tx.execute(&self.up_sql)
            .await
            .map_err(|e| MigrationError::ExecutionFailed(format!("Failed to execute up migration: {}", e)))?;
        
        Ok(())
    }
    
    async fn down(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Execute the down SQL if available
        if let Some(down_sql) = &self.down_sql {
            tx.execute(down_sql)
                .await
                .map_err(|e| MigrationError::RollbackFailed(format!("Failed to execute down migration: {}", e)))?;
        } else {
            return Err(MigrationError::RollbackFailed("No down migration available".to_string()));
        }
        
        Ok(())
    }
    
    async fn validate(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // Basic validation - check if the SQL is valid by preparing it
        tx.prepare(&self.up_sql)
            .await
            .map_err(|e| MigrationError::ValidationFailed(format!("Invalid up SQL: {}", e)))?;
        
        if let Some(down_sql) = &self.down_sql {
            tx.prepare(down_sql)
                .await
                .map_err(|e| MigrationError::ValidationFailed(format!("Invalid down SQL: {}", e)))?;
        }
        
        Ok(())
    }
    
    async fn transform_data(&self, _tx: &mut Transaction<'_, Sqlite>) -> Result<(), MigrationError> {
        // SQL migrations don't have data transformation by default
        Ok(())
    }
}

/// Migration manager for handling database migrations
pub struct MigrationManager {
    /// Database pool
    pool: Pool<Sqlite>,
    /// Migrations directory
    migrations_dir: String,
    /// Registered migrations
    migrations: HashMap<String, Arc<dyn Migration>>,
    /// Migration options
    options: MigrationOptions,
}

/// Migration options
#[derive(Debug, Clone)]
pub struct MigrationOptions {
    /// Whether to create the migrations table if it doesn't exist
    pub create_migrations_table: bool,
    /// Whether to validate migrations before running them
    pub validate_migrations: bool,
    /// Whether to allow missing down migrations
    pub allow_missing_down: bool,
    /// Whether to transform data during migrations
    pub transform_data: bool,
    /// Timeout for migrations in seconds
    pub timeout_seconds: u64,
}

impl Default for MigrationOptions {
    fn default() -> Self {
        Self {
            create_migrations_table: true,
            validate_migrations: true,
            allow_missing_down: true,
            transform_data: true,
            timeout_seconds: 60,
        }
    }
}

impl MigrationManager {
    /// Create a new migration manager
    pub fn new(pool: Pool<Sqlite>, migrations_dir: impl Into<String>, options: Option<MigrationOptions>) -> Self {
        Self {
            pool,
            migrations_dir: migrations_dir.into(),
            migrations: HashMap::new(),
            options: options.unwrap_or_default(),
        }
    }
    
    /// Initialize the migration manager
    #[instrument(skip(self))]
    pub async fn initialize(&mut self) -> Result<(), MigrationError> {
        // Create the migrations table if it doesn't exist
        if self.options.create_migrations_table {
            self.create_migrations_table().await?;
        }
        
        // Load migrations from the migrations directory
        self.load_migrations_from_directory().await?;
        
        Ok(())
    }
    
    /// Create the migrations table
    async fn create_migrations_table(&self) -> Result<(), MigrationError> {
        let query = r#"
        CREATE TABLE IF NOT EXISTS _migrations (
            version TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL,
            applied_at TIMESTAMP,
            execution_time_ms INTEGER,
            checksum TEXT,
            metadata TEXT CHECK (metadata IS NULL OR json_valid(metadata))
        );
        "#;
        
        self.pool.execute(query)
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to create migrations table: {}", e)))?;
        
        Ok(())
    }
    
    /// Load migrations from the migrations directory
    async fn load_migrations_from_directory(&mut self) -> Result<(), MigrationError> {
        // Get all SQL files in the migrations directory
        let entries = fs::read_dir(&self.migrations_dir)
            .await
            .map_err(|e| MigrationError::FileNotFound(format!("Failed to read migrations directory: {}", e)))?;
        
        let mut migration_files = Vec::new();
        
        // Collect all migration files
        let mut entries_vec = Vec::new();
        let mut entry = entries.into_iter();
        while let Some(entry_result) = entry.next() {
            let entry = entry_result.map_err(|e| MigrationError::FileNotFound(format!("Failed to read directory entry: {}", e)))?;
            entries_vec.push(entry);
        }
        
        // Process each entry
        for entry in entries_vec {
            let path = entry.path();
            
            // Skip directories and non-SQL files
            if path.is_dir() || path.extension().map_or(true, |ext| ext != "sql") {
                continue;
            }
            
            // Skip down migrations, they'll be handled with their corresponding up migrations
            if path.file_name().map_or(false, |name| name.to_string_lossy().ends_with("_down.sql")) {
                continue;
            }
            
            migration_files.push(path);
        }
        
        // Sort migration files by version
        migration_files.sort_by(|a, b| {
            let a_name = a.file_name().unwrap().to_string_lossy();
            let b_name = b.file_name().unwrap().to_string_lossy();
            
            let a_version = a_name.split('_').next().unwrap_or("");
            let b_version = b_name.split('_').next().unwrap_or("");
            
            a_version.cmp(b_version)
        });
        
        // Load each migration
        for path in migration_files {
            let migration = SqlMigration::from_file(&path).await?;
            self.register_migration(Arc::new(migration));
        }
        
        Ok(())
    }
    
    /// Register a migration
    pub fn register_migration(&mut self, migration: Arc<dyn Migration>) {
        self.migrations.insert(migration.version().to_string(), migration);
    }
    
    /// Get all migrations
    pub fn get_migrations(&self) -> Vec<Arc<dyn Migration>> {
        let mut migrations: Vec<_> = self.migrations.values().cloned().collect();
        migrations.sort_by(|a, b| a.version().cmp(b.version()));
        migrations
    }
    
    /// Get a migration by version
    pub fn get_migration(&self, version: &str) -> Option<Arc<dyn Migration>> {
        self.migrations.get(version).cloned()
    }
    
    /// Get all applied migrations
    pub async fn get_applied_migrations(&self) -> Result<Vec<MigrationRecord>, MigrationError> {
        let query = "SELECT version, name, description, status, applied_at, execution_time_ms, checksum, metadata FROM _migrations ORDER BY version";
        
        let records = sqlx::query_as!(
            MigrationRecord,
            query
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(format!("Failed to get applied migrations: {}", e)))?;
        
        Ok(records)
    }
    
    /// Get pending migrations
    pub async fn get_pending_migrations(&self) -> Result<Vec<Arc<dyn Migration>>, MigrationError> {
        // Get all applied migrations
        let applied = self.get_applied_migrations().await?;
        let applied_versions: std::collections::HashSet<String> = applied
            .iter()
            .filter(|m| m.status == MigrationStatus::Completed)
            .map(|m| m.version.clone())
            .collect();
        
        // Get all migrations that haven't been applied
        let pending: Vec<_> = self.get_migrations()
            .into_iter()
            .filter(|m| !applied_versions.contains(m.version()))
            .collect();
        
        Ok(pending)
    }
    
    /// Run all pending migrations
    #[instrument(skip(self))]
    pub async fn run_migrations(&self) -> Result<Vec<MigrationRecord>, MigrationError> {
        // Get pending migrations
        let pending = self.get_pending_migrations().await?;
        
        if pending.is_empty() {
            info!("No pending migrations to run");
            return Ok(Vec::new());
        }
        
        info!("Running {} pending migrations", pending.len());
        
        let mut results = Vec::new();
        
        // Run each migration
        for migration in pending {
            let result = self.run_migration(migration.as_ref()).await?;
            results.push(result);
        }
        
        info!("Successfully ran {} migrations", results.len());
        
        Ok(results)
    }
    
    /// Run a specific migration
    #[instrument(skip(self, migration))]
    pub async fn run_migration(&self, migration: &dyn Migration) -> Result<MigrationRecord, MigrationError> {
        let version = migration.version();
        let name = migration.name();
        
        info!("Running migration {}: {}", version, name);
        
        // Check if the migration has already been applied
        let existing = sqlx::query_as!(
            MigrationRecord,
            "SELECT version, name, description, status, applied_at, execution_time_ms, checksum, metadata FROM _migrations WHERE version = ?",
            version
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(format!("Failed to check migration status: {}", e)))?;
        
        if let Some(record) = existing {
            if record.status == MigrationStatus::Completed {
                info!("Migration {} already applied, skipping", version);
                return Ok(record);
            } else if record.status == MigrationStatus::InProgress {
                warn!("Migration {} is already in progress, resetting status", version);
                // Reset the status to pending
                sqlx::query!(
                    "UPDATE _migrations SET status = ? WHERE version = ?",
                    MigrationStatus::Pending.to_string(),
                    version
                )
                .execute(&self.pool)
                .await
                .map_err(|e| MigrationError::DatabaseError(format!("Failed to reset migration status: {}", e)))?;
            }
        }
        
        // Start a transaction
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to start transaction: {}", e)))?;
        
        // Update migration status to in progress
        let now = Utc::now();
        let start_time = std::time::Instant::now();
        
        let record = MigrationRecord {
            version: version.to_string(),
            name: name.to_string(),
            description: migration.description().map(|s| s.to_string()),
            status: MigrationStatus::InProgress,
            applied_at: Some(now),
            execution_time_ms: None,
            checksum: migration.checksum().map(|s| s.to_string()),
            metadata: migration.metadata().cloned(),
        };
        
        // Insert or update the migration record
        if existing.is_some() {
            sqlx::query!(
                "UPDATE _migrations SET name = ?, description = ?, status = ?, applied_at = ?, checksum = ?, metadata = ? WHERE version = ?",
                record.name,
                record.description,
                record.status.to_string(),
                record.applied_at,
                record.checksum,
                record.metadata.map(|m| serde_json::to_string(&m).unwrap()),
                record.version
            )
            .execute(&mut tx)
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to update migration record: {}", e)))?;
        } else {
            sqlx::query!(
                "INSERT INTO _migrations (version, name, description, status, applied_at, checksum, metadata) VALUES (?, ?, ?, ?, ?, ?, ?)",
                record.version,
                record.name,
                record.description,
                record.status.to_string(),
                record.applied_at,
                record.checksum,
                record.metadata.map(|m| serde_json::to_string(&m).unwrap())
            )
            .execute(&mut tx)
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to insert migration record: {}", e)))?;
        }
        
        // Validate the migration if enabled
        if self.options.validate_migrations {
            debug!("Validating migration {}", version);
            if let Err(e) = migration.validate(&mut tx).await {
                error!("Migration validation failed: {}", e);
                
                // Update migration status to failed
                let record = MigrationRecord {
                    version: version.to_string(),
                    name: name.to_string(),
                    description: migration.description().map(|s| s.to_string()),
                    status: MigrationStatus::Failed,
                    applied_at: Some(now),
                    execution_time_ms: Some(start_time.elapsed().as_millis() as i64),
                    checksum: migration.checksum().map(|s| s.to_string()),
                    metadata: migration.metadata().cloned(),
                };
                
                sqlx::query!(
                    "UPDATE _migrations SET status = ?, execution_time_ms = ? WHERE version = ?",
                    record.status.to_string(),
                    record.execution_time_ms,
                    record.version
                )
                .execute(&mut tx)
                .await
                .map_err(|e| MigrationError::DatabaseError(format!("Failed to update migration record: {}", e)))?;
                
                // Rollback the transaction
                tx.rollback()
                    .await
                    .map_err(|e| MigrationError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
                
                return Err(e);
            }
        }
        
        // Run the migration
        debug!("Running migration {}", version);
        if let Err(e) = migration.up(&mut tx).await {
            error!("Migration failed: {}", e);
            
            // Update migration status to failed
            let record = MigrationRecord {
                version: version.to_string(),
                name: name.to_string(),
                description: migration.description().map(|s| s.to_string()),
                status: MigrationStatus::Failed,
                applied_at: Some(now),
                execution_time_ms: Some(start_time.elapsed().as_millis() as i64),
                checksum: migration.checksum().map(|s| s.to_string()),
                metadata: migration.metadata().cloned(),
            };
            
            sqlx::query!(
                "UPDATE _migrations SET status = ?, execution_time_ms = ? WHERE version = ?",
                record.status.to_string(),
                record.execution_time_ms,
                record.version
            )
            .execute(&mut tx)
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to update migration record: {}", e)))?;
            
            // Rollback the transaction
            tx.rollback()
                .await
                .map_err(|e| MigrationError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
            
            return Err(e);
        }
        
        // Transform data if enabled
        if self.options.transform_data {
            debug!("Transforming data for migration {}", version);
            if let Err(e) = migration.transform_data(&mut tx).await {
                error!("Data transformation failed: {}", e);
                
                // Update migration status to failed
                let record = MigrationRecord {
                    version: version.to_string(),
                    name: name.to_string(),
                    description: migration.description().map(|s| s.to_string()),
                    status: MigrationStatus::Failed,
                    applied_at: Some(now),
                    execution_time_ms: Some(start_time.elapsed().as_millis() as i64),
                    checksum: migration.checksum().map(|s| s.to_string()),
                    metadata: migration.metadata().cloned(),
                };
                
                sqlx::query!(
                    "UPDATE _migrations SET status = ?, execution_time_ms = ? WHERE version = ?",
                    record.status.to_string(),
                    record.execution_time_ms,
                    record.version
                )
                .execute(&mut tx)
                .await
                .map_err(|e| MigrationError::DatabaseError(format!("Failed to update migration record: {}", e)))?;
                
                // Rollback the transaction
                tx.rollback()
                    .await
                    .map_err(|e| MigrationError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
                
                return Err(e);
            }
        }
        
        // Update migration status to completed
        let record = MigrationRecord {
            version: version.to_string(),
            name: name.to_string(),
            description: migration.description().map(|s| s.to_string()),
            status: MigrationStatus::Completed,
            applied_at: Some(now),
            execution_time_ms: Some(start_time.elapsed().as_millis() as i64),
            checksum: migration.checksum().map(|s| s.to_string()),
            metadata: migration.metadata().cloned(),
        };
        
        sqlx::query!(
            "UPDATE _migrations SET status = ?, execution_time_ms = ? WHERE version = ?",
            record.status.to_string(),
            record.execution_time_ms,
            record.version
        )
        .execute(&mut tx)
        .await
        .map_err(|e| MigrationError::DatabaseError(format!("Failed to update migration record: {}", e)))?;
        
        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;
        
        info!("Migration {} completed successfully in {}ms", version, record.execution_time_ms.unwrap_or(0));
        
        Ok(record)
    }
    
    /// Rollback a specific migration
    #[instrument(skip(self))]
    pub async fn rollback_migration(&self, version: &str) -> Result<MigrationRecord, MigrationError> {
        // Get the migration
        let migration = self.get_migration(version)
            .ok_or_else(|| MigrationError::FileNotFound(format!("Migration not found: {}", version)))?;
        
        info!("Rolling back migration {}: {}", version, migration.name());
        
        // Check if the migration has been applied
        let existing = sqlx::query_as!(
            MigrationRecord,
            "SELECT version, name, description, status, applied_at, execution_time_ms, checksum, metadata FROM _migrations WHERE version = ?",
            version
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| MigrationError::DatabaseError(format!("Failed to check migration status: {}", e)))?;
        
        if let Some(record) = &existing {
            if record.status != MigrationStatus::Completed {
                return Err(MigrationError::RollbackFailed(format!("Migration {} is not in a completed state", version)));
            }
        } else {
            return Err(MigrationError::RollbackFailed(format!("Migration {} has not been applied", version)));
        }
        
        // Start a transaction
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to start transaction: {}", e)))?;
        
        // Run the down migration
        if let Err(e) = migration.down(&mut tx).await {
            error!("Rollback failed: {}", e);
            
            // Rollback the transaction
            tx.rollback()
                .await
                .map_err(|e| MigrationError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
            
            return Err(e);
        }
        
        // Update migration status to rolled back
        let record = MigrationRecord {
            version: version.to_string(),
            name: migration.name().to_string(),
            description: migration.description().map(|s| s.to_string()),
            status: MigrationStatus::RolledBack,
            applied_at: existing.as_ref().and_then(|r| r.applied_at),
            execution_time_ms: existing.as_ref().and_then(|r| r.execution_time_ms),
            checksum: migration.checksum().map(|s| s.to_string()),
            metadata: migration.metadata().cloned(),
        };
        
        sqlx::query!(
            "UPDATE _migrations SET status = ? WHERE version = ?",
            record.status.to_string(),
            record.version
        )
        .execute(&mut tx)
        .await
        .map_err(|e| MigrationError::DatabaseError(format!("Failed to update migration record: {}", e)))?;
        
        // Commit the transaction
        tx.commit()
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;
        
        info!("Migration {} rolled back successfully", version);
        
        Ok(record)
    }
    
    /// Rollback all migrations
    #[instrument(skip(self))]
    pub async fn rollback_all(&self) -> Result<Vec<MigrationRecord>, MigrationError> {
        // Get all applied migrations
        let applied = self.get_applied_migrations().await?;
        let applied: Vec<_> = applied
            .into_iter()
            .filter(|m| m.status == MigrationStatus::Completed)
            .collect();
        
        if applied.is_empty() {
            info!("No migrations to roll back");
            return Ok(Vec::new());
        }
        
        info!("Rolling back {} migrations", applied.len());
        
        let mut results = Vec::new();
        
        // Roll back each migration in reverse order
        for record in applied.iter().rev() {
            let result = self.rollback_migration(&record.version).await?;
            results.push(result);
        }
        
        info!("Successfully rolled back {} migrations", results.len());
        
        Ok(results)
    }
    
    /// Rollback to a specific version
    #[instrument(skip(self))]
    pub async fn rollback_to(&self, target_version: &str) -> Result<Vec<MigrationRecord>, MigrationError> {
        // Get all applied migrations
        let applied = self.get_applied_migrations().await?;
        let applied: Vec<_> = applied
            .into_iter()
            .filter(|m| m.status == MigrationStatus::Completed)
            .collect();
        
        if applied.is_empty() {
            info!("No migrations to roll back");
            return Ok(Vec::new());
        }
        
        // Find the target version
        let target_index = applied.iter().position(|m| m.version == target_version);
        
        if let Some(target_index) = target_index {
            // Roll back all migrations after the target version
            let to_rollback = &applied[target_index + 1..];
            
            if to_rollback.is_empty() {
                info!("No migrations to roll back");
                return Ok(Vec::new());
            }
            
            info!("Rolling back {} migrations to version {}", to_rollback.len(), target_version);
            
            let mut results = Vec::new();
            
            // Roll back each migration in reverse order
            for record in to_rollback.iter().rev() {
                let result = self.rollback_migration(&record.version).await?;
                results.push(result);
            }
            
            info!("Successfully rolled back {} migrations", results.len());
            
            Ok(results)
        } else {
            Err(MigrationError::RollbackFailed(format!("Target version {} not found", target_version)))
        }
    }
    
    /// Validate all migrations
    #[instrument(skip(self))]
    pub async fn validate_all(&self) -> Result<(), MigrationError> {
        // Get all migrations
        let migrations = self.get_migrations();
        
        if migrations.is_empty() {
            info!("No migrations to validate");
            return Ok(());
        }
        
        info!("Validating {} migrations", migrations.len());
        
        // Start a transaction
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to start transaction: {}", e)))?;
        
        // Validate each migration
        for migration in migrations {
            debug!("Validating migration {}: {}", migration.version(), migration.name());
            migration.validate(&mut tx).await?;
        }
        
        // Rollback the transaction (we don't want to apply any changes)
        tx.rollback()
            .await
            .map_err(|e| MigrationError::DatabaseError(format!("Failed to rollback transaction: {}", e)))?;
        
        info!("All migrations validated successfully");
        
        Ok(())
    }
    
    /// Check for migration version conflicts
    #[instrument(skip(self))]
    pub async fn check_version_conflicts(&self) -> Result<(), MigrationError> {
        // Get all applied migrations
        let applied = self.get_applied_migrations().await?;
        
        // Check for conflicts
        for record in applied {
            if record.status != MigrationStatus::Completed {
                continue;
            }
            
            // Get the migration
            if let Some(migration) = self.get_migration(&record.version) {
                // Check if the checksum matches
                if let (Some(applied_checksum), Some(current_checksum)) = (&record.checksum, migration.checksum()) {
                    if applied_checksum != current_checksum {
                        return Err(MigrationError::VersionConflict(
                            format!("Checksum mismatch for migration {}: applied={}, current={}", 
                                record.version, applied_checksum, current_checksum)
                        ));
                    }
                }
            } else {
                warn!("Applied migration {} not found in registered migrations", record.version);
            }
        }
        
        Ok(())
    }
}

/// Migration test utilities
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use sqlx::SqlitePool;
    
    /// Create a test migration manager
    pub async fn create_test_migration_manager() -> MigrationManager {
        // Create an in-memory database
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");
        
        // Create a migration manager
        let mut manager = MigrationManager::new(pool, "migrations", None);
        
        // Initialize the manager
        manager.initialize().await.expect("Failed to initialize migration manager");
        
        manager
    }
    
    /// Create a test SQL migration
    pub fn create_test_sql_migration(
        version: &str,
        name: &str,
        up_sql: &str,
        down_sql: Option<&str>,
    ) -> SqlMigration {
        SqlMigration::new(
            version,
            name,
            up_sql,
            down_sql.map(|s| s.to_string()),
            None,
            None,
        )
    }
}