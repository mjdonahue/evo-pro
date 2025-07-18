use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Sqlite};
use sqlx::types::Json;
use serde_json::{Value};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "registry_type")]
#[serde(rename_all = "lowercase")]
pub enum RegistryType {
    Model,
    Tool,
    Agent,
    Other,
}

/// Represents a registry entry
#[boilermates("CreateRegistry")]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Registry {
    #[boilermates(not_in("CreateRegistry"))]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub version: String, // TODO: version should be a semver
    pub registry_type: RegistryType,
    pub config: Option<Json<Value>>,     // JSON object with additional metadata
    pub is_public: bool,
    #[boilermates(not_in("CreateRegistry"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateRegistry"))]
    pub updated_at: DateTime<Utc>,
    pub workspace_id: Option<Uuid>,
}

/// Filter for querying agent registry entries
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryFilter {
    pub registry_type: Option<RegistryType>,
    pub workspace_id: Option<Uuid>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub is_public: Option<bool>,
    pub search_term: Option<String>, // Search in name and description
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new agent registry entry in the database
    #[instrument(err, skip(self, registry))]
    pub async fn create_registry(&self, registry: &Registry) -> Result<Registry> {
        let now = Utc::now();
        let config = registry.config.as_deref();
        let id = Uuid::new_v4();
        
        Ok(sqlx::query_as!(
            Registry,   
            r#"INSERT INTO registry (
                id, name, description, version, registry_type, config, is_public, created_at, updated_at, workspace_id
            ) VALUES (
             ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING id AS "id: _", name, description,
            version, registry_type AS "registry_type: RegistryType", config AS "config: _", is_public, created_at AS "created_at: _",
            updated_at AS "updated_at: _", workspace_id AS "workspace_id: _""#,    
            id,
            registry.name,
            registry.description,
            registry.version,
            registry.registry_type,
            config,
            registry.is_public, 
            now,
            now,
            registry.workspace_id,  
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get an agent registry entry by ID
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<Registry>> {
        debug!("Getting registry by ID: {}", id);

        Ok(sqlx::query_as!(
            Registry,
            r#"SELECT id AS "id: _", name, description,
            version, registry_type AS "registry_type: RegistryType", config AS "config: _", is_public AS "is_public: _", created_at AS "created_at: _",
            updated_at AS "updated_at: _", workspace_id AS "workspace_id: _"
            FROM registry WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }


    /// Get agent registry entry by name and workspace
    pub async fn get_by_name_and_workspace(
        &self,
        name: &str,
        workspace_id: &Uuid,
    ) -> Result<Option<Registry>> {
        debug!("Getting registry by name: {} and workspace: {}", name, workspace_id);

        Ok(sqlx::query_as!(
            Registry,
            r#"SELECT id AS "id: _", name, description, version, registry_type AS "registry_type: RegistryType", config AS "config: _", is_public AS "is_public: _", created_at AS "created_at: _",
            updated_at AS "updated_at: _", workspace_id AS "workspace_id: _"
            FROM registry WHERE name = ? AND workspace_id = ?"#,
            name,
            workspace_id,
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List agent registry entries with filtering
    #[instrument(err, skip(self, filter))]   
    pub async fn list_registries(&self, filter: &RegistryFilter) -> Result<Vec<Registry>> {
        debug!("Listing registries with filter: {:?}", filter); 

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, name, description, version, registry_type AS "registry_type: RegistryType", config AS "config: _", is_public, created_at AS "created_at: _",
            updated_at AS "updated_at: _", workspace_id AS "workspace_id: _"
            FROM registry"#,
        );

        let mut add_where = add_where();

        if let Some(workspace_id) = &filter.workspace_id {
            add_where(&mut qb);
            qb.push("workspace_id = ");
            qb.push_bind(workspace_id);
        }

        if let Some(version) = &filter.version {
            add_where(&mut qb);
            qb.push("version = ");
            qb.push_bind(version.clone());
        }

        if let Some(is_public) = filter.is_public {
            add_where(&mut qb);
            qb.push("is_public = ");
            qb.push_bind(is_public as i64);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(name LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(" OR description LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(")");
        }

        qb.push(" ORDER BY name ASC, version DESC, created_at DESC");

        if let Some(limit) = &filter.limit {
            add_where(&mut qb);
            qb.push(" LIMIT ");
            qb.push_bind(*limit as i64);
        }

        if let Some(offset) = &filter.offset {
            add_where(&mut qb);
            qb.push(" OFFSET ");
            qb.push_bind(*offset as i64);
        }

        Ok(qb
            .build_query_as::<'_, Registry>()
            .fetch_all(&self.pool)
            .await?)
    }


    /// Search agent registry entries
    pub async fn search(
        &self,
        search_term: &str,
        workspace_id: Option<&Uuid>,
    ) -> Result<Vec<Registry>> {
        let filter = RegistryFilter {
            workspace_id: workspace_id.copied(),
            search_term: Some(search_term.to_string()),
            ..Default::default()
        };
        debug!("Searching for registry entries with search term: {} and workspace: {:?}", search_term, workspace_id);
        Self::list_registries(self, &filter).await
    }

    /// Update an agent registry entry
    pub async fn update_registry(&self, registry: &Registry) -> Result<()> {
        debug!("Updating registry entry: {:?}", registry);

        let affected = sqlx::query!(
            r#"UPDATE registry SET 
                workspace_id = ?, name = ?, description = ?, version = ?, 
                config = ?, is_public = ?, registry_type = ?, updated_at = ?
               WHERE id = ?"#,
            registry.workspace_id,
            registry.name,
            registry.description,
            registry.version,
            registry.config,
            registry.is_public,
            registry.registry_type,
            registry.updated_at,
            registry.id
        )
        .execute(&self.pool)
        .await?;
        
        if affected.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "Registry entry with ID {} not found", registry.id
            )));
        }
        Ok(())
    }

}
