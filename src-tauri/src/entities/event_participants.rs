use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use serde_json::{Value};
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum EventParticipantRole {
    Attendee = 0,
    Organizer = 1,
    Other = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum EventParticipantStatus {
    Pending = 0,
    Accepted = 1,
    Declined = 2,
    Tentative = 3,
    Other = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[repr(i32)]
pub enum AgentRole {
    Attendee = 0,
    Organizer = 1,
    Other = 2,
}

#[boilermates("CreateEventParticipant")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct EventParticipant {
    #[boilermates(not_in("CreateEventParticipant"))]
    pub id: Uuid,
    pub event_id: Uuid,
    pub participant_id: Option<Uuid>,
    pub email: Option<String>,
    // Participation details
    pub role: EventParticipantRole,
    pub status: EventParticipantStatus,
    #[boilermates(not_in("CreateEventParticipant"))]
    pub response_at: Option<DateTime<Utc>>,
    // Agent-specific configuration
    pub agent_role: Option<AgentRole>,
    pub agent_config: Option<Json<Value>>,
    // Communication preferences
    pub notification_preferences: Option<Json<Value>>,
    #[boilermates(not_in("CreateEventParticipant"))]
    pub joined_at: Option<DateTime<Utc>>,
    #[boilermates(not_in("CreateEventParticipant"))]
    pub left_at: Option<DateTime<Utc>>,
    // Meeting participation
    pub is_present: bool,
    pub is_muted: bool,
    pub is_video_on: bool,
    pub speaking_time: Option<i32>, // in seconds
    // Participant metadata
    pub metadata: Option<Json<Value>>,
    #[boilermates(not_in("CreateEventParticipant"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateEventParticipant"))]
    pub updated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct EventParticipantFilter {
    pub event_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub role: Option<EventParticipantRole>,
    pub status: Option<EventParticipantStatus>,
    pub is_present: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}


impl DatabaseManager {
    /// Create a new event participant
    #[instrument(skip(self))]
    pub async fn create_event_participant(&self, event_participant: &EventParticipant) -> Result<EventParticipant> {
        let id = Uuid::new_v4();
        debug!("Creating event participant with ID: {}", id);

        let created_at = Utc::now();
        let updated_at = Utc::now();

        let metadata = event_participant.metadata.as_deref();
        let agent_config = event_participant.agent_config.as_deref();
        let notification_preferences = event_participant.notification_preferences.as_deref();

        Ok(sqlx::query_as!(
            EventParticipant,
            r#"INSERT INTO event_participants (
                id, event_id, participant_id, email, role, status, response_at,
                agent_role, agent_config, notification_preferences, joined_at, left_at,
                is_present, is_muted, is_video_on, speaking_time, metadata, created_at, updated_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING 
                id as "id: _", event_id as "event_id: _", participant_id as "participant_id: _", 
                email, role as "role: EventParticipantRole", status as "status: EventParticipantStatus", 
                response_at as "response_at: _", agent_role as "agent_role: AgentRole", 
                agent_config as "agent_config: _", notification_preferences as "notification_preferences: _", 
                joined_at as "joined_at: _", left_at as "left_at: _", is_present as "is_present: bool", 
                is_muted as "is_muted: bool", is_video_on as "is_video_on: bool", speaking_time as "speaking_time: i32", 
                metadata as "metadata: _", created_at as "created_at: _", updated_at as "updated_at: _""#,
            id,
            event_participant.event_id,
            event_participant.participant_id,
            event_participant.email,
            event_participant.role,
            event_participant.status,
            event_participant.response_at,
            event_participant.agent_role,
            agent_config,
            notification_preferences,
            event_participant.joined_at,
            event_participant.left_at,
            event_participant.is_present,
            event_participant.is_muted,
            event_participant.is_video_on,
            event_participant.speaking_time,
            metadata,
            created_at,
            updated_at,
        )
        .fetch_one(&self.pool)
        .await?)

    }

    /// Get event participant by ID    
    #[instrument(skip(self))]
    pub async fn get_event_participant_by_id(&self, id: &Uuid) -> Result<Option<EventParticipant>> {
        debug!("Getting event participant by ID: {}", id);

        Ok(sqlx::query_as!(
            EventParticipant,
            r#"SELECT 
                id as "id: _", event_id as "event_id: _", participant_id as "participant_id: _", 
                email, role as "role: EventParticipantRole", status as "status: EventParticipantStatus", 
                response_at as "response_at: _", agent_role as "agent_role: AgentRole", 
                agent_config as "agent_config: _", notification_preferences as "notification_preferences: _", 
                joined_at as "joined_at: _", left_at as "left_at: _", is_present as "is_present: bool", 
                is_muted as "is_muted: bool", is_video_on as "is_video_on: bool", speaking_time as "speaking_time: i32", 
                metadata as "metadata: _", created_at as "created_at: _", updated_at as "updated_at: _"
             FROM event_participants WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List event participants with filtering
    #[instrument(skip(self))]
    pub async fn list_event_participants(&self, filter: &EventParticipantFilter) -> Result<Vec<EventParticipant>> {
        debug!("Listing event participants with filter: {:?}", filter);
        let mut db: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                id as "id: _", event_id as "event_id: _", participant_id as "participant_id: _", 
                email, role as "role: EventParticipantRole", status as "status: EventParticipantStatus", 
                response_at as "response_at: _", agent_role as "agent_role: AgentRole", 
                agent_config as "agent_config: _", notification_preferences as "notification_preferences: _", 
                joined_at as "joined_at: _", left_at as "left_at: _", is_present as "is_present: bool",     
                is_muted as "is_muted: bool", is_video_on as "is_video_on: bool", speaking_time as "speaking_time: i32", 
                metadata as "metadata: _", created_at as "created_at: _", updated_at as "updated_at: _"
             FROM event_participants"#
        );

        let mut add_where = add_where();

        if let Some(event_id) = &filter.event_id {
            add_where(&mut db);
            db.push("event_id = ?");
            db.push_bind(event_id);
        }
        if let Some(participant_id) = &filter.participant_id {
            add_where(&mut db);
            db.push("participant_id = ?");
            db.push_bind(participant_id);
        }

        if let Some(participant_id) = &filter.participant_id {
            add_where(&mut db);
            db.push("participant_id = ?");
            db.push_bind(participant_id);
        }

        if let Some(role) = &filter.role {
            add_where(&mut db);
            db.push("role = ?");
            db.push_bind(role);
        }

        if let Some(status) = &filter.status {
            add_where(&mut db);
            db.push("status = ?");
            db.push_bind(status);
        }

        if let Some(is_present) = &filter.is_present {
            add_where(&mut db);
            db.push("is_present = ?");
            db.push_bind(is_present);
        }

        db.push(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            db.push(" LIMIT ");
            db.push_bind(limit);
        }
        if let Some(offset) = filter.offset {
            db.push(" OFFSET ");
            db.push_bind(offset);
        }

        Ok(db
            .build_query_as::<'_, EventParticipant>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update event participant
    #[instrument(skip(self))]
    pub async fn update(self, event_participant: &EventParticipant) -> Result<()> {
        debug!("Updating event participant with ID: {}", event_participant.id);

        let result = sqlx::query!(
            "UPDATE event_participants SET
                event_id = ?, participant_id = ?, email = ?, role = ?,
                status = ?, response_at = ?, agent_role = ?, agent_config = ?,
                notification_preferences = ?, joined_at = ?, left_at = ?, is_present = ?,
                is_muted = ?, is_video_on = ?, speaking_time = ?, metadata = ?, updated_at = ?
            WHERE id = ?",
            event_participant.event_id,
            event_participant.participant_id,
            event_participant.email,
            event_participant.role,
            event_participant.status,
            event_participant.response_at,
            event_participant.agent_role,
            event_participant.agent_config,
            event_participant.notification_preferences,
            event_participant.joined_at,
            event_participant.left_at,
            event_participant.is_present,
            event_participant.is_muted,
            event_participant.is_video_on,
            event_participant.speaking_time,
            event_participant.metadata,
            event_participant.updated_at,
            event_participant.id,
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Event participant with ID {} not found",
                event_participant.id
            )));
        }
        Ok(())
    }

    /// Delete event participant
    #[instrument(skip(self))]
    pub async fn delete(self, id: &Uuid) -> Result<()> {
        debug!("Deleting event participant with ID: {}", id);
        let result = sqlx::query!(
            "DELETE FROM event_participants WHERE id = ?",
            id,
        )
        .execute(&self.pool)
        .await?;    

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Event participant with ID {id} not found"
            )));
        }
        Ok(())
    }
}
