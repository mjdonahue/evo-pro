use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use serde_json::Value;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, specta::Type)]
pub enum AttachmentType {
    Image = 0,
    Video = 1,
    Audio = 2,
    File = 3,
    Link = 4,
}

#[boilermates("CreateAttachment")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    #[boilermates(not_in("CreateAttachment"))]
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub message_id: Uuid,
    pub file_id: Uuid,
    pub type_: AttachmentType,
    pub url: String,
    pub metadata: Option<Json<Value>>, // JSON - mimeType, dimensions, etc.
    #[boilermates(not_in("CreateAttachment"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateAttachment"))]
    #[specta(skip)]
    pub updated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, specta::Type)]
pub struct AttachmentFilter {
    pub workspace_id: Option<Uuid>,
    pub message_id: Option<Uuid>,
    pub file_id: Option<Uuid>,
    pub type_: Option<AttachmentType>,
    pub search_term: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl DatabaseManager {
    /// Create a new attachment
    #[instrument(skip(self))]
    pub async fn create_attachment(&self, attachment: &CreateAttachment) -> Result<Attachment> {
        let id = Uuid::new_v4();
        debug!("Creating attachment with ID: {}", id);
        let now = Utc::now();
        let metadata = attachment.metadata.as_deref();

        Ok(sqlx::query_as!(
            Attachment,
            r#"INSERT INTO attachments (
                    id, workspace_id, message_id, file_id, type, url, metadata, created_at, updated_at
            ) VALUES (
            ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING 
               id as "id: _", workspace_id as "workspace_id: _", message_id as "message_id: _", file_id as "file_id: _", 
               type as "type_: _", url, metadata as "metadata: _",
               created_at as "created_at: _", updated_at as "updated_at: _""#,
            id,
            attachment.workspace_id,
            attachment.message_id,
            attachment.file_id,
            attachment.type_,
            attachment.url,
            metadata,
            now,
            now,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get attachment by ID
    #[instrument(skip(self))]
    pub async fn get_attachment_by_id(&self, id: &Uuid) -> Result<Option<Attachment>> {
        debug!("Getting attachment by ID: {}", id);

        Ok(sqlx::query_as!(
            Attachment,
            r#"SELECT 
                    id as "id: _", workspace_id as "workspace_id: _", message_id as "message_id: _", 
                    file_id as "file_id: _", type as "type_: _", url, metadata as "metadata: _",
                    created_at as "created_at: _", updated_at as "updated_at: _"
                FROM attachments WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List attachments with filtering
    #[instrument(skip(self))]
    pub async fn list_attachments(&self, filter: &AttachmentFilter) -> Result<Vec<Attachment>> {
        debug!("Listing attachments with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, workspace_id, message_id, file_id, type AS type_, url, metadata,
                    created_at, updated_at
             FROM attachments"#,
        );

        let mut add_where = add_where();

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            qb.push("workspace_id = ");
            qb.push_bind(workspace_id);
        }

        if let Some(message_id) = &filter.message_id {
            add_where(&mut qb);
            qb.push("message_id = ");
            qb.push_bind(message_id);
        }

        if let Some(file_id) = &filter.file_id {
            add_where(&mut qb);
            qb.push("file_id = ");
            qb.push_bind(file_id);
        }

        if let Some(type_) = &filter.type_ {
            add_where(&mut qb);
            qb.push("type = ");
            qb.push_bind(*type_ as i32);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("url LIKE ");
            qb.push_bind(format!("%{search_term}%"));
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

        Ok(qb.build_query_as().fetch_all(&self.pool).await?)
    }

    /// Update attachment
    #[instrument(skip(self))]
    pub async fn update_attachment(self, attachment: &Attachment) -> Result<()> {
        debug!("Updating attachment with ID: {}", attachment.id);
        let result = sqlx::query!(
            "UPDATE attachments SET
                workspace_id = ?, message_id = ?, file_id = ?, type = ?,
                url = ?, metadata = ?, updated_at = ?
             WHERE id = ?",
            attachment.workspace_id,
            attachment.message_id,
            attachment.file_id,
            attachment.type_,
            attachment.url,
            attachment.metadata,
            attachment.updated_at,
            attachment.id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Attachment with ID {} not found",
                attachment.id
            )));
        }

        Ok(())
    }

    /// Delete attachment
    #[instrument(skip(self))]
    pub async fn delete_attachment(self, id: &Uuid) -> Result<()> {
        debug!("Deleting attachment with ID: {}", id);
        let result = sqlx::query!("DELETE FROM attachments WHERE id = ?", id,)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Attachment with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get attachments by message
    pub async fn get_by_message(self, message_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: Some(*message_id),
            file_id: None,
            type_: Some(AttachmentType::File),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get attachments by workspace
    pub async fn get_by_workspace(self, workspace_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: Some(*workspace_id),
            message_id: None,
            file_id: None,
            type_: Some(AttachmentType::File),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get attachments by file
    pub async fn get_by_file(self, file_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: None,
            file_id: Some(*file_id),
            type_: Some(AttachmentType::File),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get attachments by type
    pub async fn get_by_type(self, type_: AttachmentType) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: None,
            file_id: None,
            type_: Some(type_),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get image attachments by message
    pub async fn get_images_by_message(self, message_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: Some(*message_id),
            file_id: None,
            type_: Some(AttachmentType::Image),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get video attachments by message
    pub async fn get_videos_by_message(self, message_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: Some(*message_id),
            file_id: None,
            type_: Some(AttachmentType::Video),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get audio attachments by message
    pub async fn get_audio_by_message(self, message_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: Some(*message_id),
            file_id: None,
            type_: Some(AttachmentType::Audio),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get file attachments by message
    pub async fn get_files_by_message(self, message_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: Some(*message_id),
            file_id: None,
            type_: Some(AttachmentType::File),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Get link attachments by message
    pub async fn get_links_by_message(self, message_id: &Uuid) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: Some(*message_id),
            file_id: None,
            type_: Some(AttachmentType::Link),
            search_term: None,
            limit: None,
            offset: None,
        };

        self.list_attachments(&filter).await
    }

    /// Update attachment URL
    pub async fn update_url(self, id: &Uuid, url: &str) -> Result<()> {
        let now = Utc::now();

        let result = sqlx::query!(
            "UPDATE attachments SET url = ?, updated_at = ? WHERE id = ?",
            url,
            now,
            id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Attachment with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Update attachment metadata
    pub async fn update_metadata(self, id: &Uuid, metadata: Option<&str>) -> Result<()> {
        let now = Utc::now();

        let result = sqlx::query!(
            "UPDATE attachments SET metadata = ?, updated_at = ? WHERE id = ?",
            metadata,
            now,
            id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Attachment with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Count attachments by message
    #[instrument(skip(self))]
    pub async fn count_by_message(self, message_id: &Uuid) -> Result<i64> {
        Ok(sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM attachments WHERE message_id = ?",
            message_id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Count attachments by type for message
    #[instrument(skip(self))]
    pub async fn count_by_type_and_message(
        self,
        message_id: &Uuid,
        type_: AttachmentType,
    ) -> Result<i64> {
        Ok(sqlx::query_scalar!(
            "SELECT COUNT(*) as count FROM attachments WHERE message_id = ? AND type = ?",
            message_id,
            type_,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete all attachments for a message
    #[instrument(skip(self))]
    pub async fn delete_by_message(self, message_id: &Uuid) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM attachments WHERE message_id = ?", message_id,)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Delete all attachments for a file
    #[instrument(skip(self))]
    pub async fn delete_by_file(self, file_id: &Uuid) -> Result<u64> {
        let result = sqlx::query!("DELETE FROM attachments WHERE file_id = ?", file_id,)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Search attachments by URL
    #[instrument(skip(self))]
    pub async fn search_by_url(
        self,
        search_term: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Attachment>> {
        let filter = AttachmentFilter {
            workspace_id: None,
            message_id: None,
            file_id: None,
            type_: Some(AttachmentType::File),
            search_term: Some(search_term.to_string()),
            limit,
            offset: None,
        };

        self.list_attachments(&filter).await
    }
}
