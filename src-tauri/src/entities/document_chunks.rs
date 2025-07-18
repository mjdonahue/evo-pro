use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use serde_json::Value;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

/// Document chunk type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]  
pub enum DocumentChunkType {
    Semantic = 0,
    Fixed = 1,
    Paragraph = 2,
    Sentence = 3,
    Other = 4,    
}

/// Document chunk semantic level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum DocumentChunkSemanticLevel {
    Header = 0,
    Paragraph = 1,
    Sentence = 2,
    Other = 3,
}
/// Document chunk model matching the SQLite schema
#[boilermates("CreateDocumentChunk")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DocumentChunk {
    #[boilermates(not_in("CreateDocumentChunk"))]
    pub id: Uuid,
    pub document_id: Uuid,
    pub parent_chunk_id: Option<Uuid>,
    pub content: String,
    pub content_hash: String,
    pub chunk_type: DocumentChunkType,
    pub order_index: i64,
    pub semantic_level: Option<DocumentChunkSemanticLevel>,
    pub metadata: Option<Json<Value>>, // JSON object with metadata
    #[boilermates(not_in("CreateDocumentChunk"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateDocumentChunk"))]
    pub updated_at: DateTime<Utc>,
    pub embedding_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
}

/// Additional filtering options for document chunk queries
#[skip_serializing_none]    
#[derive(Debug, Default, Deserialize)]
pub struct DocumentChunkFilter {
    pub document_id: Option<Uuid>,
    pub is_embedded: Option<bool>,
    pub search_term: Option<String>,
    pub min_chunk_index: Option<i64>,
    pub max_chunk_index: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new document chunk in the database
    #[instrument(skip(self))]
    pub async fn create_document_chunk(&self, chunk: &CreateDocumentChunk) -> Result<DocumentChunk> {
        let id = Uuid::new_v4();
        debug!("Creating document chunk with ID: {}", id);
        let metadata = chunk.metadata.as_deref();
        let now = Utc::now();

        Ok(sqlx::query_as!(
            DocumentChunk,
            r#"INSERT INTO document_chunks (
                    id, document_id, parent_chunk_id, content, content_hash, chunk_type,
                    order_index, semantic_level, metadata, embedding_id, workspace_id
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING 
                    id AS "id: _", document_id AS "document_id: _", parent_chunk_id AS "parent_chunk_id: _",
                    content, content_hash, chunk_type AS "chunk_type: DocumentChunkType",
                    order_index, semantic_level AS "semantic_level: DocumentChunkSemanticLevel",
                    metadata AS "metadata: _",  created_at AS "created_at: _", updated_at AS "updated_at: _",
                    embedding_id AS "embedding_id: _",
                    workspace_id AS "workspace_id: _"
            "#,
            id,
            chunk.document_id,
            chunk.parent_chunk_id,
            chunk.content,
            chunk.content_hash,
            chunk.chunk_type,
            chunk.order_index,
            chunk.semantic_level,
            metadata,
            chunk.embedding_id,
            chunk.workspace_id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a document chunk by ID
    #[instrument(skip(self))]   
    pub async fn get_document_chunk_by_id(&self, id: &Uuid) -> Result<Option<DocumentChunk>> {
        debug!("Getting document chunk by ID: {}", id);

        Ok(sqlx::query_as!(
            DocumentChunk,
            r#"SELECT 
                    id AS "id: _", document_id AS "document_id: _", parent_chunk_id AS "parent_chunk_id: _",
                    content AS "content: _", content_hash AS "content_hash: _", chunk_type AS "chunk_type: _",
                    order_index AS "order_index: _", semantic_level AS "semantic_level: _",
                    metadata AS "metadata: _", embedding_id AS "embedding_id: _",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM document_chunks WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get a document chunk by document ID and chunk index
    #[instrument(skip(self))]
    pub async fn get_document_chunk_by_index(
        &self,  
        document_id: &Uuid,
        order_index: i64,
    ) -> Result<Option<DocumentChunk>> {
        debug!(
            "Getting document chunk for document: {} at index: {}",
            document_id, order_index
        );

       Ok(sqlx::query_as!(
            DocumentChunk,
            r#"SELECT 
                    id AS "id: _", document_id AS "document_id: _", parent_chunk_id AS "parent_chunk_id: _",
                    content AS "content: _", content_hash AS "content_hash: _", chunk_type AS "chunk_type: _",
                    order_index AS "order_index: _", semantic_level AS "semantic_level: _",
                    metadata AS "metadata: _", embedding_id AS "embedding_id: _",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM document_chunks WHERE document_id = ? AND order_index = ?"#,
            document_id,
            order_index
        )
        .fetch_optional(&self.pool)
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
            r#"SELECT id, document_id, chunk_index, content, metadata, embedding_id, is_embedded,
               created_at, updated_at 
               FROM document_chunks"#,
        );

        let mut add_where = add_where();

        if let Some(document_id) = &filter.document_id {
            add_where(&mut qb);
            qb.push("document_id = ");
            qb.push_bind(document_id);
        }

        if let Some(is_embedded) = filter.is_embedded {
            add_where(&mut qb);
            qb.push("is_embedded = ");
            qb.push_bind(if is_embedded { 1 } else { 0 });
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("content LIKE '%");
            qb.push_bind(search_term);
            qb.push("%'");
        }

        if let Some(min_chunk_index) = filter.min_chunk_index {
            add_where(&mut qb);
            qb.push("chunk_index >= ");
            qb.push_bind(min_chunk_index);
        }

        if let Some(max_chunk_index) = filter.max_chunk_index {
            add_where(&mut qb);
            qb.push("chunk_index <= ");
            qb.push_bind(max_chunk_index);
        }

        qb.push(" ORDER BY document_id, chunk_index ASC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }
        Ok(qb
            .build_query_as::<DocumentChunk>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update a document chunk
    #[instrument(err, skip(self))]
    pub async fn update_document_chunk(&self, chunk: &DocumentChunk) -> Result<()> {
        debug!("Updating document chunk with ID: {}", chunk.id);

        let affected = sqlx::query!(    
            r#"UPDATE document_chunks SET 
                document_id = ?, parent_chunk_id = ?, content = ?, content_hash = ?, chunk_type = ?,
                order_index = ?, semantic_level = ?, metadata = ?, embedding_id = ?, workspace_id = ?
            WHERE id = ?
            "#,
            chunk.document_id,
            chunk.parent_chunk_id,
            chunk.content,
            chunk.content_hash,
            chunk.chunk_type,
            chunk.order_index,
            chunk.semantic_level,
            chunk.metadata,
            chunk.embedding_id,
            chunk.workspace_id,
            chunk.id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document chunk with ID {} not found for update",
                chunk.id
            )));
        }

        Ok(())
    }

    /// Update document chunk content
    #[instrument(err, skip(self, content))]
    pub async fn update_document_chunk_content(&self, id: &Uuid, content: &str) -> Result<()> {
        debug!("Updating content for document chunk: {}", id);

        let affected = sqlx::query_as!(
            DocumentChunk,
            r#"UPDATE document_chunks SET content = ? WHERE id = ?
            "#,
            content,
            id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document chunk with ID {} not found for update",
                id
            )));
        }

        Ok(())
    }

    /// Update document chunk metadata
    #[instrument(err, skip(self, metadata))]
    pub async fn update_document_chunk_metadata(&self, id: &Uuid, metadata: &str) -> Result<()> {
        debug!("Updating metadata for document chunk: {}", id);

        let affected = sqlx::query_as!(
            DocumentChunk,
            r#"UPDATE document_chunks SET metadata = ? WHERE id = ?"#,
            metadata,
            id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document chunk with ID {} not found for update",
                id
            )));
        }

        Ok(())
    }


    /// Set document chunk embedding
    #[instrument(skip(self))]
    pub async fn set_document_chunk_embedding(&self, id: &Uuid, embedding_id: &Uuid) -> Result<()> {
        debug!(
            "Setting embedding ID {} for document chunk: {}",
            embedding_id, id
        );

        let now = Utc::now();

        let affected = sqlx::query!(
            r#"UPDATE document_chunks SET embedding_id = ?, updated_at = ? WHERE id = ?
            "#,
            embedding_id,
            now,
            id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document chunk with ID {} not found for update",
                id
            )));
        }

        Ok(())
    }

    /// Set document chunk embedded status
    #[instrument(skip(self))]
    pub async fn set_document_chunk_embedded(&self, id: &Uuid, is_embedded: bool) -> Result<()> {
        debug!(
            "Setting document chunk {} embedded status to {}",
            id, is_embedded
        );

        let now = Utc::now();

        let affected = sqlx::query_as!(
            DocumentChunk,
            r#"UPDATE document_chunks SET updated_at = ? WHERE id = ?
            "#,
            now,
            id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document chunk with ID {} not found for update",
                id
            )));
        }

        Ok(())
    }   

    /// Delete a document chunk by ID
    #[instrument(skip(self))]
    pub async fn delete_document_chunk(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting document chunk with ID: {}", id);

        let affected = sqlx::query!(
            r#"DELETE FROM document_chunks WHERE id = ?"#,
            id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Document chunk with ID {} not found for update",
                id
            )));
        }

        Ok(())
    }

    /// Delete all chunks for a document
    #[instrument(err, skip(self))]
    pub async fn delete_document_chunks_for_document(&self, document_id: &Uuid) -> Result<u64> {
        debug!("Deleting all chunks for document: {}", document_id);

        let affected = sqlx::query_as!(    
            DocumentChunk,
            r#"DELETE FROM document_chunks WHERE document_id = ?"#,
            document_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(affected)
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
    #[instrument(skip(self))]
    pub async fn get_non_embedded_chunks(&self, limit: Option<usize>) -> Result<Vec<DocumentChunk>> {
        debug!("Getting non-embedded chunks");

        let filter = DocumentChunkFilter {
            is_embedded: Some(false),
            limit,
            ..Default::default()
        };

        self.list_document_chunks(&filter).await
    }

    /// Count chunks for a document
    #[instrument(skip(self))]
    pub async fn count_chunks_for_document(&self, document_id: &Uuid) -> Result<i64> {
        debug!("Counting chunks for document: {}", document_id);

        let affected = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM document_chunks WHERE document_id = ?"#,
            document_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(affected)
    }

    /// Count embedded chunks for a document
    #[instrument(skip(self))]   
    pub async fn count_embedded_chunks_for_document(&self, document_id: &Uuid) -> Result<i64> {
        debug!("Counting embedded chunks for document: {}", document_id);   

        let affected = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as count FROM document_chunks WHERE document_id = ?"#,
            document_id,
        )       
        .fetch_one(&self.pool)
        .await?;

        Ok(affected)
    }   

    /// Get all chunks for a document
    #[instrument(skip(self))]
    pub async fn get_all_chunks(&self) -> Result<Vec<DocumentChunk>> {
        debug!("Getting all chunks");

        Ok(sqlx::query_as!(
            DocumentChunk,
            r#"SELECT 
                id AS "id: _", document_id AS "document_id: _", parent_chunk_id AS "parent_chunk_id: _",
                content AS "content: _", content_hash AS "content_hash: _", chunk_type AS "chunk_type: _",
                order_index AS "order_index: _", semantic_level AS "semantic_level: _",
                metadata AS "metadata: _", embedding_id AS "embedding_id: _",
                workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
            FROM document_chunks"#  
        )
        .fetch_all(&self.pool)
        .await?)
    }
}

