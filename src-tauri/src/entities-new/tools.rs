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
    pub tool_type: ToolType, 
    pub category: ToolCategory,
    pub definition: Option<Json<Value>>,
    pub config: Option<Json<Value>>,
    pub examples: Option<Json<Value>>,
    pub tags: Json<Value>,
    pub rating: Option<f64>,
    pub status: ToolStatus,
    pub created_by_id: Option<Uuid>,
    pub registry_id: Option<Uuid>,
    pub metadata: Option<Json<Value>>,
    #[boilermates(not_in("CreateTool"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateTool"))]
    pub updated_at: DateTime<Utc>,
    #[boilermates(not_in("CreateTool"))]
    pub workspace_id: Option<Uuid>,
}

/// Additional filtering options for tool queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct ToolFilter {
    pub name: Option<String>,
    pub tool_type: Option<i32>,
    pub category: Option<i32>,
    pub status: Option<i32>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new tool in the database   
    #[instrument(skip(self))]
    pub async fn create_tool(&self, tool: &Tool) -> Result<Tool> {
        let id = Uuid::new_v4();
        debug!("Creating tool with ID: {}", tool.id);
        let definition = tool.definition.as_ref().map(|d| d.0.clone());
        let config = tool.config.as_ref().map(|c| c.0.clone());
        let examples = tool.examples.as_ref().map(|e| e.0.clone());
        let tags = tool.tags.0.clone();
        let metadata = tool.metadata.as_ref().map(|m| m.0.clone());

        Ok(sqlx::query_as!(
            Tool,
            r#"INSERT INTO tools (
                    id, name, description, tool_type, category, definition, config, examples,
                    tags, rating, status, created_by_id, registry_id, metadata,
                    workspace_id, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING
                    id AS "id: _", name, description, tool_type as "tool_type: ToolType", category as "category: ToolCategory", 
                    definition as "definition: _", config as "config: _", examples as "examples: _" ,
                    tags as "tags: _", rating, status as "status: ToolStatus", created_by_id as "created_by_id: Uuid", registry_id as "registry_id: Uuid", metadata as "metadata: _",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            id,
            tool.name,
            tool.description,
            tool.tool_type,
            tool.category,
            definition,
            config,
            examples,
            tags,
            tool.rating,
            tool.status,
            tool.created_by_id,
            tool.registry_id,
            metadata,
            tool.workspace_id,
            tool.created_at,
            tool.updated_at
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get a tool by ID
    #[instrument(skip(self))]
    pub async fn get_tool_by_id(&self, id: &Uuid) -> Result<Option<Tool>> {
        debug!("Getting tool by ID: {}", id);
        debug!("Getting tool by ID: {:?}", id);

        Ok(sqlx::query_as!(
            Tool,
            r#"SELECT 
                    id AS "id: _", name, description, tool_type as "tool_type: ToolType", category as "category: ToolCategory", definition as "definition: _", config as "config: _", examples as "examples: _",
                    tags as "tags: _", rating, status as "status: ToolStatus", created_by_id as "created_by_id: Uuid", registry_id as "registry_id: Uuid", metadata as "metadata: _",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM tools WHERE id = $1"#,
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
                    id AS "id: _", name, description, tool_type as "tool_type: ToolType", category as "category: ToolCategory", definition as "definition: _", config as "config: _", examples as "examples: _",
                    tags as "tags: _", rating, status as "status: ToolStatus", created_by_id as "created_by_id: Uuid", registry_id as "registry_id: Uuid", metadata as "metadata: _",
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
                    id AS "id: _", name, description, tool_type as "tool_type: ToolType", category as "category: ToolCategory", definition as "definition: _", config as "config: _", examples as "examples: _",
                    tags as "tags: _", rating, status as "status: ToolStatus", created_by_id as "created_by_id: Uuid", registry_id as "registry_id: Uuid", metadata as "metadata: _",
                    workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM tools"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(name) = &filter.name {
            where_conditions.push(format!("name = '{name}'"));
        }

        if let Some(tool_type) = &filter.tool_type {
            where_conditions.push(format!("tool_type = '{tool_type}'"));
        }

        if let Some(category) = &filter.category {
            where_conditions.push(format!("category = '{category}'"));
        }

        if let Some(status) = &filter.status {
            where_conditions.push(format!("status = '{status}'"));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%' OR tool_type LIKE '%{search_term}%' OR category LIKE '%{search_term}%')"
            ));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY updated_at DESC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let tools = qb.build_query_as().fetch_all(&self.pool).await?;
        Ok(tools)
    }

    /// Update a tool
    #[instrument(skip(self))]
    pub async fn update_tool(&self, tool: &Tool) -> Result<Tool> {
        debug!("Updating tool with ID: {}", tool.id);
        debug!("Updating tool with ID: {:?}", tool.id);

        let definition = tool.definition.as_ref().map(|d| d.0.clone());
        let config = tool.config.as_ref().map(|c| c.0.clone());
        let examples = tool.examples.as_ref().map(|e| e.0.clone());
        let tags = tool.tags.0.clone();
        let metadata = tool.metadata.as_ref().map(|m| m.0.clone());

        Ok(sqlx::query_as!(
            Tool,
            r#"UPDATE tools SET 
                name = ?, description = ?, tool_type = ?, category = ?, definition = ?,
                config = ?, examples = ?, tags = ?, rating = ?, status = ?,
                workspace_id = ?, updated_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", name, description, tool_type as "tool_type: ToolType", category as "category: ToolCategory", definition as "definition: _", config as "config: _", examples as "examples: _",
                tags as "tags: _", rating, status as "status: ToolStatus", created_by_id as "created_by_id: Uuid", registry_id as "registry_id: Uuid", metadata as "metadata: _",
                workspace_id AS "workspace_id: _", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            tool.name,
            tool.description,
            tool.tool_type,
            tool.category,
            definition,
            config,
            examples,
            tags,
            tool.rating,
            tool.status,
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
