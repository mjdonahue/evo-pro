// Core storage modules
pub mod db;
pub mod manager;
pub use manager::StorageManager;
pub mod vector;
pub mod migration;
pub mod migration_test;
pub mod retention;

// Backend modules - commented out for future use when Agent entity is implemented
// pub mod r#trait;
// pub use r#trait::StorageBackend;
// pub mod postgres_backend;
// pub mod sqlite_backend;
