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

#[skip_serializing_none]
#[boilermates("CreateMemory")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Memory {
    #[boilermates(not_in("CreateMemory"))]
    pub id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub memory_type: MemoryType,
    pub content: String,
    pub summary: Option<String>,
    pub importance: f64,
    pub last_accessed_at: DateTime<Utc>,
    pub access_count: u32,
    pub metadata: Option<String>,
    pub embedding: Option<String>,
    #[boilermates(not_in("CreateMemory"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateMemory"))]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
pub enum MemoryType {
    Working = 0,
    Episodic = 1,
    Semantic = 2,
    Procedural = 3,
    Affective = 4,
    Prospective = 5,
    Source = 6,
    Social = 7,
    Implicit = 8,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub workspace_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub memory_type: Option<MemoryType>,
    pub min_importance: Option<f64>,
    pub max_importance: Option<f64>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub accessed_after: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl DatabaseManager {
    /// Create a new memory item
    pub async fn create_memory(&self, memory: &CreateMemory) -> Result<()> {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO memories (
                id, workspace_id, participant_id, conversation_id, memory_type, content,
                summary, importance, access_count, metadata, embedding
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id,
            memory.workspace_id,
            memory.participant_id,
            memory.conversation_id,
            memory.memory_type,
            memory.content,
            memory.summary,
            memory.importance,
            memory.access_count,
            memory.metadata,
            memory.embedding,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get memory item by ID
    pub async fn get_memory_by_id(&self, id: &Uuid) -> Result<Option<Memory>> {
        Ok(sqlx::query_as(
            "SELECT id, workspace_id, participant_id, conversation_id, memory_type, payload,
                    priority, confidence_score, emotional_valence, emotional_arousal,
                    last_accessed_at, metadata, created_at, updated_at, memory_source_id,
                    memory_context_id, memory_session_id, intention_id, concept_id
             FROM memories WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List memory items with filtering
    pub async fn list_memories(&self, filter: &MemoryFilter) -> Result<Vec<Memory>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, workspace_id, participant_id, conversation_id, memory_type, content,
                summary, importance, access_count, metadata, embedding, created_at, updated_at
             FROM memories",
        );

        let mut add_where = add_where();

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            qb.push("workspace_id = ");
            qb.push_bind(workspace_id);
        }

        if let Some(participant_id) = &filter.participant_id {
            add_where(&mut qb);
            qb.push("participant_id = ");
            qb.push_bind(participant_id);
        }

        if let Some(conversation_id) = &filter.conversation_id {
            add_where(&mut qb);
            qb.push("conversation_id = ");
            qb.push_bind(conversation_id);
        }

        if let Some(memory_type) = filter.memory_type {
            add_where(&mut qb);
            qb.push("memory_type = ");
            qb.push_bind(memory_type);
        }

        if let Some(min_priority) = filter.min_importance {
            add_where(&mut qb);
            qb.push("importance >= ");
            qb.push_bind(min_priority);
        }

        if let Some(max_priority) = filter.max_importance {
            add_where(&mut qb);
            qb.push("importance <= ");
            qb.push_bind(max_priority);
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

        if let Some(accessed_after) = &filter.accessed_after {
            add_where(&mut qb);
            qb.push("last_accessed_at >= ");
            qb.push_bind(accessed_after);
        }

        if let Some(search_term) = &filter.search_term {
            let like = format!("%{search_term}%");
            add_where(&mut qb);
            qb.push("(content LIKE ");
            qb.push_bind(like.clone());
            qb.push(" OR metadata LIKE ");
            qb.push_bind(like);
            qb.push(")");
        }

        qb.push(" ORDER BY importance DESC, last_accessed_at DESC");

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

    /// Update memory item
    pub async fn update_memories(&self, memory: &Memory) -> Result<()> {
        let affected = sqlx::query!(
            "UPDATE memories SET
                workspace_id = ?, participant_id = ?, conversation_id = ?, memory_type = ?, content = ?,
                summary = ?, importance = ?, access_count = ?, metadata = ?, embedding = ?
             WHERE id = ?",
            memory.workspace_id,
            memory.participant_id,
            memory.conversation_id,
            memory.memory_type,
            memory.content,
            memory.summary,
            memory.importance,
            memory.access_count,
            memory.metadata,
            memory.embedding,
            memory.id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Memory item with ID {} not found",
                memory.id
            )));
        }

        Ok(())
    }

    /// Delete memory item
    pub async fn delete_memory(&self, id: &Uuid) -> Result<()> {
        let affected = sqlx::query!("DELETE FROM memories WHERE id = ?", id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Memory item with ID {id} not found"
            )));
        }

        Ok(())
    }
}
