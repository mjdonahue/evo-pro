use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use serde_json::{Value};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, specta::Type)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentChainExecutionStatus {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
    Cancelled = 4,
}
#[boilermates("CreateAgentChainExecution")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct AgentChainExecution {
    #[boilermates(not_in("CreateAgentChainExecution"))]
    pub id: Uuid,
    pub chain_id: Uuid,
    pub conversation_id: Option<Uuid>,
    pub triggered_by_id: Option<Uuid>,
    pub status: AgentChainExecutionStatus,
    pub current_step_id: Option<Uuid>,
    #[specta(skip)]
    pub input_data: Option<Json<Value>>,
    #[specta(skip)]
    pub output_data: Option<Json<Value>>,
    #[specta(skip)]
    pub error_details: Option<Json<Value>>,
    #[specta(skip)]
    pub execution_context: Option<Json<Value>>,
    #[boilermates(not_in("CreateAgentChainExecution"))]
    #[specta(skip)]
    pub started_at: DateTime<Utc>,
    #[boilermates(not_in("CreateAgentChainExecution"))]
    #[specta(skip)]
    pub completed_at: Option<DateTime<Utc>>,
    #[boilermates(not_in("CreateAgentChainExecution"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
}

/// Filter for agent chain execution queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, specta::Type)]
pub struct AgentChainExecutionFilter {
    pub chain_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    pub triggered_by_id: Option<Uuid>,
    pub status: Option<AgentChainExecutionStatus>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new agent chain execution
    #[instrument(skip(self))]
    pub async fn create_agent_chain_execution(
        &self,
        execution: &AgentChainExecution,
    ) -> Result<AgentChainExecution> {
        let id = Uuid::new_v4();
        debug!("Creating agent chain execution with ID: {}", execution.id);
        let input_data = execution.input_data.as_deref();
        let output_data = execution.output_data.as_deref();
        let error_details = execution.error_details.as_deref();
        let execution_context = execution.execution_context.as_deref();

        Ok(sqlx::query_as!(
            AgentChainExecution,
            r#"INSERT INTO agent_chain_executions (
                id, chain_id, conversation_id, triggered_by_id, status, current_step_id,
                input_data, output_data, error_details, execution_context,
                started_at, completed_at, created_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING
                id AS "id: _", chain_id AS "chain_id: _", conversation_id AS "conversation_id: _",
                triggered_by_id AS "triggered_by_id: _", status as "status: AgentChainExecutionStatus",
                current_step_id AS "current_step_id: _", input_data AS "input_data: _",
                output_data AS "output_data: _", error_details AS "error_details: _",
                execution_context AS "execution_context: _",
                started_at AS "started_at: _", completed_at AS "completed_at: _",
                created_at AS "created_at: _""#,
            execution.id,
            execution.chain_id,
            execution.conversation_id,
            execution.triggered_by_id,
            execution.status,
            execution.current_step_id,
            input_data,
            output_data,
            error_details,
            execution_context,
            execution.started_at,
            execution.completed_at,
            execution.created_at
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get an agent chain execution by ID
    #[instrument(skip(self))]
    pub async fn get_agent_chain_execution_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<AgentChainExecution>> {
        debug!("Getting agent chain execution by ID: {}", id);
        debug!("Getting agent chain execution by ID: {:?}", id);

        Ok(sqlx::query_as!(
            AgentChainExecution,
            r#"SELECT 
                id AS "id: _", chain_id AS "chain_id: _", conversation_id AS "conversation_id: _",
                triggered_by_id AS "triggered_by_id: _", status as "status: AgentChainExecutionStatus",
                current_step_id AS "current_step_id: _", input_data AS "input_data: _",
                output_data AS "output_data: _", error_details AS "error_details: _",
                execution_context AS "execution_context: _",
                started_at AS "started_at: _", completed_at AS "completed_at: _",
                created_at AS "created_at: _"
            FROM agent_chain_executions WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List agent chain executions with filtering
    #[instrument(err, skip(self, filter))]
    pub async fn list_agent_chain_executions(
        &self,
        filter: &AgentChainExecutionFilter,
    ) -> Result<Vec<AgentChainExecution>> {
        debug!("Listing agent chain executions with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, chain_id, conversation_id, triggered_by_id, status, current_step_id,
               input_data, output_data, error_details, execution_context,
               started_at, completed_at, created_at
               FROM agent_chain_executions"#,
        );

        if let Some(chain_id) = &filter.chain_id {  
            qb.push(" AND chain_id = ");
            qb.push_bind(chain_id);
        }

        if let Some(conversation_id) = &filter.conversation_id {
            qb.push(" AND conversation_id = ");
            qb.push_bind(conversation_id);
        }

        if let Some(triggered_by_id) = &filter.triggered_by_id {
            qb.push(" AND triggered_by_id = ");
            qb.push_bind(triggered_by_id);
        }

        if let Some(status) = &filter.status {
            qb.push(" AND status = ");
            qb.push_bind(*status as i32);
        }

        qb.push(" ORDER BY started_at DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        Ok(qb
            .build_query_as::<AgentChainExecution>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update an agent chain execution
    #[instrument(skip(self))]
    pub async fn update_agent_chain_execution(
        &self,
        execution: &AgentChainExecution,
    ) -> Result<AgentChainExecution> {
        debug!("Updating agent chain execution with ID: {}", execution.id);

        Ok(sqlx::query_as!(
            AgentChainExecution,
            r#"UPDATE agent_chain_executions SET 
                chain_id = ?, conversation_id = ?, triggered_by_id = ?, status = ?, current_step_id = ?,
                input_data = ?, output_data = ?, error_details = ?, execution_context = ?,
                started_at = ?, completed_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", chain_id AS "chain_id: _", conversation_id AS "conversation_id: _",
                triggered_by_id AS "triggered_by_id: _", status as "status: AgentChainExecutionStatus",
                current_step_id AS "current_step_id: _", input_data AS "input_data: _",
                output_data AS "output_data: _", error_details AS "error_details: _",
                execution_context AS "execution_context: _",
                started_at AS "started_at: _", completed_at AS "completed_at: _",
                created_at AS "created_at: _""#,
            execution.chain_id,
            execution.conversation_id,
            execution.triggered_by_id,
            execution.status,
            execution.current_step_id,
            execution.input_data,
            execution.output_data,
            execution.error_details,
            execution.execution_context,
            execution.started_at,
            execution.completed_at,
            execution.id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Update chain execution status and current step
    #[instrument(skip(self))]
    pub async fn update_chain_execution_status(
        &self,
        id: &Uuid,
        status: AgentChainExecutionStatus,
        current_step_id: Option<Uuid>,
        error_details: Option<Json<Value>>,
    ) -> Result<AgentChainExecution> {
        debug!("Updating chain execution {} status to {:?}", id, status);

        let completed_at = if matches!(
            status,
            AgentChainExecutionStatus::Completed
            | AgentChainExecutionStatus::Failed
            | AgentChainExecutionStatus::Cancelled
        ) {
            Some(Utc::now())
        } else {
            None
        };

        Ok(sqlx::query_as!(
            AgentChainExecution,
            r#"UPDATE agent_chain_executions SET 
                status = ?, current_step_id = ?, error_details = ?, completed_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", chain_id AS "chain_id: _", conversation_id AS "conversation_id: _",
                triggered_by_id AS "triggered_by_id: _", status as "status: AgentChainExecutionStatus",
                current_step_id AS "current_step_id: _", input_data AS "input_data: _",
                output_data AS "output_data: _", error_details AS "error_details: _",
                execution_context AS "execution_context: _",
                started_at AS "started_at: _", completed_at AS "completed_at: _",
                created_at AS "created_at: _""#,
            status,
            current_step_id,
            error_details,
            completed_at,
            id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete an agent chain execution
    #[instrument(skip(self))]
    pub async fn delete_agent_chain_execution(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting agent chain execution with ID: {}", id);

        let affected = sqlx::query!("DELETE FROM agent_chain_executions WHERE id = ?", id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Agent chain execution with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Get active executions for a chain
    #[instrument(skip(self))]
    pub async fn get_active_chain_executions(
        &self,
        chain_id: &Uuid,
    ) -> Result<Vec<AgentChainExecution>> {
        debug!("Getting active executions for chain: {}", chain_id);

        Ok(sqlx::query_as!( 
            AgentChainExecution,
            r#"SELECT 
                id AS "id: _", chain_id AS "chain_id: _", conversation_id AS "conversation_id: _",
                triggered_by_id AS "triggered_by_id: _", status as "status: AgentChainExecutionStatus",
                current_step_id AS "current_step_id: _", input_data AS "input_data: _",
                output_data AS "output_data: _", error_details AS "error_details: _",
                execution_context AS "execution_context: _",
                started_at AS "started_at: _", completed_at AS "completed_at: _",
                created_at AS "created_at: _"
            FROM agent_chain_executions WHERE chain_id = ? AND status = 1"#,
            chain_id
        )
        .fetch_all(&self.pool)
        .await?)
    }

    /// Get executions for a conversation
    #[instrument(skip(self))]
    pub async fn get_conversation_executions(
        &self,
        conversation_id: &Uuid,
    ) -> Result<Vec<AgentChainExecution>> {
        debug!("Getting executions for conversation: {}", conversation_id);

        Ok(sqlx::query_as!( 
            AgentChainExecution,
            r#"SELECT 
                id AS "id: _", chain_id AS "chain_id: _", conversation_id AS "conversation_id: _",
                triggered_by_id AS "triggered_by_id: _", status as "status: AgentChainExecutionStatus",
                current_step_id AS "current_step_id: _", input_data AS "input_data: _",
                output_data AS "output_data: _", error_details AS "error_details: _",
                execution_context AS "execution_context: _",
                started_at AS "started_at: _", completed_at AS "completed_at: _",
                created_at AS "created_at: _"
            FROM agent_chain_executions WHERE conversation_id = ?"#,
            conversation_id
        )
        .fetch_all(&self.pool)
        .await?)
    }
}
