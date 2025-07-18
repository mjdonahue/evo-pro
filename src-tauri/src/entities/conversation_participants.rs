use boilermates::boilermates;
use chrono::{DateTime, Utc};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Sqlite, types::Json};
use tracing::{debug, instrument};
use uuid::Uuid;
use serde_json::Value;

use crate::entities::{Participant, ParticipantStatus, ParticipantType, PeerIdWrapper};
use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]  
pub enum ParticipantRole {
    Owner = 0,
    Admin = 1,
    Member = 2, 
    Assistant = 3,
    Observer = 4,
}
/// ConversationParticipant model matching the SQLite schema
#[skip_serializing_none]
#[boilermates("CreateConversationParticipant")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ConversationParticipant {
    pub conversation_id: Uuid,
    pub participant_id: Uuid,
    pub role: ParticipantRole,
    #[boilermates(not_in("CreateConversationParticipant"))]
    pub joined_at: DateTime<Utc>,
    #[boilermates(not_in("CreateConversationParticipant"))]
    pub left_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    #[boilermates(not_in("CreateConversationParticipant"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateConversationParticipant"))]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for conversation participant queries
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationParticipantFilter {
    pub conversation_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub role: Option<ParticipantRole>,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
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

        let now = Utc::now();

        Ok(sqlx::query_as!(
            ConversationParticipant,    
            r#"INSERT INTO conversation_participants (
                    conversation_id, participant_id, role, joined_at, left_at, is_active, created_at, updated_at
                ) VALUES (
                 ?, ?, ?, ?, ?, ?, ?, ?
                 ) RETURNING
                    conversation_id as "conversation_id: _", participant_id as "participant_id: _",
                    role as "role: ParticipantRole", joined_at as "joined_at: _", left_at as "left_at: _",
                    is_active, created_at as "created_at: _", updated_at as "updated_at: _"
            "#,
            participant.conversation_id,
            participant.participant_id,
            participant.role,
            now,
            now,
            participant.is_active,
            now,
            now,
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
                    role as "role: ParticipantRole", joined_at as "joined_at: _", left_at as "left_at: _",
                    is_active as "is_active: bool", created_at as "created_at: _", updated_at as "updated_at: _"
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
                conversation_id, participant_id, role as "role: ParticipantRole", joined_at, left_at, is_active, created_at, updated_at
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

        sqlx::query!(
            r#"UPDATE conversation_participants SET
                role = ?, left_at = ?, is_active = ?, updated_at = ?
            WHERE conversation_id = ? AND participant_id = ?"#,
            participant.role,
            participant.left_at,
            participant.is_active,
            participant.conversation_id,
            participant.participant_id,
            participant.updated_at,
        )
        .execute(&self.pool)
        .await?;

        Ok(participant.clone())
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
            .map(|p| {
                ParticipantType::from_id_triplet(p.0, p.1, p.2)
            })   
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


    pub async fn get_participant_by_peer_id(
        &self,
        conversation_id: Uuid,
        peer_id: &PeerIdWrapper,
    ) -> Result<Vec<Participant>> {
        Ok(sqlx::query!(
            r#"
            SELECT p.id AS "id: Uuid", p.user_id AS "user_id: Uuid", p.agent_id AS "agent_id: Uuid", p.contact_id AS "contact_id: Uuid",
            p.display_name, p.avatar_url, workspace_id AS "workspace_id: Uuid",
            p.status AS "status: ParticipantStatus", p.metadata AS "metadata: Json<Value>",
            p.created_at AS "created_at: DateTime<Utc>", p.updated_at AS "updated_at: DateTime<Utc>"
            FROM participants p
            INNER JOIN conversation_participants cp ON p.id = cp.participant_id
            INNER JOIN p2p_nodes pn ON p.id = pn.participant_id
            WHERE cp.conversation_id = ? AND pn.peer_id = ?
            "#,
            conversation_id,
            peer_id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| Participant {
            id: r.id,
            type_: ParticipantType::from_id_triplet(r.user_id, r.agent_id, r.contact_id),
            display_name: r.display_name,
            avatar_url: r.avatar_url,
            workspace_id: r.workspace_id,
            status: r.status,
            metadata: r.metadata,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }).collect())
    }
}
