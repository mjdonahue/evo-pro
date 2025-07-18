use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row, Sqlite};
use sqlx::types::Json;
use serde_json::{Value};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_type")]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Meeting = 0,
    Call = 1,
    Email = 2,
    Task = 3,
    Other = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "event_status")]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Scheduled = 0,
    Completed = 1,
    Cancelled = 2,
    Rescheduled = 3,
}

#[boilermates("CreateEvent")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    #[boilermates(not_in("CreateEvent"))]
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_type: EventType,
    pub status: EventStatus,
    // Event timing
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_all_day_event: bool,
    pub timezone_sid_key: Option<String>,
    // Location and format
    pub location: Option<String>,
    pub virtual_meeting_url: Option<String>,
    pub meeting_platform: Option<String>,
    // Recurrence
    pub is_recurrence: bool,
    pub recurrence_rule: Option<Json<Value>>,
    pub recurrence_parent_id: Option<Uuid>,
    // Agent participation configuration
    pub agent_participation: Option<Json<Value>>,
    pub requires_transcription: bool,
    pub requires_summarization: bool,
    pub agent_capabilities: Option<Json<Value>>,
    // Event configuration related fields
    pub is_private: bool,
    pub allow_guests: bool,
    pub max_attendees: Option<i64>,
    pub requires_approval: bool,
    // Meeting specific fields
    pub agenda: Option<String>,
    pub meeting_notes: Option<String>,
    pub transcription: Option<String>,
    pub summary: Option<String>,
    pub action_items: Option<Json<Value>>,
    // Additional meeting fields
    pub is_child_event: bool,
    pub is_group_event: bool,
    pub is_archived: bool,
    pub event_relation: Option<Json<Value>>,
    pub activity_date: Option<DateTime<Utc>>,
    pub duration_in_minutes: Option<i64>,
    pub show_as: Option<String>,
    pub is_reminder_set: bool,
    pub reminder_date_time: DateTime<Utc>,
    pub metadata: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Relationships
    pub plan_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub created_by_user_id: Option<Uuid>,
    pub last_modified_by_user_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub parent_event_id: Option<Uuid>,
}
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventFilter {
    pub event_type: Option<EventType>,
    pub status: Option<EventStatus>,
    pub workspace_id: Option<Uuid>,
    pub created_by_user_id: Option<Uuid>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub location: Option<String>,
    pub is_all_day_event: Option<bool>,
    pub is_private: Option<bool>,
    pub is_archived: Option<bool>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new event
    #[instrument(err, skip(self))]
    pub async fn create_event(&self, event: &Event) -> Result<Event> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let metadata = event.metadata.as_deref();
        let action_items = event.action_items.as_deref();
        let event_relation = event.event_relation.as_deref();
        let recurrence_rule = event.recurrence_rule.as_deref();
        let agent_participation = event.agent_participation.as_deref();
        let agent_capabilities = event.agent_capabilities.as_deref();
        
        Ok(sqlx::query_as!(
            Event,
            r#"INSERT INTO events (
                id, title, description, event_type, status, start_time, end_time, is_all_day_event,
                timezone_sid_key, location, virtual_meeting_url, meeting_platform, is_recurrence,
                recurrence_rule, recurrence_parent_id, agent_participation, requires_transcription,
                requires_summarization, agent_capabilities, is_private, allow_guests, max_attendees,
                requires_approval, agenda, meeting_notes, transcription, summary, action_items,
                is_child_event, is_group_event, is_archived, event_relation, activity_date,
                duration_in_minutes, show_as, is_reminder_set, reminder_date_time, metadata,
                created_at, updated_at, plan_id, task_id, created_by_user_id, last_modified_by_user_id,
                workspace_id, parent_event_id   
            ) VALUES (
             ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
             ) RETURNING id AS "id: _", title AS "title: _", description AS "description: _", event_type AS "event_type: _", status AS "status: _", start_time AS "start_time: _", end_time AS "end_time: _", is_all_day_event AS "is_all_day_event: _",
                timezone_sid_key AS "timezone_sid_key: _", location AS "location: _", virtual_meeting_url AS "virtual_meeting_url: _", meeting_platform AS "meeting_platform: _", is_recurrence AS "is_recurrence: _",
                recurrence_rule AS "recurrence_rule: _", recurrence_parent_id AS "recurrence_parent_id: _", agent_participation AS "agent_participation: _", requires_transcription AS "requires_transcription: _",
                requires_summarization AS "requires_summarization: _", agent_capabilities AS "agent_capabilities: _", is_private AS "is_private: _", allow_guests AS "allow_guests: _",
                max_attendees AS "max_attendees: _", requires_approval AS "requires_approval: _", agenda AS "agenda: _", meeting_notes AS "meeting_notes: _", transcription AS "transcription: _", summary AS "summary: _",
                action_items AS "action_items: _", is_child_event AS "is_child_event: _", is_group_event AS "is_group_event: _", is_archived AS "is_archived: _", event_relation AS "event_relation: _", activity_date AS "activity_date: _",
                duration_in_minutes AS "duration_in_minutes: _", show_as AS "show_as: _", is_reminder_set AS "is_reminder_set: _", reminder_date_time AS "reminder_date_time: _", metadata AS "metadata: _",
                created_at AS "created_at: _", updated_at AS "updated_at: _", plan_id AS "plan_id: _", task_id AS "task_id: _", created_by_user_id AS "created_by_user_id: _", last_modified_by_user_id AS "last_modified_by_user_id: _",
                workspace_id AS "workspace_id: _", parent_event_id AS "parent_event_id: _"
            "#,
            id,
            event.title,
            event.description,
            event.event_type,
            event.status,
            event.start_time,
            event.end_time,
            event.is_all_day_event,
            event.timezone_sid_key,
            event.location,
            event.virtual_meeting_url,  
            event.meeting_platform,
            event.is_recurrence,
            recurrence_rule,
            event.recurrence_parent_id,
            agent_participation,
            event.requires_transcription,   
            event.requires_summarization,
            agent_capabilities,
            event.is_private,
            event.allow_guests,
            event.max_attendees,
            event.requires_approval,    
            event.agenda,
            event.meeting_notes,
            event.transcription,
            event.summary,
            action_items,
            event.is_child_event,   
            event.is_group_event,
            event.is_archived,
            event_relation,
            event.activity_date,
            event.duration_in_minutes,
            event.show_as,  
            event.is_reminder_set,
            event.reminder_date_time,
            metadata,
            now,
            now,
            event.plan_id,      
            event.task_id,
            event.created_by_user_id,
            event.last_modified_by_user_id,
            event.workspace_id,
            event.parent_event_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get event by ID
    #[instrument(err, skip(self))]
    pub async fn get_event_by_id(&self, id: &Uuid) -> Result<Option<Event>> {
        debug!("Getting event by ID: {}", id);

        Ok(sqlx::query_as!(
            Event,
            r#"SELECT id AS "id: _", title, description, event_type AS "event_type: EventType", status AS "status: EventStatus",
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
             FROM events WHERE id = ?
             "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List events with filtering
    #[instrument(err, skip(self))]
    pub async fn list_events(&self, filter: &EventFilter) -> Result<Vec<Event>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id AS "id: _", title, description, event_type AS "event_type: EventType", status AS "status: EventStatus",
                    start_time AS "start_time: _", end_time AS "end_time: _", is_all_day_event,
                    timezone_sid_key, location, virtual_meeting_url, meeting_platform, is_recurrence,
                    recurrence_rule AS "recurrence_rule: _", recurrence_parent_id AS "recurrence_parent_id: _",
                    agent_participation AS "agent_participation: _", requires_transcription,
                    requires_summarization, agent_capabilities AS "agent_capabilities: _", is_private,
                    allow_guests, max_attendees, requires_approval, agenda, meeting_notes, transcription,
                    summary, action_items AS "action_items: _",
                    is_child_event, is_group_event, is_archived, event_relation AS "event_relation: _",
                    activity_date AS "activity_date: _", duration_in_minutes, show_as, is_reminder_set,
                    reminder_date_time AS "reminder_date_time: _", metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _", plan_id AS "plan_id: _",
                    task_id AS "task_id: _", created_by_user_id AS "created_by_user_id: _",
                    last_modified_by_user_id AS "last_modified_by_user_id: _", workspace_id AS "workspace_id: _",
                    parent_event_id AS "parent_event_id: _"
             FROM events"#
        );

        let mut add_where = add_where();

        if let Some(event_type) = filter.event_type {
            add_where(&mut qb);
            qb.push("event_type = ");
            qb.push_bind(event_type);
        }

        if let Some(status) = filter.status {
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status);
        }

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            qb.push("workspace_id = ");
            qb.push_bind(workspace_id);
        }

        if let Some(created_by_user_id) = &filter.created_by_user_id {
            add_where(&mut qb);
            qb.push("created_by_user_id = ");
            qb.push_bind(created_by_user_id);
        }

        if let Some(start_date) = &filter.start_date {
            add_where(&mut qb);
            qb.push("start_time >= ");
            qb.push_bind(start_date);
        }

        if let Some(end_date) = &filter.end_date {
            add_where(&mut qb);
            qb.push("end_time <= ");
            qb.push_bind(end_date);
        }

        if let Some(location) = &filter.location {
            add_where(&mut qb);
            qb.push("location LIKE ");
            qb.push_bind(format!("%{location}%"));
        }

        if let Some(is_all_day_event) = filter.is_all_day_event {
            add_where(&mut qb);
            qb.push("is_all_day_event = ");
            qb.push_bind(is_all_day_event);
        }

        if let Some(is_private) = filter.is_private {
            add_where(&mut qb);
            qb.push("is_private = ");
            qb.push_bind(is_private);
        }

        if let Some(is_archived) = filter.is_archived {
            add_where(&mut qb);
            qb.push("is_archived = ");
            qb.push_bind(is_archived);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(title LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(" OR description LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(" OR agenda LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(")");
        }

        qb.push(" ORDER BY start_time ASC");

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

    /// Update event
        #[instrument(err, skip(self))]
    pub async fn update_event(&self, event: &Event) -> Result<()> {
        let affected = sqlx::query!(
            r#"UPDATE events SET
                title = ?, description = ?, event_type = ?, status = ?, start_time = ?, end_time = ?,
                is_all_day_event = ?, timezone_sid_key = ?, location = ?, virtual_meeting_url = ?,
                meeting_platform = ?, is_recurrence = ?, recurrence_rule = ?, recurrence_parent_id = ?,
                agent_participation = ?, requires_transcription = ?, requires_summarization = ?,
                agent_capabilities = ?, is_private = ?, allow_guests = ?, max_attendees = ?,
                requires_approval = ?, agenda = ?, meeting_notes = ?, transcription = ?,
                summary = ?, action_items = ?, is_child_event = ?, is_group_event = ?,
                is_archived = ?, event_relation = ?, activity_date = ?, duration_in_minutes = ?,
                show_as = ?, is_reminder_set = ?, reminder_date_time = ?, metadata = ?,
                updated_at = ?, plan_id = ?, task_id = ?, last_modified_by_user_id = ?,
                workspace_id = ?, parent_event_id = ?
             WHERE id = ?"#,
            event.title,
            event.description,
            event.event_type,
            event.status,
            event.start_time,
            event.end_time,
            event.is_all_day_event,    
            event.timezone_sid_key,
            event.location,
            event.virtual_meeting_url,
            event.meeting_platform,
            event.is_recurrence,
            event.recurrence_rule,
            event.recurrence_parent_id,
            event.agent_participation,
            event.requires_transcription,
            event.requires_summarization,
            event.agent_capabilities,
            event.is_private,
            event.allow_guests,
            event.max_attendees,
            event.requires_approval,
            event.agenda,
            event.meeting_notes,
            event.transcription,
            event.summary,
            event.action_items,
            event.is_child_event,
            event.is_group_event,
            event.is_archived,
            event.event_relation,
            event.activity_date,
            event.duration_in_minutes,
            event.show_as,
            event.is_reminder_set,
            event.reminder_date_time,
            event.metadata,
            event.updated_at,
            event.plan_id,
            event.task_id,
            event.last_modified_by_user_id,
            event.workspace_id,
            event.parent_event_id,
            event.id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Event with ID {} not found",
                event.id
            )));
        }

        Ok(())
    }

    /// Delete event
    #[instrument(err, skip(self))]  
    pub async fn delete_event(&self, id: &Uuid) -> Result<()> {
        let affected = sqlx::query!("DELETE FROM events WHERE id = ?", id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Event with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get events by workspace
    #[instrument(err, skip(self))]
    pub async fn get_events_by_workspace(&self, workspace_id: &Uuid) -> Result<Vec<Event>> {
        let filter = EventFilter {
            workspace_id: Some(*workspace_id),
            event_type: None,
            status: None,
            created_by_user_id: None,
            start_date: None,
            end_date: None,
            location: None,
            is_all_day_event: None,
            is_private: None,
            is_archived: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Ok(Self::list_events(self, &filter).await?)
    }

    /// Get upcoming events
    #[instrument(err, skip(self))]
    pub async fn get_upcoming_events(&self, limit: Option<u32>) -> Result<Vec<Event>> {
        let now = Utc::now();
        let filter = EventFilter {
            start_date: Some(now),
            status: Some(EventStatus::Scheduled),
            is_archived: Some(false),
            limit: Some(limit.unwrap_or(10) as usize),
            workspace_id: None,
            event_type: None,
            created_by_user_id: None,
            end_date: None,
            location: None,
            is_all_day_event: None,
            is_private: None,
            search_term: None,
            offset: None,
        };

        Ok(Self::list_events(self, &filter).await?)
    }

    /// Update event status
    #[instrument(err, skip(self))]
    pub async fn update_event_status(&self, id: &Uuid, status: EventStatus) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query("UPDATE events SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status as i32)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Event with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Complete event
    #[instrument(err, skip(self))]
    pub async fn complete_event(&self, id: &Uuid) -> Result<()> {
        Self::update_event_status(self, id, EventStatus::Completed).await
    }

    /// Cancel event
    #[instrument(err, skip(self))]
    pub async fn cancel_event(&self, id: &Uuid) -> Result<()> {
        Self::update_event_status(self, id, EventStatus::Cancelled).await
    }

    /// Archive event
    #[instrument(err, skip(self))]
    pub async fn archive_event(&self, id: &Uuid) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE events SET is_archived = 1, updated_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Event with ID {} not found",
                id
            )));
        }

        Ok(())
    }

    /// Count events by status
    #[instrument(err, skip(self))]
    pub async fn count_events_by_status(&self, status: EventStatus) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM events WHERE status = ?")
            .bind(status as i32)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Get events by date range
    #[instrument(err, skip(self))]
    pub async fn get_events_by_date_range(&self, start: &DateTime<Utc>, end: &DateTime<Utc>) -> Result<Vec<Event>> {
        let filter = EventFilter {
            start_date: Some(*start),
            end_date: Some(*end),
            workspace_id: None,
            event_type: None,
            status: None,
            created_by_user_id: None,
            location: None,
            is_all_day_event: None,
            is_private: None,
            is_archived: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Ok(Self::list_events(self, &filter).await?)
    }

    /// Get events by user
    #[instrument(err, skip(self))]
    pub async fn get_events_by_user(&self, user_id: &Uuid) -> Result<Vec<Event>> {
        let filter = EventFilter {
            created_by_user_id: Some(*user_id),
            workspace_id: None,
            event_type: None,
            status: None,
            start_date: None,
            end_date: None,
            location: None,
            is_all_day_event: None,
            is_private: None,
            is_archived: None,
            search_term: None,
            limit: None,
            offset: None,
        };

        Ok(Self::list_events(self, &filter).await?)
    }
}
