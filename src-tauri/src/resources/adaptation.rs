//! Resource adaptation functionality
//!
//! This module provides functionality for adapting application behavior based on
//! available system resources.

use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use tracing::{debug, error, info, warn};

use crate::error::Result;
use super::detection::{SystemResources, get_system_resources};

/// Resource profile representing the resource capabilities of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceProfile {
    /// High-end system with abundant resources
    HighEnd,
    
    /// Mid-range system with moderate resources
    MidRange,
    
    /// Low-end system with limited resources
    LowEnd,
    
    /// Mobile device with very limited resources
    Mobile,
    
    /// System running on battery power
    BatteryPowered,
    
    /// System with limited network connectivity
    LimitedConnectivity,
    
    /// Custom profile with specific constraints
    Custom(String),
}

impl ResourceProfile {
    /// Get a human-readable name for the profile
    pub fn name(&self) -> String {
        match self {
            ResourceProfile::HighEnd => "High-End".to_string(),
            ResourceProfile::MidRange => "Mid-Range".to_string(),
            ResourceProfile::LowEnd => "Low-End".to_string(),
            ResourceProfile::Mobile => "Mobile".to_string(),
            ResourceProfile::BatteryPowered => "Battery Powered".to_string(),
            ResourceProfile::LimitedConnectivity => "Limited Connectivity".to_string(),
            ResourceProfile::Custom(name) => format!("Custom: {}", name),
        }
    }
    
    /// Get the recommended batch size for this profile
    pub fn recommended_batch_size(&self) -> usize {
        match self {
            ResourceProfile::HighEnd => 1000,
            ResourceProfile::MidRange => 500,
            ResourceProfile::LowEnd => 100,
            ResourceProfile::Mobile => 50,
            ResourceProfile::BatteryPowered => 50,
            ResourceProfile::LimitedConnectivity => 20,
            ResourceProfile::Custom(_) => 100, // Default for custom profiles
        }
    }
    
    /// Get the recommended cache size for this profile (in MB)
    pub fn recommended_cache_size_mb(&self) -> usize {
        match self {
            ResourceProfile::HighEnd => 1024, // 1 GB
            ResourceProfile::MidRange => 512, // 512 MB
            ResourceProfile::LowEnd => 128, // 128 MB
            ResourceProfile::Mobile => 64, // 64 MB
            ResourceProfile::BatteryPowered => 128, // 128 MB
            ResourceProfile::LimitedConnectivity => 256, // 256 MB
            ResourceProfile::Custom(_) => 256, // Default for custom profiles
        }
    }
    
    /// Get the recommended number of worker threads for this profile
    pub fn recommended_worker_threads(&self) -> usize {
        match self {
            ResourceProfile::HighEnd => 8,
            ResourceProfile::MidRange => 4,
            ResourceProfile::LowEnd => 2,
            ResourceProfile::Mobile => 1,
            ResourceProfile::BatteryPowered => 2,
            ResourceProfile::LimitedConnectivity => 2,
            ResourceProfile::Custom(_) => 2, // Default for custom profiles
        }
    }
    
    /// Get the recommended polling interval for this profile
    pub fn recommended_polling_interval(&self) -> Duration {
        match self {
            ResourceProfile::HighEnd => Duration::from_secs(1),
            ResourceProfile::MidRange => Duration::from_secs(2),
            ResourceProfile::LowEnd => Duration::from_secs(5),
            ResourceProfile::Mobile => Duration::from_secs(10),
            ResourceProfile::BatteryPowered => Duration::from_secs(15),
            ResourceProfile::LimitedConnectivity => Duration::from_secs(30),
            ResourceProfile::Custom(_) => Duration::from_secs(5), // Default for custom profiles
        }
    }
    
    /// Get the recommended compression level for this profile (0-9)
    pub fn recommended_compression_level(&self) -> u32 {
        match self {
            ResourceProfile::HighEnd => 1, // Minimal compression, faster
            ResourceProfile::MidRange => 3,
            ResourceProfile::LowEnd => 6,
            ResourceProfile::Mobile => 9, // Maximum compression, slower but saves bandwidth
            ResourceProfile::BatteryPowered => 6,
            ResourceProfile::LimitedConnectivity => 9, // Maximum compression to save bandwidth
            ResourceProfile::Custom(_) => 6, // Default for custom profiles
        }
    }
}

/// Adaptation strategy for adjusting application behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationStrategy {
    /// Name of the strategy
    pub name: String,
    
    /// Description of the strategy
    pub description: String,
    
    /// Resource profile this strategy is designed for
    pub profile: ResourceProfile,
    
    /// Batch size for processing operations
    pub batch_size: usize,
    
    /// Cache size in megabytes
    pub cache_size_mb: usize,
    
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Polling interval for background operations
    pub polling_interval: Duration,
    
    /// Compression level (0-9)
    pub compression_level: u32,
    
    /// Whether to enable background processing
    pub enable_background_processing: bool,
    
    /// Whether to enable prefetching
    pub enable_prefetching: bool,
    
    /// Whether to enable caching
    pub enable_caching: bool,
    
    /// Custom parameters for this strategy
    pub custom_params: HashMap<String, String>,
}

impl AdaptationStrategy {
    /// Create a new adaptation strategy for a resource profile
    pub fn new(profile: ResourceProfile) -> Self {
        Self {
            name: profile.name(),
            description: format!("Adaptation strategy for {} systems", profile.name()),
            profile,
            batch_size: profile.recommended_batch_size(),
            cache_size_mb: profile.recommended_cache_size_mb(),
            worker_threads: profile.recommended_worker_threads(),
            polling_interval: profile.recommended_polling_interval(),
            compression_level: profile.recommended_compression_level(),
            enable_background_processing: true,
            enable_prefetching: true,
            enable_caching: true,
            custom_params: HashMap::new(),
        }
    }
    
    /// Add a custom parameter to the strategy
    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_params.insert(key.into(), value.into());
        self
    }
    
    /// Set the batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
    
    /// Set the cache size
    pub fn with_cache_size_mb(mut self, cache_size_mb: usize) -> Self {
        self.cache_size_mb = cache_size_mb;
        self
    }
    
    /// Set the number of worker threads
    pub fn with_worker_threads(mut self, worker_threads: usize) -> Self {
        self.worker_threads = worker_threads;
        self
    }
    
    /// Set the polling interval
    pub fn with_polling_interval(mut self, polling_interval: Duration) -> Self {
        self.polling_interval = polling_interval;
        self
    }
    
    /// Set the compression level
    pub fn with_compression_level(mut self, compression_level: u32) -> Self {
        self.compression_level = compression_level.min(9); // Ensure it's between 0-9
        self
    }
    
    /// Enable or disable background processing
    pub fn with_background_processing(mut self, enable: bool) -> Self {
        self.enable_background_processing = enable;
        self
    }
    
    /// Enable or disable prefetching
    pub fn with_prefetching(mut self, enable: bool) -> Self {
        self.enable_prefetching = enable;
        self
    }
    
    /// Enable or disable caching
    pub fn with_caching(mut self, enable: bool) -> Self {
        self.enable_caching = enable;
        self
    }
}

/// Resource manager for adapting application behavior
pub struct ResourceManager {
    /// Current resource profile
    current_profile: RwLock<ResourceProfile>,
    
    /// Current adaptation strategy
    current_strategy: RwLock<AdaptationStrategy>,
    
    /// Available adaptation strategies
    strategies: RwLock<HashMap<ResourceProfile, AdaptationStrategy>>,
    
    /// Last resource check time
    last_check: Mutex<Instant>,
    
    /// Resource check interval
    check_interval: Duration,
    
    /// Adaptation listeners
    listeners: RwLock<Vec<Box<dyn Fn(&AdaptationStrategy) + Send + Sync>>>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        // Default to mid-range profile
        let default_profile = ResourceProfile::MidRange;
        let default_strategy = AdaptationStrategy::new(default_profile);
        
        // Create strategies for all profiles
        let mut strategies = HashMap::new();
        strategies.insert(ResourceProfile::HighEnd, AdaptationStrategy::new(ResourceProfile::HighEnd));
        strategies.insert(ResourceProfile::MidRange, AdaptationStrategy::new(ResourceProfile::MidRange));
        strategies.insert(ResourceProfile::LowEnd, AdaptationStrategy::new(ResourceProfile::LowEnd));
        strategies.insert(ResourceProfile::Mobile, AdaptationStrategy::new(ResourceProfile::Mobile));
        strategies.insert(ResourceProfile::BatteryPowered, AdaptationStrategy::new(ResourceProfile::BatteryPowered));
        strategies.insert(ResourceProfile::LimitedConnectivity, AdaptationStrategy::new(ResourceProfile::LimitedConnectivity));
        
        Self {
            current_profile: RwLock::new(default_profile),
            current_strategy: RwLock::new(default_strategy),
            strategies: RwLock::new(strategies),
            last_check: Mutex::new(Instant::now()),
            check_interval: Duration::from_secs(60), // Check resources every minute by default
            listeners: RwLock::new(Vec::new()),
        }
    }
    
    /// Get the global resource manager instance
    pub fn global() -> Arc<Self> {
        lazy_static::lazy_static! {
            static ref INSTANCE: Arc<ResourceManager> = {
                let manager = ResourceManager::new();
                
                // Perform initial resource detection and adaptation
                if let Err(e) = manager.detect_and_adapt() {
                    error!("Failed to perform initial resource detection: {}", e);
                }
                
                Arc::new(manager)
            };
        }
        
        INSTANCE.clone()
    }
    
    /// Set the resource check interval
    pub fn set_check_interval(&self, interval: Duration) {
        *self.last_check.lock().unwrap() = Instant::now();
        self.check_interval = interval;
    }
    
    /// Get the current resource profile
    pub fn current_profile(&self) -> ResourceProfile {
        *self.current_profile.read().unwrap()
    }
    
    /// Get the current adaptation strategy
    pub fn current_strategy(&self) -> AdaptationStrategy {
        self.current_strategy.read().unwrap().clone()
    }
    
    /// Register an adaptation strategy for a profile
    pub fn register_strategy(&self, profile: ResourceProfile, strategy: AdaptationStrategy) {
        let mut strategies = self.strategies.write().unwrap();
        strategies.insert(profile, strategy);
    }
    
    /// Add a listener for adaptation changes
    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(&AdaptationStrategy) + Send + Sync + 'static,
    {
        let mut listeners = self.listeners.write().unwrap();
        listeners.push(Box::new(listener));
    }
    
    /// Detect system resources and adapt application behavior
    pub fn detect_and_adapt(&self) -> Result<()> {
        // Check if it's time to detect resources
        let mut last_check = self.last_check.lock().unwrap();
        if last_check.elapsed() < self.check_interval {
            return Ok(());
        }
        *last_check = Instant::now();
        
        // Get system resources
        let resources = get_system_resources()?;
        
        // Determine the appropriate resource profile
        let profile = self.determine_profile(&resources);
        
        // Update the current profile if it has changed
        let current_profile = *self.current_profile.read().unwrap();
        if profile != current_profile {
            info!("Resource profile changed from {:?} to {:?}", current_profile, profile);
            *self.current_profile.write().unwrap() = profile;
            
            // Get the adaptation strategy for this profile
            let strategy = {
                let strategies = self.strategies.read().unwrap();
                strategies.get(&profile).cloned().unwrap_or_else(|| AdaptationStrategy::new(profile))
            };
            
            // Update the current strategy
            *self.current_strategy.write().unwrap() = strategy.clone();
            
            // Notify listeners
            let listeners = self.listeners.read().unwrap();
            for listener in listeners.iter() {
                listener(&strategy);
            }
            
            info!("Adapted to new resource profile: {}", strategy.name);
            debug!("Adaptation strategy: {:?}", strategy);
        }
        
        Ok(())
    }
    
    /// Determine the appropriate resource profile based on system resources
    fn determine_profile(&self, resources: &SystemResources) -> ResourceProfile {
        // Check for battery power
        if let Some(battery) = &resources.battery {
            if battery.state == super::detection::BatteryState::Discharging && battery.percentage < 50.0 {
                return ResourceProfile::BatteryPowered;
            }
        }
        
        // Check for limited connectivity
        if resources.network.connectivity != super::detection::NetworkConnectivity::Full {
            return ResourceProfile::LimitedConnectivity;
        }
        
        // Determine profile based on CPU, memory, and disk
        let cpu_cores = resources.cpu.logical_cores;
        let memory_gb = resources.memory.total as f64 / 1024.0 / 1024.0 / 1024.0;
        
        if cpu_cores >= 8 && memory_gb >= 16.0 {
            ResourceProfile::HighEnd
        } else if cpu_cores >= 4 && memory_gb >= 8.0 {
            ResourceProfile::MidRange
        } else if cpu_cores >= 2 && memory_gb >= 4.0 {
            ResourceProfile::LowEnd
        } else {
            ResourceProfile::Mobile
        }
    }
    
    /// Force a specific resource profile
    pub fn force_profile(&self, profile: ResourceProfile) -> Result<()> {
        info!("Forcing resource profile to {:?}", profile);
        
        // Update the current profile
        *self.current_profile.write().unwrap() = profile;
        
        // Get the adaptation strategy for this profile
        let strategy = {
            let strategies = self.strategies.read().unwrap();
            strategies.get(&profile).cloned().unwrap_or_else(|| AdaptationStrategy::new(profile))
        };
        
        // Update the current strategy
        *self.current_strategy.write().unwrap() = strategy.clone();
        
        // Notify listeners
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(&strategy);
        }
        
        Ok(())
    }
    
    /// Get the adaptation strategy for a specific profile
    pub fn get_strategy(&self, profile: ResourceProfile) -> Option<AdaptationStrategy> {
        let strategies = self.strategies.read().unwrap();
        strategies.get(&profile).cloned()
    }
    
    /// Start background resource detection and adaptation
    pub fn start_background_adaptation(manager: Arc<Self>, interval: Duration) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            info!("Starting background resource adaptation with interval {:?}", interval);
            
            loop {
                std::thread::sleep(interval);
                
                if let Err(e) = manager.detect_and_adapt() {
                    error!("Error in background resource adaptation: {}", e);
                }
            }
        })
    }
}

/// Get the current adaptation strategy
pub fn get_current_strategy() -> Result<AdaptationStrategy> {
    Ok(ResourceManager::global().current_strategy())
}

/// Get the current resource profile
pub fn get_current_profile() -> Result<ResourceProfile> {
    Ok(ResourceManager::global().current_profile())
}

/// Force a specific resource profile
pub fn force_profile(profile: ResourceProfile) -> Result<()> {
    ResourceManager::global().force_profile(profile)
}

/// Add a listener for adaptation changes
pub fn add_adaptation_listener<F>(listener: F)
where
    F: Fn(&AdaptationStrategy) + Send + Sync + 'static,
{
    ResourceManager::global().add_listener(listener);
}