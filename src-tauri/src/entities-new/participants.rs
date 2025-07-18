use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ConversationParticipantRole {
    Owner = 0,
    Admin = 1,
    Member = 2,
    Guest = 3,
    Observer = 4,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase", tag = "type", content = "inner_id")]
pub enum ParticipantType {
    User(Uuid),
    Agent(Uuid),
    Contact(Uuid),
    System,
}

impl ParticipantType {
    pub fn into_id_triplet(self) -> (Option<Uuid>, Option<Uuid>, Option<Uuid>) {
        match self {
            ParticipantType::User(id) => (Some(id), None, None),
            ParticipantType::Agent(id) => (None, Some(id), None),
            ParticipantType::Contact(id) => (None, None, Some(id)),
            ParticipantType::System => (None, None, None),
        }
    }

    pub fn from_id_triplet(
        user_id: Option<Uuid>,
        agent_id: Option<Uuid>,
        contact_id: Option<Uuid>,
    ) -> Self {
        match (user_id, agent_id, contact_id) {
            (Some(id), _, _) => ParticipantType::User(id),
            (_, Some(id), _) => ParticipantType::Agent(id),
            (_, _, Some(id)) => ParticipantType::Contact(id),
            _ => ParticipantType::System,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ParticipantStatus {
    Active = 0,
    Inactive = 1,
    Busy = 2,
    Offline = 3,
}

#[boilermates("CreateParticipant")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    #[boilermates(not_in("CreateParticipant"))]
    pub id: Uuid,
    pub workspace_id: Uuid,
    #[serde(rename = "type")]
    pub type_: ParticipantType,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub status: ParticipantStatus,
    pub metadata: Option<Json<Value>>, // JSON
    #[boilermates(not_in("CreateParticipant"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateParticipant"))]
    pub updated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantFilter {
    pub workspace_id: Option<Uuid>,
    pub type_: Option<ParticipantType>,
    pub group_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub task_id: Option<Uuid>,
    pub role: Option<ConversationParticipantRole>,
    pub participant_type: Option<ParticipantType>,
    pub status: Option<ParticipantStatus>,
    pub search_term: Option<String>,
    pub active_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl DatabaseManager {
    /// Create a new participant in the database
    #[instrument(skip(self))]
    pub async fn create_participant(&self, participant: &CreateParticipant) -> Result<Participant> {
        let id = Uuid::new_v4();
        debug!("Creating participant with ID: {}", id);

        let (user_id, agent_id, contact_id) = participant.type_.into_id_triplet();

        let row = sqlx::query!(
            r#"INSERT INTO participants (
                    id, workspace_id, user_id, agent_id, contact_id,
                    name, description, avatar, status, metadata
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING
                    id as "id: Uuid", workspace_id as "workspace_id: Uuid", user_id as "user_id: Uuid",
                    agent_id as "agent_id: Uuid", contact_id as "contact_id: Uuid", 
                    name, description, avatar, status as "status: ParticipantStatus", metadata as "metadata: Json<Value>",
                    created_at AS "created_at!: DateTime<Utc>", updated_at AS "updated_at!: DateTime<Utc>""#,
            id,
            participant.workspace_id,
            user_id,
            agent_id,
            contact_id,
            participant.name,
            participant.description,
            participant.avatar,
            participant.status,
            participant.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Participant {
            id: row.id,
            workspace_id: row.workspace_id,
            type_: ParticipantType::from_id_triplet(row.user_id, row.agent_id, row.contact_id),
            name: row.name,
            description: row.description,
            avatar: row.avatar,
            status: row.status,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Get participant by ID
    #[instrument(skip(self))]
    pub async fn get_participant(&self, id: &Uuid) -> Result<Option<Participant>> {
        debug!("Getting participant by ID: {}", id);
        let Some(row) = sqlx::query!(
            r#"SELECT
                id as "id: Uuid", workspace_id as "workspace_id: Uuid", user_id as "user_id: Uuid",
                agent_id as "agent_id: Uuid", contact_id as "contact_id: Uuid",
                name, description, avatar, status as "status: ParticipantStatus",
                metadata as "metadata: Json<Value>",
                created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>"
            FROM participants WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };
        Ok(Some(Participant {
            id: row.id,
            workspace_id: row.workspace_id,
            type_: ParticipantType::from_id_triplet(row.user_id, row.agent_id, row.contact_id),
            name: row.name,
            description: row.description,
            avatar: row.avatar,
            status: row.status,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }))
    }

    /// Update participant
    #[instrument(skip(self))]
    pub async fn update_participant(&self, participant: &Participant) -> Result<Participant> {
        let metadata = participant.metadata.as_deref();

        let (user_id, agent_id, contact_id) = participant.type_.into_id_triplet();

        let row = sqlx::query!(
            r#"UPDATE participants SET
                workspace_id = ?, user_id = ?, agent_id = ?, contact_id = ?,
                name = ?, description = ?, avatar = ?,
                status = ?, metadata = ?
             WHERE id = ? RETURNING
                id as "id: Uuid", workspace_id as "workspace_id: Uuid", user_id as "user_id: Uuid",
                agent_id as "agent_id: Uuid", contact_id as "contact_id: Uuid",
                name, description, avatar,
                status as "status: ParticipantStatus", metadata as "metadata: Json<Value>",
                created_at as "created_at: DateTime<Utc>", updated_at as "updated_at: DateTime<Utc>""#,
            participant.workspace_id,
            user_id,
            agent_id,
            contact_id,
            participant.name,
            participant.description,
            participant.avatar,
            participant.status,
            metadata,
            participant.id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(Participant {
            id: row.id,
            workspace_id: row.workspace_id,
            type_: ParticipantType::from_id_triplet(row.user_id, row.agent_id, row.contact_id),
            name: row.name,
            description: row.description,
            avatar: row.avatar,
            status: row.status,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Delete participant by ID
    #[instrument(skip(self))]
    pub async fn delete_participant(&self, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM participants WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Participant with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Delete participants by workspace
    #[instrument(skip(self))]
    pub async fn delete_participants_by_workspace(&self, workspace_id: &Uuid) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM participants WHERE workspace_id = ?")
            .bind(workspace_id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(affected)
    }

    /// Update participant status
    #[instrument(skip(self))]
    pub async fn update_participant_status(
        &self,
        id: &Uuid,
        status: ParticipantStatus,
    ) -> Result<()> {
        let affected =
            sqlx::query("UPDATE participants SET status = ?, updated_at = ? WHERE id = ?")
                .bind(status)
                .bind(Utc::now())
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Participant with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Update participant role
    #[instrument(skip(self))]
    pub async fn update_participant_role(
        &self,
        id: &Uuid,
        role: ConversationParticipantRole,
    ) -> Result<()> {
        let affected = sqlx::query("UPDATE participants SET role = ?, updated_at = ? WHERE id = ?")
            .bind(role)
            .bind(Utc::now())
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Participant with ID {id} not found"
            )));
        }

        Ok(())
    }
}
