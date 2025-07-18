use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::prelude::FromRow;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

/// File type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Text = 0,
    Image = 1,
    Video = 2,
    Audio = 3,
    PDF = 4,
}

impl Default for FileType {
    fn default() -> Self {
        Self::Text
    }
}

impl TryFrom<i32> for FileType {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(FileType::Text),
            1 => Ok(FileType::Image),
            2 => Ok(FileType::Video),
            3 => Ok(FileType::Audio),
            4 => Ok(FileType::PDF),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid FileType: {value}"
            ))),
        }
    }
}

/// File model matching the SQLite schema
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub owner_id: Uuid,
    pub title: String,
    pub content: String,
    pub type_: FileType,
    pub file_path: String,
    pub url: String,
    pub hash: String,
    pub is_indexed: bool,
    pub is_embedded: bool,
    pub metadata: serde_json::Value, // JSON object
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for File {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            owner_id: Uuid::new_v4(),
            title: String::new(),
            content: String::new(),
            type_: FileType::default(),
            file_path: String::new(),
            url: String::new(),
            hash: String::new(),
            is_indexed: false,
            is_embedded: false,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}

impl DatabaseManager {
    /// Create a new file in the database
    /// This method automatically generates `created_at`, `updated_at` (RFC3339 strings),
    /// and `modified` (Unix ms) timestamps.
    /// The caller is responsible for setting the `id` field on the `file` struct before calling this method.   
    #[instrument(skip(self))]
    pub async fn create_file(&self, file: &File) -> Result<()> {
        debug!("Creating file with ID: {}", file.id);

        let metadata_str = serde_json::to_string(&file.metadata).map_err(|e| {
            AppError::DeserializationError(format!("Failed to serialize metadata: {e}"))
        })?;

        let _result = sqlx::query(
            "INSERT INTO files (id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(file.id)
        .bind(file.workspace_id)
        .bind(file.owner_id)
        .bind(&file.title)
        .bind(&file.content)
        .bind(file.type_ as i32)
        .bind(&file.file_path)
        .bind(&file.url)
        .bind(&file.hash)
        .bind(file.is_indexed)
        .bind(file.is_embedded)
        .bind(&metadata_str)
        .bind(file.created_at)
        .bind(file.updated_at)
        .execute(&self.pool)    
        .await?;

        Ok(())
    }

    /// Get a file by ID
    #[instrument(skip(self))]
    pub async fn get_file_by_id(&self, id: &Uuid) -> Result<Option<File>> {
        debug!("Getting file by ID: {}", id);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Get a file by title
    #[instrument(skip(self))]
    pub async fn get_file_by_title(&self, title: &str) -> Result<Option<File>> {
        debug!("Getting file by title: {}", title);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE title = ?"
        )
        .bind(title)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Get a file by content
    #[instrument(skip(self))]
    pub async fn get_file_by_content(&self, content: &str) -> Result<Option<File>> {
        debug!("Getting file by content: {}", content);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE content = ?"
        )
        .bind(content)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Get a file by metadata
    #[instrument(skip(self))]
    pub async fn get_file_by_metadata(&self, metadata: &str) -> Result<Option<File>> {
        debug!("Getting file by metadata: {}", metadata);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE metadata = ?"
        )
        .bind(metadata)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Get a file by type
    #[instrument(skip(self))]
    pub async fn get_file_by_type(&self, type_: &FileType) -> Result<Option<File>> {
        debug!("Getting file by type: {:?}", type_);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE type_ = ?"
        )
        .bind(*type_ as i32)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Get a file by created_at
    #[instrument(skip(self))]
    pub async fn get_file_by_created_at(&self, created_at: &DateTime<Utc>) -> Result<Option<File>> {
        debug!("Getting file by created_at: {}", created_at);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE created_at = ?"
        )
        .bind(created_at)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }

    /// Get a file by updated_at
    #[instrument(skip(self))]
    pub async fn get_file_by_updated_at(&self, updated_at: &DateTime<Utc>) -> Result<Option<File>> {
        debug!("Getting file by updated_at: {}", updated_at);

        let row = sqlx::query(
            "SELECT id, workspace_id, owner_id, title, content, type_, file_path, url, hash, is_indexed, is_embedded, metadata, created_at, updated_at FROM files WHERE updated_at = ?"
        )
        .bind(updated_at)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let metadata_str: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_str).map_err(|e| {
                AppError::DeserializationError(format!("Failed to deserialize metadata: {e}"))
            })?;

            let file = File {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                workspace_id: row
                    .get::<Vec<u8>, _>("workspace_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                owner_id: row
                    .get::<Vec<u8>, _>("owner_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                title: row.get("title"),
                content: row.get("content"),
                type_: FileType::try_from(row.get::<i64, _>("type_") as i32)?,
                file_path: row.get("file_path"),
                url: row.get("url"),
                hash: row.get("hash"),
                is_indexed: row.get::<i64, _>("is_indexed") != 0,
                is_embedded: row.get::<i64, _>("is_embedded") != 0,
                metadata,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(file))
        } else {
            Ok(None)
        }
    }
}
