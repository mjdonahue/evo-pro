use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pMessageQueue {
    pub id: Uuid,
    pub from_peer_id: String,
    pub to_peer_id: String,
    pub message_type: P2pMessageType,
    pub priority: P2pMessagePriority,
    pub payload: String, // JSON
    pub conversation_id: Option<Uuid>,
    pub agent_chain_execution_id: Option<Uuid>,
    pub status: P2pMessageStatus,
    pub retry_count: i32,
    pub max_retries: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub error_details: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum P2pMessageType {
    AgentMessage = 0,
    SystemMessage = 1,
    Heartbeat = 2,
}

impl TryFrom<i32> for P2pMessageType {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(P2pMessageType::AgentMessage),
            1 => Ok(P2pMessageType::SystemMessage),
            2 => Ok(P2pMessageType::Heartbeat),
            _ => Err(AppError::ValidationError(
                "Invalid P2P message type".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum P2pMessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Urgent = 3,
}

impl TryFrom<i32> for P2pMessagePriority {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(P2pMessagePriority::Low),
            1 => Ok(P2pMessagePriority::Normal),
            2 => Ok(P2pMessagePriority::High),
            3 => Ok(P2pMessagePriority::Urgent),
            _ => Err(AppError::ValidationError(
                "Invalid P2P message priority".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum P2pMessageStatus {
    Pending = 0,
    Sent = 1,
    Delivered = 2,
    Failed = 3,
    Expired = 4,
}

impl TryFrom<i32> for P2pMessageStatus {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(P2pMessageStatus::Pending),
            1 => Ok(P2pMessageStatus::Sent),
            2 => Ok(P2pMessageStatus::Delivered),
            3 => Ok(P2pMessageStatus::Failed),
            4 => Ok(P2pMessageStatus::Expired),
            _ => Err(AppError::ValidationError(
                "Invalid P2P message status".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pMessageQueueFilter {
    pub from_peer_id: Option<String>,
    pub to_peer_id: Option<String>,
    pub message_type: Option<P2pMessageType>,
    pub priority: Option<P2pMessagePriority>,
    pub status: Option<P2pMessageStatus>,
    pub conversation_id: Option<Uuid>,
    pub agent_chain_execution_id: Option<Uuid>,
    pub pending_only: Option<bool>,
    pub failed_only: Option<bool>,
    pub min_priority: Option<P2pMessagePriority>,
    pub expired_only: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub expires_before: Option<DateTime<Utc>>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl P2pMessageQueue {
    /// Create a new P2P message queue entry
    pub async fn create(pool: &Pool<Sqlite>, message: &P2pMessageQueue) -> Result<()> {
        sqlx::query(
            "INSERT INTO p2p_message_queue (
                id, from_peer_id, to_peer_id, message_type, priority, payload,
                conversation_id, agent_chain_execution_id, status, retry_count, max_retries,
                expires_at, sent_at, delivered_at, error_details, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(message.id)
        .bind(&message.from_peer_id)
        .bind(&message.to_peer_id)
        .bind(message.message_type as i32)
        .bind(message.priority as i32)
        .bind(&message.payload)
        .bind(message.conversation_id)
        .bind(message.agent_chain_execution_id)
        .bind(message.status as i32)
        .bind(message.retry_count)
        .bind(message.max_retries)
        .bind(message.expires_at)
        .bind(message.sent_at)
        .bind(message.delivered_at)
        .bind(&message.error_details)
        .bind(message.created_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get P2P message by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &Uuid) -> Result<Option<P2pMessageQueue>> {
        let row = sqlx::query(
            "SELECT id, from_peer_id, to_peer_id, message_type, priority, payload,
                    conversation_id, agent_chain_execution_id, status, retry_count, max_retries,
                    expires_at, sent_at, delivered_at, error_details, created_at
             FROM p2p_message_queue WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(P2pMessageQueue {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                from_peer_id: row.get("from_peer_id"),
                to_peer_id: row.get("to_peer_id"),
                message_type: P2pMessageType::try_from(row.get::<i32, _>("message_type"))?,
                priority: P2pMessagePriority::try_from(row.get::<i32, _>("priority"))?,
                payload: row.get("payload"),
                conversation_id: row
                    .get::<Option<Vec<u8>>, _>("conversation_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                agent_chain_execution_id: row
                    .get::<Option<Vec<u8>>, _>("agent_chain_execution_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                status: P2pMessageStatus::try_from(row.get::<i32, _>("status"))?,
                retry_count: row.get("retry_count"),
                max_retries: row.get("max_retries"),
                expires_at: row.get("expires_at"),
                sent_at: row.get("sent_at"),
                delivered_at: row.get("delivered_at"),
                error_details: row.get("error_details"),
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// List P2P messages with filtering
    pub async fn list(
        pool: &Pool<Sqlite>,
        filter: &P2pMessageQueueFilter,
    ) -> Result<Vec<P2pMessageQueue>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, from_peer_id, to_peer_id, message_type, priority, payload,
                    conversation_id, agent_chain_execution_id, status, retry_count, max_retries,
                    expires_at, sent_at, delivered_at, error_details, created_at
             FROM p2p_message_queue",
        );

        let mut where_conditions = Vec::new();

        if let Some(from_peer_id) = &filter.from_peer_id {
            where_conditions.push(format!("from_peer_id = '{from_peer_id}'"));
        }

        if let Some(to_peer_id) = &filter.to_peer_id {
            where_conditions.push(format!("to_peer_id = '{to_peer_id}'"));
        }

        if let Some(message_type) = filter.message_type {
            where_conditions.push(format!("message_type = {}", message_type as i32));
        }

        if let Some(priority) = filter.priority {
            where_conditions.push(format!("priority = {}", priority as i32));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = {}", status as i32));
        }

        if let Some(conversation_id) = &filter.conversation_id {
            where_conditions.push(format!("conversation_id = '{conversation_id}'"));
        }

        if let Some(agent_chain_execution_id) = &filter.agent_chain_execution_id {
            where_conditions.push(format!(
                "agent_chain_execution_id = '{agent_chain_execution_id}'"
            ));
        }

        if filter.pending_only.unwrap_or(false) {
            where_conditions.push("status = 0".to_string()); // Pending status
        }

        if filter.failed_only.unwrap_or(false) {
            where_conditions.push("status = 3".to_string()); // Failed status
        }

        if let Some(min_priority) = filter.min_priority {
            where_conditions.push(format!("priority >= {}", min_priority as i32));
        }

        if filter.expired_only.unwrap_or(false) {
            let now = Utc::now();
            where_conditions.push(format!(
                "expires_at < '{}'",
                now.format("%Y-%m-%d %H:%M:%S")
            ));
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

        if let Some(expires_before) = &filter.expires_before {
            where_conditions.push(format!(
                "expires_at <= '{}'",
                expires_before.format("%Y-%m-%d %H:%M:%S")
            ));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY priority DESC, created_at ASC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build().fetch_all(pool).await?;
        let mut messages = Vec::new();

        for row in rows {
            messages.push(P2pMessageQueue {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                from_peer_id: row.get("from_peer_id"),
                to_peer_id: row.get("to_peer_id"),
                message_type: P2pMessageType::try_from(row.get::<i32, _>("message_type"))?,
                priority: P2pMessagePriority::try_from(row.get::<i32, _>("priority"))?,
                payload: row.get("payload"),
                conversation_id: row
                    .get::<Option<Vec<u8>>, _>("conversation_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                agent_chain_execution_id: row
                    .get::<Option<Vec<u8>>, _>("agent_chain_execution_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                status: P2pMessageStatus::try_from(row.get::<i32, _>("status"))?,
                retry_count: row.get("retry_count"),
                max_retries: row.get("max_retries"),
                expires_at: row.get("expires_at"),
                sent_at: row.get("sent_at"),
                delivered_at: row.get("delivered_at"),
                error_details: row.get("error_details"),
                created_at: row.get("created_at"),
            });
        }

        Ok(messages)
    }

    /// Update P2P message
    pub async fn update(pool: &Pool<Sqlite>, message: &P2pMessageQueue) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE p2p_message_queue SET
                from_peer_id = ?, to_peer_id = ?, message_type = ?, priority = ?, payload = ?,
                conversation_id = ?, agent_chain_execution_id = ?, status = ?, retry_count = ?,
                max_retries = ?, expires_at = ?, sent_at = ?, delivered_at = ?, error_details = ?
             WHERE id = ?",
        )
        .bind(&message.from_peer_id)
        .bind(&message.to_peer_id)
        .bind(message.message_type as i32)
        .bind(message.priority as i32)
        .bind(&message.payload)
        .bind(message.conversation_id)
        .bind(message.agent_chain_execution_id)
        .bind(message.status as i32)
        .bind(message.retry_count)
        .bind(message.max_retries)
        .bind(message.expires_at)
        .bind(message.sent_at)
        .bind(message.delivered_at)
        .bind(&message.error_details)
        .bind(message.id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {} not found",
                message.id
            )));
        }

        Ok(())
    }

    /// Delete P2P message
    pub async fn delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM p2p_message_queue WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get pending messages (ready to send)
    pub async fn get_pending_messages(
        pool: &Pool<Sqlite>,
        limit: Option<u32>,
    ) -> Result<Vec<P2pMessageQueue>> {
        let filter = P2pMessageQueueFilter {
            from_peer_id: None,
            to_peer_id: None,
            message_type: None,
            priority: None,
            status: Some(P2pMessageStatus::Pending),
            conversation_id: None,
            agent_chain_execution_id: None,
            pending_only: Some(true),
            failed_only: None,
            min_priority: None,
            expired_only: None,
            created_after: None,
            created_before: None,
            expires_before: None,
            limit,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get high priority pending messages
    pub async fn get_high_priority_pending(
        pool: &Pool<Sqlite>,
        limit: Option<u32>,
    ) -> Result<Vec<P2pMessageQueue>> {
        let filter = P2pMessageQueueFilter {
            from_peer_id: None,
            to_peer_id: None,
            message_type: None,
            priority: None,
            status: Some(P2pMessageStatus::Pending),
            conversation_id: None,
            agent_chain_execution_id: None,
            pending_only: Some(true),
            failed_only: None,
            min_priority: Some(P2pMessagePriority::High),
            expired_only: None,
            created_after: None,
            created_before: None,
            expires_before: None,
            limit,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get failed messages that can be retried
    pub async fn get_retryable_failed_messages(
        pool: &Pool<Sqlite>,
        limit: Option<u32>,
    ) -> Result<Vec<P2pMessageQueue>> {
        let rows = sqlx::query(
            "SELECT id, from_peer_id, to_peer_id, message_type, priority, payload,
                    conversation_id, agent_chain_execution_id, status, retry_count, max_retries,
                    expires_at, sent_at, delivered_at, error_details, created_at
             FROM p2p_message_queue 
             WHERE status = 3 AND retry_count < max_retries 
             ORDER BY priority DESC, created_at ASC
             LIMIT ?",
        )
        .bind(limit.unwrap_or(100) as i64)
        .fetch_all(pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(P2pMessageQueue {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                from_peer_id: row.get("from_peer_id"),
                to_peer_id: row.get("to_peer_id"),
                message_type: P2pMessageType::try_from(row.get::<i32, _>("message_type"))?,
                priority: P2pMessagePriority::try_from(row.get::<i32, _>("priority"))?,
                payload: row.get("payload"),
                conversation_id: row
                    .get::<Option<Vec<u8>>, _>("conversation_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                agent_chain_execution_id: row
                    .get::<Option<Vec<u8>>, _>("agent_chain_execution_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                status: P2pMessageStatus::try_from(row.get::<i32, _>("status"))?,
                retry_count: row.get("retry_count"),
                max_retries: row.get("max_retries"),
                expires_at: row.get("expires_at"),
                sent_at: row.get("sent_at"),
                delivered_at: row.get("delivered_at"),
                error_details: row.get("error_details"),
                created_at: row.get("created_at"),
            });
        }

        Ok(messages)
    }

    /// Mark message as sent
    pub async fn mark_as_sent(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE p2p_message_queue SET status = 1, sent_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Mark message as delivered
    pub async fn mark_as_delivered(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE p2p_message_queue SET status = 2, delivered_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Mark message as failed and increment retry count
    pub async fn mark_as_failed(
        pool: &Pool<Sqlite>,
        id: &Uuid,
        error_details: Option<String>,
    ) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE p2p_message_queue SET status = 3, retry_count = retry_count + 1, error_details = ? WHERE id = ?"
        )
        .bind(&error_details)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Reset message for retry
    pub async fn reset_for_retry(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE p2p_message_queue SET status = 0, sent_at = NULL, delivered_at = NULL, error_details = NULL WHERE id = ?"
        )
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "P2P message with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Mark expired messages
    pub async fn mark_expired_messages(pool: &Pool<Sqlite>) -> Result<u64> {
        let now = Utc::now();

        let affected = sqlx::query(
            "UPDATE p2p_message_queue SET status = 4 WHERE expires_at < ? AND status IN (0, 1)",
        )
        .bind(now)
        .execute(pool)
        .await?
        .rows_affected();

        Ok(affected)
    }

    /// Get expired messages
    pub async fn get_expired_messages(pool: &Pool<Sqlite>) -> Result<Vec<P2pMessageQueue>> {
        let filter = P2pMessageQueueFilter {
            from_peer_id: None,
            to_peer_id: None,
            message_type: None,
            priority: None,
            status: None,
            conversation_id: None,
            agent_chain_execution_id: None,
            pending_only: None,
            failed_only: None,
            min_priority: None,
            expired_only: Some(true),
            created_after: None,
            created_before: None,
            expires_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Count messages by status
    pub async fn count_by_status(pool: &Pool<Sqlite>, status: P2pMessageStatus) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM p2p_message_queue WHERE status = ?")
            .bind(status as i32)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count messages by priority
    pub async fn count_by_priority(
        pool: &Pool<Sqlite>,
        priority: P2pMessagePriority,
    ) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM p2p_message_queue WHERE priority = ?")
            .bind(priority as i32)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Get queue statistics
    pub async fn get_queue_stats(pool: &Pool<Sqlite>) -> Result<P2pQueueStats> {
        let stats_row = sqlx::query(
            "SELECT 
                COUNT(CASE WHEN status = 0 THEN 1 END) as pending_count,
                COUNT(CASE WHEN status = 1 THEN 1 END) as sent_count,
                COUNT(CASE WHEN status = 2 THEN 1 END) as delivered_count,
                COUNT(CASE WHEN status = 3 THEN 1 END) as failed_count,
                COUNT(CASE WHEN status = 4 THEN 1 END) as expired_count,
                COUNT(CASE WHEN priority = 0 THEN 1 END) as low_priority,
                COUNT(CASE WHEN priority = 1 THEN 1 END) as normal_priority,
                COUNT(CASE WHEN priority = 2 THEN 1 END) as high_priority,
                COUNT(CASE WHEN priority = 3 THEN 1 END) as urgent_priority,
                AVG(retry_count) as avg_retry_count
             FROM p2p_message_queue",
        )
        .fetch_one(pool)
        .await?;

        Ok(P2pQueueStats {
            pending_messages: stats_row.get::<i64, _>("pending_count") as u32,
            sent_messages: stats_row.get::<i64, _>("sent_count") as u32,
            delivered_messages: stats_row.get::<i64, _>("delivered_count") as u32,
            failed_messages: stats_row.get::<i64, _>("failed_count") as u32,
            expired_messages: stats_row.get::<i64, _>("expired_count") as u32,
            low_priority_messages: stats_row.get::<i64, _>("low_priority") as u32,
            normal_priority_messages: stats_row.get::<i64, _>("normal_priority") as u32,
            high_priority_messages: stats_row.get::<i64, _>("high_priority") as u32,
            urgent_priority_messages: stats_row.get::<i64, _>("urgent_priority") as u32,
            average_retry_count: stats_row
                .get::<Option<f64>, _>("avg_retry_count")
                .unwrap_or(0.0),
        })
    }

    /// Clean up old messages
    pub async fn cleanup_old_messages(
        pool: &Pool<Sqlite>,
        cutoff_date: &DateTime<Utc>,
    ) -> Result<u64> {
        let affected = sqlx::query(
            "DELETE FROM p2p_message_queue WHERE created_at < ? AND status IN (2, 3, 4)",
        )
        .bind(cutoff_date)
        .execute(pool)
        .await?
        .rows_affected();

        Ok(affected)
    }

    /// Delete messages for conversation
    pub async fn delete_by_conversation(
        pool: &Pool<Sqlite>,
        conversation_id: &Uuid,
    ) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM p2p_message_queue WHERE conversation_id = ?")
            .bind(conversation_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Delete messages for agent chain execution
    pub async fn delete_by_agent_chain_execution(
        pool: &Pool<Sqlite>,
        agent_chain_execution_id: &Uuid,
    ) -> Result<u64> {
        let affected =
            sqlx::query("DELETE FROM p2p_message_queue WHERE agent_chain_execution_id = ?")
                .bind(agent_chain_execution_id)
                .execute(pool)
                .await?
                .rows_affected();

        Ok(affected)
    }

    /// Bulk update message status
    pub async fn bulk_update_status(
        pool: &Pool<Sqlite>,
        message_ids: &[Uuid],
        status: P2pMessageStatus,
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let mut total_affected = 0u64;

        for message_id in message_ids {
            let affected = sqlx::query("UPDATE p2p_message_queue SET status = ? WHERE id = ?")
                .bind(status as i32)
                .bind(message_id)
                .execute(&mut *tx)
                .await?
                .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Get messages for peer
    pub async fn get_messages_for_peer(
        pool: &Pool<Sqlite>,
        peer_id: &str,
    ) -> Result<Vec<P2pMessageQueue>> {
        let filter = P2pMessageQueueFilter {
            from_peer_id: None,
            to_peer_id: Some(peer_id.to_string()),
            message_type: None,
            priority: None,
            status: None,
            conversation_id: None,
            agent_chain_execution_id: None,
            pending_only: None,
            failed_only: None,
            min_priority: None,
            expired_only: None,
            created_after: None,
            created_before: None,
            expires_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get messages from peer
    pub async fn get_messages_from_peer(
        pool: &Pool<Sqlite>,
        peer_id: &str,
    ) -> Result<Vec<P2pMessageQueue>> {
        let filter = P2pMessageQueueFilter {
            from_peer_id: Some(peer_id.to_string()),
            to_peer_id: None,
            message_type: None,
            priority: None,
            status: None,
            conversation_id: None,
            agent_chain_execution_id: None,
            pending_only: None,
            failed_only: None,
            min_priority: None,
            expired_only: None,
            created_after: None,
            created_before: None,
            expires_before: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pQueueStats {
    pub pending_messages: u32,
    pub sent_messages: u32,
    pub delivered_messages: u32,
    pub failed_messages: u32,
    pub expired_messages: u32,
    pub low_priority_messages: u32,
    pub normal_priority_messages: u32,
    pub high_priority_messages: u32,
    pub urgent_priority_messages: u32,
    pub average_retry_count: f64,
}
