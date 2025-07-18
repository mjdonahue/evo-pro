use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

/// API key model matching the SQLite schema
#[boilermates("CreateApiKey")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    #[boilermates(not_in("CreateApiKey"))]
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: String,
    pub key_hash: String,
    pub scopes: String, // JSON array of scopes
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit: Option<i64>,
    pub is_active: bool,
    #[boilermates(not_in("CreateApiKey"))]
    #[specta(skip)]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateApiKey"))]
    #[specta(skip)]
    pub updated_at: DateTime<Utc>,
    #[boilermates(not_in("CreateApiKey"))]
    #[specta(skip)]
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Additional filtering options for API key queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize, specta::Type)]
pub struct ApiKeyFilter {
    pub account_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new API key in the database
    #[instrument(skip(self))]
    pub async fn create_api_key(&self, api_key: &ApiKey) -> Result<ApiKey> {
        let id = Uuid::new_v4();
        debug!("Creating API key with ID: {}", id);

        Ok(sqlx::query_as!(
            ApiKey,
            r#"INSERT INTO api_keys (
                    id, account_id, name, description, key_hash, scopes,
                    expires_at, rate_limit, is_active
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING id AS "id: _", account_id AS "account_id: _", name, description,
                key_hash, scopes, expires_at AS "expires_at: _", rate_limit AS "rate_limit: _",
                is_active AS "is_active: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                last_used_at AS "last_used_at: _""#,
            id,
            api_key.account_id,
            api_key.name,
            api_key.description,
            api_key.key_hash,
            api_key.scopes,
            api_key.expires_at,
            api_key.rate_limit,
            api_key.is_active,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Get an API key by ID
    #[instrument(skip(self))]
    pub async fn get_api_key_by_id(&self, id: &Uuid) -> Result<Option<ApiKey>> {
        debug!("Getting API key by ID: {}", id);

        Ok(sqlx::query_as!(
            ApiKey,
            r#"SELECT 
                    id AS "id: _", account_id AS "account_id: _", name, description,
                    key_hash, scopes, expires_at AS "expires_at: _", rate_limit AS "rate_limit: _",
                    is_active AS "is_active: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                    last_used_at AS "last_used_at: _"
                FROM api_keys WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get an API key by hash
    #[instrument(skip(self, key_hash))]
    pub async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        debug!("Getting API key by hash");

        Ok(sqlx::query_as!(
            ApiKey,
            r#"SELECT 
                    id AS "id: _", account_id AS "account_id: _", name, description,
                    key_hash, scopes, expires_at AS "expires_at: _", rate_limit AS "rate_limit: _",
                    is_active AS "is_active: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                    last_used_at AS "last_used_at: _"
                FROM api_keys WHERE key_hash = ?"#,
            key_hash
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// Get an API key by name for an account
    #[instrument(skip(self))]
    pub async fn get_api_key_by_name(
        &self,
        account_id: &Uuid,
        name: &str,
    ) -> Result<Option<ApiKey>> {
        debug!(
            "Getting API key for account: {} with name: {}",
            account_id, name
        );

        let row = sqlx::query(
            r#"SELECT 
                    id, account_id, name, description, 
                    key_hash, scopes, expires_at, rate_limit,
                    is_active, created_at, updated_at,
                    last_used_at
                FROM api_keys 
                WHERE account_id = ? AND name = ?"#,
        )
        .bind(account_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let api_key = ApiKey {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                account_id: row
                    .get::<Vec<u8>, _>("account_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                key_hash: row.get("key_hash"),
                scopes: row.get("scopes"),
                expires_at: row.get("expires_at"),
                rate_limit: row.get("rate_limit"),
                is_active: row.get::<i64, _>("is_active") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_used_at: row.get("last_used_at"),
            };
            Ok(Some(api_key))
        } else {
            Ok(None)
        }
    }

    /// List and filter API keys
    #[instrument(err, skip(self, filter))]
    pub async fn list_api_keys(&self, filter: &ApiKeyFilter) -> Result<Vec<ApiKey>> {
        debug!("Listing API keys with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, account_id, name, description, 
               key_hash, scopes, expires_at, rate_limit,
               is_active, created_at, updated_at,
               last_used_at 
               FROM api_keys"#,
        );

        let mut where_conditions: Vec<String> = Vec::new();

        if let Some(account_id) = &filter.account_id {
            where_conditions.push(format!("account_id = '{account_id}'"));
        }

        if let Some(is_active) = filter.is_active {
            where_conditions.push(format!("is_active = {}", if is_active { 1 } else { 0 }));
        }

        if let Some(search_term) = &filter.search_term {
            where_conditions.push(format!(
                "(name LIKE '%{search_term}%' OR description LIKE '%{search_term}%')"
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

        let mut api_keys = Vec::new();
        for row in rows {
            let api_key = ApiKey {
                id: row
                    .get::<Vec<u8>, _>("id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                account_id: row
                    .get::<Vec<u8>, _>("account_id")
                    .try_into()
                    .map_err(|_| AppError::DatabaseError("Invalid UUID".to_string()))?,
                name: row.get("name"),
                description: row.get("description"),
                key_hash: row.get("key_hash"),
                scopes: row.get("scopes"),
                expires_at: row.get("expires_at"),
                rate_limit: row.get("rate_limit"),
                is_active: row.get::<i64, _>("is_active") != 0,
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                last_used_at: row.get("last_used_at"),
            };
            api_keys.push(api_key);
        }

        Ok(api_keys)
    }

    /// Update an API key
    #[instrument(err, skip(self))]
    pub async fn update_api_key(&self, api_key: &ApiKey) -> Result<()> {
        debug!("Updating API key with ID: {}", api_key.id);

        let affected = sqlx::query(
            "UPDATE api_keys SET 
                account_id = ?, name = ?, description = ?, scopes = ?,
                expires_at = ?, rate_limit = ?, is_active = ?, updated_at = ?, last_used_at = ?
            WHERE id = ?",
        )
        .bind(api_key.account_id)
        .bind(&api_key.name)
        .bind(&api_key.description)
        .bind(&api_key.scopes)
        .bind(api_key.expires_at)
        .bind(api_key.rate_limit)
        .bind(api_key.is_active)
        .bind(api_key.updated_at)
        .bind(api_key.last_used_at)
        .bind(api_key.id)
        .execute(&self.pool)
        .await?
        .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {} not found for update",
                api_key.id
            )));
        }

        Ok(())
    }

    /// Update API key scopes
    #[instrument(err, skip(self, scopes))]
    pub async fn update_api_key_scopes(&self, id: &Uuid, scopes: &str) -> Result<()> {
        debug!("Updating scopes for API key: {}", id);

        let now = Utc::now();

        let affected = sqlx::query("UPDATE api_keys SET scopes = ?, updated_at = ? WHERE id = ?")
            .bind(scopes)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update API key expiration
    #[instrument(err, skip(self))]
    pub async fn update_api_key_expiration(
        &self,
        id: &Uuid,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        debug!("Updating expiration for API key: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE api_keys SET expires_at = ?, updated_at = ? WHERE id = ?")
                .bind(expires_at)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update API key rate limit
    #[instrument(err, skip(self))]
    pub async fn update_api_key_rate_limit(
        &self,
        id: &Uuid,
        rate_limit: Option<i64>,
    ) -> Result<()> {
        debug!("Updating rate limit for API key: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE api_keys SET rate_limit = ?, updated_at = ? WHERE id = ?")
                .bind(rate_limit)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Set API key active status
    #[instrument(err, skip(self))]
    pub async fn set_api_key_active(&self, id: &Uuid, is_active: bool) -> Result<()> {
        debug!("Setting API key {} active status to {}", id, is_active);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE api_keys SET is_active = ?, updated_at = ? WHERE id = ?")
                .bind(is_active)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Update API key last used timestamp
    #[instrument(err, skip(self))]
    pub async fn update_api_key_last_used(&self, id: &Uuid) -> Result<()> {
        debug!("Updating last used timestamp for API key: {}", id);

        let now = Utc::now();

        let affected =
            sqlx::query("UPDATE api_keys SET last_used_at = ?, updated_at = ? WHERE id = ?")
                .bind(now)
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await?
                .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {id} not found for update"
            )));
        }

        Ok(())
    }

    /// Delete an API key by ID
    #[instrument(err, skip(self))]
    pub async fn delete_api_key(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting API key with ID: {}", id);

        let affected = sqlx::query("DELETE FROM api_keys WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        if affected == 0 {
            return Err(AppError::NotFoundError(format!(
                "API key with ID {id} not found for delete"
            )));
        }

        Ok(())
    }

    /// Delete all API keys for an account
    #[instrument(err, skip(self))]
    pub async fn delete_api_keys_for_account(&self, account_id: &Uuid) -> Result<usize> {
        debug!("Deleting all API keys for account: {}", account_id);

        let affected = sqlx::query("DELETE FROM api_keys WHERE account_id = ?")
            .bind(account_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected as usize)
    }

    /// Get all API keys for an account
    #[instrument(skip(self))]
    pub async fn get_api_keys_for_account(&self, account_id: &Uuid) -> Result<Vec<ApiKey>> {
        debug!("Getting all API keys for account: {}", account_id);

        let filter = ApiKeyFilter {
            account_id: Some(*account_id),
            ..Default::default()
        };

        self.list_api_keys(&filter).await
    }

    /// Get active API keys for an account
    #[instrument(skip(self))]
    pub async fn get_active_api_keys_for_account(&self, account_id: &Uuid) -> Result<Vec<ApiKey>> {
        debug!("Getting active API keys for account: {}", account_id);

        let filter = ApiKeyFilter {
            account_id: Some(*account_id),
            is_active: Some(true),
            ..Default::default()
        };

        self.list_api_keys(&filter).await
    }

    /// Count API keys for an account
    #[instrument(skip(self))]
    pub async fn count_api_keys_for_account(&self, account_id: &Uuid) -> Result<i64> {
        debug!("Counting API keys for account: {}", account_id);

        let row = sqlx::query("SELECT COUNT(*) as count FROM api_keys WHERE account_id = ?")
            .bind(account_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    /// Count active API keys for an account
    #[instrument(skip(self))]
    pub async fn count_active_api_keys_for_account(&self, account_id: &Uuid) -> Result<i64> {
        debug!("Counting active API keys for account: {}", account_id);

        let row = sqlx::query(
            "SELECT COUNT(*) as count FROM api_keys WHERE account_id = ? AND is_active = 1",
        )
        .bind(account_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("count"))
    }

    /// Check if an API key is valid (active, not expired)
    #[instrument(skip(self))]
    pub async fn is_api_key_valid(&self, key_hash: &str) -> Result<bool> {
        debug!("Checking if API key is valid");

        let now = Utc::now();

        let row = sqlx::query(
            r#"SELECT COUNT(*) as count FROM api_keys 
               WHERE key_hash = ? AND is_active = 1 
               AND (expires_at IS NULL OR expires_at > ?)"#,
        )
        .bind(key_hash)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
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
    async fn test_create_and_get_api_key() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let expires_at = now + chrono::Duration::days(30);
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "A test API key".to_string(),
            key_hash: "hashed_api_key_123".to_string(),
            scopes: r#"["read", "write"]"#.to_string(),
            expires_at: Some(expires_at),
            rate_limit: Some(100),
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Get the API key by ID
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key");
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, api_key_id);
        assert_eq!(retrieved.account_id, account_id);
        assert_eq!(retrieved.name, "Test API Key");
        assert_eq!(retrieved.description, "A test API key");
        assert_eq!(retrieved.key_hash, "hashed_api_key_123");
        assert_eq!(retrieved.scopes, r#"["read", "write"]"#);
        assert_eq!(
            retrieved.expires_at.unwrap().timestamp(),
            expires_at.timestamp()
        );
        assert_eq!(retrieved.rate_limit, Some(100));
        assert_eq!(retrieved.is_active, true);
        assert_eq!(retrieved.last_used_at, None);

        // Get the API key by hash
        let retrieved = db
            .get_api_key_by_hash("hashed_api_key_123")
            .await
            .expect("Failed to get API key by hash");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, api_key_id);

        // Get the API key by name
        let retrieved = db
            .get_api_key_by_name(&account_id, "Test API Key")
            .await
            .expect("Failed to get API key by name");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, api_key_id);
    }

    #[tokio::test]
    async fn test_list_api_keys() {
        let db = DatabaseManager::setup_test_db().await;
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        // Create multiple API keys
        for i in 1..=3 {
            let api_key_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i + 1)).unwrap();

            let now = Utc::now();
            let api_key = ApiKey {
                id: api_key_id,
                account_id,
                name: format!("API Key {}", i),
                description: format!("Description for API key {}", i),
                key_hash: format!("hashed_key_{}", i),
                scopes: r#"["read"]"#.to_string(),
                expires_at: if i == 1 {
                    None
                } else {
                    Some(now + chrono::Duration::days(30))
                },
                rate_limit: if i == 2 { Some(100) } else { None },
                is_active: i != 3, // Make the last one inactive
                created_at: now,
                updated_at: now,
                last_used_at: if i == 1 { Some(now) } else { None },
            };

            db.create_api_key(&api_key)
                .await
                .expect("Failed to create API key");
        }

        // List all API keys
        let filter = ApiKeyFilter::default();
        let api_keys = db
            .list_api_keys(&filter)
            .await
            .expect("Failed to list API keys");
        assert_eq!(api_keys.len(), 3);

        // Filter by account_id
        let filter = ApiKeyFilter {
            account_id: Some(account_id),
            ..Default::default()
        };
        let api_keys = db
            .list_api_keys(&filter)
            .await
            .expect("Failed to list API keys");
        assert_eq!(api_keys.len(), 3);

        // Filter by is_active
        let filter = ApiKeyFilter {
            is_active: Some(true),
            ..Default::default()
        };
        let api_keys = db
            .list_api_keys(&filter)
            .await
            .expect("Failed to list API keys");
        assert_eq!(api_keys.len(), 2);
        assert!(api_keys.iter().all(|k| k.is_active));

        // Filter by search term
        let filter = ApiKeyFilter {
            search_term: Some("API Key 2".to_string()),
            ..Default::default()
        };
        let api_keys = db
            .list_api_keys(&filter)
            .await
            .expect("Failed to list API keys");
        assert_eq!(api_keys.len(), 1);
        assert_eq!(api_keys[0].name, "API Key 2");

        // Test get_api_keys_for_account
        let api_keys = db
            .get_api_keys_for_account(&account_id)
            .await
            .expect("Failed to get API keys for account");
        assert_eq!(api_keys.len(), 3);

        // Test get_active_api_keys_for_account
        let api_keys = db
            .get_active_api_keys_for_account(&account_id)
            .await
            .expect("Failed to get active API keys for account");
        assert_eq!(api_keys.len(), 2);
        assert!(api_keys.iter().all(|k| k.is_active));

        // Test count_api_keys_for_account
        let count = db
            .count_api_keys_for_account(&account_id)
            .await
            .expect("Failed to count API keys");
        assert_eq!(count, 3);

        // Test count_active_api_keys_for_account
        let count = db
            .count_active_api_keys_for_account(&account_id)
            .await
            .expect("Failed to count active API keys");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn test_update_api_key() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Original API Key".to_string(),
            description: "Original description".to_string(),
            key_hash: "original_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Update the API key
        let updated_at = Utc::now();
        let last_used_at = Utc::now();
        let expires_at = updated_at + chrono::Duration::days(30);
        let updated_api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Updated API Key".to_string(),
            description: "Updated description".to_string(),
            key_hash: "original_hash".to_string(), // Key hash shouldn't change
            scopes: r#"["read", "write"]"#.to_string(),
            expires_at: Some(expires_at),
            rate_limit: Some(200),
            is_active: false,
            created_at: api_key.created_at,
            updated_at,
            last_used_at: Some(last_used_at),
        };

        db.update_api_key(&updated_api_key)
            .await
            .expect("Failed to update API key");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.name, "Updated API Key");
        assert_eq!(retrieved.description, "Updated description");
        assert_eq!(retrieved.key_hash, "original_hash");
        assert_eq!(retrieved.scopes, r#"["read", "write"]"#);
        assert_eq!(
            retrieved.expires_at.unwrap().timestamp(),
            expires_at.timestamp()
        );
        assert_eq!(retrieved.rate_limit, Some(200));
        assert_eq!(retrieved.is_active, false);
        assert_eq!(
            retrieved.last_used_at.unwrap().timestamp(),
            last_used_at.timestamp()
        );
    }

    #[tokio::test]
    async fn test_update_api_key_scopes() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "Test description".to_string(),
            key_hash: "test_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Update just the scopes
        let new_scopes = r#"["read", "write", "admin"]"#;
        db.update_api_key_scopes(&api_key_id, new_scopes)
            .await
            .expect("Failed to update API key scopes");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.scopes, new_scopes);
        assert_eq!(retrieved.name, "Test API Key"); // Other fields should remain unchanged
    }

    #[tokio::test]
    async fn test_update_api_key_expiration() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "Test description".to_string(),
            key_hash: "test_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Update the expiration
        let new_expires_at = now + chrono::Duration::days(60);
        db.update_api_key_expiration(&api_key_id, Some(new_expires_at))
            .await
            .expect("Failed to update API key expiration");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(
            retrieved.expires_at.unwrap().timestamp(),
            new_expires_at.timestamp()
        );

        // Remove the expiration
        db.update_api_key_expiration(&api_key_id, None)
            .await
            .expect("Failed to remove API key expiration");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.expires_at, None);
    }

    #[tokio::test]
    async fn test_update_api_key_rate_limit() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "Test description".to_string(),
            key_hash: "test_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Update the rate limit
        let new_rate_limit = 500;
        db.update_api_key_rate_limit(&api_key_id, Some(new_rate_limit))
            .await
            .expect("Failed to update API key rate limit");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.rate_limit, Some(new_rate_limit));

        // Remove the rate limit
        db.update_api_key_rate_limit(&api_key_id, None)
            .await
            .expect("Failed to remove API key rate limit");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.rate_limit, None);
    }

    #[tokio::test]
    async fn test_set_api_key_active() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "Test description".to_string(),
            key_hash: "test_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Set the API key as inactive
        db.set_api_key_active(&api_key_id, false)
            .await
            .expect("Failed to set API key inactive");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.is_active, false);

        // Set the API key as active again
        db.set_api_key_active(&api_key_id, true)
            .await
            .expect("Failed to set API key active");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert_eq!(retrieved.is_active, true);
    }

    #[tokio::test]
    async fn test_update_api_key_last_used() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "Test description".to_string(),
            key_hash: "test_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Update the last used timestamp
        db.update_api_key_last_used(&api_key_id)
            .await
            .expect("Failed to update API key last used");

        // Get the updated API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to get API key")
            .unwrap();
        assert!(retrieved.last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_is_api_key_valid() {
        let db = DatabaseManager::setup_test_db().await;
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let now = Utc::now();

        // Create an active, non-expiring API key
        let valid_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();
        let valid_key = ApiKey {
            id: valid_key_id,
            account_id,
            name: "Valid Key".to_string(),
            description: "Valid API key".to_string(),
            key_hash: "valid_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };
        db.create_api_key(&valid_key)
            .await
            .expect("Failed to create valid API key");

        // Create an active, future-expiring API key
        let future_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000003").unwrap();
        let future_key = ApiKey {
            id: future_key_id,
            account_id,
            name: "Future Key".to_string(),
            description: "Future-expiring API key".to_string(),
            key_hash: "future_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: Some(now + chrono::Duration::days(30)),
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };
        db.create_api_key(&future_key)
            .await
            .expect("Failed to create future-expiring API key");

        // Create an active, expired API key
        let expired_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000004").unwrap();
        let expired_key = ApiKey {
            id: expired_key_id,
            account_id,
            name: "Expired Key".to_string(),
            description: "Expired API key".to_string(),
            key_hash: "expired_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: Some(now - chrono::Duration::days(1)),
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };
        db.create_api_key(&expired_key)
            .await
            .expect("Failed to create expired API key");

        // Create an inactive API key
        let inactive_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000005").unwrap();
        let inactive_key = ApiKey {
            id: inactive_key_id,
            account_id,
            name: "Inactive Key".to_string(),
            description: "Inactive API key".to_string(),
            key_hash: "inactive_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: false,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };
        db.create_api_key(&inactive_key)
            .await
            .expect("Failed to create inactive API key");

        // Test validity
        assert!(
            db.is_api_key_valid("valid_hash")
                .await
                .expect("Failed to check valid key")
        );
        assert!(
            db.is_api_key_valid("future_hash")
                .await
                .expect("Failed to check future key")
        );
        assert!(
            !db.is_api_key_valid("expired_hash")
                .await
                .expect("Failed to check expired key")
        );
        assert!(
            !db.is_api_key_valid("inactive_hash")
                .await
                .expect("Failed to check inactive key")
        );
        assert!(
            !db.is_api_key_valid("nonexistent_hash")
                .await
                .expect("Failed to check nonexistent key")
        );
    }

    #[tokio::test]
    async fn test_delete_api_key() {
        let db = DatabaseManager::setup_test_db().await;
        let api_key_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000002").unwrap();

        let now = Utc::now();
        let api_key = ApiKey {
            id: api_key_id,
            account_id,
            name: "Test API Key".to_string(),
            description: "Test description".to_string(),
            key_hash: "test_hash".to_string(),
            scopes: r#"["read"]"#.to_string(),
            expires_at: None,
            rate_limit: None,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        };

        // Create the API key
        db.create_api_key(&api_key)
            .await
            .expect("Failed to create API key");

        // Delete the API key
        db.delete_api_key(&api_key_id)
            .await
            .expect("Failed to delete API key");

        // Try to get the deleted API key
        let retrieved = db
            .get_api_key_by_id(&api_key_id)
            .await
            .expect("Failed to query API key");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_delete_api_keys_for_account() {
        let db = DatabaseManager::setup_test_db().await;
        let account_id = Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap();

        // Create multiple API keys for the same account
        for i in 1..=3 {
            let api_key_id =
                Uuid::from_str(&format!("00000000-0000-0000-0000-00000000000{}", i + 1)).unwrap();

            let now = Utc::now();
            let api_key = ApiKey {
                id: api_key_id,
                account_id,
                name: format!("API Key {}", i),
                description: format!("Description for API key {}", i),
                key_hash: format!("hash_{}", i),
                scopes: r#"["read"]"#.to_string(),
                expires_at: None,
                rate_limit: None,
                is_active: true,
                created_at: now,
                updated_at: now,
                last_used_at: None,
            };

            db.create_api_key(&api_key)
                .await
                .expect("Failed to create API key");
        }

        // Verify we have 3 API keys for the account
        let count = db
            .count_api_keys_for_account(&account_id)
            .await
            .expect("Failed to count API keys");
        assert_eq!(count, 3);

        // Delete all API keys for the account
        let deleted_count = db
            .delete_api_keys_for_account(&account_id)
            .await
            .expect("Failed to delete API keys for account");
        assert_eq!(deleted_count, 3);

        // Verify all API keys are deleted
        let count = db
            .count_api_keys_for_account(&account_id)
            .await
            .expect("Failed to count API keys");
        assert_eq!(count, 0);
    }
}
