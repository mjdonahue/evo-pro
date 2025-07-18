use crate::{
    error::{AppError, Result},
    storage::db::DatabaseManager,
};
use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::{Pool, QueryBuilder, Row, Sqlite, prelude::FromRow};
use uuid::Uuid;

#[skip_serializing_none]
#[boilermates("CreateMemoryItem")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MemoryItem {
    #[boilermates(not_in("CreateMemoryItem"))]
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub participant_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub memory_type: MemoryType,
    pub payload: String, // JSON
    pub priority: f64,
    pub confidence_score: f64,
    pub emotional_valence: f64,
    pub emotional_arousal: f64,
    pub last_accessed_at: DateTime<Utc>,
    pub metadata: Option<String>, // JSON
    #[boilermates(not_in("CreateMemoryItem"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateMemoryItem"))]
    pub updated_at: DateTime<Utc>,
    pub memory_source_id: Option<Uuid>,
    pub memory_context_id: Option<Uuid>,
    pub memory_session_id: Option<Uuid>,
    pub intention_id: Option<Uuid>,
    pub concept_id: Option<Uuid>,
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
pub struct MemoryItemFilter {
    pub workspace_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub memory_type: Option<MemoryType>,
    pub min_priority: Option<f64>,
    pub max_priority: Option<f64>,
    pub min_confidence: Option<f64>,
    pub max_confidence: Option<f64>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub accessed_after: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl DatabaseManager {
    /// Create a new memory item
    pub async fn create_memory_item(&self, memory_item: &CreateMemoryItem) -> Result<()> {
        let id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO memory_items (
                id, workspace_id, participant_id, conversation_id, memory_type, payload,
                priority, confidence_score, emotional_valence, emotional_arousal,
                last_accessed_at, metadata, memory_source_id,
                memory_context_id, memory_session_id, intention_id, concept_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id,
            memory_item.workspace_id,
            memory_item.participant_id,
            memory_item.conversation_id,
            memory_item.memory_type,
            memory_item.payload,
            memory_item.priority,
            memory_item.confidence_score,
            memory_item.emotional_valence,
            memory_item.emotional_arousal,
            memory_item.last_accessed_at,
            memory_item.metadata,
            memory_item.memory_source_id,
            memory_item.memory_context_id,
            memory_item.memory_session_id,
            memory_item.intention_id,
            memory_item.concept_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get memory item by ID
    pub async fn get_memory_item_by_id(&self, id: &Uuid) -> Result<Option<MemoryItem>> {
        Ok(sqlx::query_as(
            "SELECT id, workspace_id, participant_id, conversation_id, memory_type, payload,
                    priority, confidence_score, emotional_valence, emotional_arousal,
                    last_accessed_at, metadata, created_at, updated_at, memory_source_id,
                    memory_context_id, memory_session_id, intention_id, concept_id
             FROM memory_items WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List memory items with filtering
    pub async fn list_memory_items(&self, filter: &MemoryItemFilter) -> Result<Vec<MemoryItem>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, workspace_id, participant_id, conversation_id, memory_type, payload,
                    priority, confidence_score, emotional_valence, emotional_arousal,
                    last_accessed_at, metadata, created_at, updated_at, memory_source_id,
                    memory_context_id, memory_session_id, intention_id, concept_id
             FROM memory_items",
        );

        let mut where_conditions = Vec::new();

        if let Some(workspace_id) = &filter.workspace_id {
            where_conditions.push(format!("workspace_id = '{workspace_id}'"));
        }

        if let Some(participant_id) = &filter.participant_id {
            where_conditions.push(format!("participant_id = '{participant_id}'"));
        }

        if let Some(conversation_id) = &filter.conversation_id {
            where_conditions.push(format!("conversation_id = '{conversation_id}'"));
        }

        if let Some(memory_type) = filter.memory_type {
            where_conditions.push(format!("memory_type = {}", memory_type as i32));
        }

        if let Some(min_priority) = filter.min_priority {
            where_conditions.push(format!("priority >= {min_priority}"));
        }

        if let Some(max_priority) = filter.max_priority {
            where_conditions.push(format!("priority <= {max_priority}"));
        }

        if let Some(min_confidence) = filter.min_confidence {
            where_conditions.push(format!("confidence_score >= {min_confidence}"));
        }

        if let Some(max_confidence) = filter.max_confidence {
            where_conditions.push(format!("confidence_score <= {max_confidence}"));
        }

        if let Some(created_after) = &filter.created_after {
            where_conditions.push(format!(
                "created_at >= '{}'",
                created_after.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(created_before) = &filter.created_before {
            where_conditions.push(format!(
                "created_at <= '{}'",
                created_before.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(accessed_after) = &filter.accessed_after {
            where_conditions.push(format!(
                "last_accessed_at >= '{}'",
                accessed_after.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(payload LIKE '%{search_term}%' OR metadata LIKE '%{search_term}%')"
            ));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY priority DESC, last_accessed_at DESC");

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
    pub async fn update_memory_items(&self, memory_item: &MemoryItem) -> Result<()> {
        let affected = sqlx::query!(
            "UPDATE memory_items SET
                workspace_id = ?, participant_id = ?, conversation_id = ?, memory_type = ?,
                payload = ?, priority = ?, confidence_score = ?, emotional_valence = ?,
                emotional_arousal = ?, last_accessed_at = ?, metadata = ?, updated_at = ?,
                memory_source_id = ?, memory_context_id = ?, memory_session_id = ?,
                intention_id = ?, concept_id = ?
             WHERE id = ?",
            memory_item.workspace_id,
            memory_item.participant_id,
            memory_item.conversation_id,
            memory_item.memory_type,
            memory_item.payload,
            memory_item.priority,
            memory_item.confidence_score,
            memory_item.emotional_valence,
            memory_item.emotional_arousal,
            memory_item.last_accessed_at,
            memory_item.metadata,
            memory_item.updated_at,
            memory_item.memory_source_id,
            memory_item.memory_context_id,
            memory_item.memory_session_id,
            memory_item.intention_id,
            memory_item.concept_id,
            memory_item.id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Memory item with ID {} not found",
                memory_item.id
            )));
        }

        Ok(())
    }

    /// Delete memory item
    pub async fn delete_memory_item(&self, id: &Uuid) -> Result<()> {
        let affected = sqlx::query!("DELETE FROM memory_items WHERE id = ?", id)
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
