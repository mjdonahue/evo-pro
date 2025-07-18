use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub recipient_id: Uuid,
    pub message: String,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub task_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationFilter {
    pub recipient_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub is_read: Option<bool>,
    pub unread_only: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub search_term: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl Notification {
    /// Create a new notification
    pub async fn create(pool: &Pool<Sqlite>, notification: &Notification) -> Result<()> {
        sqlx::query(
            "INSERT INTO notifications (
                id, recipient_id, message, is_read, created_at, updated_at,
                task_id, event_id, agent_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(notification.id)
        .bind(notification.recipient_id)
        .bind(&notification.message)
        .bind(notification.is_read)
        .bind(notification.created_at)
        .bind(notification.updated_at)
        .bind(notification.task_id)
        .bind(notification.event_id)
        .bind(notification.agent_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get notification by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &Uuid) -> Result<Option<Notification>> {
        let row = sqlx::query(
            "SELECT id, recipient_id, message, is_read, created_at, updated_at,
                    task_id, event_id, agent_id
             FROM notifications WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(Notification {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                recipient_id: row
                    .get::<Vec<u8>, _>("recipient_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                message: row.get("message"),
                is_read: row.get::<i64, _>("is_read") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                task_id: row
                    .get::<Option<Vec<u8>>, _>("task_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                event_id: row
                    .get::<Option<Vec<u8>>, _>("event_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                agent_id: row
                    .get::<Option<Vec<u8>>, _>("agent_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List notifications with filtering
    pub async fn list(
        pool: &Pool<Sqlite>,
        filter: &NotificationFilter,
    ) -> Result<Vec<Notification>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, recipient_id, message, is_read, created_at, updated_at,
                    task_id, event_id, agent_id
             FROM notifications",
        );

        let mut where_conditions = Vec::new();

        if let Some(recipient_id) = &filter.recipient_id {
            where_conditions.push(format!("recipient_id = '{recipient_id}'"));
        }

        if let Some(task_id) = &filter.task_id {
            where_conditions.push(format!("task_id = '{task_id}'"));
        }

        if let Some(event_id) = &filter.event_id {
            where_conditions.push(format!("event_id = '{event_id}'"));
        }

        if let Some(agent_id) = &filter.agent_id {
            where_conditions.push(format!("agent_id = '{agent_id}'"));
        }

        if let Some(is_read) = filter.is_read {
            where_conditions.push(format!("is_read = {}", if is_read { 1 } else { 0 }));
        }

        if filter.unread_only.unwrap_or(false) {
            where_conditions.push("is_read = 0".to_string());
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

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!("message LIKE '%{search_term}%'"));
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

        let rows = qb.build().fetch_all(pool).await?;
        let mut notifications = Vec::new();

        for row in rows {
            notifications.push(Notification {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                recipient_id: row
                    .get::<Vec<u8>, _>("recipient_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                message: row.get("message"),
                is_read: row.get::<i64, _>("is_read") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                task_id: row
                    .get::<Option<Vec<u8>>, _>("task_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                event_id: row
                    .get::<Option<Vec<u8>>, _>("event_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
                agent_id: row
                    .get::<Option<Vec<u8>>, _>("agent_id")
                    .map(|v| {
                        v.try_into()
                            .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))
                    })
                    .transpose()?,
            });
        }

        Ok(notifications)
    }

    /// Update notification
    pub async fn update(pool: &Pool<Sqlite>, notification: &Notification) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE notifications SET
                recipient_id = ?, message = ?, is_read = ?, updated_at = ?,
                task_id = ?, event_id = ?, agent_id = ?
             WHERE id = ?",
        )
        .bind(notification.recipient_id)
        .bind(&notification.message)
        .bind(notification.is_read)
        .bind(notification.updated_at)
        .bind(notification.task_id)
        .bind(notification.event_id)
        .bind(notification.agent_id)
        .bind(notification.id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Notification with ID {} not found",
                notification.id
            )));
        }

        Ok(())
    }

    /// Delete notification
    pub async fn delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM notifications WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Notification with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get notifications for recipient
    pub async fn get_by_recipient(
        pool: &Pool<Sqlite>,
        recipient_id: &Uuid,
    ) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: Some(*recipient_id),
            task_id: None,
            event_id: None,
            agent_id: None,
            is_read: None,
            unread_only: None,
            created_after: None,
            created_before: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get unread notifications for recipient
    pub async fn get_unread_by_recipient(
        pool: &Pool<Sqlite>,
        recipient_id: &Uuid,
    ) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: Some(*recipient_id),
            task_id: None,
            event_id: None,
            agent_id: None,
            is_read: Some(false),
            unread_only: Some(true),
            created_after: None,
            created_before: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get notifications for task
    pub async fn get_by_task(pool: &Pool<Sqlite>, task_id: &Uuid) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: None,
            task_id: Some(*task_id),
            event_id: None,
            agent_id: None,
            is_read: None,
            unread_only: None,
            created_after: None,
            created_before: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get notifications for event
    pub async fn get_by_event(pool: &Pool<Sqlite>, event_id: &Uuid) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: None,
            task_id: None,
            event_id: Some(*event_id),
            agent_id: None,
            is_read: None,
            unread_only: None,
            created_after: None,
            created_before: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get notifications for agent
    pub async fn get_by_agent(pool: &Pool<Sqlite>, agent_id: &Uuid) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: None,
            task_id: None,
            event_id: None,
            agent_id: Some(*agent_id),
            is_read: None,
            unread_only: None,
            created_after: None,
            created_before: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Mark notification as read
    pub async fn mark_as_read(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE notifications SET is_read = 1, updated_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Notification with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Mark notification as unread
    pub async fn mark_as_unread(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE notifications SET is_read = 0, updated_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Notification with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Mark all notifications as read for recipient
    pub async fn mark_all_as_read_for_recipient(
        pool: &Pool<Sqlite>,
        recipient_id: &Uuid,
    ) -> Result<u64> {
        let now = Utc::now();

        let affected = sqlx::query(
            "UPDATE notifications SET is_read = 1, updated_at = ? WHERE recipient_id = ? AND is_read = 0"
        )
        .bind(now)
        .bind(recipient_id)
        .execute(pool)
        .await?
        .rows_affected();

        Ok(affected)
    }

    /// Count unread notifications for recipient
    pub async fn count_unread_for_recipient(
        pool: &Pool<Sqlite>,
        recipient_id: &Uuid,
    ) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM notifications WHERE recipient_id = ? AND is_read = 0",
        )
        .bind(recipient_id)
        .fetch_one(pool)
        .await?;

        Ok(row.get("count"))
    }

    /// Count total notifications for recipient
    pub async fn count_for_recipient(pool: &Pool<Sqlite>, recipient_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM notifications WHERE recipient_id = ?")
            .bind(recipient_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Delete all read notifications for recipient
    pub async fn delete_read_for_recipient(
        pool: &Pool<Sqlite>,
        recipient_id: &Uuid,
    ) -> Result<u64> {
        let affected =
            sqlx::query("DELETE FROM notifications WHERE recipient_id = ? AND is_read = 1")
                .bind(recipient_id)
                .execute(pool)
                .await?
                .rows_affected();

        Ok(affected)
    }

    /// Delete notifications older than specified date
    pub async fn delete_older_than(
        pool: &Pool<Sqlite>,
        cutoff_date: &DateTime<Utc>,
    ) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM notifications WHERE created_at < ?")
            .bind(cutoff_date)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Bulk mark notifications as read
    pub async fn bulk_mark_as_read(pool: &Pool<Sqlite>, notification_ids: &[Uuid]) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let now = Utc::now();
        let mut total_affected = 0u64;

        for notification_id in notification_ids {
            let affected =
                sqlx::query("UPDATE notifications SET is_read = 1, updated_at = ? WHERE id = ?")
                    .bind(now)
                    .bind(notification_id)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Bulk delete notifications
    pub async fn bulk_delete(pool: &Pool<Sqlite>, notification_ids: &[Uuid]) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let mut total_affected = 0u64;

        for notification_id in notification_ids {
            let affected = sqlx::query("DELETE FROM notifications WHERE id = ?")
                .bind(notification_id)
                .execute(&mut *tx)
                .await?
                .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }

    /// Search notifications by message content
    pub async fn search_by_message(
        pool: &Pool<Sqlite>,
        search_term: &str,
        limit: Option<u32>,
    ) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: None,
            task_id: None,
            event_id: None,
            agent_id: None,
            is_read: None,
            unread_only: None,
            created_after: None,
            created_before: None,
            search_term: Some(search_term.to_string()),
            limit,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get recent notifications for recipient
    pub async fn get_recent_for_recipient(
        pool: &Pool<Sqlite>,
        recipient_id: &Uuid,
        limit: u32,
    ) -> Result<Vec<Notification>> {
        let filter = NotificationFilter {
            recipient_id: Some(*recipient_id),
            task_id: None,
            event_id: None,
            agent_id: None,
            is_read: None,
            unread_only: None,
            created_after: None,
            created_before: None,
            search_term: None,
            limit: Some(limit),
            offset: None,
        };

        Self::list(pool, &filter).await
    }
}
