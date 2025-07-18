use chrono::{DateTime, Utc};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::entities::messages::Message;
use crate::entities::events::Event;
use crate::error::Result;
use crate::storage::db::DatabaseManager;

impl DatabaseManager {
    /// Get old messages by user that are older than the cutoff date
    #[instrument(err, skip(self))]
    pub async fn get_old_messages_by_user(&self, user_id: &Uuid, cutoff_date: DateTime<Utc>) -> Result<Vec<Message>> {
        debug!("Getting old messages for user {} before {}", user_id, cutoff_date);

        Ok(sqlx::query_as!(
            Message,
            r#"SELECT 
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_id AS "parent_id: _", content, status as "status: _",
                refs AS "refs: _", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                reply_to_id AS "reply_to_id: _", branch_conversation_id AS "branch_conversation_id: _",
                parent_message_id AS "parent_message_id: _"
            FROM messages 
            WHERE sender_id = ? AND created_at < ?
            ORDER BY created_at"#,
            user_id,
            cutoff_date
        )
        .fetch_all(&self.pool)
        .await?)
    }

    /// Delete old messages by user that are older than the cutoff date
    #[instrument(err, skip(self))]
    pub async fn delete_old_messages_by_user(&self, user_id: &Uuid, cutoff_date: DateTime<Utc>) -> Result<i64> {
        debug!("Deleting old messages for user {} before {}", user_id, cutoff_date);

        let result = sqlx::query!(
            "DELETE FROM messages WHERE sender_id = ? AND created_at < ?",
            user_id,
            cutoff_date
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get old events by user that are older than the cutoff date
    #[instrument(err, skip(self))]
    pub async fn get_old_events_by_user(&self, user_id: &Uuid, cutoff_date: DateTime<Utc>) -> Result<Vec<Event>> {
        debug!("Getting old events for user {} before {}", user_id, cutoff_date);

        Ok(sqlx::query_as!(
            Event,
            r#"SELECT id AS "id: _", title, description, event_type AS "event_type: _", status AS "status: _",
                    start_time AS "start_time: _", end_time AS "end_time: _", is_all_day_event, timezone_sid_key AS "timezone_sid_key: _",
                    location AS "location: _", virtual_meeting_url AS "virtual_meeting_url: _", meeting_platform AS "meeting_platform: _",
                    is_recurrence, recurrence_rule AS "recurrence_rule: _", recurrence_parent_id AS "recurrence_parent_id: _",
                    agent_participation AS "agent_participation: _", requires_transcription, requires_summarization,
                    agent_capabilities AS "agent_capabilities: _", is_private, allow_guests, max_attendees, requires_approval,
                    agenda, meeting_notes, transcription, summary, action_items AS "action_items: _", is_child_event, is_group_event,
                    is_archived, event_relation AS "event_relation: _", activity_date AS "activity_date: _",
                    duration_in_minutes AS "duration_in_minutes: _", show_as AS "show_as: _", is_reminder_set, reminder_date_time AS "reminder_date_time: _",
                    metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                    plan_id AS "plan_id: _", task_id AS "task_id: _", created_by_user_id AS "created_by_user_id: _",
                    last_modified_by_user_id AS "last_modified_by_user_id: _", workspace_id AS "workspace_id: _",
                    parent_event_id AS "parent_event_id: _"
             FROM events 
             WHERE created_by_user_id = ? AND end_time < ?
             ORDER BY end_time"#,
            user_id,
            cutoff_date
        )
        .fetch_all(&self.pool)
        .await?)
    }

    /// Delete old events by user that are older than the cutoff date
    #[instrument(err, skip(self))]
    pub async fn delete_old_events_by_user(&self, user_id: &Uuid, cutoff_date: DateTime<Utc>) -> Result<i64> {
        debug!("Deleting old events for user {} before {}", user_id, cutoff_date);

        let result = sqlx::query!(
            "DELETE FROM events WHERE created_by_user_id = ? AND end_time < ?",
            user_id,
            cutoff_date
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get messages by conversation ID with optional date range and deleted status
    #[instrument(err, skip(self))]
    pub async fn get_messages_by_conversation_id(
        &self,
        conversation_id: &Uuid,
        include_deleted: bool,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>> {
        debug!(
            "Getting messages for conversation {} (include_deleted: {}, start_date: {:?}, end_date: {:?})",
            conversation_id, include_deleted, start_date, end_date
        );

        let mut query = String::from(
            r#"SELECT 
                id AS "id: _", conversation_id AS "conversation_id: _", workspace_id AS "workspace_id: _", sender_id AS "sender_id: _",
                parent_id AS "parent_id: _", content, status as "status: _",
                refs AS "refs: _", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                reply_to_id AS "reply_to_id: _", branch_conversation_id AS "branch_conversation_id: _",
                parent_message_id AS "parent_message_id: _"
            FROM messages 
            WHERE conversation_id = ?"#,
        );

        // Add date range filters if provided
        if let Some(start) = start_date {
            query.push_str(" AND created_at >= ?");
        }

        if let Some(end) = end_date {
            query.push_str(" AND created_at <= ?");
        }

        // Add status filter if not including deleted
        if !include_deleted {
            query.push_str(" AND status != 3"); // 3 = Failed status
        }

        query.push_str(" ORDER BY created_at");

        // Build the query with the appropriate bindings
        let mut query_builder = sqlx::query_as::<_, Message>(&query).bind(conversation_id);

        if let Some(start) = start_date {
            query_builder = query_builder.bind(start);
        }

        if let Some(end) = end_date {
            query_builder = query_builder.bind(end);
        }

        Ok(query_builder.fetch_all(&self.pool).await?)
    }

    /// Get events by user ID with optional date range and deleted status
    #[instrument(err, skip(self))]
    pub async fn get_events_by_user_id(
        &self,
        user_id: &Uuid,
        include_deleted: bool,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<Event>> {
        debug!(
            "Getting events for user {} (include_deleted: {}, start_date: {:?}, end_date: {:?})",
            user_id, include_deleted, start_date, end_date
        );

        let mut query = String::from(
            r#"SELECT id AS "id: _", title, description, event_type AS "event_type: _", status AS "status: _",
                    start_time AS "start_time: _", end_time AS "end_time: _", is_all_day_event, timezone_sid_key AS "timezone_sid_key: _",
                    location AS "location: _", virtual_meeting_url AS "virtual_meeting_url: _", meeting_platform AS "meeting_platform: _",
                    is_recurrence, recurrence_rule AS "recurrence_rule: _", recurrence_parent_id AS "recurrence_parent_id: _",
                    agent_participation AS "agent_participation: _", requires_transcription, requires_summarization,
                    agent_capabilities AS "agent_capabilities: _", is_private, allow_guests, max_attendees, requires_approval,
                    agenda, meeting_notes, transcription, summary, action_items AS "action_items: _", is_child_event, is_group_event,
                    is_archived, event_relation AS "event_relation: _", activity_date AS "activity_date: _",
                    duration_in_minutes AS "duration_in_minutes: _", show_as AS "show_as: _", is_reminder_set, reminder_date_time AS "reminder_date_time: _",
                    metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                    plan_id AS "plan_id: _", task_id AS "task_id: _", created_by_user_id AS "created_by_user_id: _",
                    last_modified_by_user_id AS "last_modified_by_user_id: _", workspace_id AS "workspace_id: _",
                    parent_event_id AS "parent_event_id: _"
             FROM events 
             WHERE created_by_user_id = ?"#,
        );

        // Add date range filters if provided
        if let Some(start) = start_date {
            query.push_str(" AND start_time >= ?");
        }

        if let Some(end) = end_date {
            query.push_str(" AND end_time <= ?");
        }

        // Add status filter if not including deleted
        if !include_deleted {
            query.push_str(" AND status != 2"); // 2 = Cancelled status
        }

        query.push_str(" ORDER BY start_time");

        // Build the query with the appropriate bindings
        let mut query_builder = sqlx::query_as::<_, Event>(&query).bind(user_id);

        if let Some(start) = start_date {
            query_builder = query_builder.bind(start);
        }

        if let Some(end) = end_date {
            query_builder = query_builder.bind(end);
        }

        Ok(query_builder.fetch_all(&self.pool).await?)
    }
}