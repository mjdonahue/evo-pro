//! Transparent Data Usage Reporting Service
//!
//! This module implements a transparent data usage reporting system that allows
//! users to see what data is being collected about them and how it's being used.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::Json;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;
use crate::services::privacy_analytics::{AnalyticsEventType, PrivacyAnalyticsService};

/// Represents a data usage category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DataUsageCategory {
    /// User profile data
    Profile,
    /// User preferences
    Preferences,
    /// Messages and conversations
    Messages,
    /// Feature usage analytics
    FeatureUsage,
    /// Performance metrics
    Performance,
    /// Error reports
    ErrorReports,
    /// User interface interactions
    UserInterface,
}

/// Represents a data usage report for a specific user
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataUsageReport {
    /// User ID
    pub user_id: Uuid,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report period start
    pub period_start: DateTime<Utc>,
    /// Report period end
    pub period_end: DateTime<Utc>,
    /// Data usage by category
    pub usage_by_category: Vec<CategoryUsage>,
    /// Data sharing summary
    pub data_sharing: DataSharingSummary,
    /// Data retention summary
    pub retention_summary: RetentionSummary,
    /// User consent status
    pub consent_status: ConsentStatus,
}

/// Represents usage statistics for a specific data category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryUsage {
    /// Data category
    pub category: DataUsageCategory,
    /// Number of data points collected
    pub data_points_count: i64,
    /// Last collection timestamp
    pub last_collected: Option<DateTime<Utc>>,
    /// Description of what data is collected
    pub data_description: String,
    /// How the data is used
    pub usage_description: String,
    /// Whether the user has consented to this category
    pub has_consent: bool,
}

/// Represents a summary of data sharing practices
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataSharingSummary {
    /// Whether data is shared with third parties
    pub shared_with_third_parties: bool,
    /// List of third parties data is shared with
    pub third_parties: Vec<String>,
    /// Purpose of data sharing
    pub sharing_purpose: String,
    /// Whether data is anonymized before sharing
    pub anonymized_before_sharing: bool,
}

/// Represents a summary of data retention policies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionSummary {
    /// Retention period for each category
    pub retention_periods: Vec<CategoryRetention>,
    /// When data will be automatically deleted
    pub next_scheduled_deletion: Option<DateTime<Utc>>,
    /// Whether the user can request manual deletion
    pub can_request_deletion: bool,
}

/// Represents retention policy for a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRetention {
    /// Data category
    pub category: DataUsageCategory,
    /// Retention period in days
    pub retention_days: i64,
    /// Whether data is anonymized after retention period
    pub anonymize_after_retention: bool,
}

/// Represents the user's consent status for different data categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsentStatus {
    /// Whether the user has consented to profile data collection
    pub profile: bool,
    /// Whether the user has consented to preferences data collection
    pub preferences: bool,
    /// Whether the user has consented to messages data collection
    pub messages: bool,
    /// Whether the user has consented to feature usage analytics
    pub feature_usage: bool,
    /// Whether the user has consented to performance metrics
    pub performance: bool,
    /// Whether the user has consented to error reports
    pub error_reports: bool,
    /// Whether the user has consented to user interface analytics
    pub user_interface: bool,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Service for transparent data usage reporting
pub struct DataUsageReportingService {
    /// Database manager
    db: DatabaseManager,
    /// Privacy analytics service
    analytics_service: PrivacyAnalyticsService,
}

impl DataUsageReportingService {
    /// Create a new data usage reporting service
    pub fn new(db: DatabaseManager) -> Self {
        let analytics_service = PrivacyAnalyticsService::new(db.clone());
        Self { db, analytics_service }
    }

    /// Generate a data usage report for a specific user
    #[instrument(skip(self), err)]
    pub async fn generate_report(
        &self,
        user_id: &Uuid,
        period_start: Option<DateTime<Utc>>,
        period_end: Option<DateTime<Utc>>,
    ) -> Result<DataUsageReport> {
        debug!("Generating data usage report for user: {}", user_id);

        // Use default period if not specified (last 30 days)
        let end = period_end.unwrap_or_else(Utc::now);
        let start = period_start.unwrap_or_else(|| end - chrono::Duration::days(30));

        // Get user consent status
        let consent = self.get_user_consent_status(user_id).await?;

        // Generate usage by category
        let usage_by_category = self.generate_category_usage(user_id, start, end).await?;

        // Generate data sharing summary
        let data_sharing = self.generate_data_sharing_summary().await?;

        // Generate retention summary
        let retention_summary = self.generate_retention_summary(user_id).await?;

        // Create the report
        let report = DataUsageReport {
            user_id: *user_id,
            generated_at: Utc::now(),
            period_start: start,
            period_end: end,
            usage_by_category,
            data_sharing,
            retention_summary,
            consent_status: consent,
        };

        info!("Generated data usage report for user: {}", user_id);
        Ok(report)
    }

    /// Get the user's consent status
    async fn get_user_consent_status(&self, user_id: &Uuid) -> Result<ConsentStatus> {
        // Get analytics consent from the analytics service
        let analytics_consent = self.analytics_service.get_consent(user_id).await?;

        // Get data retention preferences
        let retention_prefs = self.get_retention_preferences(user_id).await?;

        // Default values if no consent record exists
        let (feature_usage, performance, error_reports, user_interface, last_updated) = 
            if let Some(consent) = analytics_consent {
                (
                    consent.feature_usage,
                    consent.performance,
                    consent.error_reporting,
                    consent.user_interface,
                    consent.updated_at,
                )
            } else {
                (false, false, false, false, Utc::now())
            };

        // Create consent status
        let consent_status = ConsentStatus {
            profile: retention_prefs.profile_consent,
            preferences: retention_prefs.preferences_consent,
            messages: retention_prefs.messages_consent,
            feature_usage,
            performance,
            error_reports,
            user_interface,
            last_updated,
        };

        Ok(consent_status)
    }

    /// Get user's data retention preferences
    async fn get_retention_preferences(&self, user_id: &Uuid) -> Result<UserDataPreferences> {
        // Get user from database
        let user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

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

    /// Generate usage statistics for each data category
    async fn generate_category_usage(
        &self,
        user_id: &Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<CategoryUsage>> {
        let mut categories = Vec::new();

        // Get analytics events for the user
        let analytics_events = self.analytics_service.get_events(None, Some(start), Some(end), None).await?;

        // Filter events by session ID (in a real implementation, we would have a way to link sessions to users)
        // For now, we'll use a placeholder approach
        let user_sessions = self.get_user_sessions(user_id).await?;
        let user_events: Vec<_> = analytics_events.into_iter()
            .filter(|event| user_sessions.contains(&event.session_id))
            .collect();

        // Get consent status
        let consent = self.get_user_consent_status(user_id).await?;

        // Profile data usage
        let profile_count = self.count_profile_data_accesses(user_id, start, end).await?;
        categories.push(CategoryUsage {
            category: DataUsageCategory::Profile,
            data_points_count: profile_count,
            last_collected: self.get_last_profile_access(user_id).await?,
            data_description: "Basic profile information including name, email, and preferences".to_string(),
            usage_description: "Used to personalize your experience and provide account functionality".to_string(),
            has_consent: consent.profile,
        });

        // Messages data usage
        let messages_count = self.count_messages(user_id, start, end).await?;
        categories.push(CategoryUsage {
            category: DataUsageCategory::Messages,
            data_points_count: messages_count,
            last_collected: self.get_last_message_timestamp(user_id).await?,
            data_description: "Message content and metadata".to_string(),
            usage_description: "Used to provide messaging functionality and conversation history".to_string(),
            has_consent: consent.messages,
        });

        // Feature usage analytics
        let feature_events: Vec<_> = user_events.iter()
            .filter(|e| e.event_type == AnalyticsEventType::FeatureUsage)
            .collect();
        categories.push(CategoryUsage {
            category: DataUsageCategory::FeatureUsage,
            data_points_count: feature_events.len() as i64,
            last_collected: feature_events.first().map(|e| e.timestamp),
            data_description: "Which features you use and how often".to_string(),
            usage_description: "Used to improve features and understand user preferences".to_string(),
            has_consent: consent.feature_usage,
        });

        // Performance metrics
        let performance_events: Vec<_> = user_events.iter()
            .filter(|e| e.event_type == AnalyticsEventType::Performance)
            .collect();
        categories.push(CategoryUsage {
            category: DataUsageCategory::Performance,
            data_points_count: performance_events.len() as i64,
            last_collected: performance_events.first().map(|e| e.timestamp),
            data_description: "App performance metrics like load times and resource usage".to_string(),
            usage_description: "Used to identify and fix performance issues".to_string(),
            has_consent: consent.performance,
        });

        // Error reports
        let error_events: Vec<_> = user_events.iter()
            .filter(|e| e.event_type == AnalyticsEventType::Error)
            .collect();
        categories.push(CategoryUsage {
            category: DataUsageCategory::ErrorReports,
            data_points_count: error_events.len() as i64,
            last_collected: error_events.first().map(|e| e.timestamp),
            data_description: "Error details when the app encounters problems".to_string(),
            usage_description: "Used to identify and fix bugs and improve stability".to_string(),
            has_consent: consent.error_reports,
        });

        // UI interactions
        let ui_events: Vec<_> = user_events.iter()
            .filter(|e| e.event_type == AnalyticsEventType::UserInterface)
            .collect();
        categories.push(CategoryUsage {
            category: DataUsageCategory::UserInterface,
            data_points_count: ui_events.len() as i64,
            last_collected: ui_events.first().map(|e| e.timestamp),
            data_description: "How you interact with the user interface".to_string(),
            usage_description: "Used to improve usability and user experience".to_string(),
            has_consent: consent.user_interface,
        });

        Ok(categories)
    }

    /// Get user sessions (placeholder implementation)
    async fn get_user_sessions(&self, user_id: &Uuid) -> Result<Vec<String>> {
        // In a real implementation, we would query a sessions table
        // For now, we'll use a placeholder approach
        Ok(vec![format!("user-session-{}", user_id)])
    }

    /// Count profile data accesses (placeholder implementation)
    async fn count_profile_data_accesses(&self, _user_id: &Uuid, _start: DateTime<Utc>, _end: DateTime<Utc>) -> Result<i64> {
        // In a real implementation, we would track and count profile data accesses
        // For now, we'll use a placeholder value
        Ok(5)
    }

    /// Get last profile access timestamp (placeholder implementation)
    async fn get_last_profile_access(&self, _user_id: &Uuid) -> Result<Option<DateTime<Utc>>> {
        // In a real implementation, we would track and return the last profile access timestamp
        // For now, we'll use a placeholder value
        Ok(Some(Utc::now() - chrono::Duration::hours(12)))
    }

    /// Count messages in the given period
    async fn count_messages(&self, user_id: &Uuid, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<i64> {
        let count = sqlx::query!(
            "SELECT COUNT(*) as count FROM messages 
             WHERE sender_id = ? AND created_at >= ? AND created_at <= ?",
            user_id, start, end
        )
        .fetch_one(&self.db.pool)
        .await?
        .count;

        Ok(count)
    }

    /// Get last message timestamp
    async fn get_last_message_timestamp(&self, user_id: &Uuid) -> Result<Option<DateTime<Utc>>> {
        let result = sqlx::query!(
            "SELECT created_at FROM messages 
             WHERE sender_id = ? 
             ORDER BY created_at DESC LIMIT 1",
            user_id
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(result.map(|r| r.created_at))
    }

    /// Generate data sharing summary
    async fn generate_data_sharing_summary(&self) -> Result<DataSharingSummary> {
        // In a real implementation, this would be based on actual data sharing practices
        // For now, we'll use a placeholder implementation
        let summary = DataSharingSummary {
            shared_with_third_parties: false,
            third_parties: Vec::new(),
            sharing_purpose: "Your data is not shared with any third parties".to_string(),
            anonymized_before_sharing: true,
        };

        Ok(summary)
    }

    /// Generate retention summary
    async fn generate_retention_summary(&self, user_id: &Uuid) -> Result<RetentionSummary> {
        // Get user's data retention preferences
        let prefs = self.get_retention_preferences(user_id).await?;

        // Create category retention objects
        let mut retention_periods = Vec::new();

        retention_periods.push(CategoryRetention {
            category: DataUsageCategory::Profile,
            retention_days: prefs.profile_retention_days,
            anonymize_after_retention: prefs.anonymize_profile,
        });

        retention_periods.push(CategoryRetention {
            category: DataUsageCategory::Messages,
            retention_days: prefs.messages_retention_days,
            anonymize_after_retention: prefs.anonymize_messages,
        });

        retention_periods.push(CategoryRetention {
            category: DataUsageCategory::FeatureUsage,
            retention_days: 90, // Default for analytics data
            anonymize_after_retention: true,
        });

        retention_periods.push(CategoryRetention {
            category: DataUsageCategory::Performance,
            retention_days: 90, // Default for analytics data
            anonymize_after_retention: true,
        });

        retention_periods.push(CategoryRetention {
            category: DataUsageCategory::ErrorReports,
            retention_days: 90, // Default for analytics data
            anonymize_after_retention: true,
        });

        retention_periods.push(CategoryRetention {
            category: DataUsageCategory::UserInterface,
            retention_days: 90, // Default for analytics data
            anonymize_after_retention: true,
        });

        // Calculate next scheduled deletion
        // In a real implementation, this would be based on actual scheduled deletions
        let next_deletion = Utc::now() + chrono::Duration::days(prefs.messages_retention_days);

        let summary = RetentionSummary {
            retention_periods,
            next_scheduled_deletion: Some(next_deletion),
            can_request_deletion: true,
        };

        Ok(summary)
    }

    /// Update user data preferences
    #[instrument(skip(self), err)]
    pub async fn update_user_data_preferences(
        &self,
        user_id: &Uuid,
        preferences: UserDataPreferences,
    ) -> Result<()> {
        // Get user from database
        let mut user = match self.db.get_user_by_id(user_id).await? {
            Some(user) => user,
            None => return Err(AppError::NotFoundError(format!("User with ID {} not found", user_id))),
        };

        // Update user preferences
        let mut user_prefs = if let Some(prefs) = user.preferences {
            prefs.0
        } else {
            json!({})
        };

        if let Value::Object(ref mut map) = user_prefs {
            map.insert("data_preferences".to_string(), json!(preferences));
        }

        user.preferences = Some(Json(user_prefs));
        self.db.update_user(&user).await?;

        // Update analytics consent
        self.analytics_service.update_consent(
            user_id,
            preferences.feature_usage_consent,
            preferences.performance_consent,
            preferences.error_reporting_consent,
            preferences.ui_analytics_consent,
        ).await?;

        info!("Updated data preferences for user: {}", user_id);
        Ok(())
    }
}

/// User data preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataPreferences {
    /// Whether the user has consented to profile data collection
    pub profile_consent: bool,
    /// Whether the user has consented to preferences data collection
    pub preferences_consent: bool,
    /// Whether the user has consented to messages data collection
    pub messages_consent: bool,
    /// Whether the user has consented to feature usage analytics
    pub feature_usage_consent: bool,
    /// Whether the user has consented to performance analytics
    pub performance_consent: bool,
    /// Whether the user has consented to error reporting
    pub error_reporting_consent: bool,
    /// Whether the user has consented to UI analytics
    pub ui_analytics_consent: bool,
    /// Retention period for profile data in days
    pub profile_retention_days: i64,
    /// Retention period for messages in days
    pub messages_retention_days: i64,
    /// Whether to anonymize profile data after retention period
    pub anonymize_profile: bool,
    /// Whether to anonymize messages after retention period
    pub anonymize_messages: bool,
}

impl Default for UserDataPreferences {
    fn default() -> Self {
        Self {
            profile_consent: true,
            preferences_consent: true,
            messages_consent: true,
            feature_usage_consent: false,
            performance_consent: false,
            error_reporting_consent: true,
            ui_analytics_consent: false,
            profile_retention_days: 730, // 2 years
            messages_retention_days: 365, // 1 year
            anonymize_profile: true,
            anonymize_messages: false,
        }
    }
}

// Tauri command for generating a data usage report
#[tauri::command]
pub async fn generate_data_usage_report(
    user_id: String,
    start_date: Option<String>,
    end_date: Option<String>,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<DataUsageReport, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    // Parse dates if provided
    let start_date = start_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
    let end_date = end_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
    
    let service = DataUsageReportingService::new(db.inner().clone());
    
    match service.generate_report(&user_id, start_date, end_date).await {
        Ok(report) => Ok(report),
        Err(e) => Err(format!("Failed to generate data usage report: {}", e)),
    }
}

// Tauri command for updating user data preferences
#[tauri::command]
pub async fn update_data_preferences(
    user_id: String,
    preferences: UserDataPreferences,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let user_id = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;
    
    let service = DataUsageReportingService::new(db.inner().clone());
    
    match service.update_user_data_preferences(&user_id, preferences).await {
        Ok(()) => Ok("Data preferences updated successfully".to_string()),
        Err(e) => Err(format!("Failed to update data preferences: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::users::{User, UserStatus, UserRole};
    use sqlx::types::Json;
    use serde_json::json;

    #[tokio::test]
    async fn test_data_usage_report_generation() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = DataUsageReportingService::new(db.clone());
        
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
                    "messages_consent": true,
                    "feature_usage_consent": true,
                    "performance_consent": false,
                    "error_reporting_consent": true,
                    "ui_analytics_consent": false,
                    "profile_retention_days": 730,
                    "messages_retention_days": 365,
                    "anonymize_profile": true,
                    "anonymize_messages": false
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
        
        // Set user consent for analytics
        analytics_service.update_consent(
            &user_id,
            true, // feature_usage
            false, // performance
            true, // error_reporting
            false, // user_interface
        ).await.unwrap();
        
        // Generate a data usage report
        let report = service.generate_report(&user_id, None, None).await.unwrap();
        
        // Verify the report
        assert_eq!(report.user_id, user_id);
        assert_eq!(report.consent_status.feature_usage, true);
        assert_eq!(report.consent_status.performance, false);
        assert_eq!(report.consent_status.error_reports, true);
        assert_eq!(report.consent_status.user_interface, false);
        
        // Verify data sharing summary
        assert_eq!(report.data_sharing.shared_with_third_parties, false);
        
        // Verify retention summary
        assert!(report.retention_summary.can_request_deletion);
        assert_eq!(report.retention_summary.retention_periods.len(), 6);
    }
    
    #[tokio::test]
    async fn test_update_data_preferences() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = DataUsageReportingService::new(db.clone());
        
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
        
        // Update data preferences
        let prefs = UserDataPreferences {
            profile_consent: true,
            preferences_consent: true,
            messages_consent: false,
            feature_usage_consent: true,
            performance_consent: true,
            error_reporting_consent: false,
            ui_analytics_consent: true,
            profile_retention_days: 365,
            messages_retention_days: 180,
            anonymize_profile: false,
            anonymize_messages: true,
        };
        
        service.update_user_data_preferences(&user_id, prefs.clone()).await.unwrap();
        
        // Verify preferences were updated
        let updated_user = db.get_user_by_id(&user_id).await.unwrap().unwrap();
        let updated_prefs = updated_user.preferences.unwrap().0;
        
        assert!(updated_prefs.get("data_preferences").is_some());
        
        // Verify analytics consent was updated
        let consent = analytics_service.get_consent(&user_id).await.unwrap().unwrap();
        assert_eq!(consent.feature_usage, true);
        assert_eq!(consent.performance, true);
        assert_eq!(consent.error_reporting, false);
        assert_eq!(consent.user_interface, true);
        
        // Generate a report and verify the updated preferences
        let report = service.generate_report(&user_id, None, None).await.unwrap();
        assert_eq!(report.consent_status.messages, false);
        assert_eq!(report.consent_status.feature_usage, true);
        assert_eq!(report.consent_status.performance, true);
        assert_eq!(report.consent_status.error_reports, false);
        assert_eq!(report.consent_status.user_interface, true);
    }
}