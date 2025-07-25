use boilermates::boilermates;
use kameo::Reply;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::{prelude::FromRow, types::Json, QueryBuilder, Sqlite};
use tracing::{instrument, debug};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ConversationStatus {
    #[default]
    Active = 0, 
    Archived = 1,
    Deleted = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ConversationType {
    Direct = 0,
    Group = 1, 
    Channel = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ConversationParticipantRole {
    Admin = 0,
    Member = 1, 
}
/// Conversation model matching the SQLite schema
#[skip_serializing_none]
#[boilermates("CreateConversation")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Reply)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    #[boilermates(not_in("CreateConversation"))]
    pub id: Uuid,
    pub title: String,
    pub conversation_type: ConversationType, // 'direct', 'group', 'channel'
    pub status: ConversationStatus, // 'active', 'archived', 'deleted'
    pub parent_conversation_id: Option<Uuid>,
    pub metadata: Option<Json<Value>>, // JSON object
    #[boilermates(not_in("CreateConversation"))]
    pub last_message_at: Option<DateTime<Utc>>,
    #[boilermates(not_in("CreateConversation"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateConversation"))]
    pub updated_at: DateTime<Utc>,
    pub workspace_id: Option<Uuid>,
}

/// Additional filtering options for conversation queries
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationFilter {
    pub status: Option<ConversationStatus>,
    pub conversation_type: Option<ConversationType>,
    pub parent_conversation_id: Option<Uuid>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
        /// Create a new conversation in the database
    #[instrument(err, skip(self))]
    pub async fn create_conversation(&self, conversation: &CreateConversation) -> Result<Conversation> {
        let id = Uuid::new_v4();
        debug!("Creating conversation with ID: {}", id);
        let now = Utc::now();
        let metadata = conversation.metadata.as_deref();
        Ok(sqlx::query_as!(
            Conversation,
            r#"INSERT INTO conversations (  
                id, title, conversation_type, status, parent_conversation_id, metadata, last_message_at, created_at, updated_at, workspace_id
            ) VALUES (  
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            RETURNING id AS "id: _", title AS "title: _", conversation_type AS "conversation_type: ConversationType",
            status AS "status: _", parent_conversation_id AS "parent_conversation_id: _",
            metadata AS "metadata: _", last_message_at AS "last_message_at: _", created_at AS "created_at: _", updated_at AS "updated_at: _", workspace_id AS "workspace_id: _"
        "#,
        id,
        conversation.title,
        conversation.conversation_type,
        conversation.status,
        conversation.parent_conversation_id,
        metadata,
        None::<DateTime<Utc>>, // last_message_at starts as None
        now,
        now,
        conversation.workspace_id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a conversation by ID
    #[instrument(err, skip(self))]
    pub async fn get_conversation_by_id(&self, id: &Uuid) -> Result<Option<Conversation>> {
        debug!("Getting conversation by ID: {}", id);

        Ok(sqlx::query_as!(
            Conversation,
            r#"
            SELECT id AS "id: _", title AS "title: _", conversation_type AS "conversation_type: _",
            status AS "status: _", parent_conversation_id AS "parent_conversation_id: _",
            metadata AS "metadata: _", last_message_at AS "last_message_at: _", created_at AS "created_at: _", updated_at AS "updated_at: _", workspace_id AS "workspace_id: _"
            FROM conversations WHERE id = ? 
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter conversations
    #[instrument(err, skip(self, filter))]
    pub async fn list_conversations(
        &self,
        filter: &ConversationFilter,
    ) -> Result<Vec<Conversation>> {
        debug!("Listing conversations with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
                r#"SELECT DISTINCT t.id, t.title, t.conversation_type, t.status, t.parent_conversation_id, t.metadata, t.last_message_at, t.created_at, t.updated_at, t.workspace_id FROM conversations t"#,
        );

        let mut add_where = add_where();

        if let Some(status) = filter.status {
            add_where(&mut qb);
            qb.push("t.status = ");
            qb.push_bind(status);
        }

        if let Some(conversation_type) = filter.conversation_type {
            add_where(&mut qb);
            qb.push("t.conversation_type = ");
            qb.push_bind(conversation_type);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("t.title LIKE ");
            qb.push_bind(format!("%{search_term}%"));
        }

        qb.push(" ORDER BY t.updated_at DESC");

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

    /// Update a conversation
    #[instrument(err, skip(self))]
    pub async fn update_conversation(&self, conversation: &Conversation) -> Result<()> {
        debug!("Updating conversation with ID: {}", conversation.id);

        let affected = sqlx::query!(
            "UPDATE conversations SET 
                title = ?, conversation_type = ?, status = ?, parent_conversation_id = ?,
                metadata = ?, last_message_at = ?, updated_at = ?, workspace_id = ?
            WHERE id = ?",
            conversation.title,
            conversation.conversation_type,
            conversation.status,
            conversation.parent_conversation_id,
            conversation.metadata,
            conversation.last_message_at,
            conversation.updated_at,
            conversation.workspace_id,
            conversation.id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Conversation with ID {} not found for update",
                conversation.id
            )));
        }

        Ok(())
    }

    /// Delete a conversation by ID
    #[instrument(err, skip(self))]
    pub async fn delete_conversation(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting conversation with ID: {}", id);
        // Delete the conversation
        let affected = sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Conversation with ID {id} not found for delete"
            )));
        }
        Ok(())
    }

    /// Add a participant to a conversation
    #[instrument(err, skip(self))]
    pub async fn add_participant(
        &self,
        conversation_id: &Uuid,
        participant_id: &Uuid,
        role: ConversationParticipantRole,
    ) -> Result<()> {
        debug!(
            "Adding participant {} to conversation {}",
            participant_id, conversation_id
        );
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO conversation_participants (conversation_id, role, joined_at, is_active) VALUES (?, ?, ?, ?)",
        conversation_id,
        role,
        now, 
        true)
        .execute(&self.pool)    
        .await?;

        Ok(())
    }
}
