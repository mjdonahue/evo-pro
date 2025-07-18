use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use serde_json::{Value};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

// Define ContentType locally since llm module doesn't exist

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Text = 0,
    Image = 1,
    Audio = 2,
    Video = 3,
    Document = 4,
    Other = 5,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum SenderType {
    User = 0,
    Agent = 1,
    System = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text = 0,
    Command = 1,
    System = 2,
    Err = 3, // Can't use 'Error' because it's causes conflicts when implementing traits that have an
             // associated type named 'Error'
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Sent = 0,
    Delivered = 1,
    Read = 2,
    Failed = 3,
}
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FileAttachment {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub type_: ContentType,
    pub url: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Message model matching the SQLite schema
#[boilermates("CreateMessage")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[boilermates(not_in("CreateMessage"))]
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub workspace_id: Uuid,
    pub sender_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub type_: MessageType, // 'text', 'command', 'system', 'error'
    pub content: Json<Value>,  // JSON object with raw and parsed content
    pub status: MessageStatus, // 'sent', 'delivered', 'read', 'failed'
    pub refs: Option<Json<Value>>,
    pub related_episode_id: Option<Uuid>,
    pub branch_conversation_id: Option<Uuid>,
    pub reply_to_id: Option<Uuid>,
    pub metadata: Option<Json<Value>>, // JSON with attachments, reactions, etc.
    #[boilermates(not_in("CreateMessage"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateMessage"))]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for message queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct MessageFilter {
    pub conversation_id: Option<Uuid>,
    pub sender_id: Option<Uuid>,
    pub type_: Option<MessageType>,
    pub status: Option<MessageStatus>,
    pub after_date: Option<DateTime<Utc>>,
    pub before_date: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new message in the database
    #[instrument(err, skip(self))]
    pub async fn create_message(&self, message: &Message) -> Result<Message> {
        let id = Uuid::new_v4();
        debug!("Creating message with ID: {}", id);
        let content = message.content.as_ref();
        let refs = message.refs.as_ref();
        let metadata = message.metadata.as_ref();

        Ok(sqlx::query_as!(
            Message,
            r#"INSERT INTO messages (
                id, conversation_id, workspace_id, sender_id, parent_id, type, content,
                status, branch_conversation_id, metadata, refs, related_episode_id,
                reply_to_id, created_at, updated_at 
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_id AS "parent_id: _", type AS "type_: MessageType", content AS "content: _",
                status AS "status: MessageStatus", branch_conversation_id AS "branch_conversation_id: _",
                metadata AS "metadata: _", refs AS "refs: _", related_episode_id AS "related_episode_id: _",
                reply_to_id AS "reply_to_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            id,
            message.conversation_id,
            message.workspace_id,
            message.sender_id,
            message.parent_id,
            message.type_,
            content,
            message.status,
            message.branch_conversation_id,
            metadata,
            refs,
            message.related_episode_id,
            message.reply_to_id,
            message.created_at,
            message.updated_at
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a message by ID
    #[instrument(err, skip(self))]
    pub async fn get_message_by_id(&self, id: &Uuid) -> Result<Option<Message>> {
        debug!("Getting message by ID: {}", id);

        Ok(sqlx::query_as!(
            Message,
            r#"SELECT
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_id AS "parent_id: _", type AS "type_: MessageType", content AS "content: _",
                status AS "status: MessageStatus", branch_conversation_id AS "branch_conversation_id: _",
                metadata AS "metadata: _", refs AS "refs: _", related_episode_id AS "related_episode_id: _",
                reply_to_id AS "reply_to_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
            FROM messages WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Update a message
    #[instrument(skip(self))]
    pub async fn update_message(&self, message: &Message) -> Result<Message> {
        debug!("Updating message with ID: {}", message.id);
        let content = message.content.as_ref();
        let refs = message.refs.as_ref();
        let metadata = message.metadata.as_ref();

        Ok(sqlx::query_as!(
            Message,
            r#"UPDATE messages SET 
                conversation_id = ?, workspace_id = ?, sender_id = ?, parent_id = ?, type = ?, content = ?,
                status = ?, refs = ?, related_episode_id = ?, branch_conversation_id = ?, metadata = ?,
                reply_to_id = ?, updated_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_id AS "parent_id: _", type AS "type_: MessageType", content AS "content: _",
                status AS "status: MessageStatus", branch_conversation_id AS "branch_conversation_id: _",
                metadata AS "metadata: _", refs AS "refs: _", related_episode_id AS "related_episode_id: _",
                reply_to_id AS "reply_to_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            message.conversation_id,
            message.workspace_id,
            message.sender_id,
            message.parent_id,
            message.type_,
            content,
            message.status,
            refs,
            message.related_episode_id,
            message.branch_conversation_id,
            metadata,
            message.reply_to_id,   
            message.updated_at,
            message.id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete a message by ID
    #[instrument(err, skip(self))]
    pub async fn delete_message(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting message with ID: {}", id);

        let affected = sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::QueryError(format!("Failed to delete message: {e}")))?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Update message status
    #[instrument(err, skip(self))]
    pub async fn update_message_status(&self, id: &Uuid, status: MessageStatus) -> Result<()> {
        debug!("Updating status for message with ID: {}", id);

        let affected = sqlx::query("UPDATE messages SET status = ? WHERE id = ?")
            .bind(status as i32)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::QueryError(format!("Failed to update message status: {e}")))?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// List messages with optional filtering
    #[instrument(err, skip(self))]
    pub async fn list_messages(&self, filter: Option<&MessageFilter>) -> Result<Vec<Message>> {
        let mut query_builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            r#"SELECT 
                id, conversation_id, workspace_id, sender_id, parent_id, type as "type_: MessageType", content, status as "status: MessageStatus",
                refs, related_episode_id, branch_conversation_id, reply_to_id, metadata, created_at, updated_at
            FROM messages WHERE 1=1"#,
        );

        if let Some(filter) = filter {
            if let Some(conversation_id) = &filter.conversation_id {
                query_builder.push(" AND conversation_id = ");
                query_builder.push_bind(*conversation_id);
            }

            if let Some(sender_id) = &filter.sender_id {
                query_builder.push(" AND sender_id = ");
                query_builder.push_bind(*sender_id);
            }

            if let Some(type_) = &filter.type_ {
                query_builder.push(" AND type = "); 
                query_builder.push_bind(type_.clone() as i32);
            }

            if let Some(status) = &filter.status {
                query_builder.push(" AND status = ");
                query_builder.push_bind(status.clone() as i32);
            }

            if let Some(after) = &filter.after_date {
                query_builder.push(" AND created_at >= ");
                query_builder.push_bind(after.clone());
            }

            if let Some(before) = &filter.before_date {
                query_builder.push(" AND created_at <= ");
                query_builder.push_bind(before.clone());
            }

            if let Some(search) = &filter.search_term {
                query_builder.push(" AND content LIKE ");
                let search_param = format!("%{search}%");
                query_builder.push_bind(search_param);
            }

            // Default sort order is by creation time
            query_builder.push(" ORDER BY created_at");

            if let Some(limit) = filter.limit {
                query_builder.push(" LIMIT ");
                query_builder.push_bind(limit as i64);

                if let Some(offset) = filter.offset {
                    query_builder.push(" OFFSET ");
                    query_builder.push_bind(offset as i64);
                }
            }
        }

        debug!("Listing messages with query: {}", query_builder.sql());

        let messages = query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::QueryError(format!("Failed to list messages: {e}")))?;

        Ok(messages)
    }

    /// Get messages for a conversation with pagination
    #[instrument(err, skip(self))]
    pub async fn get_conversation_messages(
        &self,
        conversation_id: Uuid,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>> {
        debug!(
            "Getting messages for conversation {} with limit {} and offset {}",
            conversation_id, limit, offset
        );

        let filter = MessageFilter {
            conversation_id: Some(conversation_id),
            limit: Some(limit),
            offset: Some(offset),
            ..Default::default()
        };

        self.list_messages(Some(&filter)).await
    }
}

/// Create a new Message object with default values
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    // Removed: use rusqlite::params;

    #[tokio::test]
    async fn test_create_and_get_message() {
        let repo = DatabaseManager::setup_test_db().await;
        let conversation_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let sender_id = Uuid::new_v4();

        let message = Message {
            id: Uuid::new_v4(),
            conversation_id,
            workspace_id,
            sender_id,
            parent_id: None,
            type_: MessageType::Text,
            content: Json(json!({"text": "Hello, world!"})),
            status: MessageStatus::Sent,
            refs: None,
            related_episode_id: None,
            branch_conversation_id: None,
            reply_to_id: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create_message(&message).await.unwrap();

        let retrieved = repo
            .get_message_by_id(&message.id)
            .await
            .unwrap()
            .expect("Should have returned Some");
        assert_eq!(retrieved.id, message.id);
        assert_eq!(retrieved.conversation_id, message.conversation_id);
        assert_eq!(retrieved.sender_id, message.sender_id);
        assert_eq!(retrieved.content, message.content);
    }

    #[tokio::test]
    async fn test_update_message_status() {
        let repo = DatabaseManager::setup_test_db().await;
        let conversation_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let message = Message {
            id: Uuid::new_v4(),
            conversation_id,
            workspace_id,
            sender_id: user_id,
            parent_id: None,
            type_: MessageType::Text,
            content: Json(json!({"text": "Hello, world!"})),
            status: MessageStatus::Sent,
            refs: None,
            related_episode_id: None,
            branch_conversation_id: None,
            reply_to_id: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        repo.create_message(&message).await.unwrap();
        repo.update_message_status(&message.id, MessageStatus::Delivered)
            .await
            .unwrap();

        let retrieved = repo
            .get_message_by_id(&message.id)
            .await
            .unwrap()
            .expect("Message not found");

        assert_eq!(retrieved.status as i32, MessageStatus::Delivered as i32);
    }

    #[tokio::test]
    async fn test_get_conversation_messages() {
        let repo = DatabaseManager::setup_test_db().await;

        let conversation_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        // Create multiple messages
        let message1 = Message {
            id: Uuid::new_v4(),
            conversation_id,
            workspace_id,
            sender_id: user_id,
            parent_id: None,
            type_: MessageType::Text,
            content: Json(json!({"text": "Message 1"})),
            status: MessageStatus::Sent,
            refs: None,
            related_episode_id: None,
            branch_conversation_id: None,
            reply_to_id: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let message2 = Message {
            id: Uuid::new_v4(),
            conversation_id,
            workspace_id,
            sender_id: agent_id,
            parent_id: Some(message1.id),
            type_: MessageType::Text,
            content: Json(json!({"text": "Message 2"})),
            status: MessageStatus::Sent,
            refs: None,
            related_episode_id: None,
            branch_conversation_id: None,
            reply_to_id: Some(message1.id),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        repo.create_message(&message1).await.unwrap();
        repo.create_message(&message2).await.unwrap();

        let messages = repo
            .get_conversation_messages(conversation_id, 10, 0)
            .await
            .unwrap();
        assert_eq!(messages.len(), 2);
    }
}
