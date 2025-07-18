use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

/// Document type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum DocumentType {
    Text = 0,
    Pdf = 1,
    Image = 2,
    Audio = 3,
    Video = 4,
    Webpage = 5,
    Email = 6,
    Other = 7,
}

/// Document model matching the SQLite schema
/// #[skip_serializing_none]
#[boilermates("CreateDocument")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub owner_id: Option<Uuid>,
    pub name: String,
    #[boilermates(not_in("CreateDocument"))]
    #[specta(skip)]
    pub description: Option<String>,
    pub document_type: DocumentType,
    pub mime_type: String,
    pub size_bytes: i64,
    pub content: Option<String>,
    #[specta(skip)]
    pub metadata: Option<Json<Value>>, // JSON object with metadata
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub hash: Option<String>,
    pub is_indexed: bool,
    pub is_embedded: bool,
    #[boilermates(not_in("CreateDocument"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateDocument"))]
    #[specta(skip)]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for document queries
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type, Default)]
pub struct DocumentFilter {
    pub workspace_id: Option<String>,
    pub owner_id: Option<String>,
    pub document_type: Option<DocumentType>,
    pub is_indexed: Option<bool>,
    pub is_embedded: Option<bool>,
    pub search_term: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new document in the database
    #[instrument(err, skip(self))]
    pub async fn create_document(&self, document: &Document) -> Result<Document> {
        let id = Uuid::new_v4();
        debug!("Creating document with ID: {}", id);

        Ok(sqlx::query_as!(
            Document,
            r#"INSERT INTO documents (
                    id, workspace_id, owner_id, name, description, document_type, mime_type,
                    size_bytes, content, metadata, file_path, url, hash, is_indexed, is_embedded,
                    created_at, updated_at
                ) VALUES (  
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", workspace_id AS "workspace_id: _", owner_id AS "owner_id: _",
                    name, description, document_type AS "document_type: _", mime_type, size_bytes,
                    content, metadata AS "metadata: _", file_path, url, hash, is_indexed, is_embedded,
                    created_at AS "created_at: _", updated_at AS "updated_at: _""#, 
            id,
            document.workspace_id,
            document.owner_id,
            document.name,
            document.description,
            document.document_type,  
            document.mime_type,
            document.size_bytes,
            document.content,
            document.metadata,
            document.file_path,
            document.url,  
            document.hash,
            document.is_indexed,
            document.is_embedded,
            document.created_at,
            document.updated_at
        ).fetch_one(&self.pool)
            .await?)
    }

    /// Get a document by ID
    #[instrument(err, skip(self))]
    pub async fn get_document_by_id(&self, id: &Uuid) -> Result<Option<Document>> {
        debug!("Getting document by ID: {}", id);

        Ok(sqlx::query_as!(
            Document,
            r#"SELECT 
                    id AS "id: _", workspace_id AS "workspace_id: _", owner_id AS "owner_id: _",
                    name, description, document_type AS "document_type: _", mime_type, size_bytes,
                    content, metadata AS "metadata: _", file_path, url, hash, is_indexed, is_embedded,
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM documents WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get a document by hash
    #[instrument(err, skip(self))]
    pub async fn get_document_by_hash(&self, hash: &str) -> Result<Option<Document>> {
        debug!("Getting document by hash: {}", hash);

 
        Ok(sqlx::query_as!(
            Document,
            r#"SELECT 
                    id AS "id: _", workspace_id AS "workspace_id: _", owner_id AS "owner_id: _",
                    name, description, document_type AS "document_type: _", mime_type, size_bytes,
                    content, metadata AS "metadata: _", file_path, url, hash, is_indexed, is_embedded,
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM documents WHERE hash = ?"#,
            hash
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter documents
    #[instrument(err, skip(self, filter))]
    pub async fn list_documents(&self, filter: &DocumentFilter) -> Result<Vec<Document>> {
        debug!("Listing documents with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, workspace_id, owner_id, name, description, document_type, mime_type, size_bytes,
               content, metadata, file_path, url, hash, is_indexed, is_embedded,
               created_at, updated_at 
               FROM documents"#,
        );

        let mut add_where = add_where();

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(workspace_id)?;
            qb.push("workspace_id = ");
            qb.push_bind(uuid);
        }

        if let Some(owner_id) = &filter.owner_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(owner_id)?;
            qb.push("owner_id = ");
            qb.push_bind(uuid);
        }

        if let Some(document_type) = filter.document_type {
            add_where(&mut qb);
            qb.push("document_type = ");
            qb.push_bind(document_type as i64);
        }

        if let Some(is_indexed) = filter.is_indexed {
            add_where(&mut qb);
            qb.push("is_indexed = ");
            qb.push_bind(if is_indexed { 1 } else { 0 });
        }

        if let Some(is_embedded) = filter.is_embedded {
            add_where(&mut qb);
            qb.push("is_embedded = ");
            qb.push_bind(if is_embedded { 1 } else { 0 });
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(name LIKE '%");
            qb.push_bind(search_term);
            qb.push("%' OR description LIKE '%");
            qb.push_bind(search_term);
            qb.push("%' OR content LIKE '%");
            qb.push_bind(search_term);
            qb.push("%')");
        }

        if let Some(created_after) = &filter.created_after {
            add_where(&mut qb);
            qb.push("created_at >= ");
            qb.push_bind(created_after);
        }

        if let Some(created_before) = &filter.created_before {
            add_where(&mut qb);
            qb.push("created_at <= ");
            qb.push_bind(created_before);
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

        Ok(qb
            .build_query_as::<'_, Document>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update a document
    #[instrument(err, skip(self))]
    pub async fn update_document(&self, document: &Document) -> Result<bool> {
        debug!("Updating document with ID: {}", document.id);

      
        let rows = sqlx::query!(
            "UPDATE documents SET 
                workspace_id = ?, owner_id = ?, name = ?, description = ?, document_type = ?,
                mime_type = ?, size_bytes = ?, content = ?, metadata = ?, file_path = ?,
                url = ?, hash = ?, is_indexed = ?, is_embedded = ?, updated_at = ?
            WHERE id = ?",
            document.workspace_id,
            document.owner_id,
            document.name,
            document.description,
            document.document_type,
            document.mime_type,
            document.size_bytes,
            document.content,
            document.metadata,
            document.file_path,
            document.url,
            document.hash,
            document.is_indexed,
            document.is_embedded,
            document.updated_at,
            document.id
        )
        .execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }


    /// Update document content
    #[instrument(err, skip(self, content))]
    pub async fn update_document_content(&self, id: &Uuid, content: &str) -> Result<bool> {
        debug!("Updating content for document: {}", id);

        let now = Utc::now();

        let rows = sqlx::query("UPDATE documents SET content = ?, updated_at = ? WHERE id = ?")
            .bind(content)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if rows.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document with ID {id} not found for update"
            )));
        }

        Ok(rows.rows_affected() > 0)
    }

    /// Update document metadata
    #[instrument(err, skip(self, metadata))]
    pub async fn update_document_metadata(&self, id: &Uuid, metadata: &str) -> Result<bool> {
        debug!("Updating metadata for document: {}", id);

        let now = Utc::now();

        let rows = sqlx::query!(
            r#"UPDATE documents SET metadata = ?, updated_at = ? WHERE id = ?"#,
                metadata,
                now,
                id
            )
            .execute(&self.pool)
            .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Set document indexed status
    #[instrument(err, skip(self))]
    pub async fn set_document_indexed(&self, id: &Uuid, is_indexed: bool) -> Result<bool> {
        debug!("Setting document {} indexed status to {}", id, is_indexed);

        let now = Utc::now();

        let rows = sqlx::query!(
            r#"UPDATE documents SET is_indexed = ?, updated_at = ? WHERE id = ?"#,
            is_indexed,
            now,
            id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Set document embedded status
    #[instrument(err, skip(self))]
    pub async fn set_document_embedded(&self, id: &Uuid, is_embedded: bool) -> Result<()> {
        debug!("Setting document {} embedded status to {}", id, is_embedded);

        let now = Utc::now();

        let rows = sqlx::query!(
            r#"UPDATE documents SET is_embedded = ?, updated_at = ? WHERE id = ?"#,
            is_embedded,
            now,
            id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Delete a document by ID
    #[instrument(err, skip(self))]
    pub async fn delete_document(&self, id: &Uuid) -> Result<bool> {
        debug!("Deleting document with ID: {}", id);

        let rows = sqlx::query!("DELETE FROM documents WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Get documents for a workspace
    #[instrument(err, skip(self))]
    pub async fn get_documents_for_workspace(
        &self,
        workspace_id: &Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<Document>> {
        debug!("Getting documents for workspace: {}", workspace_id);

        let filter = DocumentFilter {
            workspace_id: Some(workspace_id.to_string()),
            limit,
            ..Default::default()
        };

        self.list_documents(&filter).await
    }

    /// Get documents for an owner
    #[instrument(err, skip(self))]
    pub async fn get_documents_for_owner(
        &self,
        owner_id: &Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<Document>> {
        debug!("Getting documents for owner: {}", owner_id);
        let filter = DocumentFilter {
            owner_id: Some(owner_id.to_string()),
            limit,
            ..Default::default()
        };

        self.list_documents(&filter).await
    }

    /// Count documents by type
    #[instrument(err, skip(self))]
    pub async fn count_documents_by_type(
        &self,
        document_type: Option<DocumentType>,
    ) -> Result<i64> {
        debug!("Counting documents by type: {:?}", document_type);

        let count = match document_type {
            Some(doc_type) => {
                let row = sqlx::query!(
                    r#"SELECT COUNT(*) as count FROM documents WHERE document_type = ?"#,
                    doc_type as i32
                ).fetch_one(&self.pool)
                .await?;
                row.get::<i64, _>("count")
            },
            None => {
                let row = sqlx::query!("SELECT COUNT(*) as count FROM documents")
                    .fetch_one(&self.pool)
                    .await?;

                row.get::<i64, _>("count")
            }
        };

        Ok(count)
    }
}
