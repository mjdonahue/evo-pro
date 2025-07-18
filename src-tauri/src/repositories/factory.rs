//! Repository factory for creating repositories
//!
//! This module provides a factory for creating repositories for different entity types.
//! It centralizes the creation of repositories and ensures that they all use the same
//! database connection pool.

use sqlx::{Pool, Sqlite};

use crate::repositories::task_repository::TaskRepository;

/// Repository factory for creating repositories
#[derive(Clone)]
pub struct RepositoryFactory {
    /// Database connection pool
    pool: Pool<Sqlite>,
}

impl RepositoryFactory {
    /// Create a new repository factory
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Get the database connection pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Create a task repository
    pub fn create_task_repository(&self) -> TaskRepository {
        TaskRepository::new(self.pool.clone())
    }

    // Add methods for creating other repositories as needed
}