//! Privacy Policy Enforcement Service
//!
//! This service manages privacy policies and enforces them throughout the application.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, error};
use sqlx::{SqlitePool, Row};

use crate::privacy::{PolicyRule, PolicyEnforcer, PolicyEnforcementResult};
use crate::error::Error;

/// Service for managing and enforcing privacy policies
#[derive(Debug)]
pub struct PrivacyPolicyService {
    /// Database connection pool
    db: SqlitePool,
    
    /// Policy enforcer instance
    enforcer: Arc<RwLock<PolicyEnforcer>>,
}

/// Configuration for the privacy policy service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPolicyConfig {
    /// Whether policy enforcement is enabled
    pub enforcement_enabled: bool,
    
    /// Whether to log policy violations
    pub log_violations: bool,
    
    /// Whether to block operations that violate policies
    pub block_violations: bool,
}

impl Default for PrivacyPolicyConfig {
    fn default() -> Self {
        Self {
            enforcement_enabled: true,
            log_violations: true,
            block_violations: true,
        }
    }
}

/// Record of a policy enforcement action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEnforcementRecord {
    /// ID of the record
    pub id: i64,
    
    /// ID of the rule that was enforced
    pub rule_id: String,
    
    /// Whether the policy check passed
    pub compliant: bool,
    
    /// Details about the enforcement result
    pub details: String,
    
    /// The context in which the policy was enforced
    pub context: String,
    
    /// Timestamp of the enforcement action
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl PrivacyPolicyService {
    /// Create a new PrivacyPolicyService
    pub async fn new(db: SqlitePool) -> Result<Self> {
        // Initialize the database tables if they don't exist
        Self::init_db(&db).await?;
        
        // Load rules from the database
        let rules = Self::load_rules_from_db(&db).await?;
        
        // Create the policy enforcer
        let enforcer = if rules.is_empty() {
            PolicyEnforcer::with_defaults()
        } else {
            PolicyEnforcer::new(rules)
        };
        
        Ok(Self {
            db,
            enforcer: Arc::new(RwLock::new(enforcer)),
        })
    }
    
    /// Initialize the database tables
    async fn init_db(db: &SqlitePool) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS privacy_policy_rules (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                category TEXT NOT NULL,
                enabled BOOLEAN NOT NULL DEFAULT 1,
                parameters TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS privacy_policy_enforcement_records (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                rule_id TEXT NOT NULL,
                compliant BOOLEAN NOT NULL,
                details TEXT NOT NULL,
                context TEXT NOT NULL,
                timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (rule_id) REFERENCES privacy_policy_rules(id)
            );
            "#,
        )
        .execute(db)
        .await
        .context("Failed to initialize privacy policy database tables")?;
        
        Ok(())
    }
    
    /// Load rules from the database
    async fn load_rules_from_db(db: &SqlitePool) -> Result<Vec<PolicyRule>> {
        let rows = sqlx::query(
            r#"
            SELECT id, description, category, enabled, parameters
            FROM privacy_policy_rules
            "#,
        )
        .fetch_all(db)
        .await
        .context("Failed to load privacy policy rules from database")?;
        
        let mut rules = Vec::with_capacity(rows.len());
        
        for row in rows {
            let id: String = row.get("id");
            let description: String = row.get("description");
            let category: String = row.get("category");
            let enabled: bool = row.get("enabled");
            let parameters_json: String = row.get("parameters");
            
            let parameters: HashMap<String, serde_json::Value> = serde_json::from_str(&parameters_json)
                .context(format!("Failed to parse parameters for rule {}", id))?;
            
            rules.push(PolicyRule {
                id,
                description,
                category,
                enabled,
                parameters,
            });
        }
        
        Ok(rules)
    }
    
    /// Save a rule to the database
    async fn save_rule_to_db(&self, rule: &PolicyRule) -> Result<()> {
        let parameters_json = serde_json::to_string(&rule.parameters)
            .context(format!("Failed to serialize parameters for rule {}", rule.id))?;
        
        sqlx::query(
            r#"
            INSERT INTO privacy_policy_rules (id, description, category, enabled, parameters)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT (id) DO UPDATE SET
                description = excluded.description,
                category = excluded.category,
                enabled = excluded.enabled,
                parameters = excluded.parameters
            "#,
        )
        .bind(&rule.id)
        .bind(&rule.description)
        .bind(&rule.category)
        .bind(rule.enabled)
        .bind(&parameters_json)
        .execute(&self.db)
        .await
        .context(format!("Failed to save rule {} to database", rule.id))?;
        
        Ok(())
    }
    
    /// Record a policy enforcement action
    async fn record_enforcement(&self, result: &PolicyEnforcementResult, context: &HashMap<String, serde_json::Value>) -> Result<i64> {
        let context_json = serde_json::to_string(context)
            .context("Failed to serialize context for enforcement record")?;
        
        let id = sqlx::query(
            r#"
            INSERT INTO privacy_policy_enforcement_records (rule_id, compliant, details, context)
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&result.rule.id)
        .bind(result.compliant)
        .bind(&result.details)
        .bind(&context_json)
        .fetch_one(&self.db)
        .await
        .context("Failed to record policy enforcement action")?
        .get::<i64, _>("id");
        
        Ok(id)
    }
    
    /// Get all policy rules
    pub async fn get_rules(&self) -> Result<Vec<PolicyRule>> {
        let enforcer = self.enforcer.read().await;
        Ok(enforcer.get_rules().to_vec())
    }
    
    /// Get rules by category
    pub async fn get_rules_by_category(&self, category: &str) -> Result<Vec<PolicyRule>> {
        let enforcer = self.enforcer.read().await;
        Ok(enforcer.get_rules_by_category(category)
            .into_iter()
            .cloned()
            .collect())
    }
    
    /// Add or update a policy rule
    pub async fn add_rule(&self, rule: PolicyRule) -> Result<()> {
        // Save to database first
        self.save_rule_to_db(&rule).await?;
        
        // Then update in-memory enforcer
        let mut enforcer = self.enforcer.write().await;
        enforcer.add_rule(rule);
        
        Ok(())
    }
    
    /// Enable or disable a policy rule
    pub async fn set_rule_enabled(&self, rule_id: &str, enabled: bool) -> Result<()> {
        // Update in-memory enforcer
        {
            let mut enforcer = self.enforcer.write().await;
            enforcer.set_rule_enabled(rule_id, enabled)?;
        }
        
        // Update in database
        sqlx::query(
            r#"
            UPDATE privacy_policy_rules
            SET enabled = ?
            WHERE id = ?
            "#,
        )
        .bind(enabled)
        .bind(rule_id)
        .execute(&self.db)
        .await
        .context(format!("Failed to update rule {} enabled status in database", rule_id))?;
        
        Ok(())
    }
    
    /// Enforce a specific policy rule
    pub async fn enforce_rule(&self, rule_id: &str, context: &HashMap<String, serde_json::Value>, config: &PrivacyPolicyConfig) -> Result<PolicyEnforcementResult> {
        // Skip enforcement if disabled
        if !config.enforcement_enabled {
            debug!("Policy enforcement is disabled, skipping rule {}", rule_id);
            return Ok(PolicyEnforcementResult {
                compliant: true,
                rule: PolicyRule {
                    id: rule_id.to_string(),
                    description: "Enforcement disabled".to_string(),
                    category: "disabled".to_string(),
                    enabled: false,
                    parameters: HashMap::new(),
                },
                details: "Policy enforcement is disabled".to_string(),
                remediation: None,
            });
        }
        
        // Enforce the rule
        let result = {
            let enforcer = self.enforcer.read().await;
            enforcer.enforce_rule(rule_id, context)?
        };
        
        // Record the enforcement action
        self.record_enforcement(&result, context).await?;
        
        // Log violations if configured
        if !result.compliant && config.log_violations {
            warn!(
                "Privacy policy violation: Rule '{}' ({}) - {}",
                result.rule.id, result.rule.description, result.details
            );
            
            if let Some(remediation) = &result.remediation {
                info!("Suggested remediation: {}", remediation);
            }
        }
        
        // If configured to block violations and the result is non-compliant, return an error
        if !result.compliant && config.block_violations {
            return Err(Error::PrivacyPolicyViolation {
                rule_id: result.rule.id.clone(),
                details: result.details.clone(),
                remediation: result.remediation.clone(),
            }.into());
        }
        
        Ok(result)
    }
    
    /// Enforce all applicable policy rules
    pub async fn enforce_all_applicable(&self, context: &HashMap<String, serde_json::Value>, config: &PrivacyPolicyConfig) -> Result<Vec<PolicyEnforcementResult>> {
        // Skip enforcement if disabled
        if !config.enforcement_enabled {
            debug!("Policy enforcement is disabled, skipping all rules");
            return Ok(Vec::new());
        }
        
        // Enforce all applicable rules
        let results = {
            let enforcer = self.enforcer.read().await;
            let result_list = enforcer.enforce_all_applicable(context);
            
            // Collect results, logging errors but not failing
            let mut results = Vec::with_capacity(result_list.len());
            for result in result_list {
                match result {
                    Ok(r) => results.push(r),
                    Err(e) => error!("Error enforcing policy rule: {}", e),
                }
            }
            
            results
        };
        
        // Record all enforcement actions
        for result in &results {
            if let Err(e) = self.record_enforcement(result, context).await {
                error!("Failed to record policy enforcement: {}", e);
            }
        }
        
        // Log violations if configured
        if config.log_violations {
            for result in &results {
                if !result.compliant {
                    warn!(
                        "Privacy policy violation: Rule '{}' ({}) - {}",
                        result.rule.id, result.rule.description, result.details
                    );
                    
                    if let Some(remediation) = &result.remediation {
                        info!("Suggested remediation: {}", remediation);
                    }
                }
            }
        }
        
        // If configured to block violations and any result is non-compliant, return an error
        if config.block_violations {
            if let Some(violation) = results.iter().find(|r| !r.compliant) {
                return Err(Error::PrivacyPolicyViolation {
                    rule_id: violation.rule.id.clone(),
                    details: violation.details.clone(),
                    remediation: violation.remediation.clone(),
                }.into());
            }
        }
        
        Ok(results)
    }
    
    /// Get recent policy enforcement records
    pub async fn get_recent_enforcement_records(&self, limit: i64) -> Result<Vec<PolicyEnforcementRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, rule_id, compliant, details, context, timestamp
            FROM privacy_policy_enforcement_records
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .context("Failed to fetch recent policy enforcement records")?;
        
        let mut records = Vec::with_capacity(rows.len());
        
        for row in rows {
            let id: i64 = row.get("id");
            let rule_id: String = row.get("rule_id");
            let compliant: bool = row.get("compliant");
            let details: String = row.get("details");
            let context: String = row.get("context");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            
            records.push(PolicyEnforcementRecord {
                id,
                rule_id,
                compliant,
                details,
                context,
                timestamp,
            });
        }
        
        Ok(records)
    }
    
    /// Get enforcement records for a specific rule
    pub async fn get_rule_enforcement_records(&self, rule_id: &str, limit: i64) -> Result<Vec<PolicyEnforcementRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, rule_id, compliant, details, context, timestamp
            FROM privacy_policy_enforcement_records
            WHERE rule_id = ?
            ORDER BY timestamp DESC
            LIMIT ?
            "#,
        )
        .bind(rule_id)
        .bind(limit)
        .fetch_all(&self.db)
        .await
        .context(format!("Failed to fetch enforcement records for rule {}", rule_id))?;
        
        let mut records = Vec::with_capacity(rows.len());
        
        for row in rows {
            let id: i64 = row.get("id");
            let rule_id: String = row.get("rule_id");
            let compliant: bool = row.get("compliant");
            let details: String = row.get("details");
            let context: String = row.get("context");
            let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
            
            records.push(PolicyEnforcementRecord {
                id,
                rule_id,
                compliant,
                details,
                context,
                timestamp,
            });
        }
        
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    
    async fn setup_test_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory SQLite database");
        
        db
    }
    
    #[tokio::test]
    async fn test_service_initialization() -> Result<()> {
        let db = setup_test_db().await;
        let service = PrivacyPolicyService::new(db).await?;
        
        let rules = service.get_rules().await?;
        assert!(!rules.is_empty(), "Service should initialize with default rules");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_add_and_get_rule() -> Result<()> {
        let db = setup_test_db().await;
        let service = PrivacyPolicyService::new(db).await?;
        
        let initial_count = service.get_rules().await?.len();
        
        // Add a new rule
        let new_rule = PolicyRule {
            id: "test_rule".to_string(),
            description: "Test rule for unit tests".to_string(),
            category: "testing".to_string(),
            enabled: true,
            parameters: HashMap::new(),
        };
        
        service.add_rule(new_rule.clone()).await?;
        
        // Verify rule was added
        let rules = service.get_rules().await?;
        assert_eq!(rules.len(), initial_count + 1, "Rule count should increase by 1");
        
        // Verify rule by category
        let category_rules = service.get_rules_by_category("testing").await?;
        assert_eq!(category_rules.len(), 1, "Should find 1 rule in 'testing' category");
        assert_eq!(category_rules[0].id, "test_rule", "Rule ID should match");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_enable_disable_rule() -> Result<()> {
        let db = setup_test_db().await;
        let service = PrivacyPolicyService::new(db).await?;
        
        // Add a rule
        let rule = PolicyRule {
            id: "toggle_rule".to_string(),
            description: "Rule to test enabling/disabling".to_string(),
            category: "testing".to_string(),
            enabled: true,
            parameters: HashMap::new(),
        };
        
        service.add_rule(rule).await?;
        
        // Disable the rule
        service.set_rule_enabled("toggle_rule", false).await?;
        
        // Verify rule is disabled
        let rules = service.get_rules().await?;
        let toggle_rule = rules.iter().find(|r| r.id == "toggle_rule").unwrap();
        assert!(!toggle_rule.enabled, "Rule should be disabled");
        
        // Enable the rule
        service.set_rule_enabled("toggle_rule", true).await?;
        
        // Verify rule is enabled
        let rules = service.get_rules().await?;
        let toggle_rule = rules.iter().find(|r| r.id == "toggle_rule").unwrap();
        assert!(toggle_rule.enabled, "Rule should be enabled");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_enforce_rule() -> Result<()> {
        let db = setup_test_db().await;
        let service = PrivacyPolicyService::new(db).await?;
        
        // Add a data retention rule
        let rule = PolicyRule {
            id: "test_retention".to_string(),
            description: "Test data retention rule".to_string(),
            category: "data_lifecycle".to_string(),
            enabled: true,
            parameters: {
                let mut params = HashMap::new();
                params.insert("retention_days".to_string(), serde_json::json!(30));
                params
            },
        };
        
        service.add_rule(rule).await?;
        
        // Test compliant case
        let mut context = HashMap::new();
        context.insert("data_age_days".to_string(), serde_json::json!(20));
        
        let config = PrivacyPolicyConfig::default();
        let result = service.enforce_rule("test_retention", &context, &config).await?;
        
        assert!(result.compliant, "Rule should be compliant");
        
        // Test non-compliant case with blocking disabled
        let mut context = HashMap::new();
        context.insert("data_age_days".to_string(), serde_json::json!(40));
        
        let mut config = PrivacyPolicyConfig::default();
        config.block_violations = false;
        
        let result = service.enforce_rule("test_retention", &context, &config).await?;
        
        assert!(!result.compliant, "Rule should not be compliant");
        assert!(result.remediation.is_some(), "Should have remediation suggestion");
        
        // Test non-compliant case with blocking enabled
        let mut config = PrivacyPolicyConfig::default();
        config.block_violations = true;
        
        let result = service.enforce_rule("test_retention", &context, &config);
        assert!(result.is_err(), "Should return error when blocking violations");
        
        Ok(())
    }
}