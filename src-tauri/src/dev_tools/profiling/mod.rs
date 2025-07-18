//! Performance Profiling Utilities
//!
//! This module provides tools for profiling the performance of the application.
//! It includes utilities for measuring execution time, memory usage, and other
//! performance metrics.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

/// Profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingConfig {
    /// Whether profiling is enabled
    pub enabled: bool,
    /// Sampling interval in milliseconds
    pub sampling_interval_ms: u64,
    /// Maximum number of samples to keep
    pub max_samples: usize,
    /// Whether to log profiling results automatically
    pub auto_log: bool,
    /// Whether to include memory profiling
    pub include_memory: bool,
    /// Whether to include CPU profiling
    pub include_cpu: bool,
    /// Whether to include I/O profiling
    pub include_io: bool,
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sampling_interval_ms: 100,
            max_samples: 1000,
            auto_log: true,
            include_memory: true,
            include_cpu: true,
            include_io: true,
        }
    }
}

/// Global profiling configuration
lazy_static::lazy_static! {
    static ref PROFILING_CONFIG: Arc<Mutex<ProfilingConfig>> = Arc::new(Mutex::new(ProfilingConfig::default()));
}

/// Get the global profiling configuration
pub fn get_profiling_config() -> Arc<Mutex<ProfilingConfig>> {
    PROFILING_CONFIG.clone()
}

/// Configure profiling
pub fn configure(config: ProfilingConfig) {
    let mut current_config = PROFILING_CONFIG.lock().unwrap();
    *current_config = config;
    
    // Apply configuration changes
    if config.enabled {
        info!("Performance profiling enabled with sampling interval of {}ms", config.sampling_interval_ms);
    } else {
        info!("Performance profiling disabled");
    }
}

/// Enable profiling
pub fn enable() {
    let mut config = PROFILING_CONFIG.lock().unwrap();
    config.enabled = true;
    info!("Performance profiling enabled");
}

/// Disable profiling
pub fn disable() {
    let mut config = PROFILING_CONFIG.lock().unwrap();
    config.enabled = false;
    info!("Performance profiling disabled");
}

/// Profile type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfileType {
    /// Function execution time
    Function,
    /// Database query execution time
    Database,
    /// Network request execution time
    Network,
    /// File I/O execution time
    FileIO,
    /// UI rendering time
    Rendering,
    /// Custom profile type
    Custom(u32),
}

/// Profile data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    /// Profile ID
    pub id: String,
    /// Profile type
    pub profile_type: ProfileType,
    /// Profile name
    pub name: String,
    /// Start time
    pub start_time: Instant,
    /// End time (if completed)
    pub end_time: Option<Instant>,
    /// Duration (if completed)
    pub duration: Option<Duration>,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl ProfileData {
    /// Create a new profile data
    pub fn new(profile_type: ProfileType, name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            profile_type,
            name: name.into(),
            start_time: Instant::now(),
            end_time: None,
            duration: None,
            context: HashMap::new(),
        }
    }
    
    /// Add context to the profile data
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
    
    /// Mark the profile as completed
    pub fn complete(&mut self) {
        let now = Instant::now();
        self.end_time = Some(now);
        self.duration = Some(now.duration_since(self.start_time));
    }
}

/// Profile manager
#[derive(Debug)]
pub struct ProfileManager {
    /// Active profiles by ID
    active_profiles: HashMap<String, ProfileData>,
    /// Completed profiles
    completed_profiles: Vec<ProfileData>,
    /// Maximum number of completed profiles to keep
    max_completed_profiles: usize,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new() -> Self {
        Self {
            active_profiles: HashMap::new(),
            completed_profiles: Vec::new(),
            max_completed_profiles: 1000,
        }
    }
    
    /// Set the maximum number of completed profiles to keep
    pub fn with_max_completed_profiles(mut self, max: usize) -> Self {
        self.max_completed_profiles = max;
        self
    }
    
    /// Start a new profile
    pub fn start_profile(&mut self, profile_type: ProfileType, name: impl Into<String>) -> String {
        let profile = ProfileData::new(profile_type, name);
        let id = profile.id.clone();
        self.active_profiles.insert(id.clone(), profile);
        id
    }
    
    /// Add context to an active profile
    pub fn add_context(&mut self, profile_id: &str, key: impl Into<String>, value: impl Into<String>) {
        if let Some(profile) = self.active_profiles.get_mut(profile_id) {
            profile.context.insert(key.into(), value.into());
        }
    }
    
    /// End a profile
    pub fn end_profile(&mut self, profile_id: &str) -> Option<Duration> {
        if let Some(mut profile) = self.active_profiles.remove(profile_id) {
            profile.complete();
            
            let duration = profile.duration;
            
            // Add to completed profiles
            self.completed_profiles.push(profile);
            
            // Trim completed profiles if needed
            if self.completed_profiles.len() > self.max_completed_profiles {
                self.completed_profiles.remove(0);
            }
            
            duration
        } else {
            None
        }
    }
    
    /// Get all completed profiles
    pub fn get_completed_profiles(&self) -> &[ProfileData] {
        &self.completed_profiles
    }
    
    /// Get completed profiles of a specific type
    pub fn get_profiles_by_type(&self, profile_type: ProfileType) -> Vec<&ProfileData> {
        self.completed_profiles
            .iter()
            .filter(|p| p.profile_type == profile_type)
            .collect()
    }
    
    /// Get profiles by name
    pub fn get_profiles_by_name(&self, name: &str) -> Vec<&ProfileData> {
        self.completed_profiles
            .iter()
            .filter(|p| p.name == name)
            .collect()
    }
    
    /// Calculate statistics for profiles of a specific type
    pub fn calculate_stats_by_type(&self, profile_type: ProfileType) -> Option<ProfileStats> {
        let profiles = self.get_profiles_by_type(profile_type);
        self.calculate_stats_for_profiles(&profiles)
    }
    
    /// Calculate statistics for profiles with a specific name
    pub fn calculate_stats_by_name(&self, name: &str) -> Option<ProfileStats> {
        let profiles = self.get_profiles_by_name(name);
        self.calculate_stats_for_profiles(&profiles)
    }
    
    /// Calculate statistics for a collection of profiles
    fn calculate_stats_for_profiles(&self, profiles: &[&ProfileData]) -> Option<ProfileStats> {
        if profiles.is_empty() {
            return None;
        }
        
        let mut durations = Vec::new();
        
        for profile in profiles {
            if let Some(duration) = profile.duration {
                durations.push(duration.as_secs_f64() * 1000.0); // Convert to milliseconds
            }
        }
        
        if durations.is_empty() {
            return None;
        }
        
        // Sort durations for percentile calculations
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let count = durations.len();
        let sum: f64 = durations.iter().sum();
        let min = durations[0];
        let max = durations[count - 1];
        let mean = sum / count as f64;
        
        // Calculate percentiles
        let p50 = percentile(&durations, 0.5);
        let p90 = percentile(&durations, 0.9);
        let p95 = percentile(&durations, 0.95);
        let p99 = percentile(&durations, 0.99);
        
        Some(ProfileStats {
            count,
            min,
            max,
            mean,
            sum,
            p50,
            p90,
            p95,
            p99,
        })
    }
    
    /// Clear all profiles
    pub fn clear(&mut self) {
        self.active_profiles.clear();
        self.completed_profiles.clear();
    }
}

/// Profile statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStats {
    /// Number of profiles
    pub count: usize,
    /// Minimum duration (ms)
    pub min: f64,
    /// Maximum duration (ms)
    pub max: f64,
    /// Mean duration (ms)
    pub mean: f64,
    /// Sum of durations (ms)
    pub sum: f64,
    /// 50th percentile (median) duration (ms)
    pub p50: f64,
    /// 90th percentile duration (ms)
    pub p90: f64,
    /// 95th percentile duration (ms)
    pub p95: f64,
    /// 99th percentile duration (ms)
    pub p99: f64,
}

/// Calculate a percentile from a sorted array of values
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    if sorted_values.len() == 1 {
        return sorted_values[0];
    }
    
    let rank = p * (sorted_values.len() - 1) as f64;
    let integer_rank = rank.floor() as usize;
    let fractional_rank = rank - integer_rank as f64;
    
    if integer_rank + 1 < sorted_values.len() {
        sorted_values[integer_rank] * (1.0 - fractional_rank) + sorted_values[integer_rank + 1] * fractional_rank
    } else {
        sorted_values[integer_rank]
    }
}

/// Global profile manager instance
lazy_static::lazy_static! {
    static ref PROFILE_MANAGER: Arc<Mutex<ProfileManager>> = Arc::new(Mutex::new(ProfileManager::new()));
}

/// Get the global profile manager instance
pub fn get_profile_manager() -> Arc<Mutex<ProfileManager>> {
    PROFILE_MANAGER.clone()
}

/// Start a new profile
pub fn start_profile(profile_type: ProfileType, name: impl Into<String>) -> Option<String> {
    let config = PROFILING_CONFIG.lock().unwrap();
    
    if !config.enabled {
        return None;
    }
    
    let mut manager = PROFILE_MANAGER.lock().unwrap();
    let id = manager.start_profile(profile_type, name);
    Some(id)
}

/// End a profile
pub fn end_profile(profile_id: &str) -> Option<Duration> {
    let config = PROFILING_CONFIG.lock().unwrap();
    
    if !config.enabled {
        return None;
    }
    
    let mut manager = PROFILE_MANAGER.lock().unwrap();
    let duration = manager.end_profile(profile_id);
    
    if config.auto_log {
        if let Some(duration) = duration {
            debug!("Profile {} completed in {:?}", profile_id, duration);
        }
    }
    
    duration
}

/// Profile a function call
pub fn profile_fn<F, R>(profile_type: ProfileType, name: impl Into<String>, f: F) -> R
where
    F: FnOnce() -> R,
{
    let profile_id = start_profile(profile_type, name);
    
    let result = f();
    
    if let Some(profile_id) = profile_id {
        end_profile(&profile_id);
    }
    
    result
}

/// Get profile statistics by type
pub fn get_stats_by_type(profile_type: ProfileType) -> Option<ProfileStats> {
    let manager = PROFILE_MANAGER.lock().unwrap();
    manager.calculate_stats_by_type(profile_type)
}

/// Get profile statistics by name
pub fn get_stats_by_name(name: &str) -> Option<ProfileStats> {
    let manager = PROFILE_MANAGER.lock().unwrap();
    manager.calculate_stats_by_name(name)
}

/// Clear all profiles
pub fn clear_profiles() {
    let mut manager = PROFILE_MANAGER.lock().unwrap();
    manager.clear();
}

/// Export profiling data as JSON
pub fn export_profiling_data() -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct ExportData {
        config: ProfilingConfig,
        profiles: Vec<ProfileDataExport>,
        stats_by_type: HashMap<String, ProfileStats>,
    }
    
    #[derive(Serialize)]
    struct ProfileDataExport {
        id: String,
        profile_type: String,
        name: String,
        duration_ms: f64,
        context: HashMap<String, String>,
    }
    
    let config = PROFILING_CONFIG.lock().unwrap();
    let manager = PROFILE_MANAGER.lock().unwrap();
    
    // Export completed profiles
    let profiles: Vec<ProfileDataExport> = manager
        .get_completed_profiles()
        .iter()
        .filter_map(|profile| {
            profile.duration.map(|duration| {
                ProfileDataExport {
                    id: profile.id.clone(),
                    profile_type: format!("{:?}", profile.profile_type),
                    name: profile.name.clone(),
                    duration_ms: duration.as_secs_f64() * 1000.0,
                    context: profile.context.clone(),
                }
            })
        })
        .collect();
    
    // Export statistics by profile type
    let mut stats_by_type = HashMap::new();
    
    for profile_type in [
        ProfileType::Function,
        ProfileType::Database,
        ProfileType::Network,
        ProfileType::FileIO,
        ProfileType::Rendering,
    ].iter() {
        if let Some(stats) = manager.calculate_stats_by_type(*profile_type) {
            stats_by_type.insert(format!("{:?}", profile_type), stats);
        }
    }
    
    let export_data = ExportData {
        config: config.clone(),
        profiles,
        stats_by_type,
    };
    
    serde_json::to_string_pretty(&export_data)
}

/// Initialize the profiling system
pub fn init() {
    info!("Initializing performance profiling system");
}

/// Macro to profile a block of code
#[macro_export]
macro_rules! profile_block {
    ($profile_type:expr, $name:expr, $($body:tt)*) => {{
        let profile_id = $crate::dev_tools::profiling::start_profile($profile_type, $name);
        let result = { $($body)* };
        if let Some(profile_id) = profile_id {
            $crate::dev_tools::profiling::end_profile(&profile_id);
        }
        result
    }};
}

/// Macro to profile a function
#[macro_export]
macro_rules! profile_fn {
    ($profile_type:expr, $name:expr, $($body:tt)*) => {
        $crate::dev_tools::profiling::profile_fn($profile_type, $name, || { $($body)* })
    };
}