//! Security Framework
//!
//! This module provides security-related utilities and services for the application,
//! including threat modeling, security testing, and secure defaults.

pub mod threat_modeling;
pub mod secure_defaults;

// Re-export commonly used items for convenience
pub use threat_modeling::{
    ThreatModel, Threat, ThreatSeverity, ThreatCategory, ThreatModelingService,
};
pub use secure_defaults::{
    SecureDefaultsConfig, SecureDefaultsService,
    DatabaseSecureDefaults, NetworkSecureDefaults, AuthenticationSecureDefaults,
    InputValidationSecureDefaults, FileOperationSecureDefaults,
};
