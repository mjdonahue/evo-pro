//! User Consent Management Service
//!
//! This module implements a centralized service for managing user consent preferences
//! across all data categories and features of the application.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::Json;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::services::privacy_analytics::PrivacyAnalyticsService;
use crate::services::data_usage_reporting::UserDataPreferences;
use crate::entities::users::User;

/// Represents the complete consent status for a user across all categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserConsentStatus {
    /// User ID
    pub user_id: Uuid,
    /// When the consent was last updated
    pub last_updated: DateTime<Utc>,
    /// Data collection and processing consent
    pub data_collection: DataCollectionConsent,
    /// Analytics and tracking consent
    pub analytics: AnalyticsConsent,
    /// Communication consent
    pub communication: CommunicationConsent,
    /// Third-party sharing consent
    pub third_party: ThirdPartyConsent,
}

/// Consent for data collection and processing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataCollectionConsent {
    /// Whether the user has consented to profile data collection
    pub profile: bool,
    /// Whether the user has consented to preferences data collection
    pub preferences: bool,
    /// Whether the user has consented to messages data collection
    pub messages: bool,
    /// Retention period for profile data in days
    pub profile_retention_days: i64,
    /// Retention period for messages in days
    pub messages_retention_days: i64,
    /// Whether to anonymize profile data after retention period
    pub anonymize_profile: bool,
    /// Whether to anonymize messages after retention period
    pub anonymize_messages: bool,
}

/// Consent for analytics and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsConsent {
    /// Whether the user has consented to feature usage analytics
    pub feature_usage: bool,
    /// Whether the user has consented to performance analytics
    pub performance: bool,
    /// Whether the user has consented to error reporting
    pub error_reporting: bool,
    /// Whether the user has consented to user interface analytics
    pub user_interface: bool,
}

/// Consent for communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationConsent {
    /// Whether the user has consented to receiving product updates
    pub product_updates: bool,
    /// Whether the user has consented to receiving feature announcements
    pub feature_announcements: bool,
    /// Whether the user has consented to receiving marketing communications
    pub marketing: bool,
}

/// Consent for third-party sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThirdPartyConsent {
    /// Whether the user has consented to sharing data with third parties
    pub allow_sharing: bool,
    /// Specific third parties the user has consented to share data with
    pub approved_parties: Vec<String>,
}

/// Service for managing user consent
pub struct ConsentManagementService {
    /// Database manager
    db: DatabaseManager,
    /// Privacy analytics service
    analytics_service: PrivacyAnalyticsService,
}

impl ConsentManagementService {
    /// Create a new consent management service
    pub fn new(db: DatabaseManager) -> Self {
        let analytics_service = PrivacyAnalyticsService::new(db.clone());
        Self { db, analytics_service }
    }

    /// Get the complete consent status for a user
    #[instrument(skip(self), err)]
    pub async fn get_user_consent(&self, user_id: &Uuid) -> Result<UserConsentStatus> {
        debug!("Getting consent status for user: {}", user_id);

        // Get user from database
        let user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

        // Get analytics consent
        let analytics_consent = self.analytics_service.get_consent(user_id).await?;

        // Get data preferences from user preferences
        let data_prefs = self.get_data_preferences(&user).await?;

        // Get communication preferences from user preferences
        let communication_prefs = self.get_communication_preferences(&user).await?;

        // Get third-party sharing preferences from user preferences
        let third_party_prefs = self.get_third_party_preferences(&user).await?;

        // Determine last updated timestamp
        let last_updated = if let Some(analytics) = &analytics_consent {
            // Use the most recent update timestamp
            if let Some(prefs) = &user.preferences {
                if let Some(updated_at) = prefs.0.get("preferences_updated_at") {
                    if let Some(prefs_updated) = updated_at.as_str().and_then(|s| DateTime::parse_from_rfc3339(s).ok()) {
                        if prefs_updated > analytics.updated_at {
                            prefs_updated.with_timezone(&Utc)
                        } else {
                            analytics.updated_at
                        }
                    } else {
                        analytics.updated_at
                    }
                } else {
                    analytics.updated_at
                }
            } else {
                analytics.updated_at
            }
        } else {
            // If no analytics consent, use current time
            Utc::now()
        };

        // Create analytics consent object
        let analytics = AnalyticsConsent {
            feature_usage: analytics_consent.as_ref().map_or(false, |c| c.feature_usage),
            performance: analytics_consent.as_ref().map_or(false, |c| c.performance),
            error_reporting: analytics_consent.as_ref().map_or(false, |c| c.error_reporting),
            user_interface: analytics_consent.as_ref().map_or(false, |c| c.user_interface),
        };

        // Create data collection consent object
        let data_collection = DataCollectionConsent {
            profile: data_prefs.profile_consent,
            preferences: data_prefs.preferences_consent,
            messages: data_prefs.messages_consent,
            profile_retention_days: data_prefs.profile_retention_days,
            messages_retention_days: data_prefs.messages_retention_days,
            anonymize_profile: data_prefs.anonymize_profile,
            anonymize_messages: data_prefs.anonymize_messages,
        };

        // Create complete consent status
        let consent_status = UserConsentStatus {
            user_id: *user_id,
            last_updated,
            data_collection,
            analytics,
            communication: communication_prefs,
            third_party: third_party_prefs,
        };

        info!("Retrieved consent status for user: {}", user_id);
        Ok(consent_status)
    }

    /// Update the complete consent status for a user
    #[instrument(skip(self, consent), err)]
    pub async fn update_user_consent(&self, user_id: &Uuid, consent: UserConsentStatus) -> Result<()> {
        debug!("Updating consent status for user: {}", user_id);

        // Get user from database
        let mut user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

        // Update analytics consent
        self.analytics_service.update_consent(
            user_id,
            consent.analytics.feature_usage,
            consent.analytics.performance,
            consent.analytics.error_reporting,
            consent.analytics.user_interface,
        ).await?;

        // Update user preferences
        let mut user_prefs = if let Some(prefs) = user.preferences {
            prefs.0
        } else {
            json!({})
        };

        // Update data preferences
        let data_prefs = UserDataPreferences {
            profile_consent: consent.data_collection.profile,
            preferences_consent: consent.data_collection.preferences,
            messages_consent: consent.data_collection.messages,
            feature_usage_consent: consent.analytics.feature_usage,
            performance_consent: consent.analytics.performance,
            error_reporting_consent: consent.analytics.error_reporting,
            ui_analytics_consent: consent.analytics.user_interface,
            profile_retention_days: consent.data_collection.profile_retention_days,
            messages_retention_days: consent.data_collection.messages_retention_days,
            anonymize_profile: consent.data_collection.anonymize_profile,
            anonymize_messages: consent.data_collection.anonymize_messages,
        };

        if let Value::Object(ref mut map) = user_prefs {
            map.insert("data_preferences".to_string(), json!(data_prefs));
            map.insert("communication_preferences".to_string(), json!(consent.communication));
            map.insert("third_party_preferences".to_string(), json!(consent.third_party));
            map.insert("preferences_updated_at".to_string(), json!(Utc::now().to_rfc3339()));
        }

        user.preferences = Some(Json(user_prefs));
        self.db.update_user(&user).await?;

        info!("Updated consent status for user: {}", user_id);
        Ok(())
    }

    /// Get data preferences from user preferences
    async fn get_data_preferences(&self, user: &User) -> Result<UserDataPreferences> {
        // Default preferences
        let mut prefs = UserDataPreferences::default();

        // Extract preferences from user preferences JSON if available
        if let Some(preferences) = &user.preferences {
            if let Some(data_prefs) = preferences.0.get("data_preferences") {
                if let Ok(parsed_prefs) = serde_json::from_value(data_prefs.clone()) {
                    prefs = parsed_prefs;
                }
            }
        }

        Ok(prefs)
    }

    /// Get communication preferences from user preferences
    async fn get_communication_preferences(&self, user: &User) -> Result<CommunicationConsent> {
        // Default preferences
        let mut prefs = CommunicationConsent {
            product_updates: false,
            feature_announcements: false,
            marketing: false,
        };

        // Extract preferences from user preferences JSON if available
        if let Some(preferences) = &user.preferences {
            if let Some(comm_prefs) = preferences.0.get("communication_preferences") {
                if let Ok(parsed_prefs) = serde_json::from_value(comm_prefs.clone()) {
                    prefs = parsed_prefs;
                }
            }
        }

        Ok(prefs)
    }

    /// Get third-party sharing preferences from user preferences
    async fn get_third_party_preferences(&self, user: &User) -> Result<ThirdPartyConsent> {
        // Default preferences
        let mut prefs = ThirdPartyConsent {
            allow_sharing: false,
            approved_parties: Vec::new(),
        };

        // Extract preferences from user preferences JSON if available
        if let Some(preferences) = &user.preferences {
            if let Some(third_party_prefs) = preferences.0.get("third_party_preferences") {
                if let Ok(parsed_prefs) = serde_json::from_value(third_party_prefs.clone()) {
                    prefs = parsed_prefs;
                }
            }
        }

        Ok(prefs)
    }
}

// Tauri command for getting user consent
#[tauri::command]
pub async fn get_user_consent(
    user_id: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<UserConsentStatus, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    let service = ConsentManagementService::new(db.inner().clone());
    
    match service.get_user_consent(&user_id).await {
        Ok(consent) => Ok(consent),
        Err(e) => Err(format!("Failed to get user consent: {}", e)),
    }
}

// Tauri command for updating user consent
#[tauri::command]
pub async fn update_user_consent(
    user_id: String,
    consent: UserConsentStatus,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    let service = ConsentManagementService::new(db.inner().clone());
    
    match service.update_user_consent(&user_id, consent).await {
        Ok(()) => Ok("User consent updated successfully".to_string()),
        Err(e) => Err(format!("Failed to update user consent: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::users::{User, UserStatus, UserRole};
    use sqlx::types::Json;
    use serde_json::json;

    #[tokio::test]
    async fn test_get_user_consent() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = ConsentManagementService::new(db.clone());
        
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
            preferences: Some(Json(json!({
                "data_preferences": {
                    "profile_consent": true,
                    "preferences_consent": true,
                    "messages_consent": false,
                    "feature_usage_consent": true,
                    "performance_consent": false,
                    "error_reporting_consent": true,
                    "ui_analytics_consent": false,
                    "profile_retention_days": 365,
                    "messages_retention_days": 180,
                    "anonymize_profile": true,
                    "anonymize_messages": false
                },
                "communication_preferences": {
                    "product_updates": true,
                    "feature_announcements": true,
                    "marketing": false
                },
                "third_party_preferences": {
                    "allow_sharing": false,
                    "approved_parties": []
                }
            }))),
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            workspace_id: None,
            public_key: vec![],
        };
        
        // Insert the user into the database
        let user_id = user.id;
        db.create_user(&user).await.unwrap();
        
        // Initialize analytics service
        let analytics_service = PrivacyAnalyticsService::new(db.clone());
        analytics_service.initialize().await.unwrap();
        
        // Set analytics consent
        analytics_service.update_consent(
            &user_id,
            true, // feature_usage
            false, // performance
            true, // error_reporting
            false, // user_interface
        ).await.unwrap();
        
        // Get user consent
        let consent = service.get_user_consent(&user_id).await.unwrap();
        
        // Verify consent values
        assert_eq!(consent.user_id, user_id);
        assert_eq!(consent.data_collection.profile, true);
        assert_eq!(consent.data_collection.preferences, true);
        assert_eq!(consent.data_collection.messages, false);
        assert_eq!(consent.analytics.feature_usage, true);
        assert_eq!(consent.analytics.performance, false);
        assert_eq!(consent.analytics.error_reporting, true);
        assert_eq!(consent.analytics.user_interface, false);
        assert_eq!(consent.communication.product_updates, true);
        assert_eq!(consent.communication.feature_announcements, true);
        assert_eq!(consent.communication.marketing, false);
        assert_eq!(consent.third_party.allow_sharing, false);
        assert_eq!(consent.third_party.approved_parties.len(), 0);
    }

    #[tokio::test]
    async fn test_update_user_consent() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = ConsentManagementService::new(db.clone());
        
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
            preferences: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            workspace_id: None,
            public_key: vec![],
        };
        
        // Insert the user into the database
        let user_id = user.id;
        db.create_user(&user).await.unwrap();
        
        // Initialize analytics service
        let analytics_service = PrivacyAnalyticsService::new(db.clone());
        analytics_service.initialize().await.unwrap();
        
        // Create new consent status
        let new_consent = UserConsentStatus {
            user_id,
            last_updated: Utc::now(),
            data_collection: DataCollectionConsent {
                profile: true,
                preferences: true,
                messages: false,
                profile_retention_days: 365,
                messages_retention_days: 180,
                anonymize_profile: true,
                anonymize_messages: false,
            },
            analytics: AnalyticsConsent {
                feature_usage: true,
                performance: false,
                error_reporting: true,
                user_interface: false,
            },
            communication: CommunicationConsent {
                product_updates: true,
                feature_announcements: true,
                marketing: false,
            },
            third_party: ThirdPartyConsent {
                allow_sharing: false,
                approved_parties: vec![],
            },
        };
        
        // Update user consent
        service.update_user_consent(&user_id, new_consent).await.unwrap();
        
        // Get updated user consent
        let updated_consent = service.get_user_consent(&user_id).await.unwrap();
        
        // Verify updated consent values
        assert_eq!(updated_consent.data_collection.profile, true);
        assert_eq!(updated_consent.data_collection.preferences, true);
        assert_eq!(updated_consent.data_collection.messages, false);
        assert_eq!(updated_consent.analytics.feature_usage, true);
        assert_eq!(updated_consent.analytics.performance, false);
        assert_eq!(updated_consent.analytics.error_reporting, true);
        assert_eq!(updated_consent.analytics.user_interface, false);
        assert_eq!(updated_consent.communication.product_updates, true);
        assert_eq!(updated_consent.communication.feature_announcements, true);
        assert_eq!(updated_consent.communication.marketing, false);
        assert_eq!(updated_consent.third_party.allow_sharing, false);
        
        // Verify analytics consent was updated
        let analytics_consent = analytics_service.get_consent(&user_id).await.unwrap().unwrap();
        assert_eq!(analytics_consent.feature_usage, true);
        assert_eq!(analytics_consent.performance, false);
        assert_eq!(analytics_consent.error_reporting, true);
        assert_eq!(analytics_consent.user_interface, false);
        
        // Verify user preferences were updated
        let updated_user = db.get_user_by_id(&user_id).await.unwrap().unwrap();
        let preferences = updated_user.preferences.unwrap().0;
        
        assert!(preferences.get("data_preferences").is_some());
        assert!(preferences.get("communication_preferences").is_some());
        assert!(preferences.get("third_party_preferences").is_some());
    }
}