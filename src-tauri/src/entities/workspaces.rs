use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use serde_json::Value;
use sqlx::types::Json;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]  
pub enum WorkspaceType {
    Personal = 0,
    Group = 1,
    Organization = 2,
}

/// Workspace status matching the SQLite schema
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

/// Workspace model matching the SQLite schema
#[boilermates("CreateWorkspace")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    #[boilermates(not_in("CreateWorkspace"))]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub workspace_type: WorkspaceType,
    pub status: WorkspaceStatus,
    pub metadata: Option<Json<Value>>,  
    #[boilermates(not_in("CreateWorkspace"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateWorkspace"))]
    pub updated_at: DateTime<Utc>,
}

/// Default workspace
impl Default for Workspace {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: None,
            workspace_type: WorkspaceType::Personal,
            status: WorkspaceStatus::Active,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Additional filtering options for workspace queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct WorkspaceFilter {
    pub workspace_type: Option<WorkspaceType>,
    pub status: Option<WorkspaceStatus>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new workspace in the database
    #[instrument(err, skip(self))]
    pub async fn create_workspace(&self, workspace: &Workspace) -> Result<Workspace> {
        let id = Uuid::new_v4();
        let metadata = workspace.metadata.as_deref();
            debug!("Creating workspace with ID: {}", workspace.id);
            let now = Utc::now();

            Ok(sqlx::query_as!(
                Workspace,
                r#"INSERT INTO workspaces (
                    id, name, description, workspace_type, status, metadata, created_at, updated_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING id AS "id: _", name, description, workspace_type AS "workspace_type: WorkspaceType", status AS "status: WorkspaceStatus", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _""#,
                id,
                workspace.name,
                workspace.description,
                workspace.workspace_type,
                workspace.status,
                metadata,
                now,
                now
            )
            .fetch_one(&self.pool)
            .await?)
    }

    /// Get a workspace by ID
    #[instrument(err, skip(self))]      
    pub async fn get_workspace_by_id(&self, id: &Uuid) -> Result<Option<Workspace>> {
        debug!("Getting workspace by ID: {}", id);

        Ok(sqlx::query_as!( 
            Workspace,
            r#"SELECT id AS "id: _", name, description, workspace_type AS "workspace_type: WorkspaceType", status AS "status: WorkspaceStatus", metadata AS "metadata: _", created_at AS "created_at: _", updated_at AS "updated_at: _"
            FROM workspaces WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter workspaces
    #[instrument(err, skip(self))]
    pub async fn list_workspaces(&self, filter: &WorkspaceFilter) -> Result<Vec<Workspace>> {
        debug!("Listing workspaces with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, name, description, workspace_type AS "workspace_type: WorkspaceType", status AS "status: WorkspaceStatus", metadata, created_at AS "created_at: _", updated_at AS "updated_at: _"
            FROM workspaces"#,
        );

        let mut add_where = add_where();

        if let Some(workspace_type) = filter.workspace_type {
            add_where(&mut qb);
            qb.push("workspace_type = ");
            qb.push_bind(workspace_type);
        }

        if let Some(status) = filter.status {
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(name LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(" OR description LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(")");
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

        Ok(qb
            .build_query_as::<'_, Workspace>()
            .fetch_all(&self.pool)
            .await?)
    }

    /// Update a workspace
    #[instrument(err, skip(self))]  
    pub async fn update_workspace(&self, workspace: &Workspace) -> Result<()> {
        debug!("Updating workspace with ID: {:?}", workspace.id);
  
        let affected = sqlx::query!(
            r#"UPDATE workspaces SET      
                name = ?, description = ?, workspace_type = ?, status = ?,
                metadata = ?, created_at = ?, updated_at = ?
            WHERE id = ?"#,
            workspace.name,
            workspace.description,
            workspace.workspace_type,
            workspace.status,
            workspace.metadata,
            workspace.created_at,
            workspace.updated_at,
            workspace.id
        )
        .execute(&self.pool)
        .await?;

        if affected.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!("Workspace with ID {} not found", workspace.id)));
        }

        Ok(())
    }

    /// Update a workspace by ID
    #[instrument(err, skip(self))]
    pub async fn update_workspace_by_id(&self, id: &Uuid, workspace: &Workspace) -> Result<()> {
        debug!("Updating workspace with ID: {}", id);

        let affected = sqlx::query!(
            r#"UPDATE workspaces SET      
                name = ?, description = ?, workspace_type = ?, status = ?,
                metadata = ?, created_at = ?, updated_at = ?
            WHERE id = ?"#,
            workspace.name,
            workspace.description,
            workspace.workspace_type,
            workspace.status,
            workspace.metadata,
            workspace.created_at,
            workspace.updated_at,
            id
        )
        .execute(&self.pool)
        .await?;

        if affected.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!("Workspace with ID {} not found", id)));
        }

        Ok(())
    }   

    /// Delete a workspace by ID
    #[instrument(skip(self))]
    pub async fn delete_workspace(&self, id: &Uuid) -> Result<()> {     
        debug!("Deleting workspace with ID: {}", id); 

        let affected = sqlx::query!(
            r#"DELETE FROM workspaces WHERE id = ?"#,
            id
        )
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!("Workspace with ID {id} not found for delete")));
        }

        Ok(())
    }

}
