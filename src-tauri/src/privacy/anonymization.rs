//! Anonymization utilities for privacy-preserving data handling
//!
//! This module provides a comprehensive set of utilities for anonymizing data
//! in various ways, including:
//! - Data masking
//! - Format-preserving anonymization
//! - Differential privacy techniques
//! - k-anonymity implementation
//! - Pseudonymization with consistent mappings

use std::collections::{HashMap, HashSet};
use std::cmp::min;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use rand::{Rng, thread_rng};
use rand::distributions::{Alphanumeric, Distribution, Standard};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

/// Anonymization strategy to use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnonymizationStrategy {
    /// No anonymization
    None,
    /// Partial masking (e.g., "j*** d**")
    PartialMask,
    /// Complete masking (e.g., "****")
    CompleteMask,
    /// Redaction (e.g., "[REDACTED]")
    Redaction,
    /// Generalization (e.g., age ranges instead of exact age)
    Generalization,
    /// Pseudonymization (consistent replacement)
    Pseudonymization,
    /// Differential privacy (adding noise)
    DifferentialPrivacy,
}

/// Configuration for anonymization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationConfig {
    /// Default strategy to use
    pub default_strategy: AnonymizationStrategy,
    /// Field-specific strategies
    pub field_strategies: HashMap<String, AnonymizationStrategy>,
    /// Epsilon value for differential privacy (lower = more privacy)
    pub epsilon: f64,
    /// Whether to preserve format when anonymizing
    pub preserve_format: bool,
    /// Custom redaction text
    pub redaction_text: Option<String>,
    /// Custom generalization ranges for numeric values
    pub generalization_ranges: Option<HashMap<String, Vec<(f64, f64)>>>,
    /// Pseudonymization salt for consistent hashing
    pub pseudonymization_salt: Option<String>,
}

impl Default for AnonymizationConfig {
    fn default() -> Self {
        Self {
            default_strategy: AnonymizationStrategy::PartialMask,
            field_strategies: HashMap::new(),
            epsilon: 1.0,
            preserve_format: true,
            redaction_text: None,
            generalization_ranges: None,
            pseudonymization_salt: None,
        }
    }
}

/// Anonymizer for consistent anonymization across multiple calls
#[derive(Debug, Clone)]
pub struct Anonymizer {
    config: AnonymizationConfig,
    pseudonym_map: Arc<Mutex<HashMap<String, String>>>,
}

impl Anonymizer {
    /// Create a new anonymizer with the given configuration
    pub fn new(config: AnonymizationConfig) -> Self {
        Self {
            config,
            pseudonym_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new anonymizer with default configuration
    pub fn default() -> Self {
        Self::new(AnonymizationConfig::default())
    }

    /// Anonymize a string value
    pub fn anonymize_string(&self, value: &str, field_name: Option<&str>) -> String {
        let strategy = self.get_strategy_for_field(field_name);

        match strategy {
            AnonymizationStrategy::None => value.to_string(),
            AnonymizationStrategy::PartialMask => self.apply_partial_mask(value),
            AnonymizationStrategy::CompleteMask => self.apply_complete_mask(value),
            AnonymizationStrategy::Redaction => self.apply_redaction(value),
            AnonymizationStrategy::Generalization => self.apply_generalization_to_string(value),
            AnonymizationStrategy::Pseudonymization => self.apply_pseudonymization(value, field_name),
            AnonymizationStrategy::DifferentialPrivacy => value.to_string(), // Not applicable to strings
        }
    }

    /// Anonymize an email address
    pub fn anonymize_email(&self, email: &str) -> String {
        if email.is_empty() || !email.contains('@') {
            return self.anonymize_string(email, Some("email"));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return self.anonymize_string(email, Some("email"));
        }

        let username = parts[0];
        let domain = parts[1];

        let strategy = self.get_strategy_for_field(Some("email"));

        match strategy {
            AnonymizationStrategy::None => email.to_string(),
            AnonymizationStrategy::PartialMask => {
                // Keep first character of username, mask the rest
                if username.len() > 1 {
                    format!("{}{}@{}", 
                        &username[0..1], 
                        "*".repeat(username.len() - 1), 
                        domain
                    )
                } else {
                    format!("{}@{}", username, domain)
                }
            },
            AnonymizationStrategy::CompleteMask => {
                format!("{}@{}", "*".repeat(username.len()), domain)
            },
            AnonymizationStrategy::Redaction => {
                let redaction_text = self.config.redaction_text.as_deref().unwrap_or("[REDACTED]");
                format!("{}@{}", redaction_text, domain)
            },
            AnonymizationStrategy::Generalization => {
                // For emails, generalization means keeping the domain but anonymizing the username
                format!("user@{}", domain)
            },
            AnonymizationStrategy::Pseudonymization => {
                let pseudonym = self.get_pseudonym(username);
                format!("{}@{}", pseudonym, domain)
            },
            AnonymizationStrategy::DifferentialPrivacy => email.to_string(), // Not applicable to emails
        }
    }

    /// Anonymize a phone number
    pub fn anonymize_phone(&self, phone: &str) -> String {
        let strategy = self.get_strategy_for_field(Some("phone"));

        match strategy {
            AnonymizationStrategy::None => phone.to_string(),
            AnonymizationStrategy::PartialMask => {
                // Keep last 4 digits, mask the rest
                if phone.len() > 4 {
                    format!("{}{}",
                        "*".repeat(phone.len() - 4),
                        &phone[phone.len() - 4..]
                    )
                } else {
                    phone.to_string()
                }
            },
            AnonymizationStrategy::CompleteMask => {
                "*".repeat(phone.len())
            },
            AnonymizationStrategy::Redaction => {
                self.config.redaction_text.as_deref().unwrap_or("[REDACTED]").to_string()
            },
            AnonymizationStrategy::Generalization => {
                // For phones, generalization means keeping country/area code
                if phone.len() > 6 {
                    let area_code = &phone[0..min(6, phone.len())];
                    format!("{}-xxxx", area_code)
                } else {
                    "xxx-xxxx".to_string()
                }
            },
            AnonymizationStrategy::Pseudonymization => {
                self.apply_pseudonymization(phone, Some("phone"))
            },
            AnonymizationStrategy::DifferentialPrivacy => phone.to_string(), // Not applicable to phones
        }
    }

    /// Anonymize a numeric value using differential privacy
    pub fn anonymize_numeric(&self, value: f64, field_name: Option<&str>) -> f64 {
        let strategy = self.get_strategy_for_field(field_name);

        match strategy {
            AnonymizationStrategy::None => value,
            AnonymizationStrategy::PartialMask | 
            AnonymizationStrategy::CompleteMask | 
            AnonymizationStrategy::Redaction => 0.0, // Not applicable to numbers
            AnonymizationStrategy::Generalization => self.apply_generalization_to_numeric(value, field_name),
            AnonymizationStrategy::Pseudonymization => value, // Not applicable to numbers
            AnonymizationStrategy::DifferentialPrivacy => self.apply_differential_privacy(value),
        }
    }

    /// Anonymize a date/time value
    pub fn anonymize_datetime(&self, datetime: &DateTime<Utc>, field_name: Option<&str>) -> DateTime<Utc> {
        let strategy = self.get_strategy_for_field(field_name);

        match strategy {
            AnonymizationStrategy::None => *datetime,
            AnonymizationStrategy::PartialMask | 
            AnonymizationStrategy::CompleteMask | 
            AnonymizationStrategy::Redaction => Utc::now(), // Not really applicable
            AnonymizationStrategy::Generalization => {
                // Round to nearest day, hour, or minute depending on sensitivity
                let date = datetime.date();
                match field_name {
                    Some("birth_date") | Some("dob") => {
                        // For birth dates, just keep the year
                        date.and_hms(0, 0, 0)
                    },
                    Some("created_at") | Some("updated_at") => {
                        // For timestamps, round to nearest hour
                        date.and_hms(datetime.hour(), 0, 0)
                    },
                    _ => {
                        // Default: round to nearest day
                        date.and_hms(0, 0, 0)
                    }
                }
            },
            AnonymizationStrategy::Pseudonymization => *datetime, // Not applicable
            AnonymizationStrategy::DifferentialPrivacy => {
                // Add random noise to the timestamp (within a day)
                let noise_seconds = thread_rng().gen_range(-43200..43200); // ±12 hours
                *datetime + chrono::Duration::seconds(noise_seconds)
            },
        }
    }

    /// Anonymize a JSON object
    pub fn anonymize_json(&self, json_value: &Value) -> Value {
        match json_value {
            Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (key, value) in map {
                    // Skip fields that should be removed entirely
                    if self.should_remove_field(key) {
                        continue;
                    }

                    // Recursively anonymize nested values
                    result.insert(key.clone(), self.anonymize_json(value));
                }
                Value::Object(result)
            },
            Value::Array(arr) => {
                let mut result = Vec::new();
                for value in arr {
                    result.push(self.anonymize_json(value));
                }
                Value::Array(result)
            },
            Value::String(s) => {
                // Check if it's an email, phone, etc. and apply appropriate anonymization
                if s.contains('@') && s.contains('.') {
                    Value::String(self.anonymize_email(s))
                } else if s.chars().all(|c| c.is_digit(10) || c == '-' || c == '.' || c == '+' || c == ' ') && s.len() >= 7 {
                    Value::String(self.anonymize_phone(s))
                } else {
                    Value::String(self.anonymize_string(s, None))
                }
            },
            Value::Number(n) => {
                if let Some(n_f64) = n.as_f64() {
                    // Try to create a new number with the anonymized value
                    match serde_json::Number::from_f64(self.anonymize_numeric(n_f64, None)) {
                        Some(new_n) => Value::Number(new_n),
                        None => json_value.clone(),
                    }
                } else {
                    json_value.clone()
                }
            },
            // Boolean and Null values don't need anonymization
            _ => json_value.clone(),
        }
    }

    /// Anonymize text by detecting and redacting sensitive information
    pub fn anonymize_text(&self, text: &str) -> String {
        // Define patterns for sensitive information
        let patterns = [
            (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL REDACTED]"), // Email
            (r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b", "[PHONE REDACTED]"), // Phone numbers
            (r"\b\d{3}[-]?\d{2}[-]?\d{4}\b", "[SSN REDACTED]"), // SSN
            (r"\b(?:\d[ -]*?){13,16}\b", "[CREDIT CARD REDACTED]"), // Credit card numbers
            // Add more patterns as needed
        ];

        let mut anonymized_text = text.to_string();
        for (pattern, replacement) in patterns.iter() {
            // In a real implementation, use a proper regex library
            // This is a simplified version for demonstration
            if anonymized_text.contains(pattern) {
                anonymized_text = anonymized_text.replace(pattern, replacement);
            }
        }

        anonymized_text
    }

    /// Apply k-anonymity to a dataset
    pub fn apply_k_anonymity<T: Clone + Eq + Hash>(
        &self,
        data: &[HashMap<String, T>],
        quasi_identifiers: &[String],
        k: usize,
    ) -> Vec<HashMap<String, T>> {
        if data.is_empty() || quasi_identifiers.is_empty() {
            return data.to_vec();
        }

        // Group records by quasi-identifiers
        let mut groups: HashMap<Vec<T>, Vec<HashMap<String, T>>> = HashMap::new();

        for record in data {
            let mut key = Vec::new();
            for qi in quasi_identifiers {
                if let Some(value) = record.get(qi) {
                    key.push(value.clone());
                }
            }

            groups.entry(key).or_insert_with(Vec::new).push(record.clone());
        }

        // Apply generalization to groups smaller than k
        let mut result = Vec::new();

        for (_, group) in groups {
            if group.len() >= k {
                // Group satisfies k-anonymity, add as is
                result.extend(group);
            } else {
                // Group doesn't satisfy k-anonymity, generalize quasi-identifiers
                let mut generalized_group = Vec::new();

                for mut record in group {
                    for qi in quasi_identifiers {
                        if let Some(value) = record.get(qi).cloned() {
                            // Apply generalization to the quasi-identifier
                            // This is a simplified version - in a real implementation,
                            // you would use domain-specific generalization hierarchies
                            record.insert(qi.clone(), self.generalize_value(value));
                        }
                    }
                    generalized_group.push(record);
                }

                result.extend(generalized_group);
            }
        }

        result
    }

    // Helper methods

    fn get_strategy_for_field(&self, field_name: Option<&str>) -> AnonymizationStrategy {
        if let Some(field) = field_name {
            self.config.field_strategies.get(field).copied().unwrap_or(self.config.default_strategy)
        } else {
            self.config.default_strategy
        }
    }

    fn apply_partial_mask(&self, value: &str) -> String {
        if value.is_empty() {
            return String::new();
        }

        if value.len() <= 2 {
            return value.to_string();
        }

        // Keep first and last character, mask the rest
        format!("{}{}{}",
            &value[0..1],
            "*".repeat(value.len() - 2),
            &value[value.len() - 1..value.len()]
        )
    }

    fn apply_complete_mask(&self, value: &str) -> String {
        "*".repeat(value.len())
    }

    fn apply_redaction(&self, _value: &str) -> String {
        self.config.redaction_text.as_deref().unwrap_or("[REDACTED]").to_string()
    }

    fn apply_generalization_to_string(&self, value: &str) -> String {
        // For strings, generalization could mean keeping first character
        if value.is_empty() {
            return String::new();
        }

        format!("{}...", &value[0..1])
    }

    fn apply_generalization_to_numeric(&self, value: f64, field_name: Option<&str>) -> f64 {
        // Check if we have custom ranges for this field
        if let Some(field) = field_name {
            if let Some(ranges) = &self.config.generalization_ranges {
                if let Some(field_ranges) = ranges.get(field) {
                    for &(min, max) in field_ranges {
                        if value >= min && value <= max {
                            // Return the midpoint of the range
                            return (min + max) / 2.0;
                        }
                    }
                }
            }
        }

        // Default generalization: round to nearest 10
        (value / 10.0).round() * 10.0
    }

    fn apply_pseudonymization(&self, value: &str, field_name: Option<&str>) -> String {
        let key = format!("{}:{}", field_name.unwrap_or(""), value);
        let mut map = self.pseudonym_map.lock().unwrap();

        if let Some(pseudonym) = map.get(&key) {
            return pseudonym.clone();
        }

        // Generate a new pseudonym
        let pseudonym = self.generate_pseudonym(value);
        map.insert(key, pseudonym.clone());
        pseudonym
    }

    fn generate_pseudonym(&self, value: &str) -> String {
        // Generate a pseudonym with similar characteristics to the original
        let len = value.len();
        let mut rng = thread_rng();

        if self.config.preserve_format {
            // Preserve the format (uppercase, lowercase, digits)
            let mut result = String::with_capacity(len);

            for c in value.chars() {
                if c.is_uppercase() {
                    result.push(rng.sample(Alphanumeric).to_ascii_uppercase());
                } else if c.is_lowercase() {
                    result.push(rng.sample(Alphanumeric).to_ascii_lowercase());
                } else if c.is_digit(10) {
                    result.push(char::from_digit(rng.gen_range(0..10), 10).unwrap());
                } else {
                    result.push(c);
                }
            }

            result
        } else {
            // Just generate a random string of the same length
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(len)
                .map(char::from)
                .collect()
        }
    }

    fn apply_differential_privacy(&self, value: f64) -> f64 {
        // Add Laplace noise calibrated to the sensitivity and epsilon
        let sensitivity = 1.0; // Assume sensitivity of 1 for simplicity
        let scale = sensitivity / self.config.epsilon;

        value + self.sample_laplace(0.0, scale)
    }

    fn sample_laplace(&self, mu: f64, scale: f64) -> f64 {
        let u = thread_rng().gen::<f64>() - 0.5;
        mu - scale * f64::signum(u) * f64::ln(1.0 - 2.0 * f64::abs(u))
    }

    fn should_remove_field(&self, field_name: &str) -> bool {
        // List of fields that should be completely removed
        let sensitive_fields = [
            "password", "password_hash", "secret", "token", "api_key", "private_key",
            "ssn", "social_security", "credit_card", "bank_account"
        ];

        sensitive_fields.contains(&field_name)
    }

    fn generalize_value<T: Clone>(&self, value: T) -> T {
        // This is a placeholder - in a real implementation, you would have
        // domain-specific generalization hierarchies
        value
    }
}

/// Utility functions for anonymization
pub mod utils {
    use super::*;
    use std::cmp::min;

    /// Anonymize an email address with configurable options
    pub fn anonymize_email(email: &str, preserve_domain: bool) -> String {
        if email.is_empty() || !email.contains('@') {
            return "*".repeat(email.len().max(1));
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return "*".repeat(email.len().max(1));
        }

        let username = parts[0];
        let domain = parts[1];

        if preserve_domain {
            format!("{}@{}", "*".repeat(username.len().max(1)), domain)
        } else {
            "*".repeat(email.len().max(1))
        }
    }

    /// Anonymize a phone number with configurable options
    pub fn anonymize_phone(phone: &str, preserve_last_digits: usize) -> String {
        let digits_to_preserve = min(preserve_last_digits, phone.len());

        if digits_to_preserve == 0 {
            return "*".repeat(phone.len().max(1));
        }

        format!("{}{}",
            "*".repeat(phone.len() - digits_to_preserve),
            &phone[phone.len() - digits_to_preserve..]
        )
    }

    /// Anonymize a name with configurable options
    pub fn anonymize_name(name: &str, preserve_initials: bool) -> String {
        if name.is_empty() {
            return String::new();
        }

        if preserve_initials {
            format!("{}.", &name[0..1])
        } else {
            "*".repeat(name.len())
        }
    }

    /// Detect and redact sensitive information in text
    pub fn redact_sensitive_info(text: &str) -> String {
        // Define patterns for sensitive information
        let patterns = [
            (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "[EMAIL REDACTED]"), // Email
            (r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b", "[PHONE REDACTED]"), // Phone numbers
            (r"\b\d{3}[-]?\d{2}[-]?\d{4}\b", "[SSN REDACTED]"), // SSN
            (r"\b(?:\d[ -]*?){13,16}\b", "[CREDIT CARD REDACTED]"), // Credit card numbers
            // Add more patterns as needed
        ];

        let mut anonymized_text = text.to_string();
        for (pattern, replacement) in patterns.iter() {
            // In a real implementation, use a proper regex library
            // This is a simplified version for demonstration
            if anonymized_text.contains(pattern) {
                anonymized_text = anonymized_text.replace(pattern, replacement);
            }
        }

        anonymized_text
    }

    /// Add noise to a numeric value (differential privacy)
    pub fn add_noise(value: f64, epsilon: f64) -> f64 {
        let sensitivity = 1.0; // Assume sensitivity of 1 for simplicity
        let scale = sensitivity / epsilon;

        let u = thread_rng().gen::<f64>() - 0.5;
        value - scale * f64::signum(u) * f64::ln(1.0 - 2.0 * f64::abs(u))
    }

    /// Generalize a date to a less precise form
    pub fn generalize_date(date: &DateTime<Utc>, precision: &str) -> DateTime<Utc> {
        let date_only = date.date();

        match precision {
            "year" => {
                // Keep only the year
                date_only.with_month(1).unwrap().with_day(1).unwrap().and_hms(0, 0, 0)
            },
            "month" => {
                // Keep year and month
                date_only.with_day(1).unwrap().and_hms(0, 0, 0)
            },
            "day" => {
                // Keep year, month, day
                date_only.and_hms(0, 0, 0)
            },
            "hour" => {
                // Keep year, month, day, hour
                date_only.and_hms(date.hour(), 0, 0)
            },
            _ => {
                // Default to day precision
                date_only.and_hms(0, 0, 0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::min;

    #[test]
    fn test_anonymize_email() {
        let anonymizer = Anonymizer::default();

        // Test partial masking (default)
        assert_eq!(anonymizer.anonymize_email("john.doe@example.com"), "j********@example.com");
        assert_eq!(anonymizer.anonymize_email("a@b.com"), "a@b.com");

        // Test with different strategies
        let config = AnonymizationConfig {
            default_strategy: AnonymizationStrategy::CompleteMask,
            field_strategies: HashMap::new(),
            ..Default::default()
        };
        let anonymizer = Anonymizer::new(config);
        assert_eq!(anonymizer.anonymize_email("john.doe@example.com"), "********@example.com");

        // Test utility function
        assert_eq!(utils::anonymize_email("john.doe@example.com", true), "********@example.com");
        assert_eq!(utils::anonymize_email("john.doe@example.com", false), "********************");
    }

    #[test]
    fn test_anonymize_phone() {
        let anonymizer = Anonymizer::default();

        // Test partial masking (default)
        assert_eq!(anonymizer.anonymize_phone("123-456-7890"), "*******7890");
        assert_eq!(anonymizer.anonymize_phone("1234"), "1234");

        // Test with different strategies
        let config = AnonymizationConfig {
            default_strategy: AnonymizationStrategy::Redaction,
            field_strategies: HashMap::new(),
            ..Default::default()
        };
        let anonymizer = Anonymizer::new(config);
        assert_eq!(anonymizer.anonymize_phone("123-456-7890"), "[REDACTED]");

        // Test utility function
        assert_eq!(utils::anonymize_phone("123-456-7890", 4), "*******7890");
        assert_eq!(utils::anonymize_phone("123-456-7890", 0), "************");
    }

    #[test]
    fn test_anonymize_json() {
        let anonymizer = Anonymizer::default();

        let json_value = json!({
            "name": "John Doe",
            "email": "john.doe@example.com",
            "phone": "123-456-7890",
            "age": 35,
            "address": {
                "street": "123 Main St",
                "city": "Anytown",
                "zip": "12345"
            },
            "credit_card": "4111-1111-1111-1111"
        });

        let anonymized = anonymizer.anonymize_json(&json_value);

        // Check that sensitive fields are anonymized
        if let Value::Object(map) = &anonymized {
            if let Some(Value::String(email)) = map.get("email") {
                assert_eq!(email, "j********@example.com");
            } else {
                panic!("Email field not found or not a string");
            }

            if let Some(Value::String(phone)) = map.get("phone") {
                assert_eq!(phone, "*******7890");
            } else {
                panic!("Phone field not found or not a string");
            }

            // Credit card should be completely masked or redacted
            if let Some(Value::String(cc)) = map.get("credit_card") {
                assert!(cc.contains("*") || cc.contains("REDACTED"));
            } else {
                panic!("Credit card field not found or not a string");
            }
        } else {
            panic!("Anonymized value is not an object");
        }
    }

    #[test]
    fn test_anonymize_text() {
        let text = "Please contact john.doe@example.com or call 123-456-7890. My SSN is 123-45-6789.";

        let anonymized = utils::redact_sensitive_info(text);

        assert!(anonymized.contains("[EMAIL REDACTED]"));
        assert!(anonymized.contains("[PHONE REDACTED]"));
        assert!(anonymized.contains("[SSN REDACTED]"));
        assert!(!anonymized.contains("john.doe@example.com"));
        assert!(!anonymized.contains("123-456-7890"));
        assert!(!anonymized.contains("123-45-6789"));
    }

    #[test]
    fn test_differential_privacy() {
        let config = AnonymizationConfig {
            default_strategy: AnonymizationStrategy::DifferentialPrivacy,
            epsilon: 1.0,
            ..Default::default()
        };
        let anonymizer = Anonymizer::new(config);

        let original = 100.0;
        let anonymized = anonymizer.anonymize_numeric(original, None);

        // The anonymized value should be different but within a reasonable range
        assert_ne!(original, anonymized);
        assert!((original - anonymized).abs() < 10.0); // This is probabilistic

        // Test utility function
        let noisy = utils::add_noise(original, 1.0);
        assert_ne!(original, noisy);
    }

    #[test]
    fn test_k_anonymity() {
        let anonymizer = Anonymizer::default();

        // Create a test dataset
        let mut data = Vec::new();
        for i in 0..10 {
            let mut record = HashMap::new();
            record.insert("age".to_string(), (20 + i) as i32);
            record.insert("zipcode".to_string(), 12345 + i as i32);
            record.insert("gender".to_string(), if i % 2 == 0 { "M" } else { "F" });
            record.insert("salary".to_string(), 50000 + i * 1000 as i32);
            data.push(record);
        }

        // Apply k-anonymity with k=3
        let quasi_identifiers = vec!["age".to_string(), "zipcode".to_string(), "gender".to_string()];
        let anonymized = anonymizer.apply_k_anonymity(&data, &quasi_identifiers, 3);

        // Count the number of records with the same quasi-identifiers
        let mut groups = HashMap::new();
        for record in &anonymized {
            let mut key = Vec::new();
            for qi in &quasi_identifiers {
                key.push(record.get(qi).unwrap().clone());
            }
            *groups.entry(key).or_insert(0) += 1;
        }

        // Each group should have at least k records
        for (_, count) in groups {
            assert!(count >= 3);
        }
    }
}
