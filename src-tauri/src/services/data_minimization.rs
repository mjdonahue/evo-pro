use chrono::{DateTime, Duration, Utc};
use serde_json::{json, Value};
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::entities::messages::Message;
use crate::entities::users::User;
use crate::error::{AppError, Result};
use crate::privacy::anonymization::{Anonymizer, AnonymizationConfig, AnonymizationStrategy};
use crate::storage::db::DatabaseManager;

/// Service for implementing data minimization strategies
pub struct DataMinimizationService {
    db: DatabaseManager,
    anonymizer: Anonymizer,
}

impl DataMinimizationService {
    /// Create a new DataMinimizationService
    pub fn new(db: DatabaseManager) -> Self {
        // Create a default anonymizer with standard configuration
        let anonymizer = Anonymizer::default();
        Self { db, anonymizer }
    }

    /// Create a new DataMinimizationService with custom anonymization configuration
    pub fn with_config(db: DatabaseManager, config: AnonymizationConfig) -> Self {
        let anonymizer = Anonymizer::new(config);
        Self { db, anonymizer }
    }

    /// Anonymize user data by replacing sensitive fields with anonymized versions
    #[instrument(skip(self, user))]
    pub async fn anonymize_user(&self, user: &mut User) -> Result<()> {
        debug!("Anonymizing user data for user ID: {}", user.id);

        // Use the anonymizer to anonymize user fields
        if let Some(email) = &user.email {
            user.email = Some(self.anonymizer.anonymize_email(email));
        }

        if let Some(phone) = &user.mobile_phone {
            user.mobile_phone = Some(self.anonymizer.anonymize_phone(phone));
        }

        if let Some(first_name) = &user.first_name {
            user.first_name = Some(
                crate::privacy::anonymization::utils::anonymize_name(first_name, true)
            );
        }

        if let Some(last_name) = &user.last_name {
            user.last_name = Some(
                crate::privacy::anonymization::utils::anonymize_name(last_name, true)
            );
        }

        // Anonymize metadata using the JSON anonymizer
        if let Some(metadata) = &user.metadata {
            let anonymized_metadata = self.anonymizer.anonymize_json(&metadata.0);
            user.metadata = Some(sqlx::types::Json(anonymized_metadata));
        }

        // Update the user in the database
        self.db.update_user(user).await?;

        info!("Successfully anonymized user data for user ID: {}", user.id);
        Ok(())
    }

    /// Anonymize message content to remove sensitive information
    #[instrument(skip(self, message))]
    pub async fn anonymize_message(&self, message: &mut Message) -> Result<()> {
        debug!("Anonymizing message data for message ID: {}", message.id);

        // Use the anonymizer to anonymize message content
        // This provides more comprehensive detection and redaction of sensitive information
        message.content = self.anonymizer.anonymize_text(&message.content);

        // Anonymize metadata using the JSON anonymizer
        if let Some(metadata) = &message.metadata {
            let anonymized_metadata = self.anonymizer.anonymize_json(&metadata.0);
            message.metadata = Some(sqlx::types::Json(anonymized_metadata));
        }

        // Update the message in the database
        self.db.update_message(message).await?;

        info!("Successfully anonymized message data for message ID: {}", message.id);
        Ok(())
    }

    /// Apply data retention policy to automatically remove old data
    #[instrument(skip(self))]
    pub async fn apply_retention_policy(&self, retention_days: i64) -> Result<()> {
        debug!("Applying data retention policy: removing data older than {} days", retention_days);

        let cutoff_date = Utc::now() - Duration::days(retention_days);

        // Delete old messages
        let deleted_messages = sqlx::query!(
            "DELETE FROM messages WHERE created_at < ? RETURNING id",
            cutoff_date
        )
        .fetch_all(&self.db.pool)
        .await?;

        info!("Deleted {} old messages", deleted_messages.len());

        // For users, we might want to anonymize rather than delete
        let old_users = sqlx::query_as!(
            User,
            r#"SELECT id AS "id: _", contact_id AS "contact_id: _", email, username, operator_agent_id AS "operator_agent_id: _", 
            display_name, first_name, last_name, mobile_phone, avatar_url, bio, status AS "status: _", 
            email_verified, phone_verified, last_seen AS "last_seen: _", primary_role AS "primary_role: _", 
            roles AS "roles: _", preferences AS "preferences: _", metadata AS "metadata: _", 
            created_at AS "created_at: _", updated_at AS "updated_at: _", workspace_id AS "workspace_id: _", 
            public_key AS "public_key: _"
            FROM users 
            WHERE last_seen < ? AND status != 3"#, // Not already deleted
            cutoff_date
        )
        .fetch_all(&self.db.pool)
        .await?;

        for mut user in old_users {
            self.anonymize_user(&mut user).await?;
        }

        info!("Anonymized {} inactive users", old_users.len());

        Ok(())
    }

    /// Minimize data collection by providing a filtered version of an entity
    /// that only includes necessary fields for the given purpose
    #[instrument(skip(self, user_id, purpose))]
    pub async fn get_minimized_user(&self, user_id: &Uuid, purpose: &str) -> Result<Value> {
        debug!("Getting minimized user data for user ID: {} for purpose: {}", user_id, purpose);

        let user = self.db.get_user_by_id(user_id).await?
            .ok_or_else(|| AppError::NotFoundError(format!("User with ID {} not found", user_id)))?;

        // Different purposes require different levels of data
        let minimized_user = match purpose {
            "display" => {
                // For display purposes, we only need basic display information
                json!({
                    "id": user.id,
                    "displayName": user.display_name,
                    "avatarUrl": user.avatar_url,
                    "status": user.status,
                })
            },
            "messaging" => {
                // For messaging, we need contact information but not full details
                json!({
                    "id": user.id,
                    "displayName": user.display_name,
                    "avatarUrl": user.avatar_url,
                    "status": user.status,
                    "lastSeen": user.last_seen,
                })
            },
            "profile" => {
                // For profile viewing, we need more details but still minimize
                json!({
                    "id": user.id,
                    "displayName": user.display_name,
                    "firstName": user.first_name,
                    "lastName": user.last_name,
                    "avatarUrl": user.avatar_url,
                    "bio": user.bio,
                    "status": user.status,
                    "lastSeen": user.last_seen,
                    "primaryRole": user.primary_role,
                })
            },
            _ => {
                // Default case with minimal information
                json!({
                    "id": user.id,
                    "displayName": user.display_name,
                })
            }
        };

        Ok(minimized_user)
    }

    /// Register a new command to expose data minimization functionality to the frontend
    pub fn register_commands(app: &mut tauri::App) -> Result<()> {
        // Commands will be registered here when implementing the frontend integration
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::types::Json;

    #[tokio::test]
    async fn test_anonymize_user() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = DataMinimizationService::new(db);

        // Create a test user
        let mut user = User {
            id: Uuid::new_v4(),
            contact_id: None,
            email: Some("test.user@example.com".to_string()),
            username: Some("testuser".to_string()),
            operator_agent_id: None,
            display_name: "Test User".to_string(),
            first_name: Some("Test".to_string()),
            last_name: Some("User".to_string()),
            mobile_phone: Some("123-456-7890".to_string()),
            avatar_url: None,
            bio: Some("This is a test user bio".to_string()),
            status: crate::entities::users::UserStatus::Active,
            email_verified: false,
            phone_verified: false,
            last_seen: Some(Utc::now()),
            primary_role: crate::entities::users::UserRole::User,
            roles: Json(json!(["user"])),
            preferences: Some(Json(json!({}))),
            metadata: Some(Json(json!({
                "address": "123 Test St",
                "dob": "1990-01-01"
            }))),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            workspace_id: None,
            public_key: vec![],
        };

        // Insert the user into the database
        let user_id = user.id;
        service.db.create_user(&user).await.unwrap();

        // Anonymize the user
        service.anonymize_user(&mut user).await.unwrap();

        // Verify anonymization
        assert_eq!(user.email, Some("t***@example.com".to_string()));
        assert_eq!(user.mobile_phone, Some("*******7890".to_string()));
        assert_eq!(user.first_name, Some("T.".to_string()));
        assert_eq!(user.last_name, Some("U.".to_string()));

        // Verify metadata fields were removed
        let metadata = user.metadata.unwrap().0;
        assert!(metadata.get("address").is_none());
        assert!(metadata.get("dob").is_none());
    }
}
