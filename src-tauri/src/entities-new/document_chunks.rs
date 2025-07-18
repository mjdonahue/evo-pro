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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")] 
pub enum DocumentChunkType {
    Text = 0,
    Image = 1,
    Audio = 2,
    Video = 3,
    Other = 4,
}   

/// Document chunk model matching the SQLite schema
#[skip_serializing_none]
#[boilermates("CreateDocumentChunk")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunk {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub document_id: Uuid,
    pub parent_chunk_id: Option<Uuid>,
    pub content: String,
    pub content_hash: String,
    pub chunk_type: DocumentChunkType,
    pub order_index: i64,
    pub start_offset: i64,
    pub end_offset: i64,
    pub token_count: Option<i32>,
    pub embedding_id: Option<Uuid>,
    pub overlap_start: Option<i32>,
    pub overlap_end: Option<i32>,
    pub semantic_level: Option<i32>,
    #[specta(skip)]
    pub metadata: Option<Json<Value>>,
    #[boilermates(not_in("CreateDocumentChunk"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateDocumentChunk"))]
    #[specta(skip)]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for document chunk queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, specta::Type)]
pub struct DocumentChunkFilter {
    pub document_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub search_term: Option<String>,
    pub min_order_index: Option<i32>,
    pub max_order_index: Option<i32>,
    pub chunk_type: Option<DocumentChunkType>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub embedding_id: Option<Uuid>, 
    }

impl DatabaseManager {
    /// Create a new document chunk in the database
    #[instrument(skip(self))]
    pub async fn create_document_chunk(&self, chunk: &DocumentChunk) -> Result<DocumentChunk> {
        let id = Uuid::new_v4();
        debug!("Creating document chunk with ID: {}", id);

        let metadata = chunk.metadata.as_deref();

        let now = Utc::now();

        Ok(sqlx::query_as!(
            DocumentChunk,
            r#"INSERT INTO document_chunks (
                    id, workspace_id, document_id, parent_chunk_id, content, content_hash, chunk_type,
                    order_index, start_offset, end_offset, token_count, embedding_id, overlap_start,
                    overlap_end, semantic_level, metadata, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", workspace_id AS "workspace_id: _", document_id AS "document_id: _",
                    parent_chunk_id AS "parent_chunk_id: _", content, content_hash, chunk_type as "chunk_type: DocumentChunkType",
                    order_index, start_offset, end_offset, token_count, embedding_id AS "embedding_id: _",
                    overlap_start, overlap_end, semantic_level, metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _""#,
        id,
        chunk.workspace_id,
        chunk.document_id,
        chunk.parent_chunk_id,
        chunk.content,
        chunk.content_hash,
        chunk.chunk_type,
        chunk.order_index,
        chunk.start_offset,
        chunk.end_offset,
        chunk.token_count,
        chunk.embedding_id,
        chunk.overlap_start,
        chunk.overlap_end,
        chunk.semantic_level,
        metadata,
        now,
        now
        ).fetch_one(&self.pool)
        .await?)
    }

    /// Get a document chunk by ID
    #[instrument(err, skip(self))]
    pub async fn get_document_chunk_by_id(&self, id: &Uuid) -> Result<Option<DocumentChunk>> {
        debug!("Getting document chunk by ID: {}", id);

        Ok(sqlx::query_as!( 
            DocumentChunk,
            r#"SELECT 
                    id AS "id: _", workspace_id AS "workspace_id: _", document_id AS "document_id: _",
                    parent_chunk_id AS "parent_chunk_id: _", content, content_hash, chunk_type,
                    order_index, start_offset, end_offset, token_count, embedding_id AS "embedding_id: _",
                    overlap_start, overlap_end, semantic_level, metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM document_chunks 
                WHERE id = ?"#,
            id
        ).fetch_optional(&self.pool)
        .await?)

    }

    /// Get a document chunk by document ID and chunk index
    #[instrument(err,skip(self))]   
    pub async fn get_document_chunk_by_index(
        &self,
        document_id: &Uuid,
        chunk_index: i64,
    ) -> Result<Option<DocumentChunk>> {
        debug!(
            "Getting document chunk for document: {} at index: {}",
            document_id, chunk_index
        );

        Ok(sqlx::query_as!(
            DocumentChunk,
            r#"SELECT 
                    id AS "id: _", workspace_id AS "workspace_id: _", document_id AS "document_id: _",
                    parent_chunk_id AS "parent_chunk_id: _", content, content_hash, chunk_type as "chunk_type: DocumentChunkType",
                    order_index, start_offset, end_offset, token_count, embedding_id AS "embedding_id: _",
                    overlap_start, overlap_end, semantic_level, metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM document_chunks 
                WHERE document_id = ? AND order_index = ?"#,
            document_id,
            chunk_index
        ).fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter document chunks
    #[instrument(err, skip(self, filter))]
    pub async fn list_document_chunks(
        &self,
        filter: &DocumentChunkFilter,
    ) -> Result<Vec<DocumentChunk>> {
        debug!("Listing document chunks with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                    id AS "id: _", workspace_id AS "workspace_id: _", document_id AS "document_id: _",
                    parent_chunk_id AS "parent_chunk_id: _", content, content_hash, chunk_type,
                    order_index, start_offset, end_offset, token_count, embedding_id AS "embedding_id: _",
                    overlap_start, overlap_end, semantic_level, metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM document_chunks"#,
        );

        let mut add_where = add_where();

        if let Some(document_id) = &filter.document_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(document_id.to_string().as_str())?;
            qb.push("document_id = ");
            qb.push_bind(uuid);
        }

        if let Some(embedding_id) = filter.embedding_id {
            add_where(&mut qb);
            qb.push("embedding_id = ");
            qb.push_bind(embedding_id);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("content LIKE '%");
            qb.push_bind(search_term);
            qb.push("%'");
        }

        if let Some(min_order_index) = filter.min_order_index {
            add_where(&mut qb);
            qb.push("order_index >= ");
            qb.push_bind(min_order_index);
        }

        if let Some(max_order_index) = filter.max_order_index {
            add_where(&mut qb);
            qb.push("order_index <= ");
            qb.push_bind(max_order_index);
        }

        if !add_where.is_empty() {
            qb.push(" WHERE ");
            qb.push(add_where(&mut qb));
        }

        qb.push(" ORDER BY document_id, order_index ASC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        Ok(qb.build_query_as::<'_, DocumentChunk>()
        .fetch_all(&self.pool)
        .await?)
    }

    /// Update a document chunk
    #[instrument(err, skip(self))]
    pub async fn update_document_chunk(&self, chunk: &DocumentChunk) -> Result<bool> {
        debug!("Updating document chunk with ID: {}", chunk.id);    

        let rows = sqlx::query!(    
            r#"UPDATE document_chunks SET 
                document_id = ?, parent_chunk_id = ?, content = ?, content_hash = ?, chunk_type = ?,
                order_index = ?, start_offset = ?, end_offset = ?, token_count = ?, embedding_id = ?,
                overlap_start = ?, overlap_end = ?, semantic_level = ?, metadata = ?,
                updated_at = ?      
            WHERE id = ?"#,
            chunk.document_id,
            chunk.parent_chunk_id,
            chunk.content,
            chunk.content_hash,
            chunk.chunk_type,
            chunk.order_index,
            chunk.start_offset,
            chunk.end_offset,
            chunk.token_count,
            chunk.embedding_id,
            chunk.overlap_start,
            chunk.overlap_end,
            chunk.semantic_level,
            chunk.metadata,
            chunk.updated_at,
            chunk.id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Update document chunk content
    #[instrument(err, skip(self, content))]
    pub async fn update_document_chunk_content(&self, id: &Uuid, content: &str) -> Result<bool> {
        debug!("Updating content for document chunk: {}", id);

        let rows = sqlx::query!(
            r#"UPDATE document_chunks SET content = ? WHERE id = ?"#,
            content,
            id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Update document chunk metadata
    #[instrument(err, skip(self, metadata))]
    pub async fn update_document_chunk_metadata(&self, id: &Uuid, metadata: &str) -> Result<bool> {
        debug!("Updating metadata for document chunk: {}", id);

        let rows = sqlx::query!(
            r#"UPDATE document_chunks SET metadata = ? WHERE id = ?"#,
            metadata,
            id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Set document chunk embedding
    #[instrument(skip(self))]
    pub async fn set_document_chunk_embedding(&self, id: &Uuid, embedding_id: &Uuid) -> Result<bool> {
        debug!(
            "Setting embedding ID {} for document chunk: {}",
            embedding_id, id
        );

        let now = Utc::now();

        let rows = sqlx::query!(
            r#"UPDATE document_chunks SET embedding_id = ?, updated_at = ? WHERE id = ?"#,
            embedding_id,
            now,
            id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Set document chunk overlap
    #[instrument(skip(self))]
    pub async fn set_document_chunk_overlap(&self, id: &Uuid, overlap_start: i32, overlap_end: i32) -> Result<bool> {
        debug!("Setting overlap for document chunk: {}", id);

        let now = Utc::now();

        let rows = sqlx::query!(
            r#"UPDATE document_chunks SET overlap_start = ?, overlap_end = ?, updated_at = ? WHERE id = ?"#,
            overlap_start,
            overlap_end,
            now,
            id
        ).execute(&self.pool)
        .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Delete a document chunk by ID
    #[instrument(err, skip(self))]
    pub async fn delete_document_chunk(&self, id: &Uuid) -> Result<bool> {
        debug!("Deleting document chunk with ID: {}", id);

        let rows = sqlx::query!("DELETE FROM document_chunks WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        Ok(rows.rows_affected() > 0)
    }

    /// Delete all chunks for a document
    #[instrument(err, skip(self))]
    pub async fn delete_document_chunks_for_document(&self, document_id: &Uuid) -> Result<bool> {
        debug!("Deleting all chunks for document: {}", document_id);

        let rows = sqlx::query!("DELETE FROM document_chunks WHERE document_id = ?", document_id)
            .execute(&self.pool)
            .await?;

        Ok(rows.rows_affected() > 0)
    }
    
    /// Get all chunks for a document
    #[instrument(err, skip(self))]
    pub async fn get_document_chunks_for_document(
        &self,
        document_id: &Uuid,
    ) -> Result<Vec<DocumentChunk>> {
        debug!("Getting all chunks for document: {}", document_id);

        let filter = DocumentChunkFilter {
            document_id: Some(*document_id),
            ..Default::default()
        };

        self.list_document_chunks(&filter).await
    }

    /// Get non-embedded chunks
    #[instrument(err, skip(self))]
    pub async fn get_non_embedded_chunks(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<DocumentChunk>> {
        debug!("Getting non-embedded chunks");

        let filter = DocumentChunkFilter {
            embedding_id: None,
            limit,
            ..Default::default()
        };

        self.list_document_chunks(&filter).await
    }

    /// Count chunks for a document
    #[instrument(err, skip(self))]
    pub async fn count_chunks_for_document(&self, document_id: &Uuid) -> Result<i64> {
        debug!("Counting chunks for document: {}", document_id);

        Ok(sqlx::query!("SELECT COUNT(*) as count FROM document_chunks WHERE document_id = ?", document_id)
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count"))
    }

    /// Count embedded chunks for a document
    #[instrument(err, skip(self))]  
    pub async fn count_embedded_chunks_for_document(&self, document_id: &Uuid, embedding_id: &Uuid) -> Result<i64> { 
        debug!("Counting embedded chunks for document: {} with embedding ID: {}", document_id, embedding_id);

        let filter = DocumentChunkFilter {
            document_id: Some(*document_id),
            embedding_id: Some(embedding_id),
            ..Default::default()
        };

        self.count_chunks_for_document(document_id).await
    }
}
