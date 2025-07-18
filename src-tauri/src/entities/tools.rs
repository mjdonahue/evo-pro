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
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Rest = 0,
    Grpc = 1,
    Local = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    Operator = 0,
    Tool = 1,
    System = 2,
    Other = 3,
}

/// Tool model matching the SQLite schema
#[boilermates("CreateTool")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    #[boilermates(not_in("CreateTool"))]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: ToolCategory,
    pub definition: Option<Json<Value>>,
    pub config: Option<Json<Value>>,
    pub auth_required: bool,
    pub is_active: bool,
    pub workspace_id: Option<Uuid>,
    pub created_by_id: Option<Uuid>,
    #[boilermates(not_in("CreateTool"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateTool"))]
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for tool queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct ToolFilter {
    pub name: Option<String>,
    pub tool_type: Option<ToolType>,
    pub category: Option<ToolCategory>,
    pub status: Option<ToolStatus>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new tool in the database   
    #[instrument(err, skip(self, tool))]    
    pub async fn create_tool(&self, tool: &Tool) -> Result<Tool> {
        let id = Uuid::new_v4();
        debug!("Creating tool with ID: {}", tool.id);
        let definition = tool.definition.as_deref();
        let config = tool.config.as_deref();
        let now = Utc::now();

        Ok(sqlx::query_as!(
            Tool,
            r#"INSERT INTO tools (
                    id, name, description, category, definition, config,
                    auth_required, is_active,
                    workspace_id, created_by_id, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", name, description, category as "category: ToolCategory", 
                    definition as "definition: _", config as "config: _", auth_required, is_active,
                   workspace_id AS "workspace_id: _", created_by_id AS "created_by_id: Uuid", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            id,
            tool.name,
            tool.description,
            tool.category,
            definition,
            config,
            tool.auth_required,
            tool.is_active,
            tool.workspace_id,
            tool.created_by_id,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a tool by ID
    #[instrument(err, skip(self, id))]
    pub async fn get_tool_by_id(&self, id: &Uuid) -> Result<Option<Tool>> {
        debug!("Getting tool by ID: {}", id);
        debug!("Getting tool by ID: {:?}", id);

        Ok(sqlx::query_as!(
            Tool,
            r#"SELECT 
                    id AS "id: _", name, description, category as "category: ToolCategory", definition as "definition: _", config as "config: _", auth_required, is_active,
                    created_by_id as "created_by_id: Uuid",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM tools WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get a tool by name
    #[instrument(skip(self))]
    pub async fn get_tool_by_name(&self, name: &str) -> Result<Option<Tool>> {
        debug!("Getting tool by name: {}", name);

        Ok(sqlx::query_as!(
            Tool,
            r#"SELECT 
                    id AS "id: _", name, description, category as "category: ToolCategory", definition as "definition: _", config as "config: _", auth_required, is_active,
                    created_by_id as "created_by_id: Uuid",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM tools WHERE name = ?"#,
            name
        )
        .fetch_optional(&self.pool)
        .await?)    
    }

    /// List and filter tools
    #[instrument(skip(self, filter))]
    pub async fn list_tools(&self, filter: &ToolFilter) -> Result<Vec<Tool>> {
        debug!("Listing tools with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                    id AS "id: _", name, description, category as "category: ToolCategory", definition as "definition: _", config as "config: _", auth_required, is_active,
                    created_by_id as "created_by_id: Uuid",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM tools"#,
        );

        let mut add_where = add_where();

        if let Some(name) = &filter.name {
            add_where(&mut qb);
            qb.push("name = ");
            qb.push_bind(name);
        }

        if let Some(category) = &filter.category {
            add_where(&mut qb);
            qb.push("category = ");
            qb.push_bind(category);
        }

        if let Some(status) = &filter.status {
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(name LIKE '%");
            qb.push_bind(search_term);
            qb.push("%' OR description LIKE '%");
            qb.push_bind(search_term);
            qb.push("%' OR category LIKE '%");
            qb.push_bind(search_term);
            qb.push("%')");
        }

        qb.push(" ORDER BY name ASC, created_at DESC");

        if let Some(limit) = filter.limit {
            add_where(&mut qb);
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            add_where(&mut qb);
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        Ok(qb
            .build_query_as::<'_, Tool>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update a tool
    #[instrument(err, skip(self, tool))]
    pub async fn update_tool(&self, tool: &Tool) -> Result<Tool> {
        debug!("Updating tool with ID: {}", tool.id);
        debug!("Updating tool with ID: {:?}", tool.id);

        let definition = tool.definition.as_deref();
        let config = tool.config.as_deref();

        Ok(sqlx::query_as!(
            Tool,
            r#"UPDATE tools SET 
                name = ?, description = ?, category = ?, definition = ?,
                config = ?, auth_required = ?, is_active = ?,
                workspace_id = ?, updated_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", name, description, category as "category: ToolCategory", definition as "definition: _", config as "config: _", auth_required, is_active,
                created_by_id as "created_by_id: Uuid",
                workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            tool.name,
            tool.description,
            tool.category,
            definition,
            config,
            tool.auth_required,
            tool.is_active,
            tool.workspace_id,
            tool.updated_at,
            tool.id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Delete a tool by ID
    #[instrument(err, skip(self))]
    pub async fn delete_tool(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting tool with ID: {}", id);

        let affected = sqlx::query("DELETE FROM tools WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Tool with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Set tool active status
    #[instrument(err, skip(self))]
    pub async fn set_tool_active(&self, id: &Uuid, is_active: bool) -> Result<()> {
        debug!("Setting tool {} active status to {}", id, is_active);

        let affected = sqlx::query("UPDATE tools SET status = ? WHERE id = ?")
            .bind(is_active as i32)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Tool with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update tool configuration
    #[instrument(err, skip(self))]
    pub async fn update_tool_config(&self, id: &Uuid, config: &str) -> Result<()> {
        debug!("Updating configuration for tool: {}", id);

        let affected = sqlx::query("UPDATE tools SET config = ? WHERE id = ?")
            .bind(config)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Tool with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update tool definition
    #[instrument(err, skip(self))]
    pub async fn update_tool_definition(&self, id: &Uuid, definition: &str) -> Result<()> {
        debug!("Updating definition for tool: {}", id);

        let affected = sqlx::query("UPDATE tools SET definition = ? WHERE id = ?")
            .bind(definition)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Tool with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update tool examples
    #[instrument(err, skip(self))]
    pub async fn update_tool_examples(&self, id: &Uuid, examples: &str) -> Result<()> {
        debug!("Updating examples for tool: {}", id);
        let affected = sqlx::query("UPDATE tools SET examples = ? WHERE id = ?")
            .bind(examples)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "Tool with ID {id} not found for update"
            )));
        }

        Ok(())
    }
}
