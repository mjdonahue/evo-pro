//! Progressive enhancement functionality
//!
//! This module provides functionality for enabling enhanced features
//! on capable devices, improving the user experience when resources permit.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

use super::adaptation::{ResourceProfile, AdaptationStrategy, ResourceManager};

/// Enhanced feature that can be enabled on capable devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnhancedFeature {
    /// High-resolution textures and images
    HighResolutionAssets,
    
    /// Advanced visual effects (shadows, reflections, etc.)
    AdvancedVisualEffects,
    
    /// Real-time data updates
    RealTimeUpdates,
    
    /// Background data processing
    BackgroundProcessing,
    
    /// Predictive prefetching of data
    PredictivePrefetching,
    
    /// Advanced analytics and metrics
    AdvancedAnalytics,
    
    /// Multi-threaded operations
    MultiThreadedOperations,
    
    /// Hardware acceleration
    HardwareAcceleration,
    
    /// Custom enhanced feature
    Custom(String),
}

impl EnhancedFeature {
    /// Get a human-readable name for the feature
    pub fn name(&self) -> String {
        match self {
            EnhancedFeature::HighResolutionAssets => "High-Resolution Assets".to_string(),
            EnhancedFeature::AdvancedVisualEffects => "Advanced Visual Effects".to_string(),
            EnhancedFeature::RealTimeUpdates => "Real-Time Updates".to_string(),
            EnhancedFeature::BackgroundProcessing => "Background Processing".to_string(),
            EnhancedFeature::PredictivePrefetching => "Predictive Prefetching".to_string(),
            EnhancedFeature::AdvancedAnalytics => "Advanced Analytics".to_string(),
            EnhancedFeature::MultiThreadedOperations => "Multi-Threaded Operations".to_string(),
            EnhancedFeature::HardwareAcceleration => "Hardware Acceleration".to_string(),
            EnhancedFeature::Custom(name) => format!("Custom: {}", name),
        }
    }
    
    /// Get the minimum resource profile required for this feature
    pub fn minimum_profile(&self) -> ResourceProfile {
        match self {
            EnhancedFeature::HighResolutionAssets => ResourceProfile::MidRange,
            EnhancedFeature::AdvancedVisualEffects => ResourceProfile::HighEnd,
            EnhancedFeature::RealTimeUpdates => ResourceProfile::MidRange,
            EnhancedFeature::BackgroundProcessing => ResourceProfile::LowEnd,
            EnhancedFeature::PredictivePrefetching => ResourceProfile::MidRange,
            EnhancedFeature::AdvancedAnalytics => ResourceProfile::MidRange,
            EnhancedFeature::MultiThreadedOperations => ResourceProfile::MidRange,
            EnhancedFeature::HardwareAcceleration => ResourceProfile::MidRange,
            EnhancedFeature::Custom(_) => ResourceProfile::HighEnd, // Default to high-end for custom features
        }
    }
    
    /// Check if this feature is available for a given resource profile
    pub fn is_available_for(&self, profile: ResourceProfile) -> bool {
        // Order profiles from lowest to highest capability
        let profile_rank = match profile {
            ResourceProfile::Mobile => 0,
            ResourceProfile::LowEnd => 1,
            ResourceProfile::BatteryPowered => 1, // Same as low-end
            ResourceProfile::LimitedConnectivity => 1, // Same as low-end
            ResourceProfile::MidRange => 2,
            ResourceProfile::HighEnd => 3,
            ResourceProfile::Custom(_) => 2, // Default to mid-range for custom profiles
        };
        
        let min_profile_rank = match self.minimum_profile() {
            ResourceProfile::Mobile => 0,
            ResourceProfile::LowEnd => 1,
            ResourceProfile::BatteryPowered => 1,
            ResourceProfile::LimitedConnectivity => 1,
            ResourceProfile::MidRange => 2,
            ResourceProfile::HighEnd => 3,
            ResourceProfile::Custom(_) => 2,
        };
        
        profile_rank >= min_profile_rank
    }
}

/// Progressive enhancement manager
pub struct EnhancementManager {
    /// Enabled enhanced features
    enabled_features: HashMap<EnhancedFeature, bool>,
    
    /// Feature configuration parameters
    feature_params: HashMap<EnhancedFeature, HashMap<String, String>>,
}

impl EnhancementManager {
    /// Create a new enhancement manager
    pub fn new() -> Self {
        let mut enabled_features = HashMap::new();
        let feature_params = HashMap::new();
        
        // Initialize all features as disabled
        for feature in Self::all_features() {
            enabled_features.insert(feature, false);
        }
        
        Self {
            enabled_features,
            feature_params,
        }
    }
    
    /// Get all available enhanced features
    pub fn all_features() -> Vec<EnhancedFeature> {
        vec![
            EnhancedFeature::HighResolutionAssets,
            EnhancedFeature::AdvancedVisualEffects,
            EnhancedFeature::RealTimeUpdates,
            EnhancedFeature::BackgroundProcessing,
            EnhancedFeature::PredictivePrefetching,
            EnhancedFeature::AdvancedAnalytics,
            EnhancedFeature::MultiThreadedOperations,
            EnhancedFeature::HardwareAcceleration,
        ]
    }
    
    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: EnhancedFeature) -> bool {
        *self.enabled_features.get(&feature).unwrap_or(&false)
    }
    
    /// Enable a feature
    pub fn enable_feature(&mut self, feature: EnhancedFeature) {
        self.enabled_features.insert(feature, true);
        info!("Enhanced feature enabled: {}", feature.name());
    }
    
    /// Disable a feature
    pub fn disable_feature(&mut self, feature: EnhancedFeature) {
        self.enabled_features.insert(feature, false);
        info!("Enhanced feature disabled: {}", feature.name());
    }
    
    /// Set a feature parameter
    pub fn set_feature_param(&mut self, feature: EnhancedFeature, key: impl Into<String>, value: impl Into<String>) {
        let params = self.feature_params.entry(feature).or_insert_with(HashMap::new);
        params.insert(key.into(), value.into());
    }
    
    /// Get a feature parameter
    pub fn get_feature_param(&self, feature: EnhancedFeature, key: &str) -> Option<&String> {
        self.feature_params.get(&feature).and_then(|params| params.get(key))
    }
    
    /// Update enabled features based on the current resource profile
    pub fn update_for_profile(&mut self, profile: ResourceProfile) {
        info!("Updating enhanced features for profile: {:?}", profile);
        
        for feature in Self::all_features() {
            let available = feature.is_available_for(profile);
            
            if available {
                self.enable_feature(feature);
            } else {
                self.disable_feature(feature);
            }
        }
    }
}

/// Global enhancement manager instance
lazy_static::lazy_static! {
    static ref GLOBAL_ENHANCEMENT_MANAGER: std::sync::RwLock<EnhancementManager> = 
        std::sync::RwLock::new(EnhancementManager::new());
}

/// Initialize the enhancement system
pub fn initialize() {
    // Register a listener for adaptation changes
    super::adaptation::add_adaptation_listener(|strategy| {
        let profile = strategy.profile;
        
        // Update the enhancement manager for the new profile
        if let Ok(mut manager) = GLOBAL_ENHANCEMENT_MANAGER.write() {
            manager.update_for_profile(profile);
        } else {
            warn!("Failed to update enhancement manager for profile: {:?}", profile);
        }
    });
    
    // Perform initial update based on current profile
    if let Ok(profile) = super::adaptation::get_current_profile() {
        if let Ok(mut manager) = GLOBAL_ENHANCEMENT_MANAGER.write() {
            manager.update_for_profile(profile);
        }
    }
    
    info!("Progressive enhancement system initialized");
}

/// Check if an enhanced feature is enabled
pub fn is_feature_enabled(feature: EnhancedFeature) -> bool {
    if let Ok(manager) = GLOBAL_ENHANCEMENT_MANAGER.read() {
        manager.is_feature_enabled(feature)
    } else {
        warn!("Failed to read enhancement manager state");
        false
    }
}

/// Get all enabled enhanced features
pub fn get_enabled_features() -> Vec<EnhancedFeature> {
    if let Ok(manager) = GLOBAL_ENHANCEMENT_MANAGER.read() {
        EnhancementManager::all_features()
            .into_iter()
            .filter(|feature| manager.is_feature_enabled(*feature))
            .collect()
    } else {
        warn!("Failed to read enhancement manager state");
        Vec::new()
    }
}

/// Force enable an enhanced feature (for testing or user preference)
pub fn force_enable_feature(feature: EnhancedFeature) {
    if let Ok(mut manager) = GLOBAL_ENHANCEMENT_MANAGER.write() {
        manager.enable_feature(feature);
    } else {
        warn!("Failed to write to enhancement manager");
    }
}

/// Force disable an enhanced feature (for testing or user preference)
pub fn force_disable_feature(feature: EnhancedFeature) {
    if let Ok(mut manager) = GLOBAL_ENHANCEMENT_MANAGER.write() {
        manager.disable_feature(feature);
    } else {
        warn!("Failed to write to enhancement manager");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature_availability() {
        // High-end profile should have all features available
        assert!(EnhancedFeature::HighResolutionAssets.is_available_for(ResourceProfile::HighEnd));
        assert!(EnhancedFeature::AdvancedVisualEffects.is_available_for(ResourceProfile::HighEnd));
        assert!(EnhancedFeature::RealTimeUpdates.is_available_for(ResourceProfile::HighEnd));
        
        // Mid-range profile should have most features available
        assert!(EnhancedFeature::HighResolutionAssets.is_available_for(ResourceProfile::MidRange));
        assert!(!EnhancedFeature::AdvancedVisualEffects.is_available_for(ResourceProfile::MidRange));
        assert!(EnhancedFeature::RealTimeUpdates.is_available_for(ResourceProfile::MidRange));
        
        // Low-end profile should have limited features available
        assert!(!EnhancedFeature::HighResolutionAssets.is_available_for(ResourceProfile::LowEnd));
        assert!(!EnhancedFeature::AdvancedVisualEffects.is_available_for(ResourceProfile::LowEnd));
        assert!(!EnhancedFeature::RealTimeUpdates.is_available_for(ResourceProfile::LowEnd));
        assert!(EnhancedFeature::BackgroundProcessing.is_available_for(ResourceProfile::LowEnd));
    }
    
    #[test]
    fn test_enhancement_manager() {
        let mut manager = EnhancementManager::new();
        
        // Initially all features should be disabled
        assert!(!manager.is_feature_enabled(EnhancedFeature::HighResolutionAssets));
        
        // Enable a feature
        manager.enable_feature(EnhancedFeature::HighResolutionAssets);
        assert!(manager.is_feature_enabled(EnhancedFeature::HighResolutionAssets));
        
        // Update for high-end profile should enable most features
        manager.update_for_profile(ResourceProfile::HighEnd);
        assert!(manager.is_feature_enabled(EnhancedFeature::HighResolutionAssets));
        assert!(manager.is_feature_enabled(EnhancedFeature::AdvancedVisualEffects));
        
        // Update for low-end profile should disable most features
        manager.update_for_profile(ResourceProfile::LowEnd);
        assert!(!manager.is_feature_enabled(EnhancedFeature::HighResolutionAssets));
        assert!(!manager.is_feature_enabled(EnhancedFeature::AdvancedVisualEffects));
        assert!(manager.is_feature_enabled(EnhancedFeature::BackgroundProcessing));
    }
}