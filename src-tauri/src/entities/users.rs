use boilermates::boilermates;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use sqlx::types::Json;
use sqlx::{FromRow, QueryBuilder, Sqlite};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::utils::add_where;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_status")]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active = 0,
    Inactive = 1,
    Suspended = 2,
    Deleted = 3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "user_role")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User = 0,
    Admin = 1,
    Agent = 2,
    Contact = 3,
    Other = 4,
}

/// User model matching the SQLite schema
#[boilermates("CreateUser")]
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[boilermates(not_in("CreateUser"))]
    pub id: Uuid, // BLOB in SQLite
    pub contact_id: Option<Uuid>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub operator_agent_id: Option<Uuid>,
    pub display_name: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub mobile_phone: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub status: UserStatus,
    pub email_verified: bool,
    pub phone_verified: bool,
    pub last_seen: Option<DateTime<Utc>>,
    pub primary_role: UserRole,
    pub roles: Json<Value>,            // JSON array in SQLite
    pub preferences: Option<Json<Value>>,      // JSON object
    pub metadata: Option<Json<Value>>, // JSON object
    #[boilermates(not_in("CreateUser"))]
    pub created_at: DateTime<Utc>,
    #[boilermates(not_in("CreateUser"))]
    pub updated_at: DateTime<Utc>,
    pub workspace_id: Option<Uuid>,
    pub public_key: Vec<u8>,
}

/// Additional filtering options for user queries
#[skip_serializing_none]
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFilter {
    pub status: Option<UserStatus>, 
    pub primary_role: Option<UserRole>,
    pub search_term: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl DatabaseManager {
    /// Create a new user
    #[instrument(err, skip(self, user))]
    pub async fn create_user(&self, user: &User) -> Result<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let metadata = user.metadata.as_deref();
        let preferences = user.preferences.as_deref();
        let roles = user.roles.as_ref();

        Ok(sqlx::query_as!(
            User,
            r#"INSERT INTO users (
                id, contact_id, email, username, operator_agent_id, display_name, first_name, last_name,
                mobile_phone, avatar_url, bio, status, email_verified, phone_verified, last_seen,
                primary_role, roles, preferences, metadata, created_at, updated_at, workspace_id, public_key
            ) VALUES (
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            ) RETURNING id AS "id: _", contact_id AS "contact_id: _", email, username, operator_agent_id AS "operator_agent_id: _", display_name, first_name, last_name,
                mobile_phone, avatar_url, bio, status AS "status: UserStatus", email_verified, phone_verified, last_seen AS "last_seen: _", primary_role AS "primary_role: UserRole", roles AS "roles: _",
                preferences AS "preferences: _", metadata AS "metadata: _", created_at AS "created_at: _",
                updated_at AS "updated_at: _", workspace_id AS "workspace_id: _", public_key AS "public_key: _""#,
            id,
            user.contact_id,
            user.email,
            user.username,
            user.operator_agent_id,
            user.display_name,
            user.first_name,
            user.last_name,
            user.mobile_phone,
            user.avatar_url,
            user.bio,
            user.status,
            user.email_verified,
            user.phone_verified,
            user.last_seen,
            user.primary_role,
            roles,
            preferences,
            metadata,
            now,
            now,
            user.workspace_id,
            user.public_key
        )
        .fetch_one(&self.pool)
        .await?)
    }

    #[instrument(err, skip(self, id))]
    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        debug!("Getting user by ID: {}", id);

        Ok(sqlx::query_as!( 
            User,
            r#"SELECT id AS "id: _", contact_id AS "contact_id: _", email, username, operator_agent_id AS "operator_agent_id: _", display_name, first_name, last_name,
                mobile_phone, avatar_url, bio, status AS "status: UserStatus", email_verified, phone_verified, last_seen AS "last_seen: _", primary_role AS "primary_role: UserRole", roles AS "roles: _", 
                preferences AS "preferences: _", metadata AS "metadata: _", created_at AS "created_at: _",
                updated_at AS "updated_at: _", workspace_id AS "workspace_id: _", public_key AS "public_key: _"
                FROM users WHERE id = ?"#,  
            id
        )
        .fetch_optional(&self.pool)
        .await?)
    }

    #[instrument(err, skip(self, user))]
    pub async fn update_user(&self, user: &User) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE users SET
            contact_id = ?, email = ?, username = ?, display_name = ?, first_name = ?, last_name = ?,
            mobile_phone = ?, workspace_id = ?, avatar_url = ?, bio = ?,
            status = ?, email_verified = ?, phone_verified = ?, last_seen = ?,
            roles = ?, preferences = ?, metadata = ?
            WHERE id = ?",
            user.contact_id,   
            user.email,
            user.username,
            user.display_name,
            user.first_name,
            user.last_name,
            user.mobile_phone,
            user.workspace_id,
            user.avatar_url,
            user.bio,
            user.status,
            user.email_verified,
            user.phone_verified,
            user.last_seen,
            user.roles,
            user.preferences,
            user.metadata,
            user.id
        )
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFoundError(format!(
                "User with ID {} not found",
                user.id
            )));
        }
        Ok(())
    }

    // Advanced query with JSON operations
    #[instrument(err, skip(self, role))]
    pub async fn get_users_by_role(&self, role: &str) -> Result<Vec<User>> {
        Ok(sqlx::query_as!(
            User,
            r#"
            SELECT id AS "id: _", contact_id AS "contact_id: _", email, username, display_name, first_name, last_name,
                mobile_phone, workspace_id AS "workspace_id: _", avatar_url, bio,
                status AS "status: UserStatus", email_verified, phone_verified, last_seen AS "last_seen: _", 
                primary_role AS "primary_role: UserRole", roles AS "roles: _", 
                preferences AS "preferences: _", metadata AS "metadata: _", created_at AS "created_at: _",
                updated_at AS "updated_at: _", operator_agent_id AS "operator_agent_id: _", public_key AS "public_key: _"
            FROM users
            WHERE JSON_EXTRACT(roles, '$[0]') = ?
            ORDER BY created_at DESC
            "#,
            role
        )
        .fetch_all(&self.pool)
        .await?)
    }

    #[instrument(err, skip(self, filter))]
    pub async fn list_users(&self, filter: &UserFilter) -> Result<Vec<User>> {
        let mut qb: QueryBuilder<Sqlite> = QueryBuilder::new(
            r#"SELECT 
                    id AS "id: _", contact_id AS "contact_id: _", email, username, display_name, first_name, last_name,
                    mobile_phone, workspace_id AS "workspace_id: _", avatar_url, bio,
                    status AS "status: UserStatus", email_verified, phone_verified, last_seen AS "last_seen: _", 
                    primary_role AS "primary_role: UserRole", roles AS "roles: _", preferences AS "preferences: _", metadata AS "metadata: _",
                    created_at AS "created_at: _", updated_at AS "updated_at: _", operator_agent_id AS "operator_agent_id: _", public_key AS "public_key: _"
                FROM users WHERE status != 2"#,
        );

        let mut add_where = add_where();

        if let Some(status) = &filter.status {
            add_where(&mut qb);
            qb.push("status = ");
            qb.push_bind(status.clone());
        }

        if let Some(role) = &filter.primary_role {
            add_where(&mut qb);
            qb.push("JSON_EXTRACT(roles, '$[0]') = ");  
            qb.push_bind(role);
        }

        if let Some(search_term) = &filter.search_term {
            add_where(&mut qb);
            let pattern = format!("%{search_term}%");
            qb.push("(display_name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR first_name LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" OR last_name LIKE ");
            qb.push_bind(pattern);
            qb.push(")");
        }

        if let Some(limit) = filter.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit as i64);
        }

        if let Some(offset) = filter.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset as i64);
        }

        Ok(qb
            .build_query_as::<'_, User>()
            .fetch_all(&self.pool)
            .await?)    
    }   
}
