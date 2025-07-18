use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Worker = 0,
    Operator = 1,
    System = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active = 0,
    Inactive = 1,
    Deleted = 2,
}

#[skip_serializing_none]
#[boilermates("CreateAgent")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    #[boilermates(not_in("CreateAgent"))]
    pub id: Uuid,
    pub model_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub version: String,
    #[specta(skip)]
    pub config: Option<Json<Value>>,
    #[specta(skip)]
    pub capabilities: Option<Json<Value>>,
    #[specta(skip)]
    pub memory_config: Option<Json<Value>>,
    #[specta(skip)]
    pub planning_config: Option<Json<Value>>,
    #[specta(skip)]
    pub topic_classifier: Option<Json<Value>>,
    #[specta(skip)]
    pub routing_rules: Option<Json<Value>>,
    pub context_window: i32,
    #[specta(skip)]
    pub tool_config: Option<Json<Value>>,
    #[specta(skip)]
    pub metrics: Option<Json<Value>>,
    #[specta(skip)]
    pub personality: Option<Json<Value>>,
    pub security: Option<Json<Value>>,
    pub is_public: bool,
    pub is_user_operator: bool,
    pub operator_level: i64,    
    #[specta(skip)]
    pub delegation_rules: Option<Json<Value>>,
    #[specta(skip)]
    pub total_sessions: i64,
    pub successful_sessions: i64,
    pub average_session_time: f64,
    #[boilermates(not_in("CreateAgent"))]
    #[specta(skip)]
    pub last_interaction_at: Option<DateTime<Utc>>,
    #[specta(skip)]
    pub metadata: Option<Json<Value>>,
    #[boilermates(not_in("CreateAgent"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateAgent"))]
    #[specta(skip)]
    pub updated_at: DateTime<Utc>,
    pub created_by_id: Option<Uuid>,
    pub operator_user_id: Option<Uuid>,
    pub parent_agent_id: Option<Uuid>,
    pub registry_id: Option<Uuid>,
    pub workspace_id: Uuid,
}
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct AgentMetrics {
    pub total_sessions: i64,
    pub successful_sessions: i64,
    pub average_session_time: f64,
    pub last_interaction_at: Option<String>,
    pub success_rate: f64,
}
/// Additional filtering options for agent queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, specta::Type)]
pub struct AgentFilter {
    pub workspace_id: Option<String>,
    pub status: Option<AgentStatus>,
    pub agent_type: Option<AgentType>,
    pub is_public: Option<bool>,
    pub is_user_operator: Option<bool>,
    pub created_by_id: Option<String>,
    pub operator_user_id: Option<String>,
    pub parent_agent_id: Option<String>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub status: Option<AgentStatus>,
    pub config: Option<Json<Value>>,
    pub capabilities: Option<Vec<String>>,
    pub memory_config: Option<Json<Value>>,
    pub planning_config: Option<Json<Value>>,
    pub context_window: Option<i32>,
    pub tool_config: Option<Json<Value>>,
    pub personality: Option<Json<Value>>,
    pub security: Option<Json<Value>>,
    pub is_public: Option<bool>,
    pub delegation_rules: Option<Json<Value>>,
    pub metadata: Option<Json<Value>>,
}

impl DatabaseManager {
    /// Create a new agent in the database
    #[instrument(skip(self))]
    pub async fn create_agent(&self, agent: &CreateAgent) -> Result<Agent> {
        let id = Uuid::new_v4();
        debug!("Creating agent with ID: {}", id);

        Ok(sqlx::query_as!(
            Agent,
            r#"INSERT INTO agents (
                    id, model_id, participant_id, name, description, avatar_url, "type", status, version, 
                    config, capabilities, memory_config, planning_config, topic_classifier, routing_rules, context_window, 
                    tool_config, metrics, personality, security, is_public, is_user_operator, 
                    operator_level, delegation_rules, total_sessions, successful_sessions, average_session_time, metadata,
                    created_by_id, operator_user_id, parent_agent_id, registry_id, workspace_id
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING 
                    id AS "id: _", model_id AS "model_id: _", participant_id AS "participant_id: _", name, description, avatar_url, "type" AS "agent_type: AgentType", status as "status: AgentStatus", version,
                    config AS "config: _", capabilities AS "capabilities: _", memory_config AS "memory_config: _",
                    planning_config AS "planning_config: _", topic_classifier AS "topic_classifier: _",
                    routing_rules AS "routing_rules: _", context_window AS "context_window: i32", tool_config AS "tool_config: _",
                    metrics AS "metrics: _", personality AS "personality: _", security AS "security: _",
                    is_public, is_user_operator, operator_level AS "operator_level: i32", delegation_rules AS "delegation_rules: _",
                    total_sessions AS "total_sessions: i32", successful_sessions AS "successful_sessions: i32", average_session_time, last_interaction_at AS "last_interaction_at: _", metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _",
                    created_by_id AS "created_by_id: _", operator_user_id AS "operator_user_id: _",
                    parent_agent_id AS "parent_agent_id: _", registry_id AS "registry_id: _",
                    workspace_id AS "workspace_id: _""#,
            id,
            agent.name,
            agent.description,
            agent.avatar_url,
            agent.agent_type,
            agent.status,
            agent.version,
            agent.config,
            agent.capabilities,
            agent.memory_config,
            agent.planning_config,
            agent.topic_classifier,
            agent.routing_rules,
            agent.context_window,
            agent.tool_config,
            agent.metrics,
            agent.personality,
            agent.security,
            agent.is_public,
            agent.is_user_operator,
            agent.operator_level,
            agent.delegation_rules,
            agent.total_sessions,
            agent.successful_sessions,
            agent.average_session_time,
            agent.metadata,
            agent.model_id,
            agent.participant_id,
            agent.created_by_id,
            agent.operator_user_id,
            agent.parent_agent_id,
            agent.registry_id,
            agent.workspace_id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get an agent by ID
    #[instrument(skip(self))]
    pub async fn get_agent_by_id(&self, id: &Uuid) -> Result<Option<Agent>> {
        debug!("Getting agent by ID: {}", id);

        Ok(sqlx::query_as!(
            Agent,
            r#"SELECT 
                    id AS "id: _", model_id AS "model_id: _", participant_id AS "participant_id: _",
                    name, description, avatar_url, "type" AS "agent_type: AgentType", status AS "status: AgentStatus", version,
                    config AS "config: _", capabilities AS "capabilities: _", memory_config AS "memory_config: _",
                    planning_config AS "planning_config: _", topic_classifier AS "topic_classifier: _", routing_rules AS "routing_rules: _",
                    context_window AS "context_window: i32", tool_config AS "tool_config: _", metrics AS "metrics: _", personality AS "personality: _", security AS "security: _", is_public,
                    is_user_operator, operator_level AS "operator_level: i32", delegation_rules AS "delegation_rules: _", total_sessions AS "total_sessions: i32",
                    successful_sessions AS "successful_sessions: i32", average_session_time, last_interaction_at AS "last_interaction_at: _", metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _",
                    created_by_id AS "created_by_id: _", operator_user_id AS "operator_user_id: _",
                    parent_agent_id AS "parent_agent_id: _", registry_id AS "registry_id: _",
                    workspace_id AS "workspace_id: _"
                FROM agents WHERE id = ? AND status != 2"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter agents
    #[instrument(err, skip(self, filter))]
    pub async fn list_agents(&self, filter: &AgentFilter) -> Result<Vec<Agent>> {
        debug!("Listing agents with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                    id, model_id, participant_id, name, description, avatar_url, "type" as agent_type, status, version,
                    config, capabilities, memory_config, planning_config, topic_classifier, routing_rules,
                    context_window, tool_config, metrics, personality, security, is_public,
                    is_user_operator, operator_level, delegation_rules, total_sessions,
                    successful_sessions, average_session_time, last_interaction_at, metadata,
                    created_at, updated_at, created_by_id, operator_user_id, parent_agent_id,
                    registry_id, workspace_id FROM agents WHERE status != 2"#,
        );

        let mut add_where = add_where();


        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(workspace_id)?;
            qb.push("workspace_id = ");
            qb.push_bind(uuid);
        }

        if let Some(status) = &filter.status {
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status.clone());
        }

        if let Some(agent_type) = &filter.agent_type {
            add_where(&mut qb);
            qb.push("\"type\" = ");
            qb.push_bind(agent_type.clone());
        }

        if let Some(is_public) = &filter.is_public {
            add_where(&mut qb);
            qb.push("is_public = ");
            qb.push_bind(is_public);
        }

        if let Some(is_user_operator) = &filter.is_user_operator {
            add_where(&mut qb);
            qb.push("is_user_operator = ");
            qb.push_bind(is_user_operator);
        }

        if let Some(created_by_id) = &filter.created_by_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(created_by_id)?;
            qb.push("created_by_id = ");
            qb.push_bind(uuid);
        }

        if let Some(operator_user_id) = &filter.operator_user_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(operator_user_id)?;
            qb.push("operator_user_id = ");
            qb.push_bind(uuid);
        }

        if let Some(parent_agent_id) = &filter.parent_agent_id {
            add_where(&mut qb);
            let uuid = Uuid::parse_str(parent_agent_id)?;
            qb.push("parent_agent_id = ");
            qb.push_bind(uuid);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            let pattern = format!("%{search_term}%");
            qb.push("(name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR description LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
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

        Ok(qb
            .build_query_as::<'_, Agent>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update an agent
    #[instrument(skip(self))]
    pub async fn update_agent(&self, agent: &Agent) -> Result<()> {
        debug!("Updating agent with ID: {}", agent.id);
        let rows = sqlx::query!(
            r#"
            UPDATE agents SET
                name = ?,
                description = ?,
                avatar_url = ?,
                status = ?,
                config = ?,
                capabilities = ?,
                memory_config = ?,
                planning_config = ?,
                topic_classifier = ?,
                routing_rules = ?,
                context_window = ?,
                tool_config = ?,
                metrics = ?,
                personality = ?,
                security = ?,
                is_public = ?,
                is_user_operator = ?,
                operator_level = ?,
                delegation_rules = ?,
                total_sessions = ?,
                successful_sessions = ?,
                average_session_time = ?,
                last_interaction_at = ?,
                metadata = ?
            WHERE id = ?
            "#,
            agent.name,
            agent.description,
            agent.avatar_url,
            agent.status,
            agent.config,
            agent.capabilities,
            agent.memory_config,
            agent.planning_config,
            agent.topic_classifier,
            agent.routing_rules,
            agent.context_window,
            agent.tool_config,
            agent.metrics,
            agent.personality,
            agent.security,
            agent.is_public,
            agent.is_user_operator,
            agent.operator_level,
            agent.delegation_rules,
            agent.total_sessions,
            agent.successful_sessions,
            agent.average_session_time,
            agent.last_interaction_at,
            agent.metadata,
            agent.id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent with ID {} not found for update",
                agent.id
            )));
        }
        Ok(())
    }

    /// Delete an agent by ID (soft delete)
    #[instrument(err, skip(self))]
    pub async fn delete_agent(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting agent with ID: {}", id);

        let affected = sqlx::query!("UPDATE agents SET status = 2, updated_at = CURRENT_TIMESTAMP WHERE id = ?", id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Update agent metrics
    #[instrument(skip(self))]
    pub async fn update_agent_metrics(
        &self,
        id: &Uuid,
        session_duration: f64,
        success: bool,
    ) -> Result<()> {
        debug!("Updating metrics for agent with ID: {}", id);

        let query = if success {
            r#"
            UPDATE agents 
            SET total_sessions = total_sessions + 1,
                successful_sessions = successful_sessions + 1,
                average_session_time = (average_session_time * total_sessions + ?) / (total_sessions + 1),
                last_interaction_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        } else {
            r#"
            UPDATE agents 
            SET total_sessions = total_sessions + 1,
                average_session_time = (average_session_time * total_sessions + ?) / (total_sessions + 1),
                last_interaction_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#
        };

        sqlx::query(query)
            .bind(session_duration)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
