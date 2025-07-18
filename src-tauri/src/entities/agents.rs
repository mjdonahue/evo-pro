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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Worker = 0,
    Operator = 1,
    System = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Active = 0,
    Inactive = 1,
    Deleted = 2,
}

#[skip_serializing_none]
#[boilermates("CreateAgent")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    #[boilermates(not_in("CreateAgent"))]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub version: String,
    pub config: Option<Json<Value>>,
    pub tool_config: Option<Json<Value>>,
    pub context_window: i64,
    pub parent_agent_id: Option<Uuid>,
    pub operator_level: i64,    
    pub delegation_rules: Option<Json<Value>>,
    pub performance_metrics: Option<Json<Value>>,
    #[boilermates(not_in("CreateAgent"))]
    pub last_interaction_at: Option<DateTime<Utc>>,
    #[boilermates(not_in("CreateAgent"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateAgent"))]
    pub updated_at: DateTime<Utc>,
    pub model_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub created_by_id: Option<Uuid>,
    pub operator_user_id: Option<Uuid>,
    pub registry_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_sessions: i64,
    pub successful_sessions: i64,
    pub average_session_time: f64,
    pub last_interaction_at: Option<String>,
    pub success_rate: f64,
}
/// Additional filtering options for agent queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct AgentFilter {
    pub workspace_id: Option<String>,
    pub status: Option<AgentStatus>,
    pub agent_type: Option<AgentType>,
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
    pub agent_type: Option<AgentType>,
    pub status: Option<AgentStatus>,
    pub config: Option<Json<Value>>,
    pub tool_config: Option<Json<Value>>,
    pub context_window: Option<i64>,
    pub parent_agent_id: Option<Uuid>,
    pub operator_level: Option<i64>,
    pub delegation_rules: Option<Json<Value>>,
    pub performance_metrics: Option<Json<Value>>,
    pub last_interaction_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub model_id: Option<Uuid>,
    pub participant_id: Option<Uuid>,
    pub created_by_id: Option<Uuid>,
    pub operator_user_id: Option<Uuid>,
    pub registry_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
}

impl DatabaseManager {
    /// Create a new agent in the database
    #[instrument(skip(self))]
    pub async fn create_agent(&self, agent: &CreateAgent) -> Result<Agent> {
        let id = Uuid::new_v4();
        debug!("Creating agent with ID: {}", id);

        let now = Utc::now();
        let config = agent.config.as_ref();
        let tool_config = agent.tool_config.as_ref();
        let delegation_rules = agent.delegation_rules.as_ref();
        let performance_metrics = agent.performance_metrics.as_ref();

        Ok(sqlx::query_as!(
            Agent,
            r#"INSERT INTO agents (
                    id, name, description, avatar_url, agent_type, status, version, 
                    config, tool_config, context_window, parent_agent_id, operator_level, delegation_rules, performance_metrics,
                    last_interaction_at, created_at, updated_at, model_id, participant_id, workspace_id, registry_id, created_by_id, operator_user_id
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING 
                    id AS "id: _", name, description, avatar_url, agent_type AS "agent_type: AgentType", status as "status: AgentStatus", version,
                    config AS "config: _", tool_config AS "tool_config: _", context_window, parent_agent_id AS "parent_agent_id: _", operator_level, delegation_rules AS "delegation_rules: _", performance_metrics AS "performance_metrics: _",
                    last_interaction_at AS "last_interaction_at: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                    model_id AS "model_id: _", participant_id AS "participant_id: _", workspace_id AS "workspace_id: _", registry_id AS "registry_id: _",
                    created_by_id AS "created_by_id: _", operator_user_id AS "operator_user_id: _"
            "#,
            id,
            agent.name,
            agent.description,
            agent.avatar_url,
            agent.agent_type,
            agent.status,
            agent.version,
            config,
            tool_config,
            agent.context_window,
            agent.parent_agent_id,
            agent.operator_level,
            delegation_rules,
            performance_metrics,
            now,
            now,
            now,
            agent.model_id,
            agent.participant_id,
            agent.workspace_id,
            agent.registry_id,
            agent.created_by_id,
            agent.operator_user_id,
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
                    name, description, avatar_url, agent_type AS "agent_type: AgentType", status AS "status: AgentStatus", version,
                    config AS "config: _", tool_config AS "tool_config: _", context_window AS "context_window: i32", operator_level AS "operator_level: i32", delegation_rules AS "delegation_rules: _", performance_metrics AS "performance_metrics: _",
                    last_interaction_at AS "last_interaction_at: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
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
                    id, name, description, avatar_url, agent_type, status, version,
                    config, tool_config, context_window, parent_agent_id, operator_level, delegation_rules, performance_metrics,
                    last_interaction_at, created_at, updated_at, model_id, participant_id, workspace_id, registry_id, created_by_id, operator_user_id
                    FROM agents WHERE status != 2"#,
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
                context_window = ?,
                tool_config = ?,
                operator_level = ?,
                delegation_rules = ?,
                performance_metrics = ?,
                last_interaction_at = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
            agent.name,
            agent.description,
            agent.avatar_url,
            agent.status,
            agent.config,
            agent.context_window,
            agent.tool_config,
            agent.operator_level,
            agent.delegation_rules,
            agent.performance_metrics,
            agent.last_interaction_at,
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

        let now = Utc::now();
        let result = if success {
            sqlx::query!(
                r#"
                UPDATE agents 
                SET last_interaction_at = ?,
                    updated_at = ?
                WHERE id = ?
                "#,
                now, now, id
            )
            .execute(&self.pool)
            .await?
        } else {
            sqlx::query!(
                r#"
                UPDATE agents 
                SET last_interaction_at = ?,
                    updated_at = ?
                WHERE id = ?
                "#,
                now, now, id
            )
            .execute(&self.pool)
            .await?
        };

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent with ID {id} not found for update metrics"
            )));
        }

        Ok(())
    }
}
