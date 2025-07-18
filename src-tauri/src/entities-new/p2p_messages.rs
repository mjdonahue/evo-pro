use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

/// P2P message status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum P2pMessageStatus {
    Pending = 0,
    Sent = 1,
    Delivered = 2,
    Read = 3,
    Failed = 4,
}

impl TryFrom<i32> for P2pMessageStatus {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(P2pMessageStatus::Pending),
            1 => Ok(P2pMessageStatus::Sent),
            2 => Ok(P2pMessageStatus::Delivered),
            3 => Ok(P2pMessageStatus::Read),
            4 => Ok(P2pMessageStatus::Failed),
            _ => Err(AppError::DeserializationError(format!(
                "Invalid P2pMessageStatus: {value}"
            ))),
        }
    }
}

/// P2P message model matching the SQLite schema
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct P2pMessage {
    pub id: Uuid,
    pub sender_id: String,
    pub recipient_id: String,
    pub message_type: String,
    pub content: String,
    pub status: P2pMessageStatus,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub read_at: Option<DateTime<Utc>>,
    pub metadata: String, // JSON object with additional metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for P2P message queries
#[derive(Debug, Default, Deserialize)]
pub struct P2pMessageFilter {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub sender_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub recipient_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub status: Option<P2pMessageStatus>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub created_after: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub created_before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new P2P message in the database
    #[instrument(skip(self))]
    pub async fn create_p2p_message(&self, message: &P2pMessage) -> Result<()> {
        debug!("Creating P2P message with ID: {}", message.id);

        let _result = sqlx::query(
            "INSERT INTO p2p_messages (
                    id, sender_id, recipient_id, message_type, content, status,
                    sent_at, delivered_at, read_at, metadata, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                )",
        )
        .bind(message.id)
        .bind(&message.sender_id)
        .bind(&message.recipient_id)
        .bind(&message.message_type)
        .bind(&message.content)
        .bind(message.status as i32)
        .bind(message.sent_at)
        .bind(message.delivered_at)
        .bind(message.read_at)
        .bind(&message.metadata)
        .bind(message.created_at)
        .bind(message.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a P2P message by ID
    #[instrument(skip(self))]
    pub async fn get_p2p_message_by_id(&self, id: &Uuid) -> Result<Option<P2pMessage>> {
        debug!("Getting P2P message by ID: {}", id);

        let row = sqlx::query(
            r#"SELECT 
                    id, sender_id, recipient_id, message_type, content,
                    status, sent_at, delivered_at, read_at, metadata, created_at, updated_at
                FROM p2p_messages WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let message = P2pMessage {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                sender_id: row.get("sender_id"),
                recipient_id: row.get("recipient_id"),
                message_type: row.get("message_type"),
                content: row.get("content"),
                status: P2pMessageStatus::try_from(row.get::<i64, _>("status") as i32)?,
                sent_at: row.get("sent_at"),
                delivered_at: row.get("delivered_at"),
                read_at: row.get("read_at"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(message))
        } else {
            Ok(None)
        }
    }

    /// List and filter P2P messages
    #[instrument(err, skip(self, filter))]
    pub async fn list_p2p_messages(&self, filter: &P2pMessageFilter) -> Result<Vec<P2pMessage>> {
        debug!("Listing P2P messages with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, sender_id, recipient_id, message_type, content,
               status, sent_at, delivered_at, read_at, metadata, created_at, updated_at 
               FROM p2p_messages"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(sender_id) = &filter.sender_id {
            where_conditions.push(format!("sender_id = '{sender_id}'"));
        }

        if let Some(recipient_id) = &filter.recipient_id {
            where_conditions.push(format!("recipient_id = '{recipient_id}'"));
        }

        if let Some(message_type) = &filter.message_type {
            where_conditions.push(format!("message_type = '{message_type}'"));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = {}", status as i64));
        }

        if let Some(created_after) = &filter.created_after {
            where_conditions.push(format!("created_at >= '{created_after}'"));
        }

        if let Some(created_before) = &filter.created_before {
            where_conditions.push(format!("created_at <= '{created_before}'"));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(content LIKE '%{search_term}%' OR metadata LIKE '%{search_term}%')"
            ));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
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

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut messages = Vec::new();
        for row in rows {
            let message = P2pMessage {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                sender_id: row.get("sender_id"),
                recipient_id: row.get("recipient_id"),
                message_type: row.get("message_type"),
                content: row.get("content"),
                status: P2pMessageStatus::try_from(row.get::<i64, _>("status") as i32)?,
                sent_at: row.get("sent_at"),
                delivered_at: row.get("delivered_at"),
                read_at: row.get("read_at"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            messages.push(message);
        }

        Ok(messages)
    }

    /// Update a P2P message
    #[instrument(err, skip(self))]
    pub async fn update_p2p_message(&self, message: &P2pMessage) -> Result<()> {
        debug!("Updating P2P message with ID: {}", message.id);

        let affected = sqlx::query(
            "UPDATE p2p_messages SET 
                sender_id = ?, recipient_id = ?, message_type = ?, content = ?,
                status = ?, sent_at = ?, delivered_at = ?, read_at = ?, metadata = ?,
                updated_at = ?
            WHERE id = ?",
        )
        .bind(&message.sender_id)
        .bind(&message.recipient_id)
        .bind(&message.message_type)
        .bind(&message.content)
        .bind(message.status as i32)
        .bind(message.sent_at)
        .bind(message.delivered_at)
        .bind(message.read_at)
        .bind(&message.metadata)
        .bind(message.updated_at)
        .bind(message.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {} not found for update",
                message.id
            )));
        }

        Ok(())
    }

    /// Update P2P message status
    #[instrument(err, skip(self))]
    pub async fn update_p2p_message_status(
        &self,
        id: &Uuid,
        status: P2pMessageStatus,
    ) -> Result<()> {
        debug!("Updating status for P2P message: {} to {:?}", id, status);

        let now = Utc::now();

        // Update timestamps based on status
        let (sent_at, delivered_at, read_at) = match status {
            P2pMessageStatus::Sent => (Some(now), None, None),
            P2pMessageStatus::Delivered => (None, Some(now), None),
            P2pMessageStatus::Read => (None, None, Some(now)),
            _ => (None, None, None),
        };

        let affected = sqlx::query(
            r#"UPDATE p2p_messages SET 
                status = ?,
                sent_at = CASE 
                    WHEN ? IS NOT NULL THEN ? 
                    ELSE sent_at 
                END,
                delivered_at = CASE 
                    WHEN ? IS NOT NULL THEN ? 
                    ELSE delivered_at 
                END,
                read_at = CASE 
                    WHEN ? IS NOT NULL THEN ? 
                    ELSE read_at 
                END,
                updated_at = ?
            WHERE id = ?"#,
        )
        .bind(status as i32)
        .bind(sent_at)
        .bind(sent_at)
        .bind(delivered_at)
        .bind(delivered_at)
        .bind(read_at)
        .bind(read_at)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update P2P message content
    #[instrument(err, skip(self, content))]
    pub async fn update_p2p_message_content(&self, id: &Uuid, content: &str) -> Result<()> {
        debug!("Updating content for P2P message: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE p2p_messages SET content = ?, updated_at = ? WHERE id = ?")
                .bind(content)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update P2P message metadata
    #[instrument(err, skip(self, metadata))]
    pub async fn update_p2p_message_metadata(&self, id: &Uuid, metadata: &str) -> Result<()> {
        debug!("Updating metadata for P2P message: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE p2p_messages SET metadata = ?, updated_at = ? WHERE id = ?")
                .bind(metadata)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Mark P2P message as sent
    #[instrument(err, skip(self))]
    pub async fn mark_p2p_message_as_sent(&self, id: &Uuid) -> Result<()> {
        debug!("Marking P2P message as sent: {}", id);
        self.update_p2p_message_status(id, P2pMessageStatus::Sent)
            .await
    }

    /// Mark P2P message as delivered
    #[instrument(err, skip(self))]
    pub async fn mark_p2p_message_as_delivered(&self, id: &Uuid) -> Result<()> {
        debug!("Marking P2P message as delivered: {}", id);
        self.update_p2p_message_status(id, P2pMessageStatus::Delivered)
            .await
    }

    /// Mark P2P message as read
    #[instrument(err, skip(self))]
    pub async fn mark_p2p_message_as_read(&self, id: &Uuid) -> Result<()> {
        debug!("Marking P2P message as read: {}", id);
        self.update_p2p_message_status(id, P2pMessageStatus::Read)
            .await
    }

    /// Mark P2P message as failed
    #[instrument(err, skip(self))]
    pub async fn mark_p2p_message_as_failed(&self, id: &Uuid) -> Result<()> {
        debug!("Marking P2P message as failed: {}", id);
        self.update_p2p_message_status(id, P2pMessageStatus::Failed)
            .await
    }

    /// Delete a P2P message by ID
    #[instrument(err, skip(self))]
    pub async fn delete_p2p_message(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting P2P message with ID: {}", id);

        let affected = sqlx::query("DELETE FROM p2p_messages WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Get P2P messages by sender ID
    #[instrument(skip(self))]
    pub async fn get_p2p_messages_by_sender_id(
        &self,
        sender_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<P2pMessage>> {
        debug!("Getting P2P messages for sender: {}", sender_id);

        let filter = P2pMessageFilter {
            sender_id: Some(sender_id.to_string()),
            limit,
            ..Default::default()
        };

        self.list_p2p_messages(&filter).await
    }

    /// Get P2P messages by recipient ID
    #[instrument(skip(self))]
    pub async fn get_p2p_messages_by_recipient_id(
        &self,
        recipient_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<P2pMessage>> {
        debug!("Getting P2P messages for recipient: {}", recipient_id);

        let filter = P2pMessageFilter {
            recipient_id: Some(recipient_id.to_string()),
            limit,
            ..Default::default()
        };

        self.list_p2p_messages(&filter).await
    }

    /// Get conversation between two peers
    #[instrument(skip(self))]
    pub async fn get_p2p_conversation(
        &self,
        peer1_id: &str,
        peer2_id: &str,
        limit: Option<usize>,
    ) -> Result<Vec<P2pMessage>> {
        debug!(
            "Getting P2P conversation between {} and {}",
            peer1_id, peer2_id
        );

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, sender_id, recipient_id, message_type, content,
               status, sent_at, delivered_at, read_at, metadata, created_at, updated_at 
               FROM p2p_messages
               WHERE (sender_id = ? AND recipient_id = ?) OR (sender_id = ? AND recipient_id = ?)"#,
        );

        qb.push_bind(peer1_id);
        qb.push_bind(peer2_id);
        qb.push_bind(peer2_id);
        qb.push_bind(peer1_id);

        qb.push(" ORDER BY created_at DESC");

        if let Some(limit) = limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut messages = Vec::new();
        for row in rows {
            let message = P2pMessage {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                sender_id: row.get("sender_id"),
                recipient_id: row.get("recipient_id"),
                message_type: row.get("message_type"),
                content: row.get("content"),
                status: P2pMessageStatus::try_from(row.get::<i64, _>("status") as i32)?,
                sent_at: row.get("sent_at"),
                delivered_at: row.get("delivered_at"),
                read_at: row.get("read_at"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            messages.push(message);
        }

        Ok(messages)
    }

    /// Get pending outgoing P2P messages
    #[instrument(skip(self))]
    pub async fn get_pending_outgoing_p2p_messages(
        &self,
        sender_id: &str,
    ) -> Result<Vec<P2pMessage>> {
        debug!(
            "Getting pending outgoing P2P messages for sender: {}",
            sender_id
        );

        let filter = P2pMessageFilter {
            sender_id: Some(sender_id.to_string()),
            status: Some(P2pMessageStatus::Pending),
            ..Default::default()
        };

        self.list_p2p_messages(&filter).await
    }

    /// Count P2P messages by status
    #[instrument(skip(self))]
    pub async fn count_p2p_messages_by_status(&self, status: P2pMessageStatus) -> Result<i64> {
        debug!("Counting P2P messages with status: {:?}", status);

        let row = sqlx::query("SELECT COUNT(*) as count FROM p2p_messages WHERE status = ?")
            .bind(status as i32)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::DatabaseManager;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_and_get_p2p_message() {
        let db = DatabaseManager::setup_test_db().await;
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Hello, Node2!".to_string(),
            status: P2pMessageStatus::Pending,
            sent_at: None,
            delivered_at: None,
            read_at: None,
            metadata: r#"{"priority": "normal"}"#.to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the message
        db.create_p2p_message(&message)
            .await
            .expect("Failed to create P2P message");

        // Get the message by ID
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, message_id);
        assert_eq!(retrieved.sender_id, "node1");
        assert_eq!(retrieved.recipient_id, "node2");
        assert_eq!(retrieved.message_type, "text");
        assert_eq!(retrieved.content, "Hello, Node2!");
        assert_eq!(retrieved.status, P2pMessageStatus::Pending);
        assert_eq!(retrieved.sent_at, None);
        assert_eq!(retrieved.delivered_at, None);
        assert_eq!(retrieved.read_at, None);
        assert_eq!(retrieved.metadata, r#"{"priority": "normal"}"#);
    }

    #[tokio::test]
    async fn test_list_p2p_messages() {
        let db = DatabaseManager::setup_test_db().await;

        // Create multiple messages
        for i in 1..=3 {
            let message_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i)).unwrap();

            let now = Utc::now();
            let message = P2pMessage {
                id: message_id,
                sender_id: if i % 2 == 0 {
                    "node2".to_string()
                } else {
                    "node1".to_string()
                },
                recipient_id: if i % 2 == 0 {
                    "node1".to_string()
                } else {
                    "node2".to_string()
                },
                message_type: "text".to_string(),
                content: format!("Message {}", i),
                status: match i {
                    1 => P2pMessageStatus::Sent,
                    2 => P2pMessageStatus::Delivered,
                    _ => P2pMessageStatus::Read,
                },
                sent_at: if i >= 1 { Some(now) } else { None },
                delivered_at: if i >= 2 { Some(now) } else { None },
                read_at: if i >= 3 { Some(now) } else { None },
                metadata: "{}".to_string(),
                created_at: now,
                updated_at: now,
            };

            db.create_p2p_message(&message)
                .await
                .expect("Failed to create P2P message");
        }

        // List all messages
        let filter = P2pMessageFilter::default();
        let messages = db
            .list_p2p_messages(&filter)
            .await
            .expect("Failed to list P2P messages");
        assert_eq!(messages.len(), 3);

        // Filter by sender_id
        let filter = P2pMessageFilter {
            sender_id: Some("node1".to_string()),
            ..Default::default()
        };
        let messages = db
            .list_p2p_messages(&filter)
            .await
            .expect("Failed to list P2P messages");
        assert_eq!(messages.len(), 2);
        assert!(messages.iter().all(|m| m.sender_id == "node1"));

        // Filter by recipient_id
        let filter = P2pMessageFilter {
            recipient_id: Some("node1".to_string()),
            ..Default::default()
        };
        let messages = db
            .list_p2p_messages(&filter)
            .await
            .expect("Failed to list P2P messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].recipient_id, "node1");

        // Filter by status
        let filter = P2pMessageFilter {
            status: Some(P2pMessageStatus::Delivered),
            ..Default::default()
        };
        let messages = db
            .list_p2p_messages(&filter)
            .await
            .expect("Failed to list P2P messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].status, P2pMessageStatus::Delivered);

        // Test get_p2p_messages_by_sender_id
        let messages = db
            .get_p2p_messages_by_sender_id("node1", None)
            .await
            .expect("Failed to get P2P messages by sender");
        assert_eq!(messages.len(), 2);
        assert!(messages.iter().all(|m| m.sender_id == "node1"));

        // Test get_p2p_messages_by_recipient_id
        let messages = db
            .get_p2p_messages_by_recipient_id("node2", None)
            .await
            .expect("Failed to get P2P messages by recipient");
        assert_eq!(messages.len(), 1);
        assert!(messages.iter().all(|m| m.recipient_id == "node2"));

        // Test get_p2p_conversation
        let messages = db
            .get_p2p_conversation("node1", "node2", None)
            .await
            .expect("Failed to get P2P conversation");
        assert_eq!(messages.len(), 3);

        // Test count_p2p_messages_by_status
        let count = db
            .count_p2p_messages_by_status(P2pMessageStatus::Read)
            .await
            .expect("Failed to count P2P messages");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_update_p2p_message() {
        let db = DatabaseManager::setup_test_db().await;
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Original content".to_string(),
            status: P2pMessageStatus::Pending,
            sent_at: None,
            delivered_at: None,
            read_at: None,
            metadata: "{}".to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the message
        db.create_p2p_message(&message)
            .await
            .expect("Failed to create P2P message");

        // Update the message
        let updated_message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Updated content".to_string(),
            status: P2pMessageStatus::Sent,
            sent_at: Some(Utc::now()),
            delivered_at: None,
            read_at: None,
            metadata: r#"{"priority": "high"}"#.to_string(),
            created_at: message.created_at,
            updated_at: Utc::now(),
        };

        db.update_p2p_message(&updated_message)
            .await
            .expect("Failed to update P2P message");

        // Get the updated message
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message")
            .unwrap();
        assert_eq!(retrieved.content, "Updated content");
        assert_eq!(retrieved.status, P2pMessageStatus::Sent);
        assert!(retrieved.sent_at.is_some());
        assert_eq!(retrieved.metadata, r#"{"priority": "high"}"#);
    }

    #[tokio::test]
    async fn test_update_p2p_message_status() {
        let db = DatabaseManager::setup_test_db().await;
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Test message".to_string(),
            status: P2pMessageStatus::Pending,
            sent_at: None,
            delivered_at: None,
            read_at: None,
            metadata: "{}".to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the message
        db.create_p2p_message(&message)
            .await
            .expect("Failed to create P2P message");

        // Update to Sent status
        db.update_p2p_message_status(&message_id, P2pMessageStatus::Sent)
            .await
            .expect("Failed to update P2P message status");
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message")
            .unwrap();
        assert_eq!(retrieved.status, P2pMessageStatus::Sent);
        assert!(retrieved.sent_at.is_some());
        assert!(retrieved.delivered_at.is_none());
        assert!(retrieved.read_at.is_none());

        // Update to Delivered status
        db.update_p2p_message_status(&message_id, P2pMessageStatus::Delivered)
            .await
            .expect("Failed to update P2P message status");
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message")
            .unwrap();
        assert_eq!(retrieved.status, P2pMessageStatus::Delivered);
        assert!(retrieved.sent_at.is_some()); // Should still have sent_at
        assert!(retrieved.delivered_at.is_some());
        assert!(retrieved.read_at.is_none());

        // Update to Read status
        db.update_p2p_message_status(&message_id, P2pMessageStatus::Read)
            .await
            .expect("Failed to update P2P message status");
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message")
            .unwrap();
        assert_eq!(retrieved.status, P2pMessageStatus::Read);
        assert!(retrieved.sent_at.is_some());
        assert!(retrieved.delivered_at.is_some());
        assert!(retrieved.read_at.is_some());
    }

    #[tokio::test]
    async fn test_update_p2p_message_content() {
        let db = DatabaseManager::setup_test_db().await;
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Original content".to_string(),
            status: P2pMessageStatus::Pending,
            sent_at: None,
            delivered_at: None,
            read_at: None,
            metadata: "{}".to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the message
        db.create_p2p_message(&message)
            .await
            .expect("Failed to create P2P message");

        // Update just the content
        let new_content = "This is the updated content with more details";
        db.update_p2p_message_content(&message_id, new_content)
            .await
            .expect("Failed to update P2P message content");

        // Get the updated message
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message")
            .unwrap();
        assert_eq!(retrieved.content, new_content);
        assert_eq!(retrieved.status, P2pMessageStatus::Pending); // Other fields should remain unchanged
    }

    #[tokio::test]
    async fn test_update_p2p_message_metadata() {
        let db = DatabaseManager::setup_test_db().await;
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Test message".to_string(),
            status: P2pMessageStatus::Pending,
            sent_at: None,
            delivered_at: None,
            read_at: None,
            metadata: "{}".to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the message
        db.create_p2p_message(&message)
            .await
            .expect("Failed to create P2P message");

        // Update the metadata
        let new_metadata = r#"{"priority": "high", "encrypted": true, "retry_count": 3}"#;
        db.update_p2p_message_metadata(&message_id, new_metadata)
            .await
            .expect("Failed to update P2P message metadata");

        // Get the updated message
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get P2P message")
            .unwrap();
        assert_eq!(retrieved.metadata, new_metadata);
        assert_eq!(retrieved.content, "Test message"); // Other fields should remain unchanged
    }

    #[tokio::test]
    async fn test_mark_p2p_message_status_helpers() {
        let db = DatabaseManager::setup_test_db().await;

        // Create messages for each status transition
        for i in 1..=4 {
            let message_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i)).unwrap();

            let now = Utc::now();
            let message = P2pMessage {
                id: message_id,
                sender_id: "node1".to_string(),
                recipient_id: "node2".to_string(),
                message_type: "text".to_string(),
                content: format!("Message {}", i),
                status: P2pMessageStatus::Pending,
                sent_at: None,
                delivered_at: None,
                read_at: None,
                metadata: "{}".to_string(),
                created_at: now,
                updated_at: now,
            };

            db.create_p2p_message(&message)
                .await
                .expect("Failed to create P2P message");
        }

        // Test mark_p2p_message_as_sent
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        db.mark_p2p_message_as_sent(&message_id)
            .await
            .expect("Failed to mark message as sent");
        let message = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get message")
            .unwrap();
        assert_eq!(message.status, P2pMessageStatus::Sent);
        assert!(message.sent_at.is_some());

        // Test mark_p2p_message_as_delivered
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        db.mark_p2p_message_as_delivered(&message_id)
            .await
            .expect("Failed to mark message as delivered");
        let message = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get message")
            .unwrap();
        assert_eq!(message.status, P2pMessageStatus::Delivered);
        assert!(message.delivered_at.is_some());

        // Test mark_p2p_message_as_read
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap();
        db.mark_p2p_message_as_read(&message_id)
            .await
            .expect("Failed to mark message as read");
        let message = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get message")
            .unwrap();
        assert_eq!(message.status, P2pMessageStatus::Read);
        assert!(message.read_at.is_some());

        // Test mark_p2p_message_as_failed
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();
        db.mark_p2p_message_as_failed(&message_id)
            .await
            .expect("Failed to mark message as failed");
        let message = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to get message")
            .unwrap();
        assert_eq!(message.status, P2pMessageStatus::Failed);
    }

    #[tokio::test]
    async fn test_delete_p2p_message() {
        let db = DatabaseManager::setup_test_db().await;
        let message_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let message = P2pMessage {
            id: message_id,
            sender_id: "node1".to_string(),
            recipient_id: "node2".to_string(),
            message_type: "text".to_string(),
            content: "Test message".to_string(),
            status: P2pMessageStatus::Pending,
            sent_at: None,
            delivered_at: None,
            read_at: None,
            metadata: "{}".to_string(),
            created_at: now,
            updated_at: now,
        };

        // Create the message
        db.create_p2p_message(&message)
            .await
            .expect("Failed to create P2P message");

        // Delete the message
        db.delete_p2p_message(&message_id)
            .await
            .expect("Failed to delete P2P message");

        // Try to get the deleted message
        let retrieved = db
            .get_p2p_message_by_id(&message_id)
            .await
            .expect("Failed to query P2P message");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_get_pending_outgoing_p2p_messages() {
        let db = DatabaseManager::setup_test_db().await;

        // Create a mix of pending and non-pending messages
        for i in 1..=4 {
            let message_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i)).unwrap();

            let now = Utc::now();
            let message = P2pMessage {
                id: message_id,
                sender_id: "node1".to_string(),
                recipient_id: "node2".to_string(),
                message_type: "text".to_string(),
                content: format!("Message {}", i),
                status: if i <= 2 {
                    P2pMessageStatus::Pending
                } else {
                    P2pMessageStatus::Sent
                },
                sent_at: if i > 2 { Some(now) } else { None },
                delivered_at: None,
                read_at: None,
                metadata: "{}".to_string(),
                created_at: now,
                updated_at: now,
            };

            db.create_p2p_message(&message)
                .await
                .expect("Failed to create P2P message");
        }

        // Get pending outgoing messages
        let messages = db
            .get_pending_outgoing_p2p_messages("node1")
            .await
            .expect("Failed to get pending outgoing messages");
        assert_eq!(messages.len(), 2);
        assert!(
            messages
                .iter()
                .all(|m| m.status == P2pMessageStatus::Pending)
        );
    }
}
