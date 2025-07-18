//! Fallback strategies for constrained environments
//!
//! This module provides fallback implementations for resource-intensive features
//! that can be used in constrained environments to maintain functionality with
//! reduced resource usage.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

use super::adaptation::{ResourceProfile, AdaptationStrategy};
use super::enhancement::EnhancedFeature;

/// Fallback strategy for a resource-intensive feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStrategy {
    /// Name of the fallback strategy
    pub name: String,
    
    /// Description of the fallback strategy
    pub description: String,
    
    /// The feature this strategy provides a fallback for
    pub feature: EnhancedFeature,
    
    /// Resource usage reduction percentage (0-100)
    pub resource_reduction: u8,
    
    /// Functionality preservation percentage (0-100)
    pub functionality_preservation: u8,
    
    /// Custom parameters for this strategy
    pub custom_params: HashMap<String, String>,
}

impl FallbackStrategy {
    /// Create a new fallback strategy
    pub fn new(feature: EnhancedFeature, name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            feature,
            resource_reduction: 50, // Default 50% resource reduction
            functionality_preservation: 80, // Default 80% functionality preservation
            custom_params: HashMap::new(),
        }
    }
    
    /// Set the resource reduction percentage
    pub fn with_resource_reduction(mut self, percentage: u8) -> Self {
        self.resource_reduction = percentage.min(100);
        self
    }
    
    /// Set the functionality preservation percentage
    pub fn with_functionality_preservation(mut self, percentage: u8) -> Self {
        self.functionality_preservation = percentage.min(100);
        self
    }
    
    /// Add a custom parameter to the strategy
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_params.insert(key.into(), value.into());
        self
    }
}

/// Fallback manager for managing fallback strategies
pub struct FallbackManager {
    /// Available fallback strategies
    strategies: HashMap<EnhancedFeature, Vec<FallbackStrategy>>,
    
    /// Currently active fallback strategies
    active_strategies: HashMap<EnhancedFeature, FallbackStrategy>,
}

impl FallbackManager {
    /// Create a new fallback manager
    pub fn new() -> Self {
        let mut manager = Self {
            strategies: HashMap::new(),
            active_strategies: HashMap::new(),
        };
        
        // Register default fallback strategies
        manager.register_default_strategies();
        
        manager
    }
    
    /// Register default fallback strategies
    fn register_default_strategies(&mut self) {
        // High Resolution Assets fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::HighResolutionAssets,
                "Medium Resolution Assets",
                "Use medium resolution textures and images to reduce memory usage"
            )
            .with_resource_reduction(50)
            .with_functionality_preservation(90)
        );
        
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::HighResolutionAssets,
                "Low Resolution Assets",
                "Use low resolution textures and images to minimize memory usage"
            )
            .with_resource_reduction(80)
            .with_functionality_preservation(70)
        );
        
        // Advanced Visual Effects fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::AdvancedVisualEffects,
                "Basic Visual Effects",
                "Use simplified visual effects with reduced complexity"
            )
            .with_resource_reduction(60)
            .with_functionality_preservation(85)
        );
        
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::AdvancedVisualEffects,
                "Minimal Visual Effects",
                "Use only essential visual effects with minimal resource usage"
            )
            .with_resource_reduction(90)
            .with_functionality_preservation(60)
        );
        
        // Real-Time Updates fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::RealTimeUpdates,
                "Periodic Updates",
                "Update data at regular intervals instead of in real-time"
            )
            .with_resource_reduction(70)
            .with_functionality_preservation(85)
            .with_param("update_interval_seconds", "30")
        );
        
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::RealTimeUpdates,
                "Manual Updates",
                "Update data only when explicitly requested by the user"
            )
            .with_resource_reduction(95)
            .with_functionality_preservation(50)
        );
        
        // Background Processing fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::BackgroundProcessing,
                "Limited Background Processing",
                "Run background tasks with reduced frequency and priority"
            )
            .with_resource_reduction(60)
            .with_functionality_preservation(80)
            .with_param("max_concurrent_tasks", "1")
            .with_param("task_interval_seconds", "60")
        );
        
        // Predictive Prefetching fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::PredictivePrefetching,
                "Basic Prefetching",
                "Prefetch only the most commonly accessed data"
            )
            .with_resource_reduction(70)
            .with_functionality_preservation(75)
            .with_param("prefetch_limit", "10")
        );
        
        // Advanced Analytics fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::AdvancedAnalytics,
                "Basic Analytics",
                "Collect and process only essential analytics data"
            )
            .with_resource_reduction(80)
            .with_functionality_preservation(70)
        );
        
        // Multi-Threaded Operations fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::MultiThreadedOperations,
                "Limited Threading",
                "Use a reduced number of threads for parallel operations"
            )
            .with_resource_reduction(50)
            .with_functionality_preservation(85)
            .with_param("max_threads", "2")
        );
        
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::MultiThreadedOperations,
                "Single-Threaded Operations",
                "Use single-threaded operations to minimize resource usage"
            )
            .with_resource_reduction(90)
            .with_functionality_preservation(60)
        );
        
        // Hardware Acceleration fallbacks
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::HardwareAcceleration,
                "Selective Hardware Acceleration",
                "Use hardware acceleration only for critical operations"
            )
            .with_resource_reduction(60)
            .with_functionality_preservation(80)
        );
        
        self.register_strategy(
            FallbackStrategy::new(
                EnhancedFeature::HardwareAcceleration,
                "Software Rendering",
                "Use software rendering instead of hardware acceleration"
            )
            .with_resource_reduction(100)
            .with_functionality_preservation(50)
        );
    }
    
    /// Register a fallback strategy
    pub fn register_strategy(&mut self, strategy: FallbackStrategy) {
        let feature = strategy.feature;
        let strategies = self.strategies.entry(feature).or_insert_with(Vec::new);
        strategies.push(strategy);
    }
    
    /// Get all fallback strategies for a feature
    pub fn get_strategies_for_feature(&self, feature: EnhancedFeature) -> Vec<FallbackStrategy> {
        self.strategies.get(&feature).cloned().unwrap_or_default()
    }
    
    /// Get the active fallback strategy for a feature
    pub fn get_active_strategy(&self, feature: EnhancedFeature) -> Option<&FallbackStrategy> {
        self.active_strategies.get(&feature)
    }
    
    /// Activate the most appropriate fallback strategy for a feature based on the resource profile
    pub fn activate_fallback(&mut self, feature: EnhancedFeature, profile: ResourceProfile) {
        let strategies = self.get_strategies_for_feature(feature);
        if strategies.is_empty() {
            return;
        }
        
        // Select the appropriate strategy based on the resource profile
        let strategy = match profile {
            ResourceProfile::Mobile | ResourceProfile::LimitedConnectivity => {
                // For very constrained environments, use the strategy with the highest resource reduction
                strategies.iter()
                    .max_by_key(|s| s.resource_reduction)
                    .unwrap()
                    .clone()
            },
            ResourceProfile::LowEnd | ResourceProfile::BatteryPowered => {
                // For low-end devices, balance resource reduction and functionality
                strategies.iter()
                    .max_by_key(|s| s.resource_reduction + s.functionality_preservation)
                    .unwrap()
                    .clone()
            },
            _ => {
                // For mid-range and high-end devices, prioritize functionality
                strategies.iter()
                    .max_by_key(|s| s.functionality_preservation)
                    .unwrap()
                    .clone()
            },
        };
        
        info!("Activating fallback strategy for {}: {}", feature.name(), strategy.name);
        self.active_strategies.insert(feature, strategy);
    }
    
    /// Deactivate the fallback strategy for a feature
    pub fn deactivate_fallback(&mut self, feature: EnhancedFeature) {
        if self.active_strategies.remove(&feature).is_some() {
            info!("Deactivated fallback strategy for {}", feature.name());
        }
    }
    
    /// Update fallback strategies based on the current resource profile
    pub fn update_for_profile(&mut self, profile: ResourceProfile, strategy: &AdaptationStrategy) {
        info!("Updating fallback strategies for profile: {:?}", profile);
        
        for feature in super::enhancement::EnhancementManager::all_features() {
            // If the feature is available for this profile, deactivate any fallbacks
            if feature.is_available_for(profile) {
                self.deactivate_fallback(feature);
            } else {
                // Otherwise, activate an appropriate fallback
                self.activate_fallback(feature, profile);
            }
        }
    }
}

/// Global fallback manager instance
lazy_static::lazy_static! {
    static ref GLOBAL_FALLBACK_MANAGER: std::sync::RwLock<FallbackManager> = 
        std::sync::RwLock::new(FallbackManager::new());
}

/// Initialize the fallback system
pub fn initialize() {
    // Register a listener for adaptation changes
    super::adaptation::add_adaptation_listener(|strategy| {
        let profile = strategy.profile;
        
        // Update the fallback manager for the new profile
        if let Ok(mut manager) = GLOBAL_FALLBACK_MANAGER.write() {
            manager.update_for_profile(profile, strategy);
        } else {
            warn!("Failed to update fallback manager for profile: {:?}", profile);
        }
    });
    
    // Perform initial update based on current profile
    if let Ok(profile) = super::adaptation::get_current_profile() {
        if let Ok(strategy) = super::adaptation::get_current_strategy() {
            if let Ok(mut manager) = GLOBAL_FALLBACK_MANAGER.write() {
                manager.update_for_profile(profile, &strategy);
            }
        }
    }
    
    info!("Fallback strategy system initialized");
}

/// Check if a fallback strategy is active for a feature
pub fn is_fallback_active(feature: EnhancedFeature) -> bool {
    if let Ok(manager) = GLOBAL_FALLBACK_MANAGER.read() {
        manager.get_active_strategy(feature).is_some()
    } else {
        warn!("Failed to read fallback manager state");
        false
    }
}

/// Get the active fallback strategy for a feature
pub fn get_active_fallback(feature: EnhancedFeature) -> Option<FallbackStrategy> {
    if let Ok(manager) = GLOBAL_FALLBACK_MANAGER.read() {
        manager.get_active_strategy(feature).cloned()
    } else {
        warn!("Failed to read fallback manager state");
        None
    }
}

/// Get all active fallback strategies
pub fn get_active_fallbacks() -> HashMap<EnhancedFeature, FallbackStrategy> {
    if let Ok(manager) = GLOBAL_FALLBACK_MANAGER.read() {
        manager.active_strategies.clone()
    } else {
        warn!("Failed to read fallback manager state");
        HashMap::new()
    }
}

/// Force activate a specific fallback strategy for a feature
pub fn force_activate_fallback(feature: EnhancedFeature, strategy_name: &str) -> bool {
    if let Ok(mut manager) = GLOBAL_FALLBACK_MANAGER.write() {
        let strategies = manager.get_strategies_for_feature(feature);
        if let Some(strategy) = strategies.iter().find(|s| s.name == strategy_name) {
            manager.active_strategies.insert(feature, strategy.clone());
            info!("Forced activation of fallback strategy for {}: {}", feature.name(), strategy_name);
            true
        } else {
            warn!("Fallback strategy not found: {}", strategy_name);
            false
        }
    } else {
        warn!("Failed to write to fallback manager");
        false
    }
}

/// Force deactivate the fallback strategy for a feature
pub fn force_deactivate_fallback(feature: EnhancedFeature) {
    if let Ok(mut manager) = GLOBAL_FALLBACK_MANAGER.write() {
        manager.deactivate_fallback(feature);
    } else {
        warn!("Failed to write to fallback manager");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fallback_strategy_creation() {
        let strategy = FallbackStrategy::new(
            EnhancedFeature::HighResolutionAssets,
            "Test Strategy",
            "A test strategy"
        )
        .with_resource_reduction(75)
        .with_functionality_preservation(60)
        .with_param("test_param", "test_value");
        
        assert_eq!(strategy.name, "Test Strategy");
        assert_eq!(strategy.feature, EnhancedFeature::HighResolutionAssets);
        assert_eq!(strategy.resource_reduction, 75);
        assert_eq!(strategy.functionality_preservation, 60);
        assert_eq!(strategy.custom_params.get("test_param"), Some(&"test_value".to_string()));
    }
    
    #[test]
    fn test_fallback_manager() {
        let mut manager = FallbackManager::new();
        
        // Test that default strategies are registered
        let high_res_strategies = manager.get_strategies_for_feature(EnhancedFeature::HighResolutionAssets);
        assert!(!high_res_strategies.is_empty());
        
        // Test activating a fallback
        manager.activate_fallback(EnhancedFeature::HighResolutionAssets, ResourceProfile::Mobile);
        assert!(manager.get_active_strategy(EnhancedFeature::HighResolutionAssets).is_some());
        
        // Test deactivating a fallback
        manager.deactivate_fallback(EnhancedFeature::HighResolutionAssets);
        assert!(manager.get_active_strategy(EnhancedFeature::HighResolutionAssets).is_none());
    }
}