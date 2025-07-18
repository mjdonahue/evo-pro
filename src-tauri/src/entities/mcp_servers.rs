use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

/// MCP server model matching the SQLite schema
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub host: String,
    pub port: i64,
    pub api_key: Option<String>,
    pub auth_token: Option<String>,
    pub is_active: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Additional filtering options for MCP server queries
#[derive(Debug, Default, Deserialize)]
pub struct McpServerFilter {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search_term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub limit: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new MCP server in the database
    #[instrument(skip(self))]
    pub async fn create_mcp_server(&self, server: &McpServer) -> Result<()> {
        debug!("Creating MCP server with ID: {}", server.id);

        // If this server is set as default, unset any existing default
        if server.is_default {
            self.unset_default_mcp_server().await?;
        }

        let _result = sqlx::query(
            "INSERT INTO mcp_servers (
                    id, name, description, host, port, api_key, auth_token,
                    is_active, is_default, created_at, updated_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                )",
        )
        .bind(server.id)
        .bind(&server.name)
        .bind(&server.description)
        .bind(&server.host)
        .bind(server.port)
        .bind(&server.api_key)
        .bind(&server.auth_token)
        .bind(server.is_active)
        .bind(server.is_default)
        .bind(server.created_at)
        .bind(server.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get an MCP server by ID
    #[instrument(skip(self))]
    pub async fn get_mcp_server_by_id(&self, id: &Uuid) -> Result<Option<McpServer>> {
        debug!("Getting MCP server by ID: {}", id);

        let row = sqlx::query(
            r#"SELECT 
                    id, name, description, host, port, api_key, auth_token,
                    is_active, is_default, created_at, updated_at
                FROM mcp_servers WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let server = McpServer {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                host: row.get("host"),
                port: row.get("port"),
                api_key: row.get("api_key"),
                auth_token: row.get("auth_token"),
                is_active: row.get::<i64, _>("is_active") != 0,
                is_default: row.get::<i64, _>("is_default") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(server))
        } else {
            Ok(None)
        }
    }

    /// Get an MCP server by name
    #[instrument(skip(self))]
    pub async fn get_mcp_server_by_name(&self, name: &str) -> Result<Option<McpServer>> {
        debug!("Getting MCP server by name: {}", name);

        let row = sqlx::query(
            r#"SELECT 
                    id, name, description, host, port, api_key, auth_token,
                    is_active, is_default, created_at, updated_at
                FROM mcp_servers WHERE name = ?"#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let server = McpServer {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                host: row.get("host"),
                port: row.get("port"),
                api_key: row.get("api_key"),
                auth_token: row.get("auth_token"),
                is_active: row.get::<i64, _>("is_active") != 0,
                is_default: row.get::<i64, _>("is_default") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(server))
        } else {
            Ok(None)
        }
    }

    /// Get the default MCP server
    #[instrument(skip(self))]
    pub async fn get_default_mcp_server(&self) -> Result<Option<McpServer>> {
        debug!("Getting default MCP server");

        let row = sqlx::query(
            r#"SELECT 
                    id, name, description, host, port, api_key, auth_token,
                    is_active, is_default, created_at, updated_at
                FROM mcp_servers WHERE is_default = 1"#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let server = McpServer {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                host: row.get("host"),
                port: row.get("port"),
                api_key: row.get("api_key"),
                auth_token: row.get("auth_token"),
                is_active: row.get::<i64, _>("is_active") != 0,
                is_default: row.get::<i64, _>("is_default") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(server))
        } else {
            Ok(None)
        }
    }

    /// List and filter MCP servers
    #[instrument(err, skip(self, filter))]
    pub async fn list_mcp_servers(&self, filter: &McpServerFilter) -> Result<Vec<McpServer>> {
        debug!("Listing MCP servers with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, name, description, host, port, api_key, auth_token,
               is_active, is_default, created_at, updated_at 
               FROM mcp_servers"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(is_active) = filter.is_active {
            where_conditions.push(format!("is_active = {}", if is_active { 1 } else { 0 }));
        }

        if let Some(is_default) = filter.is_default {
            where_conditions.push(format!("is_default = {}", if is_default { 1 } else { 0 }));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%' OR host LIKE '%{search_term}%')"
            ));
        }

        if !where_conditions.is_empty() {
            qb.push(" WHERE ");
            qb.push(where_conditions.join(" AND "));
        }

        qb.push(" ORDER BY name ASC");

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut servers = Vec::new();
        for row in rows {
            let server = McpServer {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                host: row.get("host"),
                port: row.get("port"),
                api_key: row.get("api_key"),
                auth_token: row.get("auth_token"),
                is_active: row.get::<i64, _>("is_active") != 0,
                is_default: row.get::<i64, _>("is_default") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            servers.push(server);
        }

        Ok(servers)
    }

    /// Update an MCP server
    #[instrument(err, skip(self))]
    pub async fn update_mcp_server(&self, server: &McpServer) -> Result<()> {
        debug!("Updating MCP server with ID: {}", server.id);

        // If this server is being set as default, unset any existing default
        if server.is_default {
            self.unset_default_mcp_server().await?;
        }

        let affected = sqlx::query(
            "UPDATE mcp_servers SET 
                name = ?, description = ?, host = ?, port = ?, api_key = ?,
                auth_token = ?, is_active = ?, is_default = ?, updated_at = ?
            WHERE id = ?",
        )
        .bind(&server.name)
        .bind(&server.description)
        .bind(&server.host)
        .bind(server.port)
        .bind(&server.api_key)
        .bind(&server.auth_token)
        .bind(server.is_active)
        .bind(server.is_default)
        .bind(server.updated_at)
        .bind(server.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP server with ID {} not found for update",
                server.id
            )));
        }

        Ok(())
    }

    /// Update MCP server connection details
    #[instrument(err, skip(self))]
    pub async fn update_mcp_server_connection(
        &self,
        id: &Uuid,
        host: &str,
        port: i64,
    ) -> Result<()> {
        debug!("Updating connection details for MCP server: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE mcp_servers SET host = ?, port = ?, updated_at = ? WHERE id = ?")
                .bind(host)
                .bind(port)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP server with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update MCP server authentication
    #[instrument(err, skip(self))]
    pub async fn update_mcp_server_auth(
        &self,
        id: &Uuid,
        api_key: Option<&str>,
        auth_token: Option<&str>,
    ) -> Result<()> {
        debug!("Updating authentication for MCP server: {}", id);

        let now = Utc::now();

        let affected = sqlx::query(
            "UPDATE mcp_servers SET api_key = ?, auth_token = ?, updated_at = ? WHERE id = ?",
        )
        .bind(api_key)
        .bind(auth_token)
        .bind(now)
        .bind(id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP server with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Set MCP server active status
    #[instrument(err, skip(self))]
    pub async fn set_mcp_server_active(&self, id: &Uuid, is_active: bool) -> Result<()> {
        debug!("Setting MCP server {} active status to {}", id, is_active);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE mcp_servers SET is_active = ?, updated_at = ? WHERE id = ?")
                .bind(is_active)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP server with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Set MCP server as default
    #[instrument(err, skip(self))]
    pub async fn set_mcp_server_default(&self, id: &Uuid) -> Result<()> {
        debug!("Setting MCP server {} as default", id);

        // First, unset any existing default
        self.unset_default_mcp_server().await?;

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE mcp_servers SET is_default = 1, updated_at = ? WHERE id = ?")
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP server with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Unset any default MCP server
    #[instrument(err, skip(self))]
    pub async fn unset_default_mcp_server(&self) -> Result<()> {
        debug!("Unsetting any default MCP server");

        let now = Utc::now();

        let _result = sqlx::query(
            "UPDATE mcp_servers SET is_default = 0, updated_at = ? WHERE is_default = 1",
        )
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete an MCP server by ID
    #[instrument(err, skip(self))]
    pub async fn delete_mcp_server(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting MCP server with ID: {}", id);

        let affected = sqlx::query("DELETE FROM mcp_servers WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "MCP server with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Count all MCP servers
    #[instrument(skip(self))]
    pub async fn count_mcp_servers(&self) -> Result<i64> {
        debug!("Counting MCP servers");

        let row = sqlx::query("SELECT COUNT(*) as count FROM mcp_servers")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count"))
    }

    /// Count active MCP servers
    #[instrument(skip(self))]
    pub async fn count_active_mcp_servers(&self) -> Result<i64> {
        debug!("Counting active MCP servers");

        let row = sqlx::query("SELECT COUNT(*) as count FROM mcp_servers WHERE is_active = 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::DatabaseManager;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::str::FromStr;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_and_get_mcp_server() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Test Server".to_string(),
            description: "A test MCP server".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            api_key: Some("test-api-key".to_string()),
            auth_token: None,
            is_active: true,
            is_default: true,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Get the MCP server by ID
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, server_id);
        assert_eq!(retrieved.name, "Test Server");
        assert_eq!(retrieved.description, "A test MCP server");
        assert_eq!(retrieved.host, "localhost");
        assert_eq!(retrieved.port, 8080);
        assert_eq!(retrieved.api_key, Some("test-api-key".to_string()));
        assert_eq!(retrieved.auth_token, None);
        assert_eq!(retrieved.is_active, true);
        assert_eq!(retrieved.is_default, true);

        // Get the MCP server by name
        let retrieved = db
            .get_mcp_server_by_name("Test Server")
            .await
            .expect("Failed to get MCP server by name");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, server_id);

        // Get the default MCP server
        let retrieved = db
            .get_default_mcp_server()
            .await
            .expect("Failed to get default MCP server");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, server_id);
    }

    #[tokio::test]
    async fn test_list_mcp_servers() {
        let db = DatabaseManager::setup_test_db().await;

        // Create multiple MCP servers
        for i in 1..=3 {
            let server_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i)).unwrap();

            let now = Utc::now();
            let server = McpServer {
                id: server_id,
                name: format!("Server {}", i),
                description: format!("Description for server {}", i),
                host: format!("host{}.example.com", i),
                port: 8080 + i as i64,
                api_key: Some(format!("api-key-{}", i)),
                auth_token: None,
                is_active: i != 3,  // Make the last one inactive
                is_default: i == 1, // Make the first one default
                created_at: now,
                updated_at: now,
            };

            db.create_mcp_server(&server)
                .await
                .expect("Failed to create MCP server");
        }

        // List all MCP servers
        let filter = McpServerFilter::default();
        let servers = db
            .list_mcp_servers(&filter)
            .await
            .expect("Failed to list MCP servers");
        assert_eq!(servers.len(), 3);

        // Filter by is_active
        let filter = McpServerFilter {
            is_active: Some(true),
            ..Default::default()
        };
        let servers = db
            .list_mcp_servers(&filter)
            .await
            .expect("Failed to list MCP servers");
        assert_eq!(servers.len(), 2);
        assert!(servers.iter().all(|s| s.is_active));

        // Filter by is_default
        let filter = McpServerFilter {
            is_default: Some(true),
            ..Default::default()
        };
        let servers = db
            .list_mcp_servers(&filter)
            .await
            .expect("Failed to list MCP servers");
        assert_eq!(servers.len(), 1);
        assert!(servers.iter().all(|s| s.is_default));

        // Filter by search term
        let filter = McpServerFilter {
            search_term: Some("Server 2".to_string()),
            ..Default::default()
        };
        let servers = db
            .list_mcp_servers(&filter)
            .await
            .expect("Failed to list MCP servers");
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].name, "Server 2");

        // Test count_mcp_servers
        let count = db
            .count_mcp_servers()
            .await
            .expect("Failed to count MCP servers");
        assert_eq!(count, 3);

        // Test count_active_mcp_servers
        let count = db
            .count_active_mcp_servers()
            .await
            .expect("Failed to count active MCP servers");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_update_mcp_server() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Original Server".to_string(),
            description: "Original description".to_string(),
            host: "original.example.com".to_string(),
            port: 8080,
            api_key: Some("original-key".to_string()),
            auth_token: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Update the MCP server
        let updated_server = McpServer {
            id: server_id,
            name: "Updated Server".to_string(),
            description: "Updated description".to_string(),
            host: "updated.example.com".to_string(),
            port: 9090,
            api_key: None,
            auth_token: Some("updated-token".to_string()),
            is_active: false,
            is_default: true,
            created_at: server.created_at,
            updated_at: Utc::now(),
        };

        db.update_mcp_server(&updated_server)
            .await
            .expect("Failed to update MCP server");

        // Get the updated MCP server
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server")
            .unwrap();
        assert_eq!(retrieved.name, "Updated Server");
        assert_eq!(retrieved.description, "Updated description");
        assert_eq!(retrieved.host, "updated.example.com");
        assert_eq!(retrieved.port, 9090);
        assert_eq!(retrieved.api_key, None);
        assert_eq!(retrieved.auth_token, Some("updated-token".to_string()));
        assert_eq!(retrieved.is_active, false);
        assert_eq!(retrieved.is_default, true);
    }

    #[tokio::test]
    async fn test_update_mcp_server_connection() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Test Server".to_string(),
            description: "Test description".to_string(),
            host: "original.example.com".to_string(),
            port: 8080,
            api_key: Some("test-key".to_string()),
            auth_token: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Update just the connection details
        let new_host = "new.example.com";
        let new_port = 9090;
        db.update_mcp_server_connection(&server_id, new_host, new_port)
            .await
            .expect("Failed to update MCP server connection");

        // Get the updated MCP server
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server")
            .unwrap();
        assert_eq!(retrieved.host, new_host);
        assert_eq!(retrieved.port, new_port);
        assert_eq!(retrieved.name, "Test Server"); // Other fields should remain unchanged
        assert_eq!(retrieved.api_key, Some("test-key".to_string()));
    }

    #[tokio::test]
    async fn test_update_mcp_server_auth() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Test Server".to_string(),
            description: "Test description".to_string(),
            host: "test.example.com".to_string(),
            port: 8080,
            api_key: Some("original-key".to_string()),
            auth_token: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Update just the authentication
        let new_api_key = Some("new-api-key");
        let new_auth_token = Some("new-auth-token");
        db.update_mcp_server_auth(&server_id, new_api_key, new_auth_token)
            .await
            .expect("Failed to update MCP server auth");

        // Get the updated MCP server
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server")
            .unwrap();
        assert_eq!(retrieved.api_key, Some("new-api-key".to_string()));
        assert_eq!(retrieved.auth_token, Some("new-auth-token".to_string()));
        assert_eq!(retrieved.host, "test.example.com"); // Other fields should remain unchanged
    }

    #[tokio::test]
    async fn test_set_mcp_server_active() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Test Server".to_string(),
            description: "Test description".to_string(),
            host: "test.example.com".to_string(),
            port: 8080,
            api_key: None,
            auth_token: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Set the MCP server as inactive
        db.set_mcp_server_active(&server_id, false)
            .await
            .expect("Failed to set MCP server inactive");

        // Get the updated MCP server
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server")
            .unwrap();
        assert_eq!(retrieved.is_active, false);

        // Set the MCP server as active again
        db.set_mcp_server_active(&server_id, true)
            .await
            .expect("Failed to set MCP server active");

        // Get the updated MCP server
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server")
            .unwrap();
        assert_eq!(retrieved.is_active, true);
    }

    #[tokio::test]
    async fn test_set_mcp_server_default() {
        let db = DatabaseManager::setup_test_db().await;

        // Create two MCP servers
        let server1_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let server2_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();

        let server1 = McpServer {
            id: server1_id,
            name: "Server 1".to_string(),
            description: "First server".to_string(),
            host: "server1.example.com".to_string(),
            port: 8081,
            api_key: None,
            auth_token: None,
            is_active: true,
            is_default: true, // First one is default
            created_at: now,
            updated_at: now,
        };

        let server2 = McpServer {
            id: server2_id,
            name: "Server 2".to_string(),
            description: "Second server".to_string(),
            host: "server2.example.com".to_string(),
            port: 8082,
            api_key: None,
            auth_token: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        };

        // Create both servers
        db.create_mcp_server(&server1)
            .await
            .expect("Failed to create first MCP server");
        db.create_mcp_server(&server2)
            .await
            .expect("Failed to create second MCP server");

        // Verify server1 is default
        let default = db
            .get_default_mcp_server()
            .await
            .expect("Failed to get default MCP server");
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, server1_id);

        // Set server2 as default
        db.set_mcp_server_default(&server2_id)
            .await
            .expect("Failed to set server2 as default");

        // Verify server2 is now default
        let default = db
            .get_default_mcp_server()
            .await
            .expect("Failed to get default MCP server");
        assert!(default.is_some());
        assert_eq!(default.unwrap().id, server2_id);

        // Verify server1 is no longer default
        let server1 = db
            .get_mcp_server_by_id(&server1_id)
            .await
            .expect("Failed to get server1")
            .unwrap();
        assert_eq!(server1.is_default, false);
    }

    #[tokio::test]
    async fn test_unset_default_mcp_server() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Test Server".to_string(),
            description: "Test description".to_string(),
            host: "test.example.com".to_string(),
            port: 8080,
            api_key: None,
            auth_token: None,
            is_active: true,
            is_default: true,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Verify it's the default
        let default = db
            .get_default_mcp_server()
            .await
            .expect("Failed to get default MCP server");
        assert!(default.is_some());

        // Unset any default
        db.unset_default_mcp_server()
            .await
            .expect("Failed to unset default MCP server");

        // Verify there's no default anymore
        let default = db
            .get_default_mcp_server()
            .await
            .expect("Failed to get default MCP server");
        assert!(default.is_none());

        // Verify the server still exists but is not default
        let server = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to get MCP server")
            .unwrap();
        assert_eq!(server.is_default, false);
    }

    #[tokio::test]
    async fn test_delete_mcp_server() {
        let db = DatabaseManager::setup_test_db().await;
        let server_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        let now = Utc::now();
        let server = McpServer {
            id: server_id,
            name: "Test Server".to_string(),
            description: "Test description".to_string(),
            host: "test.example.com".to_string(),
            port: 8080,
            api_key: None,
            auth_token: None,
            is_active: true,
            is_default: false,
            created_at: now,
            updated_at: now,
        };

        // Create the MCP server
        db.create_mcp_server(&server)
            .await
            .expect("Failed to create MCP server");

        // Delete the MCP server
        db.delete_mcp_server(&server_id)
            .await
            .expect("Failed to delete MCP server");

        // Try to get the deleted MCP server
        let retrieved = db
            .get_mcp_server_by_id(&server_id)
            .await
            .expect("Failed to query MCP server");
        assert!(retrieved.is_none());
    }
}
