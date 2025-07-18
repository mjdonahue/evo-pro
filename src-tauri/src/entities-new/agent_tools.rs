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

/// Represents an agent tool association
#[boilermates("CreateAgentTool")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AgentTool {
    #[boilermates(not_in("CreateAgentTool"))]
    pub id: Uuid,
    pub agent_id: Uuid,
    pub tool_id: Uuid,
    pub config: Option<Json<Value>>, // JSON object with configuration
    pub enabled: bool,
    #[boilermates(not_in("CreateAgentTool"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateAgentTool"))]
    pub updated_at: DateTime<Utc>,
}

/// Filter for querying agent tools
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct AgentToolFilter {
    pub agent_id: Option<Uuid>,
    pub tool_id: Option<Uuid>,
    pub enabled: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new agent tool in the database
    #[instrument(skip(self))]
    pub async fn create_agent_tool(&self, agent_tool: &AgentTool) -> Result<AgentTool> {
        let id = Uuid::new_v4();
        debug!("Creating agent tool with ID: {}", id);

        let config = agent_tool.config.as_deref();

        Ok(sqlx::query_as!(
            AgentTool,  
            r#"INSERT INTO agent_tools (
                id, agent_id, tool_id, config, enabled
                ) VALUES (  
                    ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", agent_id AS "agent_id: _", tool_id AS "tool_id: _", config as "config: _", enabled,
                    created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            id,
            agent_tool.agent_id,
            agent_tool.tool_id,
            config,
            agent_tool.enabled
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get an agent tool by ID
    #[instrument(skip(self))]
    pub async fn get_agent_tool_by_id(&self, id: &Uuid) -> Result<Option<AgentTool>> {
        debug!("Getting agent tool by ID: {}", id);

        Ok(sqlx::query_as!(
            AgentTool,
            r#"SELECT id AS "id: _", agent_id AS "agent_id: _", tool_id AS "tool_id: _", config AS "config: _", enabled, created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM agent_tools WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get an agent tool by agent and tool IDs
    #[instrument(skip(self))]
    pub async fn get_agent_tool_by_agent_and_tool(
        &self,
        agent_id: &Uuid,
        tool_id: &Uuid,
    ) -> Result<Option<AgentTool>> {
        debug!("Getting agent tool by agent and tool IDs: {}, {}", agent_id, tool_id);

        Ok(sqlx::query_as!(
            AgentTool,
            r#"SELECT id AS "id: _", agent_id AS "agent_id: _", tool_id AS "tool_id: _", config AS "config: _", enabled, created_at AS "created_at: _", updated_at AS "updated_at: _"
               FROM agent_tools WHERE agent_id = ? AND tool_id = ?"#,
            agent_id,
            tool_id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List agent tools with filtering
    #[instrument(skip(self))]
    pub async fn list_agent_tools(&self, filter: &AgentToolFilter) -> Result<Vec<AgentTool>> {
        debug!("Listing agent tools with filter: {:?}", filter);

        let mut qb = QueryBuilder::new("SELECT id, agent_id, tool_id, config, enabled, created_at, updated_at FROM agent_tools WHERE 1=1");

        if let Some(agent_id) = &filter.agent_id {
            qb.push(" AND agent_id = ");
            qb.push_bind(agent_id);
        }

        if let Some(tool_id) = &filter.tool_id {
            qb.push(" AND tool_id = ");
            qb.push_bind(tool_id);
        }

        if let Some(enabled) = filter.enabled {
            qb.push(" AND enabled = ");
            qb.push_bind(enabled);
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

        let agent_tools = qb
            .build_query_as()
            .fetch_all(&self.pool)
            .await?;

        Ok(agent_tools)
    }

    /// Get all tools for a specific agent
    #[instrument(skip(self))]
    pub async fn get_tools_for_agent(&self, agent_id: &Uuid) -> Result<Vec<AgentTool>> {
        debug!("Getting tools for agent: {}", agent_id);
        debug!("Getting tools for agent: {:?}", agent_id);

        Ok(self.list_agent_tools(&AgentToolFilter {
            agent_id: Some(*agent_id),
            ..Default::default()
        }).await?)
    }

    /// Get enabled tools for a specific agent
    #[instrument(skip(self))]
    pub async fn get_enabled_tools_for_agent(&self, agent_id: &Uuid) -> Result<Vec<AgentTool>> {
        debug!("Getting enabled tools for agent: {}", agent_id);
        debug!("Getting enabled tools for agent: {:?}", agent_id);

        Ok(self.list_agent_tools(&AgentToolFilter {
            agent_id: Some(*agent_id),
            enabled: Some(true),
            ..Default::default()
        }).await?)
    }

    /// Get all agents for a specific tool
    #[instrument(skip(self))]
    pub async fn get_agents_for_tool(&self, tool_id: &Uuid) -> Result<Vec<AgentTool>> {
        debug!("Getting agents for tool: {}", tool_id);
        debug!("Getting agents for tool: {:?}", tool_id);

        Ok(self.list_agent_tools(&AgentToolFilter {
            tool_id: Some(*tool_id),
            ..Default::default()
        }).await?)
    }

    /// Update an agent tool
    #[instrument(skip(self))]
    pub async fn update_agent_tool(&self, agent_tool: &AgentTool) -> Result<()> {
        debug!("Updating agent tool: {}", agent_tool.id);

        let affected = sqlx::query(
            r#"UPDATE agent_tools SET 
                agent_id = ?, tool_id = ?, config = ?, enabled = ?, updated_at = ?
               WHERE id = ?"#,
        )
        .bind(agent_tool.agent_id)
        .bind(agent_tool.tool_id)
        .bind(&agent_tool.config)
        .bind(agent_tool.enabled)
        .bind(Utc::now())
        .bind(agent_tool.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent tool with ID {} not found for update",
                agent_tool.id
            )));
        }

        Ok(())
    }

    /// Enable or disable an agent tool
    #[instrument(skip(self))]
    pub async fn set_enabled(&self, id: &Uuid, enabled: bool) -> Result<()> {
        debug!("Setting enabled for agent tool: {}", id);

        let affected = sqlx::query(
            "UPDATE agent_tools SET enabled = ?, updated_at = ? WHERE id = ?",
        )
        .bind(enabled)
        .bind(Utc::now())
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent tool with ID {id} not found for enable/disable update"
            )));
        }

        Ok(())
    }

    /// Update tool configuration
    #[instrument(skip(self))]
    pub async fn update_config(&self, id: &Uuid, config: Option<&Json<Value>>) -> Result<()> {
        debug!("Updating config for agent tool: {}", id);

        let affected = sqlx::query(
            "UPDATE agent_tools SET config = ?, updated_at = ? WHERE id = ?",
        )
        .bind(config)
        .bind(Utc::now())
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent tool with ID {id} not found for config update"
            )));
        }

        Ok(())
    }

    /// Enable or disable all tools for an agent
    #[instrument(skip(self))]
    pub async fn set_all_enabled_for_agent(&self, agent_id: &Uuid, enabled: bool) -> Result<u64> {
        debug!("Setting all enabled for agent: {}", agent_id);

        let affected = sqlx::query(
            "UPDATE agent_tools SET enabled = ?, updated_at = ? WHERE agent_id = ?",
        )
        .bind(enabled)
        .bind(Utc::now())
        .bind(agent_id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(affected)
    }

    /// Delete an agent tool by ID
    #[instrument(skip(self))]
    pub async fn delete_agent_tool(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting agent tool by ID: {}", id);
        debug!("Deleting agent tool by ID: {:?}", id);

        let affected = sqlx::query("DELETE FROM agent_tools WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent tool with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Delete all tools for an agent
    #[instrument(skip(self))]
    pub async fn delete_by_agent(&self, agent_id: &Uuid) -> Result<u64> {
        debug!("Deleting all tools for agent: {}", agent_id);
        debug!("Deleting all tools for agent: {:?}", agent_id);

        let affected = sqlx::query("DELETE FROM agent_tools WHERE agent_id = ?")
            .bind(agent_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Delete all agent associations for a tool
    #[instrument(skip(self))]
    pub async fn delete_by_tool(&self, tool_id: &Uuid) -> Result<u64> {
        debug!("Deleting all agent associations for tool: {}", tool_id);
        debug!("Deleting all agent associations for tool: {:?}", tool_id);

        let affected = sqlx::query("DELETE FROM agent_tools WHERE tool_id = ?")
            .bind(tool_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected)
    }
}
