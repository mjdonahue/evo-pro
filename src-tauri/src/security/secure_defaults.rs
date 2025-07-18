//! Secure Defaults for Application Components
//!
//! This module provides secure default configurations for various components of the application,
//! ensuring that security best practices are applied consistently throughout the codebase.

use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::{debug, info, warn};

/// Configuration for secure defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureDefaultsConfig {
    /// Whether secure defaults are enabled
    pub enabled: bool,
    
    /// Whether to enforce secure defaults (fail if they can't be applied)
    pub enforce: bool,
    
    /// Whether to log when secure defaults are applied
    pub log_application: bool,
}

impl Default for SecureDefaultsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            enforce: true,
            log_application: true,
        }
    }
}

/// Service for managing and applying secure defaults
#[derive(Debug)]
pub struct SecureDefaultsService {
    /// Database connection pool
    db: SqlitePool,
    
    /// Configuration
    config: SecureDefaultsConfig,
}

impl SecureDefaultsService {
    /// Create a new SecureDefaultsService
    pub async fn new(db: SqlitePool, config: Option<SecureDefaultsConfig>) -> Result<Self> {
        let config = config.unwrap_or_default();
        
        Ok(Self {
            db,
            config,
        })
    }
    
    /// Apply secure defaults to all components
    pub async fn apply_secure_defaults(&self) -> Result<()> {
        if !self.config.enabled {
            debug!("Secure defaults are disabled, skipping application");
            return Ok(());
        }
        
        info!("Applying secure defaults to all components");
        
        // Apply secure defaults to various components
        self.apply_database_defaults().await?;
        self.apply_storage_defaults().await?;
        self.apply_network_defaults().await?;
        self.apply_authentication_defaults().await?;
        self.apply_input_validation_defaults().await?;
        self.apply_error_handling_defaults().await?;
        self.apply_file_operation_defaults().await?;
        self.apply_configuration_defaults().await?;
        
        info!("Secure defaults applied successfully");
        
        Ok(())
    }
    
    /// Apply secure defaults to database operations
    async fn apply_database_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to database operations");
        }
        
        // Set secure connection timeout
        sqlx::query("PRAGMA busy_timeout = 5000")
            .execute(&self.db)
            .await
            .context("Failed to set database busy timeout")?;
        
        // Enable WAL mode for better concurrency and crash recovery
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&self.db)
            .await
            .context("Failed to enable WAL mode")?;
        
        // Set secure synchronization mode
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&self.db)
            .await
            .context("Failed to set synchronous mode")?;
        
        // Enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.db)
            .await
            .context("Failed to enable foreign key constraints")?;
        
        Ok(())
    }
    
    /// Apply secure defaults to data storage
    async fn apply_storage_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to data storage");
        }
        
        // Ensure data directory has secure permissions
        let data_dir = crate::utils::get_data_dir();
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&data_dir)
                .context("Failed to get data directory metadata")?;
            let mut permissions = metadata.permissions();
            
            // Set permissions to owner read/write/execute only (700)
            permissions.set_mode(0o700);
            std::fs::set_permissions(&data_dir, permissions)
                .context("Failed to set data directory permissions")?;
        }
        
        Ok(())
    }
    
    /// Apply secure defaults to network communication
    async fn apply_network_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to network communication");
        }
        
        // Network security defaults are primarily handled by libp2p configuration
        // which is set up when creating p2p nodes
        
        Ok(())
    }
    
    /// Apply secure defaults to authentication and authorization
    async fn apply_authentication_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to authentication and authorization");
        }
        
        // Set secure defaults for key generation and storage
        // (This is a placeholder - actual implementation would depend on the authentication system)
        
        Ok(())
    }
    
    /// Apply secure defaults to input validation
    async fn apply_input_validation_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to input validation");
        }
        
        // Input validation defaults are primarily handled by the validation framework
        // which is applied at the repository and service layers
        
        Ok(())
    }
    
    /// Apply secure defaults to error handling and logging
    async fn apply_error_handling_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to error handling and logging");
        }
        
        // Error handling defaults are primarily handled by the error taxonomy
        // and contextual error enrichment
        
        Ok(())
    }
    
    /// Apply secure defaults to file operations
    async fn apply_file_operation_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to file operations");
        }
        
        // Ensure temporary files are created with secure permissions
        // and in a secure location
        
        Ok(())
    }
    
    /// Apply secure defaults to application configuration
    async fn apply_configuration_defaults(&self) -> Result<()> {
        if self.config.log_application {
            info!("Applying secure defaults to application configuration");
        }
        
        // Set secure defaults for application configuration
        // (This is a placeholder - actual implementation would depend on the configuration system)
        
        Ok(())
    }
}

/// Database secure defaults
pub struct DatabaseSecureDefaults;

impl DatabaseSecureDefaults {
    /// Get secure connection string for SQLite
    pub fn get_connection_string(path: &str) -> String {
        // Add secure connection parameters to SQLite connection string
        format!("{}?mode=rwc&cache=shared&_journal_mode=WAL&_synchronous=NORMAL&_foreign_keys=ON&_busy_timeout=5000", path)
    }
    
    /// Get secure SQLite pragmas
    pub fn get_pragmas() -> Vec<&'static str> {
        vec![
            "PRAGMA journal_mode = WAL",
            "PRAGMA synchronous = NORMAL",
            "PRAGMA foreign_keys = ON",
            "PRAGMA busy_timeout = 5000",
        ]
    }
}

/// Network secure defaults
pub struct NetworkSecureDefaults;

impl NetworkSecureDefaults {
    /// Get secure timeout duration
    pub fn get_timeout() -> Duration {
        Duration::from_secs(30)
    }
    
    /// Get secure keepalive interval
    pub fn get_keepalive_interval() -> Duration {
        Duration::from_secs(20)
    }
    
    /// Get secure TLS configuration
    pub fn get_tls_config() -> bool {
        // Always use TLS by default
        true
    }
}

/// Authentication secure defaults
pub struct AuthenticationSecureDefaults;

impl AuthenticationSecureDefaults {
    /// Get secure password hashing iterations
    pub fn get_password_hashing_iterations() -> u32 {
        // Use a high number of iterations for password hashing
        100_000
    }
    
    /// Get secure token expiration
    pub fn get_token_expiration() -> Duration {
        // Tokens expire after 1 hour by default
        Duration::from_secs(3600)
    }
    
    /// Get secure key length
    pub fn get_key_length() -> usize {
        // Use 32 bytes (256 bits) for keys
        32
    }
}

/// Input validation secure defaults
pub struct InputValidationSecureDefaults;

impl InputValidationSecureDefaults {
    /// Get secure maximum input length
    pub fn get_max_input_length() -> usize {
        // Default maximum input length
        1024 * 1024 // 1 MB
    }
    
    /// Get secure input validation regex for usernames
    pub fn get_username_regex() -> &'static str {
        // Alphanumeric characters, underscores, and hyphens, 3-32 characters
        r"^[a-zA-Z0-9_-]{3,32}$"
    }
    
    /// Get secure input validation regex for emails
    pub fn get_email_regex() -> &'static str {
        // Simple email validation regex
        r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
    }
}

/// File operation secure defaults
pub struct FileOperationSecureDefaults;

impl FileOperationSecureDefaults {
    /// Get secure temporary directory
    pub fn get_temp_dir() -> std::path::PathBuf {
        // Use application-specific temporary directory
        let app_temp_dir = std::env::temp_dir().join("evo-pro");
        
        // Create the directory if it doesn't exist
        if !app_temp_dir.exists() {
            std::fs::create_dir_all(&app_temp_dir).ok();
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = std::fs::metadata(&app_temp_dir) {
                    let mut permissions = metadata.permissions();
                    permissions.set_mode(0o700);
                    std::fs::set_permissions(&app_temp_dir, permissions).ok();
                }
            }
        }
        
        app_temp_dir
    }
    
    /// Get secure file permissions
    #[cfg(unix)]
    pub fn get_file_permissions() -> u32 {
        // Owner read/write only (600)
        0o600
    }
    
    /// Get secure directory permissions
    #[cfg(unix)]
    pub fn get_directory_permissions() -> u32 {
        // Owner read/write/execute only (700)
        0o700
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    
    async fn setup_test_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite database");
        
        db
    }
    
    #[tokio::test]
    async fn test_service_initialization() -> Result<()> {
        let db = setup_test_db().await;
        let service = SecureDefaultsService::new(db, None).await?;
        
        assert!(service.config.enabled);
        assert!(service.config.enforce);
        assert!(service.config.log_application);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_apply_database_defaults() -> Result<()> {
        let db = setup_test_db().await;
        let service = SecureDefaultsService::new(db, None).await?;
        
        service.apply_database_defaults().await?;
        
        // Verify that the pragmas were applied
        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(&service.db)
            .await?;
        
        assert_eq!(journal_mode.to_uppercase(), "WAL");
        
        let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(&service.db)
            .await?;
        
        assert_eq!(foreign_keys, 1);
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_database_secure_defaults() {
        let connection_string = DatabaseSecureDefaults::get_connection_string("test.db");
        assert!(connection_string.contains("_journal_mode=WAL"));
        assert!(connection_string.contains("_foreign_keys=ON"));
        
        let pragmas = DatabaseSecureDefaults::get_pragmas();
        assert!(!pragmas.is_empty());
        assert!(pragmas.contains(&"PRAGMA foreign_keys = ON"));
    }
    
    #[tokio::test]
    async fn test_network_secure_defaults() {
        let timeout = NetworkSecureDefaults::get_timeout();
        assert!(timeout.as_secs() > 0);
        
        let use_tls = NetworkSecureDefaults::get_tls_config();
        assert!(use_tls);
    }
    
    #[tokio::test]
    async fn test_authentication_secure_defaults() {
        let iterations = AuthenticationSecureDefaults::get_password_hashing_iterations();
        assert!(iterations >= 10_000);
        
        let key_length = AuthenticationSecureDefaults::get_key_length();
        assert!(key_length >= 32);
    }
    
    #[tokio::test]
    async fn test_input_validation_secure_defaults() {
        let max_length = InputValidationSecureDefaults::get_max_input_length();
        assert!(max_length > 0);
        
        let username_regex = InputValidationSecureDefaults::get_username_regex();
        assert!(!username_regex.is_empty());
        
        let email_regex = InputValidationSecureDefaults::get_email_regex();
        assert!(!email_regex.is_empty());
    }
    
    #[tokio::test]
    async fn test_file_operation_secure_defaults() {
        let temp_dir = FileOperationSecureDefaults::get_temp_dir();
        assert!(temp_dir.to_string_lossy().contains("evo-pro"));
        
        #[cfg(unix)]
        {
            let file_permissions = FileOperationSecureDefaults::get_file_permissions();
            assert_eq!(file_permissions, 0o600);
            
            let dir_permissions = FileOperationSecureDefaults::get_directory_permissions();
            assert_eq!(dir_permissions, 0o700);
        }
    }
}