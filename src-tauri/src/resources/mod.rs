//! Resource detection and adaptation module
//!
//! This module provides functionality for detecting system resources and adapting
//! application behavior based on available resources.

mod detection;
mod adaptation;
mod enhancement;
mod fallback;
#[cfg(test)]
mod tests;

pub use detection::{SystemResources, ResourceDetector};
pub use adaptation::{ResourceProfile, AdaptationStrategy, ResourceManager};
pub use enhancement::{EnhancedFeature, is_feature_enabled, get_enabled_features, force_enable_feature, force_disable_feature};
pub use fallback::{FallbackStrategy, is_fallback_active, get_active_fallback, get_active_fallbacks, force_activate_fallback, force_deactivate_fallback};

/// Initialize the resource detection and adaptation system
pub fn initialize() {
    // Initialize the resource detector and manager
    let _ = ResourceManager::global();

    // Initialize the progressive enhancement system
    enhancement::initialize();

    // Initialize the fallback strategy system
    fallback::initialize();
}
