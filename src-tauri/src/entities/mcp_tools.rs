use crate::error::{AppError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub id: Uuid,
    pub mcp_server_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_enabled: bool,
    pub tool_type: McpToolType,
    pub status: McpToolStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum McpToolType {
    Rest = 0,
    Grpc = 1,
    Local = 2,
}

impl TryFrom<String> for McpToolType {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "REST" => Ok(McpToolType::Rest),
            "GRPC" => Ok(McpToolType::Grpc),
            "LOCAL" => Ok(McpToolType::Local),
            _ => Err(AppError::ValidationError(
                "Invalid MCP tool type".to_string(),
            )),
        }
    }
}

impl ToString for McpToolType {
    fn to_string(&self) -> String {
        match self {
            McpToolType::Rest => "REST".to_string(),
            McpToolType::Grpc => "GRPC".to_string(),
            McpToolType::Local => "LOCAL".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum McpToolStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

impl TryFrom<i32> for McpToolStatus {
    type Error = AppError;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(McpToolStatus::Active),
            1 => Ok(McpToolStatus::Archived),
            2 => Ok(McpToolStatus::Deleted),
            _ => Err(AppError::ValidationError(
                "Invalid MCP tool status".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolFilter {
    pub mcp_server_id: Option<Uuid>,
    pub tool_type: Option<McpToolType>,
    pub status: Option<McpToolStatus>,
    pub is_enabled: Option<bool>,
    pub search_term: Option<String>,
    pub active_only: Option<bool>,
    pub enabled_only: Option<bool>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl McpTool {
    /// Create a new MCP tool
    pub async fn create(pool: &Pool<Sqlite>, mcp_tool: &McpTool) -> Result<()> {
        sqlx::query(
            "INSERT INTO mcp_tools (
                id, mcp_server_id, name, description, is_enabled, type, status,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(mcp_tool.id)
        .bind(mcp_tool.mcp_server_id)
        .bind(&mcp_tool.name)
        .bind(&mcp_tool.description)
        .bind(mcp_tool.is_enabled)
        .bind(mcp_tool.tool_type.to_string())
        .bind(mcp_tool.status as i32)
        .bind(mcp_tool.created_at)
        .bind(mcp_tool.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get MCP tool by ID
    pub async fn get_by_id(pool: &Pool<Sqlite>, id: &Uuid) -> Result<Option<McpTool>> {
        let row = sqlx::query(
            "SELECT id, mcp_server_id, name, description, is_enabled, type, status,
                    created_at, updated_at
             FROM mcp_tools WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(McpTool {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                mcp_server_id: row
                    .get::<Vec<u8>, _>("mcp_server_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                is_enabled: row.get::<i64, _>("is_enabled") != 0,
                tool_type: McpToolType::try_from(row.get::<String, _>("type"))?,
                status: McpToolStatus::try_from(row.get::<i32, _>("status"))?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }))
        } else {
            Ok(None)
        }
    }

    /// List MCP tools with filtering
    pub async fn list(pool: &Pool<Sqlite>, filter: &McpToolFilter) -> Result<Vec<McpTool>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            "SELECT id, mcp_server_id, name, description, is_enabled, type, status,
                    created_at, updated_at
             FROM mcp_tools",
        );

        let mut where_conditions = Vec::new();

        if let Some(mcp_server_id) = &filter.mcp_server_id {
            where_conditions.push(format!("mcp_server_id = '{mcp_server_id}'"));
        }

        if let Some(tool_type) = filter.tool_type {
            where_conditions.push(format!("type = '{}'", tool_type.to_string()));
        }

        if let Some(status) = filter.status {
            where_conditions.push(format!("status = {}", status as i32));
        }

        if let Some(is_enabled) = filter.is_enabled {
            where_conditions.push(format!("is_enabled = {}", if is_enabled { 1 } else { 0 }));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%')"
            ));
        }

        if filter.active_only.unwrap_or(false) {
            where_conditions.push("status = 0".to_string()); // Active status
        }

        if filter.enabled_only.unwrap_or(false) {
            where_conditions.push("is_enabled = 1".to_string());
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
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

        let rows = qb.build().fetch_all(pool).await?;
        let mut mcp_tools = Vec::new();

        for row in rows {
            mcp_tools.push(McpTool {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                mcp_server_id: row
                    .get::<Vec<u8>, _>("mcp_server_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                is_enabled: row.get::<i64, _>("is_enabled") != 0,
                tool_type: McpToolType::try_from(row.get::<String, _>("type"))?,
                status: McpToolStatus::try_from(row.get::<i32, _>("status"))?,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }

        Ok(mcp_tools)
    }

    /// Update MCP tool
    pub async fn update(pool: &Pool<Sqlite>, mcp_tool: &McpTool) -> Result<()> {
        let affected = sqlx::query(
            "UPDATE mcp_tools SET
                mcp_server_id = ?, name = ?, description = ?, is_enabled = ?,
                type = ?, status = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(mcp_tool.mcp_server_id)
        .bind(&mcp_tool.name)
        .bind(&mcp_tool.description)
        .bind(mcp_tool.is_enabled)
        .bind(mcp_tool.tool_type.to_string())
        .bind(mcp_tool.status as i32)
        .bind(mcp_tool.updated_at)
        .bind(mcp_tool.id)
        .execute(pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP tool with ID {} not found",
                mcp_tool.id
            )));
        }

        Ok(())
    }

    /// Delete MCP tool
    pub async fn delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        let affected = sqlx::query("DELETE FROM mcp_tools WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP tool with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Get tools by MCP server
    pub async fn get_by_server(pool: &Pool<Sqlite>, mcp_server_id: &Uuid) -> Result<Vec<McpTool>> {
        let filter = McpToolFilter {
            mcp_server_id: Some(*mcp_server_id),
            tool_type: None,
            status: None,
            is_enabled: None,
            search_term: None,
            active_only: None,
            enabled_only: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get enabled tools by server
    pub async fn get_enabled_by_server(
        pool: &Pool<Sqlite>,
        mcp_server_id: &Uuid,
    ) -> Result<Vec<McpTool>> {
        let filter = McpToolFilter {
            mcp_server_id: Some(*mcp_server_id),
            tool_type: None,
            status: None,
            is_enabled: Some(true),
            search_term: None,
            active_only: Some(true),
            enabled_only: Some(true),
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Get tools by type
    pub async fn get_by_type(pool: &Pool<Sqlite>, tool_type: McpToolType) -> Result<Vec<McpTool>> {
        let filter = McpToolFilter {
            mcp_server_id: None,
            tool_type: Some(tool_type),
            status: None,
            is_enabled: None,
            search_term: None,
            active_only: None,
            enabled_only: None,
            limit: None,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Update tool status
    pub async fn update_status(
        pool: &Pool<Sqlite>,
        id: &Uuid,
        status: McpToolStatus,
    ) -> Result<()> {
        let now = Utc::now();

        let affected = sqlx::query("UPDATE mcp_tools SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status as i32)
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP tool with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Enable/disable tool
    pub async fn update_enabled(pool: &Pool<Sqlite>, id: &Uuid, is_enabled: bool) -> Result<()> {
        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE mcp_tools SET is_enabled = ?, updated_at = ? WHERE id = ?")
                .bind(is_enabled)
                .bind(now)
                .bind(id)
                .execute(pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP tool with ID {id} not found"
            )));
        }

        Ok(())
    }

    /// Archive tool
    pub async fn archive(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, McpToolStatus::Archived).await
    }

    /// Restore archived tool
    pub async fn restore(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, McpToolStatus::Active).await
    }

    /// Soft delete tool
    pub async fn soft_delete(pool: &Pool<Sqlite>, id: &Uuid) -> Result<()> {
        Self::update_status(pool, id, McpToolStatus::Deleted).await
    }

    /// Search tools by name
    pub async fn search_by_name(
        pool: &Pool<Sqlite>,
        search_term: &str,
        limit: Option<u32>,
    ) -> Result<Vec<McpTool>> {
        let filter = McpToolFilter {
            mcp_server_id: None,
            tool_type: None,
            status: None,
            is_enabled: None,
            search_term: Some(search_term.to_string()),
            active_only: None,
            enabled_only: None,
            limit,
            offset: None,
        };

        Self::list(pool, &filter).await
    }

    /// Count tools by server
    pub async fn count_by_server(pool: &Pool<Sqlite>, mcp_server_id: &Uuid) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM mcp_tools WHERE mcp_server_id = ?")
            .bind(mcp_server_id)
            .fetch_one(pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count enabled tools by server
    pub async fn count_enabled_by_server(pool: &Pool<Sqlite>, mcp_server_id: &Uuid) -> Result<i64> {
        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM mcp_tools WHERE mcp_server_id = ? AND is_enabled = 1",
        )
        .bind(mcp_server_id)
        .fetch_one(pool)
        .await?;

        Ok(row.get("count"))
    }

    /// Delete all tools for a server
    pub async fn delete_by_server(pool: &Pool<Sqlite>, mcp_server_id: &Uuid) -> Result<u64> {
        let affected = sqlx::query("DELETE FROM mcp_tools WHERE mcp_server_id = ?")
            .bind(mcp_server_id)
            .execute(pool)
            .await?
            .rows_affected();

        Ok(affected)
    }

    /// Bulk enable/disable tools
    pub async fn bulk_update_enabled(
        pool: &Pool<Sqlite>,
        tool_ids: &[Uuid],
        is_enabled: bool,
    ) -> Result<u64> {
        let mut tx = pool.begin().await?;
        let now = Utc::now();
        let mut total_affected = 0u64;

        for tool_id in tool_ids {
            let affected =
                sqlx::query("UPDATE mcp_tools SET is_enabled = ?, updated_at = ? WHERE id = ?")
                    .bind(is_enabled)
                    .bind(now)
                    .bind(tool_id)
                    .execute(&mut *tx)
                    .await?
                    .rows_affected();
            total_affected += affected;
        }

        tx.commit().await?;
        Ok(total_affected)
    }
}
