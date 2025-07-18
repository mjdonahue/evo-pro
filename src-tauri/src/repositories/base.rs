//! Base repository trait and implementations
//!
//! This module provides the base repository trait that defines the common
//! interface for all repositories, as well as some common implementations.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{Pool, Sqlite};
use std::fmt::Debug;
use uuid::Uuid;

use crate::error::Result;

/// Base repository trait that defines common operations for all repositories
#[async_trait]
pub trait Repository<T, F>
where
    T: Serialize + DeserializeOwned + Send + Sync + Debug,
    F: Send + Sync + Debug,
{
    /// Get the entity by ID
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<T>>;

    /// List entities with optional filtering
    async fn list(&self, filter: &F) -> Result<Vec<T>>;

    /// Create a new entity
    async fn create(&self, entity: &T) -> Result<T>;

    /// Update an existing entity
    async fn update(&self, entity: &T) -> Result<()>;

    /// Delete an entity by ID
    async fn delete(&self, id: &Uuid) -> Result<()>;

    /// Check if an entity exists by ID
    async fn exists(&self, id: &Uuid) -> Result<bool> {
        Ok(self.get_by_id(id).await?.is_some())
    }

    /// Count entities with optional filtering
    async fn count(&self, filter: &F) -> Result<i64>;

    /// Create multiple entities in a single transaction
    async fn batch_create(&self, entities: &[T]) -> Result<Vec<T>> {
        // Default implementation creates entities one by one
        // Implementations should override this with a more efficient approach
        let mut created_entities = Vec::with_capacity(entities.len());
        for entity in entities {
            created_entities.push(self.create(entity).await?);
        }
        Ok(created_entities)
    }

    /// Update multiple entities in a single transaction
    async fn batch_update(&self, entities: &[T]) -> Result<()> {
        // Default implementation updates entities one by one
        // Implementations should override this with a more efficient approach
        for entity in entities {
            self.update(entity).await?;
        }
        Ok(())
    }

    /// Delete multiple entities by ID in a single transaction
    async fn batch_delete(&self, ids: &[Uuid]) -> Result<()> {
        // Default implementation deletes entities one by one
        // Implementations should override this with a more efficient approach
        for id in ids {
            self.delete(id).await?;
        }
        Ok(())
    }

    /// Get multiple entities by ID in a single query
    async fn get_by_ids(&self, ids: &[Uuid]) -> Result<Vec<T>> {
        // Default implementation gets entities one by one
        // Implementations should override this with a more efficient approach
        let mut entities = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(entity) = self.get_by_id(id).await? {
                entities.push(entity);
            }
        }
        Ok(entities)
    }
}

/// Base repository implementation that provides common functionality
pub struct BaseRepository {
    /// Database connection pool
    pub pool: Pool<Sqlite>,
}

impl BaseRepository {
    /// Create a new base repository
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}
