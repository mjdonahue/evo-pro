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
pub enum AccountType {
    Personal = 0,
    Group = 1,
    Organization = 2,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active = 0,
    Archived = 1,
    Deleted = 2,
}

/// Account model matching the SQLite schema
#[boilermates("CreateAccount")]
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[boilermates(not_in("CreateAccount"))]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub workspace_id: Option<Uuid>,
    pub primary_address_id: Option<Uuid>,
    pub account_type: AccountType,
    pub status: AccountStatus,
    pub metadata: Option<Json<Value>>,
    #[boilermates(not_in("CreateAccount"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateAccount"))]
    pub updated_at: DateTime<Utc>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccountRequest {
    pub name: String,
    pub description: Option<String>,
    pub workspace_id: Option<String>,
    pub primary_address_id: Option<String>,
    pub account_type: AccountType,
    pub metadata: Option<Json<Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAccountRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub workspace_id: Option<String>,
    pub primary_address_id: Option<String>,
    pub account_type: Option<AccountType>,
    pub status: Option<AccountStatus>,
    pub metadata: Option<Json<Value>>,   
}

#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
pub struct AccountListFilter {
    pub workspace_id: Option<String>,
    pub status: Option<AccountStatus>,
    pub type_: Option<AccountType>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub workspace_id: Option<String>,
    pub primary_address_id: Option<String>,
    pub account_type: AccountType,
    pub status: AccountStatus,
    pub metadata: Option<Json<Value>>,
    pub created_at: String,
    pub updated_at: String,
}

impl DatabaseManager {
    /// Create a new account
    #[instrument(skip(self))]
    pub async fn create_account(&self, account: &CreateAccountRequest) -> Result<Account> {
        let id = Uuid::new_v4();
        debug!("Creating account with ID: {}", id);
        
        let metadata = account.metadata.as_ref();
        let now = Utc::now();


        Ok(sqlx::query_as!(
            Account,
            r#"INSERT INTO accounts (
            id, name, description, workspace_id, primary_address_id, account_type, status, metadata, created_at, updated_at
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING 
                id AS "id: _", name, description, workspace_id AS "workspace_id: _",
                primary_address_id AS "primary_address_id: _", account_type AS "account_type: AccountType", status as "status: AccountStatus",
                metadata AS "metadata: _",
                created_at AS "created_at: _", updated_at AS "updated_at: _"
            "#,
            id,
            account.name,
            account.description,
            account.workspace_id,
            account.primary_address_id,
            account.account_type,
            AccountStatus::Active,
            metadata,
            now,
            now
        )
        .fetch_one(&self.pool)
        .await?)
    }

    #[instrument(skip(self))]
    pub async fn get_account_by_id(&self, id: &Uuid) -> Result<Option<Account>> {
        debug!("Getting account by ID: {}", id);
        debug!("Getting account by ID: {:?}", id);

        Ok(sqlx::query_as!(
            Account,
            r#"SELECT
                    id AS "id: _", name, description, workspace_id AS "workspace_id: _",
                    primary_address_id AS "primary_address_id: _", account_type AS "account_type: AccountType", status as "status: AccountStatus",
                    metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM accounts WHERE id = ?"#,
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    #[instrument(skip(self))]
    pub async fn get_account_by_name(&self, name: &str) -> Result<Option<Account>> {
        debug!("Getting account by name: {}", name);

        Ok(sqlx::query_as!(
            Account,
            r#"SELECT
                    id AS "id: _", name, description, workspace_id AS "workspace_id: _",
                    primary_address_id AS "primary_address_id: _", account_type AS "account_type: AccountType", status as "status: AccountStatus",
                    metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _"
                FROM accounts WHERE name = ?"#,
            name,
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    #[instrument(skip(self))]
    pub async fn list_accounts(&self, filter: &AccountListFilter) -> Result<Vec<Account>> {
        debug!("Listing accounts with filter: {:?}", filter);
        let mut qb = QueryBuilder::new("SELECT * FROM accounts");

        if let Some(workspace_id) = &filter.workspace_id {
            qb.push(" WHERE workspace_id = ");
            qb.push_bind(workspace_id); 
        }

        if let Some(status) = &filter.status {
            qb.push(" AND status = ");
            qb.push_bind(status);
        }

        if let Some(type_) = &filter.type_ {
            qb.push(" AND type_ = ");
            qb.push_bind(type_);
        }

        if let Some(search_term) = &filter.search_term {
            qb.push(" AND name LIKE ");
            qb.push_bind(format!("%{}%", search_term));
        }

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i32);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i32);
        }

        Ok(qb
            .build_query_as::<Account>()
            .fetch_all(&self.pool)
            .await?)
    }

    #[instrument(skip(self))]
    pub async fn update_account(&self, account: &Account) -> Result<Account> {
        debug!("Updating account with ID: {}", account.id); 
        let metadata = account.metadata.as_ref();

        Ok(sqlx::query_as!(
            Account,
            r#"UPDATE accounts SET 
            name = ?, description = ?, workspace_id = ?, primary_address_id = ?, account_type = ?, status = ?, metadata = ?, updated_at = ?
            WHERE id = ? RETURNING
                id AS "id: _", name, description, workspace_id AS "workspace_id: _",
                primary_address_id AS "primary_address_id: _", account_type AS "account_type: AccountType", status as "status: AccountStatus",
                metadata AS "metadata: _",
                created_at AS "created_at: _", updated_at AS "updated_at: _""#,
            account.name,
            account.description,
            account.workspace_id,
            account.primary_address_id,
            account.account_type,
            account.status, 
            metadata,
            account.updated_at,
            account.id,
        )
        .fetch_one(&self.pool)
        .await?)
    }

    #[instrument(skip(self))]
    pub async fn delete_account(&self, id: &Uuid) -> Result<()> {
        debug!("Deleting account with ID: {}", id);

        let affected = sqlx::query!("DELETE FROM accounts WHERE id = ?", id)
            .execute(&self.pool)
            .await?
            .rows_affected();   

        if affected == 0 {
            return Err(AppError::NotFoundError(format!("Account with ID {id} not found for delete")));
        }

        Ok(())
    }
}
