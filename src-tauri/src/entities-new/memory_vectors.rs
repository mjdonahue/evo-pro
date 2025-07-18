use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

/// Memory vector source type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum MemoryVectorSourceType {
    Message = 0,
    Document = 1,
    Chunk = 2,
    Event = 3,
    Custom = 99,
}

impl TryFrom<i32> for MemoryVectorSourceType {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(MemoryVectorSourceType::Message),
            1 => Ok(MemoryVectorSourceType::Document),
            2 => Ok(MemoryVectorSourceType::Chunk),
            3 => Ok(MemoryVectorSourceType::Event),
            99 => Ok(MemoryVectorSourceType::Custom),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid MemoryVectorSourceType: {value}"
            ))),
        }
    }
}

/// Memory vector model matching the SQLite schema
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MemoryVector {
    pub id: Uuid,
    pub source_id: Uuid,
    pub source_type: MemoryVectorSourceType,
    pub text: String,
    pub vector: Vec<f32>,
    pub model_id: Option<Uuid>,
    pub metadata: String, // JSON object with additional metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for memory vector queries
#[derive(Debug, Default, Deserialize)]
pub struct MemoryVectorFilter {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_type: Option<MemoryVectorSourceType>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub created_before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new memory vector in the database
    #[instrument(skip(self))]
    pub async fn create_memory_vector(&self, memory_vector: &MemoryVector) -> Result<()> {
        debug!("Creating memory vector with ID: {}", memory_vector.id);

        // Convert vector to JSON string for database storage
        let vector_json = serde_json::to_string(&memory_vector.vector).map_err(|e| {
            AppError::DeserializationError(format!("Failed to serialize vector: {e}"))
        })?;

        let _result = sqlx::query(
            "INSERT INTO memory_vectors (
                id, source_id, source_type, text, vector, model_id,
                metadata, created_at, updated_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?
            )",
        )
        .bind(memory_vector.id)
        .bind(memory_vector.source_id)
        .bind(memory_vector.source_type as i32)
        .bind(&memory_vector.text)
        .bind(&vector_json)
        .bind(memory_vector.model_id)
        .bind(&memory_vector.metadata)
        .bind(memory_vector.created_at)
        .bind(memory_vector.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a memory vector by ID
    #[instrument(skip(self))]
    pub async fn get_memory_vector_by_id(&self, id: &Uuid) -> Result<Option<MemoryVector>> {
        debug!("Getting memory vector by ID: {}", id);

        let row = sqlx::query(
            r#"SELECT 
                id, source_id, source_type, text, vector, model_id,
                metadata, created_at, updated_at
            FROM memory_vectors WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let vector_json: String = row.get("vector");
            let vector: Vec<f32> = serde_json::from_str(&vector_json).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize vector: {e}"))
            })?;

            let memory_vector = MemoryVector {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                source_id: row
                    .get::<Vec<u8>, _>("source_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                source_type: MemoryVectorSourceType::try_from(
                    row.get::<i64, _>("source_type") as i32
                )?,
                text: row.get("text"),
                vector,
                model_id: row
                    .get::<Option<Vec<u8>>, _>("model_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(memory_vector))
        } else {
            Ok(None)
        }
    }

    /// List and filter memory vectors
    #[instrument(err, skip(self, filter))]
    pub async fn list_memory_vectors(
        &self,
        filter: &MemoryVectorFilter,
    ) -> Result<Vec<MemoryVector>> {
        debug!("Listing memory vectors with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                id, source_id, source_type, text, vector, model_id,
                metadata, created_at, updated_at
            FROM memory_vectors"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(source_id) = &filter.source_id {
            where_conditions.push(format!("source_id = '{source_id}'"));
        }

        if let Some(source_type) = filter.source_type {
            where_conditions.push(format!("source_type = {}", source_type as i64));
        }

        if let Some(model_id) = &filter.model_id {
            where_conditions.push(format!("model_id = '{model_id}'"));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(text LIKE '%{search_term}%' OR metadata LIKE '%{search_term}%')"
            ));
        }

        if let Some(created_after) = &filter.created_after {
            where_conditions.push(format!("created_at >= '{created_after}'"));
        }

        if let Some(created_before) = &filter.created_before {
            where_conditions.push(format!("created_at <= '{created_before}'"));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut memory_vectors = Vec::with_capacity(rows.len());
        for row in rows {
            let vector_json: String = row.get("vector");
            let vector: Vec<f32> = serde_json::from_str(&vector_json).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize vector: {e}"))
            })?;

            let memory_vector = MemoryVector {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                source_id: row
                    .get::<Vec<u8>, _>("source_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                source_type: MemoryVectorSourceType::try_from(
                    row.get::<i64, _>("source_type") as i32
                )?,
                text: row.get("text"),
                vector,
                model_id: row
                    .get::<Option<Vec<u8>>, _>("model_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            memory_vectors.push(memory_vector);
        }

        Ok(memory_vectors)
    }

    /// Update a memory vector
    #[instrument(err, skip(self))]
    pub async fn update_memory_vector(&self, memory_vector: &MemoryVector) -> Result<()> {
        debug!("Updating memory vector with ID: {}", memory_vector.id);

        // Convert vector to JSON string for database storage
        let vector_json = serde_json::to_string(&memory_vector.vector).map_err(|e| {
            AppError::DeserializationError(format!("Failed to serialize vector: {e}"))
        })?;

        let affected = sqlx::query(
            "UPDATE memory_vectors SET 
                source_id = ?, source_type = ?, text = ?, vector = ?, model_id = ?,
                metadata = ?, updated_at = ?
            WHERE id = ?",
        )
        .bind(memory_vector.source_id)
        .bind(memory_vector.source_type as i32)
        .bind(&memory_vector.text)
        .bind(&vector_json)
        .bind(memory_vector.model_id)
        .bind(&memory_vector.metadata)
        .bind(memory_vector.updated_at)
        .bind(memory_vector.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Memory vector with ID {} not found for update",
                memory_vector.id
            )));
        }

        Ok(())
    }

    /// Update memory vector metadata
    #[instrument(err, skip(self, metadata))]
    pub async fn update_memory_vector_metadata(&self, id: &Uuid, metadata: &str) -> Result<()> {
        debug!("Updating metadata for memory vector: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE memory_vectors SET metadata = ?, updated_at = ? WHERE id = ?")
                .bind(metadata)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Memory vector with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Delete a memory vector by ID
    #[instrument(err, skip(self))]
    pub async fn delete_memory_vector(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting memory vector with ID: {}", id);

        let affected = sqlx::query("DELETE FROM memory_vectors WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Memory vector with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Delete memory vectors by source ID and type
    #[instrument(err, skip(self))]
    pub async fn delete_memory_vectors_by_source(
        &self,
        source_id: &Uuid,
        source_type: MemoryVectorSourceType,
    ) -> Result<usize> {
        debug!(
            "Deleting memory vectors for source: {} of type: {:?}",
            source_id, source_type
        );

        let result =
            sqlx::query("DELETE FROM memory_vectors WHERE source_id = ? AND source_type = ?")
                .bind(source_id)
                .bind(source_type as i32)
                .execute(&self.pool)
                .await?;

        Ok(result.rows_affected() as usize)
    }

    /// Get memory vectors by source ID and type
    #[instrument(skip(self))]
    pub async fn get_memory_vectors_by_source(
        &self,
        source_id: &Uuid,
        source_type: MemoryVectorSourceType,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryVector>> {
        debug!(
            "Getting memory vectors for source: {} of type: {:?}",
            source_id, source_type
        );

        let filter = MemoryVectorFilter {
            source_id: Some(*source_id),
            source_type: Some(source_type),
            limit,
            ..Default::default()
        };

        self.list_memory_vectors(&filter).await
    }

    /// Get memory vectors by model ID
    #[instrument(skip(self))]
    pub async fn get_memory_vectors_by_model_id(
        &self,
        model_id: &Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryVector>> {
        debug!("Getting memory vectors for model: {}", model_id);

        let filter = MemoryVectorFilter {
            model_id: Some(*model_id),
            limit,
            ..Default::default()
        };

        self.list_memory_vectors(&filter).await
    }

    /// Search memory vectors by text
    #[instrument(skip(self, search_term))]
    pub async fn search_memory_vectors_by_text(
        &self,
        search_term: &str,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryVector>> {
        debug!("Searching memory vectors for text: {}", search_term);

        let filter = MemoryVectorFilter {
            search_term: Some(search_term.to_string()),
            limit,
            ..Default::default()
        };

        self.list_memory_vectors(&filter).await
    }

    /// Perform vector similarity search
    #[instrument(skip(self, query_vector))]
    pub async fn search_memory_vectors_by_similarity(
        &self,
        query_vector: &[f32],
        limit: Option<usize>,
        threshold: Option<f32>,
    ) -> Result<Vec<(MemoryVector, f32)>> {
        debug!("Performing vector similarity search");

        // Get all memory vectors
        let memory_vectors = self
            .list_memory_vectors(&MemoryVectorFilter::default())
            .await?;

        // Calculate cosine similarity for each vector
        let mut results: Vec<(MemoryVector, f32)> = memory_vectors
            .into_iter()
            .map(|mv| {
                let similarity = cosine_similarity(query_vector, &mv.vector);
                (mv, similarity)
            })
            .collect();

        // Filter by threshold if provided
        if let Some(threshold_value) = threshold {
            results.retain(|(_, similarity)| *similarity >= threshold_value);
        }

        // Sort by similarity (highest first)
        results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        // Apply limit if provided
        if let Some(limit_value) = limit {
            results.truncate(limit_value);
        }

        Ok(results)
    }

    /// Count memory vectors by source type
    #[instrument(skip(self))]
    pub async fn count_memory_vectors_by_source_type(
        &self,
        source_type: MemoryVectorSourceType,
    ) -> Result<i64> {
        debug!("Counting memory vectors of source type: {:?}", source_type);

        let row = sqlx::query("SELECT COUNT(*) as count FROM memory_vectors WHERE source_type = ?")
            .bind(source_type as i32)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count"))
    }
}

/// Helper function to calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::DatabaseManager;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::str::FromStr;
    use uuid::Uuid;

    fn create_test_vector(dimension: usize) -> Vec<f32> {
        let mut vector = Vec::with_capacity(dimension);
        for i in 0..dimension {
            vector.push((i as f32) / (dimension as f32));
        }
        vector
    }

    #[tokio::test]
    async fn test_create_and_get_memory_vector() {
        let db = DatabaseManager::setup_test_db().await;
        let vector_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let model_id = Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap();

        let now = Utc::now();
        let vector = MemoryVector {
            id: vector_id,
            source_id,
            source_type: MemoryVectorSourceType::Message,
            text: "This is a test message for vector embedding".to_string(),
            vector: create_test_vector(384), // Common embedding dimension
            model_id: Some(model_id),
            metadata: r#"{"importance": "high", "context": "test"}"#.to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the memory vector
        db.create_memory_vector(&vector)
            .await
            .expect("Failed to create memory vector");

        // Get the memory vector by ID
        let retrieved = db
            .get_memory_vector_by_id(&vector_id)
            .await
            .expect("Failed to get memory vector");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, vector_id);
        assert_eq!(retrieved.source_id, source_id);
        assert_eq!(retrieved.source_type, MemoryVectorSourceType::Message);
        assert_eq!(
            retrieved.text,
            "This is a test message for vector embedding"
        );
        assert_eq!(retrieved.vector.len(), 384);
        assert_eq!(retrieved.model_id, Some(model_id));
        assert_eq!(
            retrieved.metadata,
            r#"{"importance": "high", "context": "test"}"#
        );
    }

    #[tokio::test]
    async fn test_list_memory_vectors() {
        let db = DatabaseManager::setup_test_db().await;
        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let model_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        // Create multiple memory vectors
        for i in 1..=5 {
            let vector_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i + 2)).unwrap();

            let now = Utc::now();
            let vector = MemoryVector {
                id: vector_id,
                source_id,
                source_type: match i {
                    1 => MemoryVectorSourceType::Message,
                    2 => MemoryVectorSourceType::Document,
                    3 => MemoryVectorSourceType::Chunk,
                    4 => MemoryVectorSourceType::Event,
                    _ => MemoryVectorSourceType::Custom,
                },
                text: format!("Test content {}", i),
                vector: create_test_vector(384),
                model_id: Some(model_id),
                metadata: format!(r#"{{"index": {}}}"#, i),
                created_at: now,
                updated_at: now,
            };

            db.create_memory_vector(&vector)
                .await
                .expect("Failed to create memory vector");
        }

        // List all memory vectors
        let filter = MemoryVectorFilter::default();
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 5);

        // Filter by source_id
        let filter = MemoryVectorFilter {
            source_id: Some(source_id),
            ..Default::default()
        };
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 5);
        assert!(vectors.iter().all(|v| v.source_id == source_id));

        // Filter by source_type
        let filter = MemoryVectorFilter {
            source_type: Some(MemoryVectorSourceType::Message),
            ..Default::default()
        };
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].source_type, MemoryVectorSourceType::Message);

        // Filter by model_id
        let filter = MemoryVectorFilter {
            model_id: Some(model_id),
            ..Default::default()
        };
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 5);
        assert!(vectors.iter().all(|v| v.model_id == Some(model_id)));

        // Filter by search_term
        let filter = MemoryVectorFilter {
            search_term: Some("Test content 3".to_string()),
            ..Default::default()
        };
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].text, "Test content 3");

        // Test get_memory_vectors_by_source
        let vectors = db
            .get_memory_vectors_by_source(&source_id, MemoryVectorSourceType::Document, None)
            .await
            .expect("Failed to get memory vectors by source");
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].source_type, MemoryVectorSourceType::Document);

        // Test get_memory_vectors_by_model_id
        let vectors = db
            .get_memory_vectors_by_model_id(&model_id, None)
            .await
            .expect("Failed to get memory vectors by model");
        assert_eq!(vectors.len(), 5);
        assert!(vectors.iter().all(|v| v.model_id == Some(model_id)));

        // Test search_memory_vectors_by_text
        let vectors = db
            .search_memory_vectors_by_text("Test content 4", None)
            .await
            .expect("Failed to search memory vectors");
        assert_eq!(vectors.len(), 1);
        assert_eq!(vectors[0].text, "Test content 4");

        // Test count_memory_vectors_by_source_type
        let count = db
            .count_memory_vectors_by_source_type(MemoryVectorSourceType::Chunk)
            .await
            .expect("Failed to count memory vectors");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_update_memory_vector() {
        let db = DatabaseManager::setup_test_db().await;
        let vector_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let model_id = Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap();

        let now = Utc::now();
        let vector = MemoryVector {
            id: vector_id,
            source_id,
            source_type: MemoryVectorSourceType::Message,
            text: "Original text".to_string(),
            vector: create_test_vector(384),
            model_id: Some(model_id),
            metadata: r#"{"original": true}"#.to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the memory vector
        db.create_memory_vector(&vector)
            .await
            .expect("Failed to create memory vector");

        // Update the memory vector
        let new_source_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();
        let updated_vector = MemoryVector {
            id: vector_id,
            source_id: new_source_id,
            source_type: MemoryVectorSourceType::Document,
            text: "Updated text".to_string(),
            vector: create_test_vector(384), // Different vector values
            model_id: Some(model_id),
            metadata: r#"{"updated": true}"#.to_string(),
            created_at: vector.created_at,
            updated_at: Utc::now(),
        };

        db.update_memory_vector(&updated_vector)
            .await
            .expect("Failed to update memory vector");

        // Get the updated memory vector
        let retrieved = db
            .get_memory_vector_by_id(&vector_id)
            .await
            .expect("Failed to get memory vector")
            .unwrap();
        assert_eq!(retrieved.source_id, new_source_id);
        assert_eq!(retrieved.source_type, MemoryVectorSourceType::Document);
        assert_eq!(retrieved.text, "Updated text");
        assert_eq!(retrieved.metadata, r#"{"updated": true}"#);
    }

    #[tokio::test]
    async fn test_update_memory_vector_metadata() {
        let db = DatabaseManager::setup_test_db().await;
        let vector_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let vector = MemoryVector {
            id: vector_id,
            source_id,
            source_type: MemoryVectorSourceType::Message,
            text: "Test text".to_string(),
            vector: create_test_vector(384),
            model_id: None,
            metadata: r#"{"original": true}"#.to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the memory vector
        db.create_memory_vector(&vector)
            .await
            .expect("Failed to create memory vector");

        // Update just the metadata
        let new_metadata = r#"{"updated": true, "importance": "high", "tags": ["memory", "test"]}"#;
        db.update_memory_vector_metadata(&vector_id, new_metadata)
            .await
            .expect("Failed to update memory vector metadata");

        let retrieved = db
            .get_memory_vector_by_id(&vector_id)
            .await
            .expect("Failed to get memory vector")
            .unwrap();
        assert_eq!(retrieved.metadata, new_metadata);
        assert_eq!(retrieved.text, "Test text"); // Other fields should remain unchanged
    }

    #[tokio::test]
    async fn test_delete_memory_vector() {
        let db = DatabaseManager::setup_test_db().await;
        let vector_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let vector = MemoryVector {
            id: vector_id,
            source_id,
            source_type: MemoryVectorSourceType::Message,
            text: "Test text".to_string(),
            vector: create_test_vector(384),
            model_id: None,
            metadata: r#"{"test": true}"#.to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the memory vector
        db.create_memory_vector(&vector)
            .await
            .expect("Failed to create memory vector");

        // Delete the memory vector
        db.delete_memory_vector(&vector_id)
            .await
            .expect("Failed to delete memory vector");

        // Try to get the deleted memory vector
        let retrieved = db
            .get_memory_vector_by_id(&vector_id)
            .await
            .expect("Failed to query memory vector");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_memory_vectors_by_source() {
        let db = DatabaseManager::setup_test_db().await;
        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        // Create multiple memory vectors for the same source
        for i in 1..=3 {
            let vector_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i + 1)).unwrap();

            let now = Utc::now();
            let vector = MemoryVector {
                id: vector_id,
                source_id,
                source_type: MemoryVectorSourceType::Message,
                text: format!("Test text {}", i),
                vector: create_test_vector(384),
                model_id: None,
                metadata: r#"{"test": true}"#.to_string(),
                created_at: now,
                updated_at: now,
            };

            db.create_memory_vector(&vector)
                .await
                .expect("Failed to create memory vector");
        }

        // Verify memory vectors exist
        let filter = MemoryVectorFilter {
            source_id: Some(source_id),
            source_type: Some(MemoryVectorSourceType::Message),
            ..Default::default()
        };
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 3);

        // Delete all memory vectors for the source
        let deleted_count = db
            .delete_memory_vectors_by_source(&source_id, MemoryVectorSourceType::Message)
            .await
            .expect("Failed to delete memory vectors by source");
        assert_eq!(deleted_count, 3);

        // Verify memory vectors are deleted
        let vectors = db
            .list_memory_vectors(&filter)
            .await
            .expect("Failed to list memory vectors");
        assert_eq!(vectors.len(), 0);
    }

    #[tokio::test]
    async fn test_vector_similarity_search() {
        let db = DatabaseManager::setup_test_db().await;

        // Create test vectors with different similarity to the query
        let vectors = [
            // Very similar to query (0.99)
            (
                Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap(),
                vec![0.1, 0.2, 0.3, 0.4, 0.5],
                "Very similar text".to_string(),
            ),
            // Somewhat similar to query (0.85)
            (
                Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap(),
                vec![0.15, 0.25, 0.35, 0.45, 0.55],
                "Somewhat similar text".to_string(),
            ),
            // Less similar to query (0.5)
            (
                Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap(),
                vec![0.5, 0.4, 0.3, 0.2, 0.1],
                "Less similar text".to_string(),
            ),
            // Not similar to query (0.1)
            (
                Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap(),
                vec![-0.1, -0.2, -0.3, -0.4, -0.5],
                "Not similar text".to_string(),
            ),
        ];

        let source_id = Uuid::from_str("00000000-0000-0000-0000-000000000005").unwrap();
        let now = Utc::now();

        // Create memory vectors
        for (id, vec, text) in &vectors {
            let memory_vector = MemoryVector {
                id: *id,
                source_id,
                source_type: MemoryVectorSourceType::Message,
                text: text.clone(),
                vector: vec.clone(),
                model_id: None,
                metadata: r#"{"test": true}"#.to_string(),
                created_at: now,
                updated_at: now,
            };

            db.create_memory_vector(&memory_vector)
                .await
                .expect("Failed to create memory vector");
        }

        // Query vector (very similar to the first vector)
        let query_vector = vec![0.11, 0.21, 0.31, 0.41, 0.51];

        // Search with no threshold
        let results = db
            .search_memory_vectors_by_similarity(&query_vector, None, None)
            .await
            .expect("Failed to search by similarity");

        assert_eq!(results.len(), 4);

        // Results should be ordered by similarity (highest first)
        assert_eq!(results[0].0.id, vectors[0].0); // Most similar
        assert_eq!(results[1].0.id, vectors[1].0); // Second most similar
        assert_eq!(results[2].0.id, vectors[2].0); // Third most similar
        assert_eq!(results[3].0.id, vectors[3].0); // Least similar

        // Search with threshold
        let results = db
            .search_memory_vectors_by_similarity(&query_vector, None, Some(0.8))
            .await
            .expect("Failed to search by similarity with threshold");

        assert_eq!(results.len(), 2); // Only the two most similar vectors should be returned
        assert!(results[0].1 >= 0.8);
        assert!(results[1].1 >= 0.8);

        // Search with limit
        let results = db
            .search_memory_vectors_by_similarity(&query_vector, Some(2), None)
            .await
            .expect("Failed to search by similarity with limit");

        assert_eq!(results.len(), 2); // Only the two most similar vectors should be returned
        assert_eq!(results[0].0.id, vectors[0].0);
        assert_eq!(results[1].0.id, vectors[1].0);
    }

    #[test]
    fn test_cosine_similarity() {
        // Test identical vectors
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        assert!((cosine_similarity(&vec1, &vec2) - 1.0).abs() < 1e-6);

        // Test orthogonal vectors
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&vec1, &vec2) - 0.0).abs() < 1e-6);

        // Test opposite vectors
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![-1.0, -2.0, -3.0];
        assert!((cosine_similarity(&vec1, &vec2) + 1.0).abs() < 1e-6);

        // Test different length vectors
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0];
        assert_eq!(cosine_similarity(&vec1, &vec2), 0.0);

        // Test empty vectors
        let vec1: Vec<f32> = vec![];
        let vec2: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&vec1, &vec2), 0.0);

        // Test zero magnitude vector
        let vec1 = vec![0.0, 0.0, 0.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        assert_eq!(cosine_similarity(&vec1, &vec2), 0.0);
    }
}
