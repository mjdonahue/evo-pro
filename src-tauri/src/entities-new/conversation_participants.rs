use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::entities::{ConversationParticipantRole, ParticipantType};
use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

/// ConversationParticipant model matching the SQLite schema
#[skip_serializing_none]
#[boilermates("CreateConversationParticipant")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ConversationParticipant {
    pub conversation_id: Uuid,
    pub participant_id: Uuid,
    pub role: ConversationParticipantRole,
    #[boilermates(not_in("CreateConversationParticipant"))]
    #[specta(skip)]
    pub joined_at: DateTime<Utc>,
    #[boilermates(not_in("CreateConversationParticipant"))]
    #[specta(skip)]
    pub left_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    #[boilermates(not_in("CreateConversationParticipant"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
}

/// Additional filtering options for conversation participant queries
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ConversationParticipantFilter {
    pub conversation_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub role: Option<ConversationParticipantRole>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Add a participant to a conversation
    #[instrument(skip(self))]
    pub async fn add_conversation_participant(
        &self,
        participant: &CreateConversationParticipant,
    ) -> Result<ConversationParticipant> {
        debug!(
            "Adding participant {:?} to conversation {}",
            participant.participant_id, participant.conversation_id
        );

        Ok(sqlx::query_as!(
            ConversationParticipant,
            r#"INSERT INTO conversation_participants (
                    conversation_id, participant_id, role, is_active
                ) VALUES ( ?, ?, ?, ? )
                RETURNING
                    conversation_id as "conversation_id: _", participant_id as "participant_id: _",
                    role as "role: _", joined_at as "joined_at: _", left_at as "left_at: _",
                    is_active as "is_active: bool", created_at as "created_at: _""#,
            participant.conversation_id,
            participant.participant_id,
            participant.role,
            participant.is_active
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a conversation participant by conversation_id and participant_id
    #[instrument(skip(self))]
    pub async fn get_conversation_participant(
        &self,
        conversation_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<Option<ConversationParticipant>> {
        debug!(
            "Getting participant {:?} in conversation {}",
            participant_id, conversation_id
        );

        Ok(sqlx::query_as!(
            ConversationParticipant,
            r#"SELECT
                    conversation_id as "conversation_id: _", participant_id as "participant_id: _",
                    role as "role: _", joined_at as "joined_at: _", left_at as "left_at: _",
                    is_active as "is_active: bool", created_at as "created_at: _"
                FROM conversation_participants
                WHERE conversation_id = ? AND participant_id = ?"#,
            conversation_id,
            participant_id,
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter conversation participants
    #[instrument(skip(self))]
    pub async fn list_conversation_participants(
        &self,
        filter: &ConversationParticipantFilter,
    ) -> Result<Vec<ConversationParticipant>> {
        debug!(
            "Listing conversation participants with filter: {:?}",
            filter
        );

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT
                conversation_id, participant_id, role, joined_at, left_at, is_active, created_at
                FROM conversation_participants"#,
        );

        let mut add_where = add_where();

        if let Some(conversation_id) = filter.conversation_id {
            add_where(&mut qb);
            qb.push("conversation_id = ");
            qb.push_bind(conversation_id);
        }
        if let Some(participant_id) = filter.participant_id {
            add_where(&mut qb);
            qb.push("participant_id = ");
            qb.push_bind(participant_id);
        }
        if let Some(role) = filter.role {
            add_where(&mut qb);
            qb.push("role = ");
            qb.push_bind(role);
        }
        if let Some(is_active) = filter.is_active {
            add_where(&mut qb);
            qb.push("is_active = ");
            qb.push_bind(is_active);
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

        let participants = qb.build_query_as().fetch_all(&self.pool).await?;
        Ok(participants)
    }

    /// Update a conversation participant
    #[instrument(skip(self))]
    pub async fn update_conversation_participant(
        &self,
        participant: &ConversationParticipant,
    ) -> Result<ConversationParticipant> {
        debug!(
            "Updating participant {} in conversation {}",
            participant.participant_id, participant.conversation_id
        );

        Ok(sqlx::query_as!(
            ConversationParticipant,
            r#"UPDATE conversation_participants SET
                role = ?, left_at = ?, is_active = ?
            WHERE conversation_id = ? AND participant_id = ?
            RETURNING
                conversation_id AS "conversation_id: _", participant_id AS "participant_id: _",
                role AS "role: _", joined_at AS "joined_at: _", left_at AS "left_at: _",
                is_active AS "is_active: bool", created_at AS "created_at: _""#,
            participant.role,
            participant.left_at,
            participant.is_active,
            participant.conversation_id,
            participant.participant_id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Remove a participant from a conversation
    #[instrument(skip(self))]
    pub async fn remove_conversation_participant(
        &self,
        conversation_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<()> {
        debug!(
            "Removing participant {:?} from conversation {}",
            participant_id, conversation_id
        );

        let affected = sqlx::query!(
            "DELETE FROM conversation_participants WHERE conversation_id = ? AND participant_id = ?",
            conversation_id,
            participant_id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Participant {} not found in conversation {}",
                participant_id, conversation_id
            )));
        }

        Ok(())
    }

    /// Mark a participant as left from a conversation
    #[instrument(skip(self))]
    pub async fn mark_participant_left(
        &self,
        conversation_id: &Uuid,
        participant_id: &Uuid,
    ) -> Result<()> {
        debug!(
            "Marking participant {} as left from conversation {}",
            participant_id, conversation_id
        );

        let now = Utc::now();
        let affected = sqlx::query!(
            "UPDATE conversation_participants SET left_at = ?, is_active = false
            WHERE conversation_id = ? AND participant_id = ?",
            now,
            conversation_id,
            participant_id,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Participant {} not found in conversation {}",
                participant_id, conversation_id
            )));
        }

        Ok(())
    }

    /// Get all conversations for a participant
    #[instrument(skip(self))]
    pub async fn get_participant_conversations(
        &self,
        participant_id: &Uuid,
        is_active_only: bool,
    ) -> Result<Vec<Uuid>> {
        debug!("Getting conversations for participant {:?}", participant_id);

        let mut qb = QueryBuilder::new(
            "SELECT conversation_id FROM conversation_participants WHERE participant_id = ",
        );
        qb.push_bind(participant_id);

        if is_active_only {
            qb.push(" AND is_active = true");
        }

        let conversation_ids = qb.build_query_scalar().fetch_all(&self.pool).await?;
        Ok(conversation_ids)
    }

    /// Get all participants in a conversation
    #[instrument(skip(self))]
    pub async fn get_conversation_participant_ids(
        &self,
        conversation_id: &Uuid,
        is_active_only: bool,
    ) -> Result<Vec<ParticipantType>> {
        debug!("Getting participants in conversation {}", conversation_id);

        let mut qb = QueryBuilder::new(
            "SELECT user_id, agent_id, contact_id FROM conversation_participants c
            LEFT JOIN participants p ON c.participant_id = p.id
            WHERE c.conversation_id = ?",
        );
        qb.push_bind(conversation_id);

        if is_active_only {
            qb.push(" AND is_active = true");
        }

        Ok(qb
            .build_query_as::<(Option<Uuid>, Option<Uuid>, Option<Uuid>)>()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|p| ParticipantType::from_id_triplet(p.0, p.1, p.2))
            .collect())
    }

    pub async fn create_batch_participants(
        &self,
        participants: Vec<CreateConversationParticipant>,
    ) -> Result<Vec<ConversationParticipant>> {
        let mut qb = QueryBuilder::new(
            "INSERT INTO conversation_participants (conversation_id, participant_id, role, is_active) ",
        );
        qb.push_values(participants.iter(), |mut b, p| {
            b.push_bind(p.conversation_id)
                .push_bind(p.participant_id)
                .push_bind(p.role)
                .push_bind(p.is_active);
        });
        qb.push(" RETURNING conversation_id, participant_id, role, joined_at, left_at, is_active, created_at");

        Ok(qb
            .build_query_as::<ConversationParticipant>()
            .fetch_all(&self.pool)
            .await?)
    }
}
