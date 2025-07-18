//! Privacy-related utilities and services
//!
//! This module contains utilities and services for privacy-preserving data handling,
//! including anonymization, data minimization, privacy policy enforcement, and privacy-preserving analytics.

pub mod anonymization;
pub mod policy;

// Re-export commonly used items for convenience
pub use anonymization::{
    Anonymizer, AnonymizationConfig, AnonymizationStrategy,
};
pub use policy::{
    PolicyRule, PolicyEnforcer, PolicyEnforcementResult,
};
