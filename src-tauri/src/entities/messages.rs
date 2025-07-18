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
    pub sender_id: Uuid,
    pub parent_message_id: Option<Uuid>,
    pub content: String,  // JSON object with raw and parsed content
    pub status: MessageStatus, // 'sent', 'delivered', 'read', 'failed'
    pub refs: Option<Json<Value>>, // JSON array of referenced message IDs
    pub metadata: Option<Json<Value>>, // JSON with attachments, reactions, etc.
    #[boilermates(not_in("CreateMessage"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateMessage"))]
    pub updated_at: DateTime<Utc>,
    pub reply_to_id: Option<Uuid>, // For threading
    pub branch_conversation_id: Option<Uuid>, // For branching conversations
    pub parent_id: Option<Uuid>, // For threading
    pub workspace_id: Option<Uuid>,
}

/// Additional filtering options for message queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct MessageFilter {
    pub conversation_id: Option<Uuid>,
    pub sender_id: Option<Uuid>,
    pub parent_message_id: Option<Uuid>,
    pub status: Option<MessageStatus>,
    pub after_date: Option<DateTime<Utc>>,
    pub before_date: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub workspace_id: Option<Uuid>,
}

impl DatabaseManager {
    /// Create a new message in the database
    #[instrument(err, skip(self, message))]
    pub async fn create_message(&self, message: &Message) -> Result<Message> {
        debug!("Creating message with ID: {}", message.id);
        let id = Uuid::new_v4();
        let refs = message.refs.as_deref();
        let metadata = message.metadata.as_deref();
        let now = Utc::now();

        Ok(sqlx::query_as!(
            Message,
            r#"INSERT INTO messages (
                id, conversation_id, sender_id, parent_message_id, content,
                status, refs, metadata, created_at, updated_at, reply_to_id, branch_conversation_id, parent_id,
                workspace_id
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING
                id AS "id: _", conversation_id AS "conversation_id: _", sender_id AS "sender_id: _",
                parent_message_id AS "parent_message_id: _", content, status as "status: MessageStatus",
                refs AS "refs: _", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                reply_to_id AS "reply_to_id: _",
                branch_conversation_id AS "branch_conversation_id: _", parent_id AS "parent_id: _",
                workspace_id AS "workspace_id: _"
            "#,
            id,
            message.conversation_id,
            message.sender_id,
            message.parent_message_id,
            message.content,
            message.status,
            refs,
            metadata,
            now,
            now,
            message.reply_to_id,
            message.branch_conversation_id,
            message.parent_id,
            message.workspace_id,
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
                parent_id AS "parent_id: _", content, status as "status: MessageStatus",
                refs AS "refs: _", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                reply_to_id AS "reply_to_id: _", branch_conversation_id AS "branch_conversation_id: _",
                parent_message_id AS "parent_message_id: _"
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
        let refs = message.refs.as_ref();
        let metadata = message.metadata.as_ref();
        let now = Utc::now();

        Ok(sqlx::query_as!(
            Message,
            r#"UPDATE messages SET 
                conversation_id = ?, workspace_id = ?, sender_id = ?, parent_id = ?, content = ?,
                status = ?, refs = ?, metadata = ?, created_at = ?, updated_at = ?, reply_to_id = ?,
                branch_conversation_id = ?, parent_message_id = ?
            WHERE id = ? RETURNING
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_id AS "parent_id: _", content, status as "status: MessageStatus",
                refs AS "refs: _", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                reply_to_id AS "reply_to_id: _", branch_conversation_id AS "branch_conversation_id: _",
                parent_message_id AS "parent_message_id: _"
            "#,
            message.conversation_id,
            message.workspace_id,
            message.sender_id,
            message.parent_id,
            message.content,
            message.status,
            refs,
            metadata,
            now,
            now,
            message.reply_to_id,
            message.branch_conversation_id,
            message.parent_message_id,
            message.id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete a message by ID
    #[instrument(err, skip(self))]
    pub async fn delete_message(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting message with ID: {}", id);

        let affected = sqlx::query!(
            "DELETE FROM messages WHERE id = ?",
            id
        )
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

        let affected = sqlx::query!(    
            "UPDATE messages SET status = ? WHERE id = ?",
            status,
            id
        )       
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
    #[instrument(err, skip(self, filter))]
    pub async fn list_messages(&self, filter: &MessageFilter) -> Result<Vec<Message>> {
        debug!("Listing messages with filter: {:?}", filter);

        let mut query_builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(
            r#"SELECT 
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_message_id AS "parent_message_id: _", content, status as "status: MessageStatus",
                refs AS "refs: _", metadata AS "metadata: _", created_at AS "created_at: _",
                updated_at AS "updated_at: _", reply_to_id AS "reply_to_id: _",
                branch_conversation_id AS "branch_conversation_id: _", parent_id AS "parent_id: _"
            FROM messages"#,
        );

        let mut add_where = add_where();

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut query_builder);
            query_builder.push(" AND workspace_id = ");
            query_builder.push_bind(*workspace_id);
        }
        if let Some(conversation_id) = &filter.conversation_id {
            add_where(&mut query_builder);
            query_builder.push(" AND conversation_id = ");
            query_builder.push_bind(*conversation_id);
        }

        if let Some(sender_id) = &filter.sender_id {
            add_where(&mut query_builder);
            query_builder.push(" AND sender_id = ");
            query_builder.push_bind(*sender_id);
        }

        if let Some(status) = &filter.status {
            add_where(&mut query_builder);
            query_builder.push(" AND status = ");   
            query_builder.push_bind(*status as i32);
        }

        if let Some(after) = &filter.after_date {
            add_where(&mut query_builder);
            query_builder.push(" AND created_at >= ");
            query_builder.push_bind(*after);
        }

        if let Some(before) = &filter.before_date {
            add_where(&mut query_builder);
            query_builder.push(" AND created_at <= ");
            query_builder.push_bind(*before);
        }

        if let Some(search) = &filter.search_term {
            add_where(&mut query_builder);
            query_builder.push(" AND content LIKE ");
            let search_param = format!("%{search}%");
            query_builder.push_bind(search_param);
        }

        // Default sort order is by creation time
        query_builder.push(" ORDER BY created_at");

        if let Some(limit) = &filter.limit {
            add_where(&mut query_builder);
            query_builder.push(" LIMIT ");  
            query_builder.push_bind(*limit as i64);
        }

        if let Some(offset) = &filter.offset {  
            add_where(&mut query_builder);
            query_builder.push(" OFFSET ");
            query_builder.push_bind(*offset as i64);
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
        conversation_id: &Uuid,
        limit: &usize,
        offset: &usize,
    ) -> Result<Vec<Message>> {
        debug!(
            "Getting messages for conversation {} with limit {} and offset {}",
            conversation_id, limit, offset
        );

        let filter = MessageFilter {
            conversation_id: Some(*conversation_id),
            limit: Some(*limit),
            offset: Some(*offset),
            ..Default::default()
        };

        self.list_messages(&filter).await
    }
}
