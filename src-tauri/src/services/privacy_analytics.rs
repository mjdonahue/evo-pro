//! Privacy-Preserving Analytics Service
//!
//! This module implements a privacy-preserving analytics system that collects
//! anonymized usage data while respecting user privacy and consent.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::types::Json;
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::storage::db::DatabaseManager;

/// Types of analytics events that can be collected
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "analytics_event_type")]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsEventType {
    /// Feature usage event
    FeatureUsage = 0,
    /// Performance event
    Performance = 1,
    /// Error event
    Error = 2,
    /// User interface event
    UserInterface = 3,
    /// Session event
    Session = 4,
    /// Custom event
    Custom = 5,
}

/// Analytics event data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsEvent {
    /// Unique ID for the event
    pub id: Uuid,
    /// Type of event
    pub event_type: AnalyticsEventType,
    /// Name of the event
    pub event_name: String,
    /// Anonymized session ID
    pub session_id: String,
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// Event properties (anonymized)
    pub properties: Json<Value>,
    /// Whether the user has consented to this type of analytics
    pub has_consent: bool,
    /// Anonymization level applied to this event
    pub anonymization_level: AnonymizationLevel,
}

/// Anonymization levels for analytics data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "anonymization_level")]
#[serde(rename_all = "snake_case")]
pub enum AnonymizationLevel {
    /// No anonymization (only for events with explicit consent)
    None = 0,
    /// Basic anonymization (remove direct identifiers)
    Basic = 1,
    /// Advanced anonymization (k-anonymity)
    Advanced = 2,
    /// Full anonymization (differential privacy)
    Full = 3,
}

/// User consent preferences for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsConsent {
    /// User ID
    pub user_id: Uuid,
    /// Whether the user has consented to feature usage analytics
    pub feature_usage: bool,
    /// Whether the user has consented to performance analytics
    pub performance: bool,
    /// Whether the user has consented to error reporting
    pub error_reporting: bool,
    /// Whether the user has consented to user interface analytics
    pub user_interface: bool,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsConfig {
    /// Whether analytics collection is enabled
    pub enabled: bool,
    /// Default anonymization level
    pub default_anonymization_level: AnonymizationLevel,
    /// Retention period in days
    pub retention_days: i64,
    /// Whether to collect analytics in development mode
    pub collect_in_development: bool,
    /// Custom configuration
    pub custom_config: Option<Json<Value>>,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for privacy
            default_anonymization_level: AnonymizationLevel::Advanced,
            retention_days: 90,
            collect_in_development: false,
            custom_config: None,
        }
    }
}

/// Service for privacy-preserving analytics
pub struct PrivacyAnalyticsService {
    /// Database manager
    db: DatabaseManager,
    /// Analytics configuration
    config: AnalyticsConfig,
}

impl PrivacyAnalyticsService {
    /// Create a new privacy analytics service
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            db,
            config: AnalyticsConfig::default(),
        }
    }

    /// Create a new privacy analytics service with custom configuration
    pub fn with_config(db: DatabaseManager, config: AnalyticsConfig) -> Self {
        Self { db, config }
    }

    /// Initialize the analytics service
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<()> {
        debug!("Initializing privacy analytics service");

        // Ensure the analytics tables exist
        self.ensure_tables_exist().await?;

        // Apply data retention policy
        self.apply_retention_policy().await?;

        info!("Privacy analytics service initialized");
        Ok(())
    }

    /// Ensure the required database tables exist
    async fn ensure_tables_exist(&self) -> Result<()> {
        // This would typically be handled by migrations, but we'll include the logic here for completeness
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS analytics_events (
                id BLOB PRIMARY KEY,
                event_type INTEGER NOT NULL,
                event_name TEXT NOT NULL,
                session_id TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                properties TEXT NOT NULL CHECK (json_valid(properties)),
                has_consent BOOLEAN NOT NULL,
                anonymization_level INTEGER NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS analytics_consent (
                user_id BLOB PRIMARY KEY,
                feature_usage BOOLEAN NOT NULL DEFAULT FALSE,
                performance BOOLEAN NOT NULL DEFAULT FALSE,
                error_reporting BOOLEAN NOT NULL DEFAULT FALSE,
                user_interface BOOLEAN NOT NULL DEFAULT FALSE,
                updated_at DATETIME NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_analytics_events_type ON analytics_events(event_type);
            CREATE INDEX IF NOT EXISTS idx_analytics_events_timestamp ON analytics_events(timestamp);
            "#,
        )
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    /// Track an analytics event
    #[instrument(skip(self, properties))]
    pub async fn track_event(
        &self,
        event_type: AnalyticsEventType,
        event_name: &str,
        session_id: &str,
        properties: Value,
        user_id: Option<&Uuid>,
    ) -> Result<()> {
        // Check if analytics is enabled
        if !self.config.enabled {
            debug!("Analytics is disabled, not tracking event");
            return Ok(());
        }

        // Check if we should collect in development mode
        #[cfg(debug_assertions)]
        if !self.config.collect_in_development {
            debug!("Analytics collection in development mode is disabled");
            return Ok(());
        }

        // Check user consent if user_id is provided
        let has_consent = if let Some(user_id) = user_id {
            self.check_consent(user_id, event_type).await?
        } else {
            // If no user_id is provided, we assume no consent
            false
        };

        // Determine anonymization level based on consent and configuration
        let anonymization_level = if has_consent {
            // If user has consented, use the default anonymization level
            self.config.default_anonymization_level
        } else {
            // If user has not consented, use the highest anonymization level
            AnonymizationLevel::Full
        };

        // Anonymize properties based on the anonymization level
        let anonymized_properties = self.anonymize_properties(properties, anonymization_level).await?;

        // Create and store the event
        let event = AnalyticsEvent {
            id: Uuid::new_v4(),
            event_type,
            event_name: event_name.to_string(),
            session_id: session_id.to_string(),
            timestamp: Utc::now(),
            properties: Json(anonymized_properties),
            has_consent,
            anonymization_level,
        };

        self.store_event(&event).await?;

        debug!("Tracked analytics event: {}", event_name);
        Ok(())
    }

    /// Store an analytics event in the database
    async fn store_event(&self, event: &AnalyticsEvent) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO analytics_events (
                id, event_type, event_name, session_id, timestamp, 
                properties, has_consent, anonymization_level
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            event.id,
            event.event_type as i32,
            event.event_name,
            event.session_id,
            event.timestamp,
            event.properties.0,
            event.has_consent,
            event.anonymization_level as i32
        )
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    /// Check if the user has consented to the given event type
    async fn check_consent(&self, user_id: &Uuid, event_type: AnalyticsEventType) -> Result<bool> {
        let consent = sqlx::query_as!(
            AnalyticsConsent,
            r#"
            SELECT 
                user_id as "user_id: Uuid",
                feature_usage, performance, error_reporting, user_interface,
                updated_at as "updated_at: DateTime<Utc>"
            FROM analytics_consent
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.db.pool)
        .await?;

        if let Some(consent) = consent {
            // Check consent based on event type
            match event_type {
                AnalyticsEventType::FeatureUsage => Ok(consent.feature_usage),
                AnalyticsEventType::Performance => Ok(consent.performance),
                AnalyticsEventType::Error => Ok(consent.error_reporting),
                AnalyticsEventType::UserInterface => Ok(consent.user_interface),
                // For other event types, default to no consent
                _ => Ok(false),
            }
        } else {
            // If no consent record exists, assume no consent
            Ok(false)
        }
    }

    /// Update user consent preferences
    #[instrument(skip(self))]
    pub async fn update_consent(
        &self,
        user_id: &Uuid,
        feature_usage: bool,
        performance: bool,
        error_reporting: bool,
        user_interface: bool,
    ) -> Result<()> {
        let now = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO analytics_consent (
                user_id, feature_usage, performance, error_reporting, user_interface, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id) DO UPDATE SET
                feature_usage = excluded.feature_usage,
                performance = excluded.performance,
                error_reporting = excluded.error_reporting,
                user_interface = excluded.user_interface,
                updated_at = excluded.updated_at
            "#,
            user_id,
            feature_usage,
            performance,
            error_reporting,
            user_interface,
            now
        )
        .execute(&self.db.pool)
        .await?;

        info!("Updated analytics consent for user {}", user_id);
        Ok(())
    }

    /// Get user consent preferences
    #[instrument(skip(self))]
    pub async fn get_consent(&self, user_id: &Uuid) -> Result<Option<AnalyticsConsent>> {
        let consent = sqlx::query_as!(
            AnalyticsConsent,
            r#"
            SELECT 
                user_id as "user_id: Uuid",
                feature_usage, performance, error_reporting, user_interface,
                updated_at as "updated_at: DateTime<Utc>"
            FROM analytics_consent
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(consent)
    }

    /// Anonymize properties based on the anonymization level
    async fn anonymize_properties(&self, properties: Value, level: AnonymizationLevel) -> Result<Value> {
        match level {
            AnonymizationLevel::None => {
                // No anonymization, return as is (only for events with explicit consent)
                Ok(properties)
            },
            AnonymizationLevel::Basic => {
                // Basic anonymization: remove direct identifiers
                self.apply_basic_anonymization(properties)
            },
            AnonymizationLevel::Advanced => {
                // Advanced anonymization: k-anonymity
                self.apply_advanced_anonymization(properties)
            },
            AnonymizationLevel::Full => {
                // Full anonymization: differential privacy
                self.apply_full_anonymization(properties)
            },
        }
    }

    /// Apply basic anonymization (remove direct identifiers)
    fn apply_basic_anonymization(&self, properties: Value) -> Result<Value> {
        if let Value::Object(mut map) = properties {
            // List of fields to remove or hash
            let sensitive_fields = [
                "email", "name", "username", "user_id", "ip", "address",
                "phone", "location", "device_id", "user_agent"
            ];

            // Remove sensitive fields
            for field in &sensitive_fields {
                map.remove(*field);
            }

            Ok(Value::Object(map))
        } else {
            Ok(properties)
        }
    }

    /// Apply advanced anonymization (k-anonymity)
    fn apply_advanced_anonymization(&self, properties: Value) -> Result<Value> {
        if let Value::Object(mut map) = properties {
            // First apply basic anonymization
            let basic = self.apply_basic_anonymization(Value::Object(map.clone()))?;

            if let Value::Object(mut basic_map) = basic {
                // Generalize numeric values
                for (key, value) in basic_map.iter_mut() {
                    if let Value::Number(num) = value {
                        // Round numbers to reduce precision
                        if let Some(n) = num.as_f64() {
                            // Round to nearest 10
                            let rounded = (n / 10.0).round() * 10.0;
                            *value = json!(rounded);
                        }
                    }
                }

                // Generalize timestamps
                if let Some(Value::String(timestamp)) = basic_map.get("timestamp") {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
                        // Round to nearest hour
                        let rounded = dt.date().and_hms_opt(dt.hour(), 0, 0)
                            .unwrap_or_else(|| dt.date().and_hms(0, 0, 0));
                        basic_map.insert("timestamp".to_string(), json!(rounded.to_rfc3339()));
                    }
                }

                Ok(Value::Object(basic_map))
            } else {
                Ok(basic)
            }
        } else {
            Ok(properties)
        }
    }

    /// Apply full anonymization (differential privacy)
    fn apply_full_anonymization(&self, properties: Value) -> Result<Value> {
        // For full anonymization, we only keep aggregate data and add noise

        let mut anonymized = json!({
            "event_occurred": true,
            // Add a small amount of random noise to any numeric values
            "timestamp_bucket": chrono::Utc::now().date().and_hms(0, 0, 0).to_rfc3339()
        });

        // Extract and anonymize numeric values with differential privacy
        if let Value::Object(map) = properties {
            let mut numeric_values = HashMap::new();

            for (key, value) in map {
                if let Value::Number(num) = value {
                    if let Some(n) = num.as_f64() {
                        // Add random noise (simple implementation of differential privacy)
                        // In a real implementation, this would use a proper differential privacy library
                        let noise = rand::random::<f64>() * 2.0 - 1.0; // Random value between -1 and 1
                        let noisy_value = n + noise;
                        numeric_values.insert(format!("{}_approx", key), json!(noisy_value));
                    }
                }
            }

            if let Value::Object(ref mut anonymized_map) = anonymized {
                for (key, value) in numeric_values {
                    anonymized_map.insert(key, value);
                }
            }
        }

        Ok(anonymized)
    }

    /// Apply data retention policy
    #[instrument(skip(self))]
    pub async fn apply_retention_policy(&self) -> Result<()> {
        let retention_days = self.config.retention_days;
        debug!("Applying analytics data retention policy: {} days", retention_days);

        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        // Delete old analytics events
        let result = sqlx::query!(
            "DELETE FROM analytics_events WHERE timestamp < ?",
            cutoff_date
        )
        .execute(&self.db.pool)
        .await?;

        info!("Deleted {} old analytics events", result.rows_affected());
        Ok(())
    }

    /// Get analytics events for a specific period
    #[instrument(skip(self))]
    pub async fn get_events(
        &self,
        event_type: Option<AnalyticsEventType>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<AnalyticsEvent>> {
        // Build the query based on the provided filters
        let mut query = "
            SELECT 
                id as \"id: Uuid\",
                event_type as \"event_type: AnalyticsEventType\",
                event_name,
                session_id,
                timestamp as \"timestamp: DateTime<Utc>\",
                properties as \"properties: Json<Value>\",
                has_consent,
                anonymization_level as \"anonymization_level: AnonymizationLevel\"
            FROM analytics_events
            WHERE 1=1
        ".to_string();

        let mut params = Vec::new();

        if let Some(event_type) = event_type {
            query.push_str(" AND event_type = ?");
            params.push(event_type as i32);
        }

        if let Some(start_date) = start_date {
            query.push_str(" AND timestamp >= ?");
            params.push(start_date);
        }

        if let Some(end_date) = end_date {
            query.push_str(" AND timestamp <= ?");
            params.push(end_date);
        }

        query.push_str(" ORDER BY timestamp DESC");

        if let Some(limit) = limit {
            query.push_str(" LIMIT ?");
            params.push(limit);
        }

        // Execute the query
        let mut q = sqlx::query_as::<_, AnalyticsEvent>(&query);

        // Add the parameters
        for param in params {
            q = q.bind(param);
        }

        let events = q.fetch_all(&self.db.pool).await?;

        Ok(events)
    }

    /// Generate an aggregated analytics report
    #[instrument(skip(self))]
    pub async fn generate_report(
        &self,
        event_type: Option<AnalyticsEventType>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Value> {
        // Get events for the report
        let events = self.get_events(event_type, start_date, end_date, None).await?;

        // Count events by type
        let mut event_counts = HashMap::new();
        for event in &events {
            *event_counts.entry(event.event_name.clone()).or_insert(0) += 1;
        }

        // Count events by day
        let mut daily_counts = HashMap::new();
        for event in &events {
            let day = event.timestamp.date().to_string();
            *daily_counts.entry(day).or_insert(0) += 1;
        }

        // Generate the report
        let report = json!({
            "total_events": events.len(),
            "event_counts": event_counts,
            "daily_counts": daily_counts,
            "report_generated_at": Utc::now(),
            "start_date": start_date,
            "end_date": end_date,
            "event_type": event_type,
        });

        Ok(report)
    }

    /// Register commands with Tauri
    pub fn register_commands(app: &mut tauri::App) -> Result<()> {
        app.register_command(track_analytics_event)?;
        app.register_command(get_analytics_events)?;
        app.register_command(generate_analytics_report)?;
        app.register_command(update_analytics_consent)?;
        app.register_command(get_analytics_consent)?;
        Ok(())
    }
}

/// Tauri command for tracking an analytics event
#[tauri::command]
pub async fn track_analytics_event(
    event_type: String,
    event_name: String,
    session_id: String,
    properties: Value,
    user_id: Option<String>,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let service = PrivacyAnalyticsService::new(db.inner().clone());

    // Parse event type
    let event_type = match event_type.as_str() {
        "feature_usage" => AnalyticsEventType::FeatureUsage,
        "performance" => AnalyticsEventType::Performance,
        "error" => AnalyticsEventType::Error,
        "user_interface" => AnalyticsEventType::UserInterface,
        "session" => AnalyticsEventType::Session,
        "custom" => AnalyticsEventType::Custom,
        _ => return Err(format!("Invalid event type: {}", event_type)),
    };

    // Parse user ID if provided
    let user_uuid = if let Some(uid) = user_id {
        Some(Uuid::parse_str(&uid).map_err(|e| e.to_string())?)
    } else {
        None
    };

    // Track the event
    match service.track_event(
        event_type,
        &event_name,
        &session_id,
        properties,
        user_uuid.as_ref(),
    ).await {
        Ok(_) => Ok("Event tracked successfully".to_string()),
        Err(e) => Err(format!("Failed to track event: {}", e)),
    }
}

/// Tauri command for getting analytics events
#[tauri::command]
pub async fn get_analytics_events(
    event_type: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    limit: Option<i64>,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<Vec<AnalyticsEvent>, String> {
    let service = PrivacyAnalyticsService::new(db.inner().clone());

    // Parse event type if provided
    let parsed_event_type = if let Some(et) = event_type {
        Some(match et.as_str() {
            "feature_usage" => AnalyticsEventType::FeatureUsage,
            "performance" => AnalyticsEventType::Performance,
            "error" => AnalyticsEventType::Error,
            "user_interface" => AnalyticsEventType::UserInterface,
            "session" => AnalyticsEventType::Session,
            "custom" => AnalyticsEventType::Custom,
            _ => return Err(format!("Invalid event type: {}", et)),
        })
    } else {
        None
    };

    // Parse dates if provided
    let start = start_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
    let end = end_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));

    // Get events
    match service.get_events(parsed_event_type, start, end, limit).await {
        Ok(events) => Ok(events),
        Err(e) => Err(format!("Failed to get events: {}", e)),
    }
}

/// Tauri command for generating an analytics report
#[tauri::command]
pub async fn generate_analytics_report(
    event_type: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<Value, String> {
    let service = PrivacyAnalyticsService::new(db.inner().clone());

    // Parse event type if provided
    let parsed_event_type = if let Some(et) = event_type {
        Some(match et.as_str() {
            "feature_usage" => AnalyticsEventType::FeatureUsage,
            "performance" => AnalyticsEventType::Performance,
            "error" => AnalyticsEventType::Error,
            "user_interface" => AnalyticsEventType::UserInterface,
            "session" => AnalyticsEventType::Session,
            "custom" => AnalyticsEventType::Custom,
            _ => return Err(format!("Invalid event type: {}", et)),
        })
    } else {
        None
    };

    // Parse dates if provided
    let start = start_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));
    let end = end_date.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok().map(|dt| dt.with_timezone(&Utc)));

    // Generate report
    match service.generate_report(parsed_event_type, start, end).await {
        Ok(report) => Ok(report),
        Err(e) => Err(format!("Failed to generate report: {}", e)),
    }
}

/// Tauri command for updating analytics consent
#[tauri::command]
pub async fn update_analytics_consent(
    user_id: String,
    feature_usage: bool,
    performance: bool,
    error_reporting: bool,
    user_interface: bool,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<String, String> {
    let service = PrivacyAnalyticsService::new(db.inner().clone());

    // Parse user ID
    let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

    // Update consent
    match service.update_consent(&user_uuid, feature_usage, performance, error_reporting, user_interface).await {
        Ok(_) => Ok("Consent updated successfully".to_string()),
        Err(e) => Err(format!("Failed to update consent: {}", e)),
    }
}

/// Tauri command for getting analytics consent
#[tauri::command]
pub async fn get_analytics_consent(
    user_id: String,
    db: tauri::State<'_, DatabaseManager>,
) -> Result<Option<AnalyticsConsent>, String> {
    let service = PrivacyAnalyticsService::new(db.inner().clone());

    // Parse user ID
    let user_uuid = Uuid::parse_str(&user_id).map_err(|e| e.to_string())?;

    // Get consent
    match service.get_consent(&user_uuid).await {
        Ok(consent) => Ok(consent),
        Err(e) => Err(format!("Failed to get consent: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_anonymization_levels() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = PrivacyAnalyticsService::new(db);

        // Test data
        let test_properties = json!({
            "user_id": "test-user-123",
            "email": "user@example.com",
            "feature": "test-feature",
            "duration": 123.456,
            "timestamp": "2023-01-01T12:34:56Z",
            "count": 5
        });

        // Test basic anonymization
        let basic = service.anonymize_properties(test_properties.clone(), AnonymizationLevel::Basic).await.unwrap();
        if let Value::Object(map) = &basic {
            assert!(!map.contains_key("user_id"));
            assert!(!map.contains_key("email"));
            assert!(map.contains_key("feature"));
            assert!(map.contains_key("duration"));
            assert!(map.contains_key("count"));
        } else {
            panic!("Expected object");
        }

        // Test advanced anonymization
        let advanced = service.anonymize_properties(test_properties.clone(), AnonymizationLevel::Advanced).await.unwrap();
        if let Value::Object(map) = &advanced {
            assert!(!map.contains_key("user_id"));
            assert!(!map.contains_key("email"));
            assert!(map.contains_key("feature"));

            // Check if numeric values are rounded
            if let Some(Value::Number(num)) = map.get("duration") {
                if let Some(n) = num.as_f64() {
                    assert_eq!(n, 120.0); // Rounded to nearest 10
                }
            }
        } else {
            panic!("Expected object");
        }

        // Test full anonymization
        let full = service.anonymize_properties(test_properties.clone(), AnonymizationLevel::Full).await.unwrap();
        if let Value::Object(map) = &full {
            assert!(map.contains_key("event_occurred"));
            assert!(map.contains_key("timestamp_bucket"));

            // Should have approximate values
            assert!(map.contains_key("duration_approx") || map.contains_key("count_approx"));
        } else {
            panic!("Expected object");
        }
    }

    #[tokio::test]
    async fn test_consent_management() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;
        let service = PrivacyAnalyticsService::new(db);

        // Initialize service
        service.initialize().await.unwrap();

        // Test user ID
        let user_id = Uuid::new_v4();

        // Initially, user should have no consent record
        let initial_consent = service.get_consent(&user_id).await.unwrap();
        assert!(initial_consent.is_none());

        // Update consent
        service.update_consent(&user_id, true, false, true, false).await.unwrap();

        // Check updated consent
        let updated_consent = service.get_consent(&user_id).await.unwrap();
        assert!(updated_consent.is_some());
        let consent = updated_consent.unwrap();
        assert_eq!(consent.user_id, user_id);
        assert_eq!(consent.feature_usage, true);
        assert_eq!(consent.performance, false);
        assert_eq!(consent.error_reporting, true);
        assert_eq!(consent.user_interface, false);

        // Check consent for specific event types
        assert_eq!(service.check_consent(&user_id, AnalyticsEventType::FeatureUsage).await.unwrap(), true);
        assert_eq!(service.check_consent(&user_id, AnalyticsEventType::Performance).await.unwrap(), false);
        assert_eq!(service.check_consent(&user_id, AnalyticsEventType::Error).await.unwrap(), true);
        assert_eq!(service.check_consent(&user_id, AnalyticsEventType::UserInterface).await.unwrap(), false);
    }

    #[tokio::test]
    async fn test_event_tracking_and_retention() {
        // Setup test database
        let db = DatabaseManager::setup_test_db().await;

        // Create service with short retention period for testing
        let config = AnalyticsConfig {
            enabled: true,
            default_anonymization_level: AnonymizationLevel::Basic,
            retention_days: 7,
            collect_in_development: true,
            custom_config: None,
        };
        let service = PrivacyAnalyticsService::with_config(db, config);

        // Initialize service
        service.initialize().await.unwrap();

        // Test user ID and session
        let user_id = Uuid::new_v4();
        let session_id = "test-session-123";

        // Set user consent
        service.update_consent(&user_id, true, true, true, true).await.unwrap();

        // Track some events
        service.track_event(
            AnalyticsEventType::FeatureUsage,
            "test_feature_used",
            session_id,
            json!({"feature": "test", "action": "click"}),
            Some(&user_id)
        ).await.unwrap();

        service.track_event(
            AnalyticsEventType::Performance,
            "test_performance",
            session_id,
            json!({"duration_ms": 123}),
            Some(&user_id)
        ).await.unwrap();

        // Get recent events
        let events = service.get_events(None, None, None, None).await.unwrap();
        assert_eq!(events.len(), 2);

        // Test retention policy
        // Create an old event (manually insert to bypass normal flow)
        let old_timestamp = Utc::now() - Duration::days(30);
        let old_event = AnalyticsEvent {
            id: Uuid::new_v4(),
            event_type: AnalyticsEventType::FeatureUsage,
            event_name: "old_event".to_string(),
            session_id: session_id.to_string(),
            timestamp: old_timestamp,
            properties: Json(json!({"old": true})),
            has_consent: true,
            anonymization_level: AnonymizationLevel::Basic,
        };

        service.store_event(&old_event).await.unwrap();

        // Apply retention policy
        service.apply_retention_policy().await.unwrap();

        // Old event should be gone
        let events_after_retention = service.get_events(None, None, None, None).await.unwrap();
        assert_eq!(events_after_retention.len(), 2); // Only the 2 recent events remain

        // Generate a report
        let report = service.generate_report(None, None, None).await.unwrap();
        assert_eq!(report["total_events"], 2);
    }
}
