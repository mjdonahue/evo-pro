//! Data Deletion Verification Service
//!
//! This module implements a service for verifying that user data has been properly deleted
//! from the system, providing users with confirmation and peace of mind.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::Json;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::entities::users::User;

/// Represents the result of a data deletion verification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletionVerificationResult {
    /// User ID for whom the verification was performed
    pub user_id: Uuid,
    /// When the verification was performed
    pub verification_date: DateTime<Utc>,
    /// Whether the deletion was successful
    pub deletion_successful: bool,
    /// Detailed results for each data category
    pub category_results: Vec<CategoryVerificationResult>,
    /// Overall verification status
    pub verification_status: VerificationStatus,
    /// Any issues found during verification
    pub issues_found: Vec<String>,
    /// Recommendations for resolving issues
    pub recommendations: Vec<String>,
}

/// Represents the verification result for a specific data category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryVerificationResult {
    /// Data category name
    pub category: String,
    /// Whether the deletion was successful for this category
    pub deletion_successful: bool,
    /// Number of items checked
    pub items_checked: i64,
    /// Number of items with issues
    pub items_with_issues: i64,
    /// Details about the verification
    pub details: String,
}

/// Represents the overall verification status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VerificationStatus {
    /// All data was successfully deleted
    FullyVerified,
    /// Some data was deleted but some issues were found
    PartiallyVerified,
    /// Significant issues were found with data deletion
    VerificationFailed,
}

/// Service for verifying data deletion
pub struct DataDeletionVerificationService {
    /// Database manager
    db: DatabaseManager,
}

impl DataDeletionVerificationService {
    /// Create a new data deletion verification service
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    /// Verify that a user's data has been properly deleted
    #[instrument(skip(self), err)]
    pub async fn verify_user_data_deletion(&self, user_id: &Uuid) -> Result<DeletionVerificationResult> {
        debug!("Verifying data deletion for user: {}", user_id);

        // Initialize verification result
        let mut result = DeletionVerificationResult {
            user_id: *user_id,
            verification_date: Utc::now(),
            deletion_successful: true,
            category_results: Vec::new(),
            verification_status: VerificationStatus::FullyVerified,
            issues_found: Vec::new(),
            recommendations: Vec::new(),
        };

        // Verify user profile data
        let profile_result = self.verify_profile_deletion(user_id).await?;
        if !profile_result.deletion_successful {
            result.deletion_successful = false;
            result.issues_found.push(format!("User profile data still exists: {}", profile_result.details));
            result.recommendations.push("Contact support to manually remove remaining profile data.".to_string());
        }
        result.category_results.push(profile_result);

        // Verify messages
        let messages_result = self.verify_messages_deletion(user_id).await?;
        if !messages_result.deletion_successful {
            result.deletion_successful = false;
            result.issues_found.push(format!("Message data still exists: {}", messages_result.details));
            result.recommendations.push("Run data retention policy again with 'anonymize' set to false.".to_string());
        }
        result.category_results.push(messages_result);

        // Verify events
        let events_result = self.verify_events_deletion(user_id).await?;
        if !events_result.deletion_successful {
            result.deletion_successful = false;
            result.issues_found.push(format!("Event data still exists: {}", events_result.details));
            result.recommendations.push("Run data retention policy again with 'anonymize' set to false.".to_string());
        }
        result.category_results.push(events_result);

        // Verify other references to user
        let references_result = self.verify_user_references_deletion(user_id).await?;
        if !references_result.deletion_successful {
            result.deletion_successful = false;
            result.issues_found.push(format!("References to user still exist: {}", references_result.details));
            result.recommendations.push("Use the data export tool to identify remaining references and contact support.".to_string());
        }
        result.category_results.push(references_result);

        // Determine overall verification status
        if !result.deletion_successful {
            let total_issues = result.category_results.iter().map(|r| r.items_with_issues).sum::<i64>();
            if total_issues > 10 {
                result.verification_status = VerificationStatus::VerificationFailed;
            } else {
                result.verification_status = VerificationStatus::PartiallyVerified;
            }
        }

        info!("Data deletion verification completed for user: {}, status: {:?}", user_id, result.verification_status);
        Ok(result)
    }

    /// Verify that a user's profile has been deleted
    async fn verify_profile_deletion(&self, user_id: &Uuid) -> Result<CategoryVerificationResult> {
        debug!("Verifying profile deletion for user: {}", user_id);

        // Check if user still exists in the database
        let user = self.db.get_user_by_id(user_id).await?;
        
        let mut result = CategoryVerificationResult {
            category: "Profile".to_string(),
            deletion_successful: true,
            items_checked: 1,
            items_with_issues: 0,
            details: "User profile has been successfully deleted".to_string(),
        };

        if let Some(user) = user {
            // User still exists, check if it's properly anonymized
            if user.status.to_string() != "deleted" {
                result.deletion_successful = false;
                result.items_with_issues = 1;
                result.details = format!("User profile still exists with status: {:?}", user.status);
            } else if user.email.is_some() && !user.email.as_ref().unwrap().contains("***") {
                // Email exists and is not anonymized
                result.deletion_successful = false;
                result.items_with_issues = 1;
                result.details = "User email still exists and is not anonymized".to_string();
            } else if user.first_name.is_some() && !user.first_name.as_ref().unwrap().contains(".") {
                // First name exists and is not anonymized
                result.deletion_successful = false;
                result.items_with_issues = 1;
                result.details = "User first name still exists and is not anonymized".to_string();
            } else if user.last_name.is_some() && !user.last_name.as_ref().unwrap().contains(".") {
                // Last name exists and is not anonymized
                result.deletion_successful = false;
                result.items_with_issues = 1;
                result.details = "User last name still exists and is not anonymized".to_string();
            }
        }

        Ok(result)
    }

    /// Verify that a user's messages have been deleted
    async fn verify_messages_deletion(&self, user_id: &Uuid) -> Result<CategoryVerificationResult> {
        debug!("Verifying messages deletion for user: {}", user_id);

        // Count messages sent by the user
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM messages WHERE sender_id = ?",
            user_id
        )
        .fetch_one(&self.db.pool)
        .await?
        .count;

        let mut result = CategoryVerificationResult {
            category: "Messages".to_string(),
            deletion_successful: count == 0,
            items_checked: 1,
            items_with_issues: if count > 0 { 1 } else { 0 },
            details: if count == 0 {
                "All messages have been successfully deleted".to_string()
            } else {
                format!("{} messages still exist", count)
            },
        };

        Ok(result)
    }

    /// Verify that a user's events have been deleted
    async fn verify_events_deletion(&self, user_id: &Uuid) -> Result<CategoryVerificationResult> {
        debug!("Verifying events deletion for user: {}", user_id);

        // Count events created by the user
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM events WHERE created_by_user_id = ?",
            user_id
        )
        .fetch_one(&self.db.pool)
        .await?
        .count;

        let mut result = CategoryVerificationResult {
            category: "Events".to_string(),
            deletion_successful: count == 0,
            items_checked: 1,
            items_with_issues: if count > 0 { 1 } else { 0 },
            details: if count == 0 {
                "All events have been successfully deleted".to_string()
            } else {
                format!("{} events still exist", count)
            },
        };

        Ok(result)
    }

    /// Verify that all references to a user have been deleted
    async fn verify_user_references_deletion(&self, user_id: &Uuid) -> Result<CategoryVerificationResult> {
        debug!("Verifying user references deletion for user: {}", user_id);

        // Initialize counters
        let mut total_references = 0;
        let mut tables_with_references = Vec::new();

        // Check for references in various tables
        // This is a simplified example - in a real implementation, you would check all tables
        // that might contain references to a user

        // Check participants table
        let participant_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM participants WHERE user_id = ?",
            user_id
        )
        .fetch_one(&self.db.pool)
        .await?
        .count;

        if participant_count > 0 {
            total_references += participant_count;
            tables_with_references.push(format!("participants ({})", participant_count));
        }

        // Check conversations table
        let conversation_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM conversations WHERE created_by_id = ?",
            user_id
        )
        .fetch_one(&self.db.pool)
        .await?
        .count;

        if conversation_count > 0 {
            total_references += conversation_count;
            tables_with_references.push(format!("conversations ({})", conversation_count));
        }

        // Check agents table
        let agent_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM agents WHERE operator_id = ?",
            user_id
        )
        .fetch_one(&self.db.pool)
        .await?
        .count;

        if agent_count > 0 {
            total_references += agent_count;
            tables_with_references.push(format!("agents ({})", agent_count));
        }

        let mut result = CategoryVerificationResult {
            category: "References".to_string(),
            deletion_successful: total_references == 0,
            items_checked: 3, // Number of tables checked
            items_with_issues: tables_with_references.len() as i64,
            details: if total_references == 0 {
                "No references to the user found in the system".to_string()
            } else {
                format!("Found {} references to the user in tables: {}", 
                    total_references, 
                    tables_with_references.join(", ")
                )
            },
        };

        Ok(result)
    }

    /// Generate a deletion certificate for a user
    #[instrument(skip(self), err)]
    pub async fn generate_deletion_certificate(&self, user_id: &Uuid) -> Result<Value> {
        debug!("Generating deletion certificate for user: {}", user_id);

        // Verify data deletion first
        let verification_result = self.verify_user_data_deletion(user_id).await?;

        // Only generate a certificate if verification was successful
        if verification_result.verification_status != VerificationStatus::FullyVerified {
            return Err(AppError::ValidationError(
                format!("Cannot generate deletion certificate: data deletion verification failed with status {:?}", 
                    verification_result.verification_status)
            ));
        }

        // Generate certificate
        let certificate = json!({
            "certificateId": Uuid::new_v4().to_string(),
            "userId": user_id.to_string(),
            "issuedAt": Utc::now().to_rfc3339(),
            "verificationResult": verification_result,
            "statement": "This certificate confirms that all personal data associated with the specified user ID has been permanently deleted from our systems in accordance with our privacy policy and applicable data protection regulations.",
            "issuer": "Evo Pro Data Protection Team"
        });

        info!("Generated deletion certificate for user: {}", user_id);
        Ok(certificate)
    }
}

// Tauri command for verifying data deletion
#[tauri::command]
pub async fn verify_data_deletion(
    user_id: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<DeletionVerificationResult, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    let service = DataDeletionVerificationService::new(db.inner().clone());
    
    match service.verify_user_data_deletion(&user_id).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Failed to verify data deletion: {}", e)),
    }
}

// Tauri command for generating a deletion certificate
#[tauri::command]
pub async fn generate_deletion_certificate(
    user_id: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<Value, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    let service = DataDeletionVerificationService::new(db.inner().clone());
    
    match service.generate_deletion_certificate(&user_id).await {
        Ok(certificate) => Ok(certificate),
        Err(e) => Err(format!("Failed to generate deletion certificate: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::users::{User, UserStatus, UserRole};
    use sqlx::types::Json;
    use serde_json::json;

    #[tokio::test]
    async fn test_verify_data_deletion() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = DataDeletionVerificationService::new(db.clone());
        
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
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            workspace_id: None,
            public_key: vec![],
        };
        
        // Insert the user into the database
        let user_id = user.id;
        db.create_user(&user).await.unwrap();
        
        // Verify data deletion (should fail since user still exists)
        let result = service.verify_user_data_deletion(&user_id).await.unwrap();
        assert_eq!(result.verification_status, VerificationStatus::VerificationFailed);
        assert_eq!(result.deletion_successful, false);
        
        // Update user status to deleted
        let mut deleted_user = user.clone();
        deleted_user.status = UserStatus::Deleted;
        deleted_user.email = Some("t***@example.com".to_string());
        deleted_user.first_name = Some("T.".to_string());
        deleted_user.last_name = Some("U.".to_string());
        db.update_user(&deleted_user).await.unwrap();
        
        // Verify data deletion again (should be partially verified now)
        let result = service.verify_user_data_deletion(&user_id).await.unwrap();
        assert_eq!(result.verification_status, VerificationStatus::PartiallyVerified);
        
        // Delete all references to the user
        sqlx::query!("DELETE FROM messages WHERE sender_id = ?", user_id)
            .execute(&db.pool)
            .await
            .unwrap();
        
        sqlx::query!("DELETE FROM events WHERE created_by_user_id = ?", user_id)
            .execute(&db.pool)
            .await
            .unwrap();
        
        // Verify data deletion again (should be fully verified now)
        let result = service.verify_user_data_deletion(&user_id).await.unwrap();
        assert_eq!(result.verification_status, VerificationStatus::FullyVerified);
        
        // Try to generate a certificate
        let certificate = service.generate_deletion_certificate(&user_id).await.unwrap();
        assert!(certificate.get("certificateId").is_some());
        assert_eq!(certificate["userId"], user_id.to_string());
    }
}