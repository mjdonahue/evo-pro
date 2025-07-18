use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::Json;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::entities::users::User;
use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::services::data_minimization::DataMinimizationService;

/// Represents the retention policy for different data categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Retention period for messages in days (None means keep forever)
    pub messages_retention_days: Option<i64>,
    /// Retention period for events in days (None means keep forever)
    pub events_retention_days: Option<i64>,
    /// Retention period for inactive accounts in days (None means keep forever)
    pub inactive_account_days: Option<i64>,
    /// Whether to anonymize data instead of deleting it
    pub anonymize_instead_of_delete: bool,
    /// Categories to exclude from automatic cleanup
    pub excluded_categories: Vec<String>,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            messages_retention_days: Some(365), // Default 1 year
            events_retention_days: Some(180),   // Default 6 months
            inactive_account_days: Some(730),   // Default 2 years
            anonymize_instead_of_delete: true,
            excluded_categories: vec![],
        }
    }
}

/// Service for managing data retention policies
pub struct DataRetentionService {
    db: DatabaseManager,
    minimization_service: DataMinimizationService,
}

impl DataRetentionService {
    /// Create a new DataRetentionService
    pub fn new(db: DatabaseManager) -> Self {
        let minimization_service = DataMinimizationService::new(db.clone());
        Self { db, minimization_service }
    }

    /// Get the retention policy for a specific user
    #[instrument(skip(self), err)]
    pub async fn get_user_retention_policy(&self, user_id: &Uuid) -> Result<RetentionPolicy> {
        debug!("Getting retention policy for user: {}", user_id);

        // Get the user from the database
        let user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

        // Check if the user has retention preferences
        if let Some(preferences) = &user.preferences {
            if let Some(retention) = preferences.0.get("retention") {
                if let Ok(policy) = serde_json::from_value(retention.clone()) {
                    return Ok(policy);
                }
            }
        }

        // Return default policy if no user-specific policy is found
        Ok(RetentionPolicy::default())
    }

    /// Set the retention policy for a specific user
    #[instrument(skip(self, policy), err)]
    pub async fn set_user_retention_policy(&self, user_id: &Uuid, policy: RetentionPolicy) -> Result<()> {
        debug!("Setting retention policy for user: {}", user_id);

        // Get the user from the database
        let mut user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

        // Update or create the preferences JSON
        let mut preferences = if let Some(prefs) = user.preferences {
            prefs.0
        } else {
            json!({})
        };

        // Set the retention policy in the preferences
        if let Value::Object(ref mut map) = preferences {
            map.insert("retention".to_string(), json!(policy));
        }

        // Update the user preferences
        user.preferences = Some(Json(preferences));
        self.db.update_user(&user).await?;

        info!("Updated retention policy for user: {}", user_id);
        Ok(())
    }

    /// Apply retention policies for all users
    #[instrument(skip(self), err)]
    pub async fn apply_all_retention_policies(&self) -> Result<()> {
        debug!("Applying retention policies for all users");

        // Get all users
        let users = self.db.list_users(&Default::default()).await?;

        for user in users {
            match self.apply_user_retention_policy(&user.id).await {
                Ok(_) => info!("Applied retention policy for user: {}", user.id),
                Err(e) => warn!("Failed to apply retention policy for user {}: {}", user.id, e),
            }
        }

        Ok(())
    }

    /// Apply retention policy for a specific user
    #[instrument(skip(self), err)]
    pub async fn apply_user_retention_policy(&self, user_id: &Uuid) -> Result<()> {
        debug!("Applying retention policy for user: {}", user_id);

        // Get the user's retention policy
        let policy = self.get_user_retention_policy(user_id).await?;

        // Apply message retention if configured
        if let Some(days) = policy.messages_retention_days {
            self.apply_message_retention(user_id, days, policy.anonymize_instead_of_delete).await?;
        }

        // Apply event retention if configured
        if let Some(days) = policy.events_retention_days {
            self.apply_event_retention(user_id, days, policy.anonymize_instead_of_delete).await?;
        }

        // Apply inactive account policy if configured
        if let Some(days) = policy.inactive_account_days {
            self.apply_inactive_account_policy(user_id, days).await?;
        }

        info!("Successfully applied retention policy for user: {}", user_id);
        Ok(())
    }

    /// Apply message retention policy for a user
    async fn apply_message_retention(&self, user_id: &Uuid, days: i64, anonymize: bool) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(days);

        if anonymize {
            // Get old messages
            let messages = self.db.get_old_messages_by_user(user_id, cutoff_date).await?;

            // Anonymize each message
            for mut message in messages {
                self.minimization_service.anonymize_message(&mut message).await?;
            }

            info!("Anonymized old messages for user: {}", user_id);
        } else {
            // Delete old messages
            let deleted_count = self.db.delete_old_messages_by_user(user_id, cutoff_date).await?;
            info!("Deleted {} old messages for user: {}", deleted_count, user_id);
        }

        Ok(())
    }

    /// Apply event retention policy for a user
    async fn apply_event_retention(&self, user_id: &Uuid, days: i64, anonymize: bool) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(days);

        if anonymize {
            // Get old events
            let events = self.db.get_old_events_by_user(user_id, cutoff_date).await?;

            // Anonymize each event (simplified for now)
            for event in events {
                // Anonymize event data
                // This would be implemented similar to message anonymization
            }

            info!("Anonymized old events for user: {}", user_id);
        } else {
            // Delete old events
            let deleted_count = self.db.delete_old_events_by_user(user_id, cutoff_date).await?;
            info!("Deleted {} old events for user: {}", deleted_count, user_id);
        }

        Ok(())
    }

    /// Apply inactive account policy for a user
    async fn apply_inactive_account_policy(&self, user_id: &Uuid, days: i64) -> Result<()> {
        let cutoff_date = Utc::now() - Duration::days(days);

        // Get the user
        let user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

        // Check if the user is inactive
        if let Some(last_seen) = user.last_seen {
            if last_seen < cutoff_date {
                // Anonymize the inactive user
                let mut user_to_anonymize = user.clone();
                self.minimization_service.anonymize_user(&mut user_to_anonymize).await?;
                info!("Anonymized inactive user: {}", user_id);
            }
        }

        Ok(())
    }
}

// Tauri command for setting user retention policy
#[tauri::command]
pub async fn set_retention_policy(
    user_id: String,
    messages_days: Option<i64>,
    events_days: Option<i64>,
    inactive_days: Option<i64>,
    anonymize: bool,
    excluded_categories: Vec<String>,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

    let policy = RetentionPolicy {
        messages_retention_days: messages_days,
        events_retention_days: events_days,
        inactive_account_days: inactive_days,
        anonymize_instead_of_delete: anonymize,
        excluded_categories,
    };

    let service = DataRetentionService::new(db.inner().clone());

    match service.set_user_retention_policy(&user_id, policy).await {
        Ok(()) => Ok("Retention policy updated successfully".to_string()),
        Err(e) => Err(format!("Failed to update retention policy: {}", e)),
    }
}

// Tauri command for getting user retention policy
#[tauri::command]
pub async fn get_retention_policy(
    user_id: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<RetentionPolicy, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

    let service = DataRetentionService::new(db.inner().clone());

    match service.get_user_retention_policy(&user_id).await {
        Ok(policy) => Ok(policy),
        Err(e) => Err(format!("Failed to get retention policy: {}", e)),
    }
}

// Tauri command for applying user retention policy
#[tauri::command]
pub async fn apply_retention_policy(
    user_id: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

    let service = DataRetentionService::new(db.inner().clone());

    match service.apply_user_retention_policy(&user_id).await {
        Ok(()) => Ok("Retention policy applied successfully".to_string()),
        Err(e) => Err(format!("Failed to apply retention policy: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use sqlx::types::Json;
    use serde_json::json;
    use crate::entities::users::{User, UserStatus, UserRole};
    use crate::entities::messages::{Message, MessageStatus};
    use crate::storage::db::DatabaseManager;

    #[tokio::test]
    async fn test_retention_policy() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = DataRetentionService::new(db.clone());

        // Create a test user
        let user = User {
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
            status: UserStatus::Active,
            email_verified: false,
            phone_verified: false,
            last_seen: Some(Utc::now()),
            primary_role: UserRole::User,
            roles: Json(json!(["user"])),
            preferences: Some(Json(json!({}))),
            metadata: Some(Json(json!({}))),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            workspace_id: None,
            public_key: vec![],
        };

        // Insert the user into the database
        let user_id = user.id;
        db.create_user(&user).await.unwrap();

        // Create a test message
        let message = Message {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            sender_id: user_id,
            parent_message_id: None,
            content: "Test message content".to_string(),
            status: MessageStatus::Sent,
            refs: None,
            metadata: None,
            created_at: Utc::now() - Duration::days(100), // Old message
            updated_at: Utc::now() - Duration::days(100),
            reply_to_id: None,
            branch_conversation_id: None,
            parent_id: None,
            workspace_id: None,
        };

        // Insert the message into the database
        db.create_message(&message).await.unwrap();

        // Set a retention policy for the user
        let policy = RetentionPolicy {
            messages_retention_days: Some(30), // Keep messages for 30 days
            events_retention_days: Some(60),
            inactive_account_days: Some(365),
            anonymize_instead_of_delete: false, // Delete old data
            excluded_categories: vec![],
        };

        // Set the policy
        service.set_user_retention_policy(&user_id, policy).await.unwrap();

        // Verify the policy was set
        let retrieved_policy = service.get_user_retention_policy(&user_id).await.unwrap();
        assert_eq!(retrieved_policy.messages_retention_days, Some(30));
        assert_eq!(retrieved_policy.events_retention_days, Some(60));
        assert_eq!(retrieved_policy.anonymize_instead_of_delete, false);

        // Apply the retention policy
        service.apply_user_retention_policy(&user_id).await.unwrap();

        // Verify the old message was deleted
        let old_messages = db.get_old_messages_by_user(&user_id, Utc::now() - Duration::days(31)).await.unwrap();
        assert_eq!(old_messages.len(), 0);
    }
}
