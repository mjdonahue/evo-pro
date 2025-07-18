use crate::error::Result;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};
use std::{env, path::Path, str::FromStr, sync::Arc};
use tracing::{info, instrument};

use crate::storage::migration::{MigrationManager, MigrationOptions};

/// DatabaseManager handles SQLite connection pooling and database operations
#[derive(Clone)]
pub struct DatabaseManager {
    /// Connection pool for SQLite
    pub pool: Pool<Sqlite>,
    /// Path to the database file
    pub db_path: Arc<str>,
}

impl DatabaseManager {
    /// Creates a new DatabaseManager with a connection pool to the specified database
    #[instrument(err)]
    pub async fn new(db_path: &str) -> Result<Self> {
        info!("Initializing database at: {}", db_path);

        // Create the connection manager with the file path
        let pool = Pool::connect_with(
            SqliteConnectOptions::from_str(db_path)?
                .foreign_keys(!cfg!(test)) // Disable foreign keys in tests to avoid errors
                // Create the database if it doesn't exist
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                // Only use NORMAL if WAL mode is enabled
                // as it provides extra performance benefits
                // at the cost of durability
                .synchronous(SqliteSynchronous::Normal),
        )
        .await?;

        // Initialize the database schema if it doesn't exist
        let db_manager = Self {
            pool,
            db_path: db_path.into(),
        };

        Ok(db_manager)
    }

    /// Get database path
    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    #[instrument(skip(self))]
    pub async fn run_migrations(&self) -> Result<()> {
        // Determine which migrations directory to use
        let migrations_dir = if cfg!(debug_assertions) && !env::var("SEED_DB").is_ok_and(|s| &s == "false") {
            "./seeding"
        } else {
            "./migrations"
        };

        info!("Running migrations from directory: {}", migrations_dir);

        // Create migration options
        let options = MigrationOptions {
            create_migrations_table: true,
            validate_migrations: true,
            allow_missing_down: true,
            transform_data: true,
            timeout_seconds: 60,
        };

        // Create and initialize the migration manager
        let mut manager = MigrationManager::new(self.pool.clone(), migrations_dir, Some(options));
        manager.initialize().await?;

        // Check for version conflicts
        if let Err(e) = manager.check_version_conflicts().await {
            // If there's a version conflict, log it but continue
            // This allows for development flexibility while still providing warnings
            tracing::warn!("Migration version conflict detected: {}", e);
        }

        // Validate migrations
        manager.validate_all().await?;

        // Run pending migrations
        let results = manager.run_migrations().await?;

        if !results.is_empty() {
            info!("Successfully applied {} migrations", results.len());

            // Log details of applied migrations
            for record in results {
                info!("Applied migration {}: {} in {}ms", 
                    record.version, 
                    record.name,
                    record.execution_time_ms.unwrap_or(0)
                );
            }
        } else {
            info!("No migrations to apply");
        }

        Ok(())
    }

    /// Setup test database schema
    pub(crate) async fn setup_test_db() -> DatabaseManager {
        let db = DatabaseManager::new(":memory:")
            .await
            .expect("Failed to initialize database");
        db.run_migrations().await.unwrap();
        db
    }
}
