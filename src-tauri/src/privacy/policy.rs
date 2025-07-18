//! Privacy Policy Enforcement Mechanisms
//!
//! This module provides structures and functions for defining, validating,
//! and enforcing privacy policies throughout the application.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use anyhow::{Result, Context};

/// Represents a privacy policy rule that can be enforced
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Unique identifier for the rule
    pub id: String,
    
    /// Human-readable description of the rule
    pub description: String,
    
    /// The category this rule belongs to (e.g., "data_retention", "data_access")
    pub category: String,
    
    /// Whether this rule is currently enabled
    pub enabled: bool,
    
    /// Parameters that configure the rule's behavior
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Represents the result of a policy enforcement check
#[derive(Debug, Clone)]
pub struct PolicyEnforcementResult {
    /// Whether the policy check passed
    pub compliant: bool,
    
    /// The rule that was checked
    pub rule: PolicyRule,
    
    /// Details about the enforcement result
    pub details: String,
    
    /// Suggested remediation steps if not compliant
    pub remediation: Option<String>,
}

/// Manages privacy policy rules and enforces them
#[derive(Debug)]
pub struct PolicyEnforcer {
    /// All available policy rules
    rules: Vec<PolicyRule>,
}

impl PolicyEnforcer {
    /// Create a new PolicyEnforcer with the given rules
    pub fn new(rules: Vec<PolicyRule>) -> Self {
        Self { rules }
    }
    
    /// Create a new PolicyEnforcer with default rules
    pub fn with_defaults() -> Self {
        let default_rules = vec![
            PolicyRule {
                id: "data_retention".to_string(),
                description: "Enforces data retention periods for user data".to_string(),
                category: "data_lifecycle".to_string(),
                enabled: true,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("retention_days".to_string(), serde_json::json!(90));
                    params
                },
            },
            PolicyRule {
                id: "data_minimization".to_string(),
                description: "Ensures only necessary data is collected and stored".to_string(),
                category: "data_collection".to_string(),
                enabled: true,
                parameters: HashMap::new(),
            },
            PolicyRule {
                id: "consent_required".to_string(),
                description: "Verifies user consent before data processing".to_string(),
                category: "user_consent".to_string(),
                enabled: true,
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("require_explicit_consent".to_string(), serde_json::json!(true));
                    params
                },
            },
        ];
        
        Self::new(default_rules)
    }
    
    /// Get all policy rules
    pub fn get_rules(&self) -> &[PolicyRule] {
        &self.rules
    }
    
    /// Get rules by category
    pub fn get_rules_by_category(&self, category: &str) -> Vec<&PolicyRule> {
        self.rules.iter()
            .filter(|rule| rule.category == category && rule.enabled)
            .collect()
    }
    
    /// Add a new policy rule
    pub fn add_rule(&mut self, rule: PolicyRule) {
        // Check if rule with this ID already exists
        if let Some(existing_index) = self.rules.iter().position(|r| r.id == rule.id) {
            // Replace existing rule
            self.rules[existing_index] = rule;
            debug!("Updated existing policy rule: {}", self.rules[existing_index].id);
        } else {
            // Add new rule
            debug!("Added new policy rule: {}", rule.id);
            self.rules.push(rule);
        }
    }
    
    /// Enable or disable a policy rule
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) -> Result<()> {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
            debug!("Set policy rule '{}' enabled: {}", rule_id, enabled);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Policy rule not found: {}", rule_id))
        }
    }
    
    /// Enforce a specific policy rule
    pub fn enforce_rule(&self, rule_id: &str, context: &HashMap<String, serde_json::Value>) -> Result<PolicyEnforcementResult> {
        let rule = self.rules.iter()
            .find(|r| r.id == rule_id && r.enabled)
            .ok_or_else(|| anyhow::anyhow!("Policy rule not found or not enabled: {}", rule_id))?;
        
        match rule.id.as_str() {
            "data_retention" => self.enforce_data_retention(rule, context),
            "data_minimization" => self.enforce_data_minimization(rule, context),
            "consent_required" => self.enforce_consent_required(rule, context),
            _ => Err(anyhow::anyhow!("Unknown policy rule type: {}", rule.id)),
        }
    }
    
    /// Enforce all enabled policy rules that apply to the given context
    pub fn enforce_all_applicable(&self, context: &HashMap<String, serde_json::Value>) -> Vec<Result<PolicyEnforcementResult>> {
        let category = context.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("default");
        
        let applicable_rules = self.rules.iter()
            .filter(|r| r.enabled)
            .filter(|r| {
                // If context specifies rules, only enforce those
                if let Some(rules) = context.get("rules") {
                    if let Some(rules_array) = rules.as_array() {
                        return rules_array.iter()
                            .any(|rule_id| rule_id.as_str() == Some(&r.id));
                    }
                }
                
                // Otherwise, enforce by category
                r.category == category || category == "default"
            })
            .collect::<Vec<_>>();
        
        applicable_rules.iter()
            .map(|rule| self.enforce_rule(&rule.id, context))
            .collect()
    }
    
    // Implementation of specific policy enforcement rules
    
    fn enforce_data_retention(&self, rule: &PolicyRule, context: &HashMap<String, serde_json::Value>) -> Result<PolicyEnforcementResult> {
        let retention_days = rule.parameters.get("retention_days")
            .and_then(|v| v.as_i64())
            .unwrap_or(90);
        
        let data_age_days = context.get("data_age_days")
            .and_then(|v| v.as_i64())
            .context("Missing required context: data_age_days")?;
        
        let compliant = data_age_days <= retention_days;
        
        let details = if compliant {
            format!("Data age ({} days) is within retention period ({} days)", data_age_days, retention_days)
        } else {
            format!("Data age ({} days) exceeds retention period ({} days)", data_age_days, retention_days)
        };
        
        let remediation = if !compliant {
            Some("Delete or anonymize this data to comply with retention policy".to_string())
        } else {
            None
        };
        
        Ok(PolicyEnforcementResult {
            compliant,
            rule: rule.clone(),
            details,
            remediation,
        })
    }
    
    fn enforce_data_minimization(&self, rule: &PolicyRule, context: &HashMap<String, serde_json::Value>) -> Result<PolicyEnforcementResult> {
        let data_fields = context.get("data_fields")
            .and_then(|v| v.as_array())
            .context("Missing required context: data_fields")?;
        
        let required_fields = context.get("required_fields")
            .and_then(|v| v.as_array())
            .context("Missing required context: required_fields")?;
        
        // Check if any non-required fields are present
        let mut extra_fields = Vec::new();
        
        for field in data_fields {
            let field_str = field.as_str().unwrap_or_default();
            let is_required = required_fields.iter()
                .any(|req| req.as_str() == Some(field_str));
            
            if !is_required {
                extra_fields.push(field_str.to_string());
            }
        }
        
        let compliant = extra_fields.is_empty();
        
        let details = if compliant {
            "All data fields are required for the specified purpose".to_string()
        } else {
            format!("Found non-essential data fields: {}", extra_fields.join(", "))
        };
        
        let remediation = if !compliant {
            Some(format!("Remove unnecessary fields: {}", extra_fields.join(", ")))
        } else {
            None
        };
        
        Ok(PolicyEnforcementResult {
            compliant,
            rule: rule.clone(),
            details,
            remediation,
        })
    }
    
    fn enforce_consent_required(&self, rule: &PolicyRule, context: &HashMap<String, serde_json::Value>) -> Result<PolicyEnforcementResult> {
        let require_explicit = rule.parameters.get("require_explicit_consent")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let has_consent = context.get("has_consent")
            .and_then(|v| v.as_bool())
            .context("Missing required context: has_consent")?;
        
        let consent_type = context.get("consent_type")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        
        let compliant = has_consent && (!require_explicit || consent_type == "explicit");
        
        let details = if compliant {
            format!("User has provided {} consent", consent_type)
        } else if !has_consent {
            "User has not provided consent".to_string()
        } else {
            format!("User has provided {} consent, but explicit consent is required", consent_type)
        };
        
        let remediation = if !compliant {
            Some("Obtain explicit user consent before processing this data".to_string())
        } else {
            None
        };
        
        Ok(PolicyEnforcementResult {
            compliant,
            rule: rule.clone(),
            details,
            remediation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_enforcer_creation() {
        let enforcer = PolicyEnforcer::with_defaults();
        assert_eq!(enforcer.get_rules().len(), 3);
    }
    
    #[test]
    fn test_data_retention_enforcement() {
        let enforcer = PolicyEnforcer::with_defaults();
        
        // Test compliant case
        let mut context = HashMap::new();
        context.insert("data_age_days".to_string(), serde_json::json!(30));
        
        let result = enforcer.enforce_rule("data_retention", &context).unwrap();
        assert!(result.compliant);
        
        // Test non-compliant case
        let mut context = HashMap::new();
        context.insert("data_age_days".to_string(), serde_json::json!(120));
        
        let result = enforcer.enforce_rule("data_retention", &context).unwrap();
        assert!(!result.compliant);
        assert!(result.remediation.is_some());
    }
    
    #[test]
    fn test_data_minimization_enforcement() {
        let enforcer = PolicyEnforcer::with_defaults();
        
        // Test compliant case
        let mut context = HashMap::new();
        context.insert("data_fields".to_string(), serde_json::json!(["name", "email"]));
        context.insert("required_fields".to_string(), serde_json::json!(["name", "email"]));
        
        let result = enforcer.enforce_rule("data_minimization", &context).unwrap();
        assert!(result.compliant);
        
        // Test non-compliant case
        let mut context = HashMap::new();
        context.insert("data_fields".to_string(), serde_json::json!(["name", "email", "phone", "address"]));
        context.insert("required_fields".to_string(), serde_json::json!(["name", "email"]));
        
        let result = enforcer.enforce_rule("data_minimization", &context).unwrap();
        assert!(!result.compliant);
        assert!(result.remediation.is_some());
    }
    
    #[test]
    fn test_consent_required_enforcement() {
        let enforcer = PolicyEnforcer::with_defaults();
        
        // Test compliant case
        let mut context = HashMap::new();
        context.insert("has_consent".to_string(), serde_json::json!(true));
        context.insert("consent_type".to_string(), serde_json::json!("explicit"));
        
        let result = enforcer.enforce_rule("consent_required", &context).unwrap();
        assert!(result.compliant);
        
        // Test non-compliant case
        let mut context = HashMap::new();
        context.insert("has_consent".to_string(), serde_json::json!(true));
        context.insert("consent_type".to_string(), serde_json::json!("implicit"));
        
        let result = enforcer.enforce_rule("consent_required", &context).unwrap();
        assert!(!result.compliant);
        assert!(result.remediation.is_some());
    }
}