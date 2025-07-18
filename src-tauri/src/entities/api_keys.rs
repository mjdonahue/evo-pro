use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use sqlx::prelude::FromRow;
use sqlx::types::Json;
use serde_json::Value;
use sqlx::{QueryBuilder, Row, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

/// API key model matching the SQLite schema
#[boilermates("CreateApiKey")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    #[boilermates(not_in("CreateApiKey"))]
    pub id: Uuid,
    pub account_id: Uuid,
    pub name: String,
    pub description: String,
    pub key_hash: String,
    pub scopes: Json<Value>, // JSON array of scopes
    #[boilermates(not_in("CreateApiKey"))]
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit: Option<i64>,
    pub is_active: bool,
    #[boilermates(not_in("CreateApiKey"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateApiKey"))]
    pub updated_at: DateTime<Utc>,
    #[boilermates(not_in("CreateApiKey"))]
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Additional filtering options for API key queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
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
    pub async fn create_api_key(&self, api_key: &CreateApiKey) -> Result<ApiKey> {
        let id = Uuid::new_v4();
        debug!("Creating API key with ID: {}", id);

        let now = Utc::now();
        let scopes = api_key.scopes.as_ref();

        Ok(sqlx::query_as!(
            ApiKey,
            r#"INSERT INTO api_keys (
                    id, account_id, name, description, key_hash, scopes,
                    expires_at, rate_limit, is_active, created_at, updated_at, last_used_at
                ) VALUES (
                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                ) RETURNING id AS "id: _", account_id AS "account_id: _", name, description,
                key_hash, scopes AS "scopes: _", expires_at AS "expires_at: _", rate_limit AS "rate_limit: _",
                is_active AS "is_active: _", created_at AS "created_at: _", updated_at AS "updated_at: _",
                last_used_at AS "last_used_at: _""#,    
            id,
            api_key.account_id,
            api_key.name,
            api_key.description,
            api_key.key_hash,
            scopes,
            now,
            api_key.rate_limit,
            api_key.is_active, 
            now,
            now,
            now,
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
                    key_hash, scopes AS "scopes: _", expires_at AS "expires_at: _", rate_limit AS "rate_limit: _",
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
                    key_hash, scopes AS "scopes: _", expires_at AS "expires_at: _", rate_limit AS "rate_limit: _",
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

        Ok(sqlx::query_as!(
            ApiKey,
            r#"SELECT   
                    id AS "id: _", account_id AS "account_id: _", name, description, 
                    key_hash, scopes AS "scopes: _", expires_at AS "expires_at: _", rate_limit,
                    is_active, created_at AS "created_at: _", updated_at AS "updated_at: _",
                    last_used_at AS "last_used_at: _"
                FROM api_keys
                WHERE account_id = ? AND name = ?"#,
            account_id,
            name
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    /// List and filter API keys
    #[instrument(err, skip(self, filter))]
    pub async fn list_api_keys(&self, filter: &ApiKeyFilter) -> Result<Vec<ApiKey>> {
        debug!("Listing API keys with filter: {:?}", filter);

        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT id, account_id, name, description, 
               key_hash, scopes AS "scopes: _", expires_at, rate_limit,
               is_active, created_at, updated_at,
               last_used_at 
               FROM api_keys"#,
        );

        let mut add_where = add_where();

        if let Some(account_id) = &filter.account_id {
            add_where(&mut qb);
            qb.push("account_id = ");
            qb.push_bind(account_id);
        }

        if let Some(is_active) = filter.is_active {
            add_where(&mut qb);
            qb.push("is_active = ");
            qb.push_bind(if is_active { 1 } else { 0 });
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            qb.push("(name LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(" OR description LIKE ");
            qb.push_bind(format!("%{search_term}%"));
            qb.push(")");
        }

        qb.push(" ORDER BY name ASC");

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

        let rows = qb.build().fetch_all(&self.pool).await?;

        let mut api_keys = Vec::new();
        for row in rows {
            let api_key = ApiKey {
                id: row.get("id"),
                account_id: row.get("account_id"),
                name: row.get("name"),
                description: row.get("description"),
                key_hash: row.get("key_hash"),
                scopes: row.get::<Json<Value>, _>("scopes"),
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
        let now = Utc::now();
        debug!("Updating API key with ID: {}", api_key.id);

        let scopes = api_key.scopes.as_ref();
        let expires_at = api_key.expires_at.as_ref();

        sqlx::query!(
            r#"UPDATE api_keys SET
                account_id = ?, name = ?, description = ?, scopes = ?, key_hash = ?,
                expires_at = ?, rate_limit = ?, is_active = ?, updated_at = ?, last_used_at = ?
            WHERE id = ?"#,
            api_key.account_id,
            api_key.name,
            api_key.description,
            scopes,
            api_key.key_hash,
            expires_at,
            api_key.rate_limit,
            api_key.is_active, 
            now,
            api_key.last_used_at,
            api_key.id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update API key scopes
    #[instrument(err, skip(self, scopes))]
    pub async fn update_api_key_scopes(&self, id: &Uuid, scopes: &Json<Value>) -> Result<()> {
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

        sqlx::query!(
            r#"UPDATE api_keys SET expires_at = ?, updated_at = ? WHERE id = ?"#,
            expires_at,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

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

        sqlx::query!(
            r#"UPDATE api_keys SET rate_limit = ?, updated_at = ? WHERE id = ?"#,
            rate_limit,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    /// Set API key active status
    #[instrument(err, skip(self))]
    pub async fn set_api_key_active(&self, id: &Uuid, is_active: bool) -> Result<()> {
        debug!("Setting API key {} active status to {}", id, is_active);

        let now = Utc::now();

        sqlx::query!(
            r#"UPDATE api_keys SET is_active = ?, updated_at = ? WHERE id = ?"#,
            is_active,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }


    /// Update API key last used timestamp
    #[instrument(err, skip(self))]
    pub async fn update_api_key_last_used(&self, id: &Uuid) -> Result<()> {
        debug!("Updating last used timestamp for API key: {}", id);

        let now = Utc::now();

        sqlx::query!(
            r#"UPDATE api_keys SET last_used_at = ?, updated_at = ? WHERE id = ?"#,
            now,
            now,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete an API key by ID
    #[instrument(err, skip(self))]
    pub async fn delete_api_key(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting API key with ID: {}", id);

        sqlx::query!(
            r#"DELETE FROM api_keys WHERE id = ?"#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Delete all API keys for an account
    #[instrument(err, skip(self))]
    pub async fn delete_api_keys_for_account(&self, account_id: &Uuid) -> Result<u64> {
        debug!("Deleting all API keys for account: {}", account_id);

        let affected = sqlx::query!(
            r#"DELETE FROM api_keys WHERE account_id = ?"#,
            account_id
        ).execute(&self.pool)
        .await?;

        Ok(affected.rows_affected() as u64)
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

        Ok(sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM api_keys WHERE account_id = ?"#,
            account_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Count active API keys for an account
    #[instrument(skip(self))]
    pub async fn count_active_api_keys_for_account(&self, account_id: &Uuid) -> Result<i64> {
        debug!("Counting active API keys for account: {}", account_id);

        Ok(sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM api_keys WHERE account_id = ? AND is_active = 1"#,
            account_id
        )
        .fetch_one(&self.pool)
        .await?)
    }

    /// Check if an API key is valid (active, not expired)
    #[instrument(skip(self))]
    pub async fn is_api_key_valid(&self, key_hash: &str) -> Result<bool> {
        debug!("Checking if API key is valid");

        let now = Utc::now();

        let row = sqlx::query!(
            r#"SELECT COUNT(*) as count FROM api_keys   
               WHERE key_hash = ? AND is_active = 1 
               AND (expires_at IS NULL OR expires_at > ?)"#,
            key_hash,
            now
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count > 0)
    }
}
