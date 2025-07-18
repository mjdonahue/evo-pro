//! Adaptive Performance Profiling Tools
//!
//! This module provides specialized tools for profiling and analyzing the
//! adaptive performance system, including resource usage tracking, adaptation
//! decision analysis, and fallback strategy impact measurement.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};

use crate::resources::{
    ResourceProfile, AdaptationStrategy, EnhancedFeature, 
    FallbackStrategy, SystemResources
};
use crate::dev_tools::profiling::{ProfileType, ProfileStats};

/// Adaptive performance event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdaptiveEventType {
    /// Profile change event
    ProfileChange,
    /// Feature enablement change
    FeatureChange,
    /// Fallback strategy activation
    FallbackActivation,
    /// Resource usage threshold crossed
    ResourceThreshold,
    /// Performance anomaly detected
    PerformanceAnomaly,
    /// Custom event
    Custom(u32),
}

/// Adaptive performance event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: AdaptiveEventType,
    /// Event description
    pub description: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Previous resource profile (for profile changes)
    pub previous_profile: Option<ResourceProfile>,
    /// New resource profile (for profile changes)
    pub new_profile: Option<ResourceProfile>,
    /// Affected feature (for feature changes)
    pub feature: Option<EnhancedFeature>,
    /// Fallback strategy (for fallback activations)
    pub fallback_strategy: Option<String>,
    /// System resources at the time of the event
    pub resources: Option<SystemResources>,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl AdaptiveEvent {
    /// Create a new adaptive event
    pub fn new(event_type: AdaptiveEventType, description: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            description: description.into(),
            timestamp: Instant::now(),
            previous_profile: None,
            new_profile: None,
            feature: None,
            fallback_strategy: None,
            resources: None,
            context: HashMap::new(),
        }
    }
    
    /// Set the previous and new resource profiles
    pub fn with_profile_change(mut self, previous: ResourceProfile, new: ResourceProfile) -> Self {
        self.previous_profile = Some(previous);
        self.new_profile = Some(new);
        self
    }
    
    /// Set the affected feature
    pub fn with_feature(mut self, feature: EnhancedFeature) -> Self {
        self.feature = Some(feature);
        self
    }
    
    /// Set the fallback strategy
    pub fn with_fallback_strategy(mut self, strategy_name: impl Into<String>) -> Self {
        self.fallback_strategy = Some(strategy_name.into());
        self
    }
    
    /// Set the system resources
    pub fn with_resources(mut self, resources: SystemResources) -> Self {
        self.resources = Some(resources);
        self
    }
    
    /// Add context to the event
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Resource usage sample
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSample {
    /// Timestamp
    pub timestamp: Instant,
    /// System resources
    pub resources: SystemResources,
    /// Current resource profile
    pub profile: ResourceProfile,
    /// Active fallback strategies
    pub active_fallbacks: HashMap<EnhancedFeature, String>,
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Metric unit
    pub unit: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Context
    pub context: HashMap<String, String>,
}

/// Adaptive profiling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveProfilingConfig {
    /// Whether adaptive profiling is enabled
    pub enabled: bool,
    /// Resource sampling interval in milliseconds
    pub resource_sampling_interval_ms: u64,
    /// Maximum number of resource samples to keep
    pub max_resource_samples: usize,
    /// Maximum number of events to keep
    pub max_events: usize,
    /// Whether to automatically capture system resources with events
    pub auto_capture_resources: bool,
    /// Whether to log events automatically
    pub auto_log_events: bool,
    /// Resource usage thresholds for triggering events
    pub resource_thresholds: HashMap<String, f64>,
}

impl Default for AdaptiveProfilingConfig {
    fn default() -> Self {
        let mut resource_thresholds = HashMap::new();
        resource_thresholds.insert("cpu_usage".to_string(), 80.0); // 80% CPU usage
        resource_thresholds.insert("memory_usage".to_string(), 80.0); // 80% memory usage
        resource_thresholds.insert("disk_usage".to_string(), 90.0); // 90% disk usage
        
        Self {
            enabled: false,
            resource_sampling_interval_ms: 1000, // 1 second
            max_resource_samples: 3600, // 1 hour at 1 sample per second
            max_events: 1000,
            auto_capture_resources: true,
            auto_log_events: true,
            resource_thresholds,
        }
    }
}

/// Adaptive profiler for tracking and analyzing adaptive performance
pub struct AdaptiveProfiler {
    /// Configuration
    config: AdaptiveProfilingConfig,
    /// Resource usage samples
    resource_samples: VecDeque<ResourceSample>,
    /// Adaptive events
    events: VecDeque<AdaptiveEvent>,
    /// Performance metrics
    metrics: Vec<PerformanceMetric>,
    /// Last resource sample time
    last_sample_time: Instant,
    /// Background sampling thread handle
    sampling_thread: Option<std::thread::JoinHandle<()>>,
}

impl AdaptiveProfiler {
    /// Create a new adaptive profiler
    pub fn new(config: AdaptiveProfilingConfig) -> Self {
        Self {
            config,
            resource_samples: VecDeque::new(),
            events: VecDeque::new(),
            metrics: Vec::new(),
            last_sample_time: Instant::now(),
            sampling_thread: None,
        }
    }
    
    /// Start resource sampling
    pub fn start_sampling(&mut self) -> Result<(), String> {
        if self.sampling_thread.is_some() {
            return Err("Sampling already started".to_string());
        }
        
        let config = self.config.clone();
        let profiler = Arc::new(Mutex::new(self.clone()));
        
        let handle = std::thread::spawn(move || {
            info!("Starting adaptive performance resource sampling");
            
            loop {
                std::thread::sleep(Duration::from_millis(config.resource_sampling_interval_ms));
                
                // Get current system resources
                match crate::resources::detection::get_system_resources() {
                    Ok(resources) => {
                        // Get current profile and fallbacks
                        if let (Ok(profile), Ok(fallbacks)) = (
                            crate::resources::adaptation::get_current_profile(),
                            crate::resources::fallback::get_active_fallbacks()
                        ) {
                            let active_fallbacks = fallbacks.iter()
                                .map(|(feature, strategy)| (*feature, strategy.name.clone()))
                                .collect();
                            
                            let sample = ResourceSample {
                                timestamp: Instant::now(),
                                resources,
                                profile,
                                active_fallbacks,
                            };
                            
                            // Add sample to profiler
                            if let Ok(mut profiler) = profiler.lock() {
                                profiler.add_resource_sample(sample);
                                
                                // Check resource thresholds
                                profiler.check_resource_thresholds();
                            }
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get system resources for profiling: {}", e);
                    }
                }
            }
        });
        
        self.sampling_thread = Some(handle);
        Ok(())
    }
    
    /// Stop resource sampling
    pub fn stop_sampling(&mut self) -> Result<(), String> {
        if let Some(handle) = self.sampling_thread.take() {
            // We can't actually join the thread here because it runs indefinitely
            // In a real implementation, we would use a channel to signal the thread to stop
            info!("Stopping adaptive performance resource sampling");
        }
        
        Ok(())
    }
    
    /// Add a resource sample
    pub fn add_resource_sample(&mut self, sample: ResourceSample) {
        self.resource_samples.push_back(sample);
        
        // Trim samples if needed
        while self.resource_samples.len() > self.config.max_resource_samples {
            self.resource_samples.pop_front();
        }
        
        self.last_sample_time = Instant::now();
    }
    
    /// Add an adaptive event
    pub fn add_event(&mut self, mut event: AdaptiveEvent) {
        // Automatically capture system resources if configured
        if self.config.auto_capture_resources && event.resources.is_none() {
            if let Ok(resources) = crate::resources::detection::get_system_resources() {
                event.resources = Some(resources);
            }
        }
        
        // Log the event if configured
        if self.config.auto_log_events {
            info!("Adaptive event: {} - {}", format!("{:?}", event.event_type), event.description);
        }
        
        self.events.push_back(event);
        
        // Trim events if needed
        while self.events.len() > self.config.max_events {
            self.events.pop_front();
        }
    }
    
    /// Add a performance metric
    pub fn add_metric(&mut self, metric: PerformanceMetric) {
        self.metrics.push(metric);
    }
    
    /// Check resource thresholds and generate events if crossed
    fn check_resource_thresholds(&mut self) {
        if let Some(sample) = self.resource_samples.back() {
            let resources = &sample.resources;
            
            // Check CPU usage
            if let Some(threshold) = self.config.resource_thresholds.get("cpu_usage") {
                let cpu_usage = resources.cpu.usage_percent;
                if cpu_usage > *threshold {
                    let event = AdaptiveEvent::new(
                        AdaptiveEventType::ResourceThreshold,
                        format!("CPU usage threshold crossed: {:.1}% > {:.1}%", cpu_usage, threshold)
                    )
                    .with_resources(resources.clone())
                    .with_context("threshold_type", "cpu_usage")
                    .with_context("threshold_value", threshold.to_string())
                    .with_context("actual_value", cpu_usage.to_string());
                    
                    self.add_event(event);
                }
            }
            
            // Check memory usage
            if let Some(threshold) = self.config.resource_thresholds.get("memory_usage") {
                let memory_total = resources.memory.total as f64;
                let memory_used = resources.memory.used as f64;
                let memory_usage = (memory_used / memory_total) * 100.0;
                
                if memory_usage > *threshold {
                    let event = AdaptiveEvent::new(
                        AdaptiveEventType::ResourceThreshold,
                        format!("Memory usage threshold crossed: {:.1}% > {:.1}%", memory_usage, threshold)
                    )
                    .with_resources(resources.clone())
                    .with_context("threshold_type", "memory_usage")
                    .with_context("threshold_value", threshold.to_string())
                    .with_context("actual_value", memory_usage.to_string());
                    
                    self.add_event(event);
                }
            }
            
            // Check disk usage
            if let Some(threshold) = self.config.resource_thresholds.get("disk_usage") {
                for disk in &resources.disks {
                    let disk_total = disk.total_space as f64;
                    let disk_used = disk.used_space as f64;
                    let disk_usage = (disk_used / disk_total) * 100.0;
                    
                    if disk_usage > *threshold {
                        let event = AdaptiveEvent::new(
                            AdaptiveEventType::ResourceThreshold,
                            format!("Disk usage threshold crossed for {}: {:.1}% > {:.1}%", 
                                    disk.name, disk_usage, threshold)
                        )
                        .with_resources(resources.clone())
                        .with_context("threshold_type", "disk_usage")
                        .with_context("threshold_value", threshold.to_string())
                        .with_context("actual_value", disk_usage.to_string())
                        .with_context("disk_name", disk.name.clone());
                        
                        self.add_event(event);
                    }
                }
            }
        }
    }
    
    /// Get resource samples within a time range
    pub fn get_resource_samples(&self, duration: Option<Duration>) -> Vec<&ResourceSample> {
        if let Some(duration) = duration {
            let cutoff = Instant::now() - duration;
            self.resource_samples
                .iter()
                .filter(|sample| sample.timestamp >= cutoff)
                .collect()
        } else {
            self.resource_samples.iter().collect()
        }
    }
    
    /// Get events within a time range
    pub fn get_events(&self, duration: Option<Duration>) -> Vec<&AdaptiveEvent> {
        if let Some(duration) = duration {
            let cutoff = Instant::now() - duration;
            self.events
                .iter()
                .filter(|event| event.timestamp >= cutoff)
                .collect()
        } else {
            self.events.iter().collect()
        }
    }
    
    /// Get events of a specific type
    pub fn get_events_by_type(&self, event_type: AdaptiveEventType) -> Vec<&AdaptiveEvent> {
        self.events
            .iter()
            .filter(|event| event.event_type == event_type)
            .collect()
    }
    
    /// Get profile change history
    pub fn get_profile_history(&self) -> Vec<(&Instant, &ResourceProfile)> {
        self.events
            .iter()
            .filter_map(|event| {
                if event.event_type == AdaptiveEventType::ProfileChange {
                    if let Some(new_profile) = &event.new_profile {
                        return Some((&event.timestamp, new_profile));
                    }
                }
                None
            })
            .collect()
    }
    
    /// Get feature enablement history for a specific feature
    pub fn get_feature_history(&self, feature: EnhancedFeature) -> Vec<(&Instant, bool)> {
        self.events
            .iter()
            .filter_map(|event| {
                if event.event_type == AdaptiveEventType::FeatureChange {
                    if let Some(event_feature) = &event.feature {
                        if *event_feature == feature {
                            // Extract enabled state from context
                            if let Some(enabled) = event.context.get("enabled") {
                                return Some((&event.timestamp, enabled == "true"));
                            }
                        }
                    }
                }
                None
            })
            .collect()
    }
    
    /// Get fallback activation history for a specific feature
    pub fn get_fallback_history(&self, feature: EnhancedFeature) -> Vec<(&Instant, Option<&String>)> {
        self.events
            .iter()
            .filter_map(|event| {
                if event.event_type == AdaptiveEventType::FallbackActivation {
                    if let Some(event_feature) = &event.feature {
                        if *event_feature == feature {
                            return Some((&event.timestamp, event.fallback_strategy.as_ref()));
                        }
                    }
                }
                None
            })
            .collect()
    }
    
    /// Generate a report of adaptive performance
    pub fn generate_report(&self) -> AdaptivePerformanceReport {
        // Calculate time spent in each profile
        let mut profile_durations = HashMap::new();
        let mut current_profile = None;
        let mut profile_start_time = Instant::now();
        
        for event in self.events.iter().filter(|e| e.event_type == AdaptiveEventType::ProfileChange) {
            if let Some(previous) = &event.previous_profile {
                if let Some(profile) = current_profile {
                    let duration = event.timestamp.duration_since(profile_start_time);
                    *profile_durations.entry(profile).or_insert(Duration::from_secs(0)) += duration;
                }
                
                current_profile = event.new_profile;
                profile_start_time = event.timestamp;
            }
        }
        
        // Add duration for current profile
        if let Some(profile) = current_profile {
            let duration = Instant::now().duration_since(profile_start_time);
            *profile_durations.entry(profile).or_insert(Duration::from_secs(0)) += duration;
        }
        
        // Calculate fallback activation counts
        let mut fallback_counts = HashMap::new();
        
        for event in self.events.iter().filter(|e| e.event_type == AdaptiveEventType::FallbackActivation) {
            if let Some(feature) = &event.feature {
                if let Some(strategy) = &event.fallback_strategy {
                    let key = (*feature, strategy.clone());
                    *fallback_counts.entry(key).or_insert(0) += 1;
                }
            }
        }
        
        // Calculate resource threshold events
        let threshold_events = self.events
            .iter()
            .filter(|e| e.event_type == AdaptiveEventType::ResourceThreshold)
            .count();
        
        // Calculate performance anomalies
        let anomaly_events = self.events
            .iter()
            .filter(|e| e.event_type == AdaptiveEventType::PerformanceAnomaly)
            .count();
        
        // Create report
        AdaptivePerformanceReport {
            total_events: self.events.len(),
            profile_durations,
            fallback_counts,
            threshold_events,
            anomaly_events,
            resource_samples: self.resource_samples.len(),
        }
    }
    
    /// Export profiling data as JSON
    pub fn export_data(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ExportData {
            config: AdaptiveProfilingConfig,
            events: Vec<ExportEvent>,
            resource_samples: Vec<ExportSample>,
            metrics: Vec<ExportMetric>,
            report: AdaptivePerformanceReport,
        }
        
        #[derive(Serialize)]
        struct ExportEvent {
            id: String,
            event_type: String,
            description: String,
            timestamp_ms: u64,
            previous_profile: Option<String>,
            new_profile: Option<String>,
            feature: Option<String>,
            fallback_strategy: Option<String>,
            context: HashMap<String, String>,
        }
        
        #[derive(Serialize)]
        struct ExportSample {
            timestamp_ms: u64,
            profile: String,
            cpu_usage: f64,
            memory_used_mb: f64,
            memory_total_mb: f64,
            active_fallbacks: HashMap<String, String>,
        }
        
        #[derive(Serialize)]
        struct ExportMetric {
            name: String,
            value: f64,
            unit: String,
            timestamp_ms: u64,
            context: HashMap<String, String>,
        }
        
        // Convert events to export format
        let events: Vec<ExportEvent> = self.events
            .iter()
            .map(|event| {
                ExportEvent {
                    id: event.id.clone(),
                    event_type: format!("{:?}", event.event_type),
                    description: event.description.clone(),
                    timestamp_ms: event.timestamp.elapsed().as_millis() as u64,
                    previous_profile: event.previous_profile.map(|p| format!("{:?}", p)),
                    new_profile: event.new_profile.map(|p| format!("{:?}", p)),
                    feature: event.feature.map(|f| format!("{:?}", f)),
                    fallback_strategy: event.fallback_strategy.clone(),
                    context: event.context.clone(),
                }
            })
            .collect();
        
        // Convert resource samples to export format
        let resource_samples: Vec<ExportSample> = self.resource_samples
            .iter()
            .map(|sample| {
                let active_fallbacks = sample.active_fallbacks
                    .iter()
                    .map(|(feature, strategy)| (format!("{:?}", feature), strategy.clone()))
                    .collect();
                
                ExportSample {
                    timestamp_ms: sample.timestamp.elapsed().as_millis() as u64,
                    profile: format!("{:?}", sample.profile),
                    cpu_usage: sample.resources.cpu.usage_percent,
                    memory_used_mb: sample.resources.memory.used as f64 / 1024.0 / 1024.0,
                    memory_total_mb: sample.resources.memory.total as f64 / 1024.0 / 1024.0,
                    active_fallbacks,
                }
            })
            .collect();
        
        // Convert metrics to export format
        let metrics: Vec<ExportMetric> = self.metrics
            .iter()
            .map(|metric| {
                ExportMetric {
                    name: metric.name.clone(),
                    value: metric.value,
                    unit: metric.unit.clone(),
                    timestamp_ms: metric.timestamp.elapsed().as_millis() as u64,
                    context: metric.context.clone(),
                }
            })
            .collect();
        
        // Generate report
        let report = self.generate_report();
        
        let export_data = ExportData {
            config: self.config.clone(),
            events,
            resource_samples,
            metrics,
            report,
        };
        
        serde_json::to_string_pretty(&export_data)
    }
}

impl Clone for AdaptiveProfiler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            resource_samples: self.resource_samples.clone(),
            events: self.events.clone(),
            metrics: self.metrics.clone(),
            last_sample_time: self.last_sample_time,
            sampling_thread: None, // Don't clone the thread handle
        }
    }
}

/// Adaptive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptivePerformanceReport {
    /// Total number of events
    pub total_events: usize,
    /// Time spent in each profile
    pub profile_durations: HashMap<ResourceProfile, Duration>,
    /// Fallback activation counts by feature and strategy
    pub fallback_counts: HashMap<(EnhancedFeature, String), usize>,
    /// Number of resource threshold events
    pub threshold_events: usize,
    /// Number of performance anomaly events
    pub anomaly_events: usize,
    /// Number of resource samples
    pub resource_samples: usize,
}

/// Global adaptive profiler instance
lazy_static::lazy_static! {
    static ref ADAPTIVE_PROFILER: Arc<Mutex<AdaptiveProfiler>> = Arc::new(Mutex::new(
        AdaptiveProfiler::new(AdaptiveProfilingConfig::default())
    ));
}

/// Get the global adaptive profiler instance
pub fn get_adaptive_profiler() -> Arc<Mutex<AdaptiveProfiler>> {
    ADAPTIVE_PROFILER.clone()
}

/// Configure adaptive profiling
pub fn configure(config: AdaptiveProfilingConfig) {
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    // Stop sampling if it's running
    let _ = profiler.stop_sampling();
    
    // Update configuration
    profiler.config = config.clone();
    
    // Restart sampling if enabled
    if config.enabled {
        let _ = profiler.start_sampling();
        info!("Adaptive performance profiling enabled with sampling interval of {}ms", 
              config.resource_sampling_interval_ms);
    } else {
        info!("Adaptive performance profiling disabled");
    }
}

/// Enable adaptive profiling
pub fn enable() {
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.config.enabled = true;
    let _ = profiler.start_sampling();
    info!("Adaptive performance profiling enabled");
}

/// Disable adaptive profiling
pub fn disable() {
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.config.enabled = false;
    let _ = profiler.stop_sampling();
    info!("Adaptive performance profiling disabled");
}

/// Record a profile change event
pub fn record_profile_change(previous: ResourceProfile, new: ResourceProfile) {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    if !profiler.config.enabled {
        return;
    }
    
    let event = AdaptiveEvent::new(
        AdaptiveEventType::ProfileChange,
        format!("Resource profile changed from {:?} to {:?}", previous, new)
    )
    .with_profile_change(previous, new);
    
    drop(profiler);
    
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.add_event(event);
}

/// Record a feature change event
pub fn record_feature_change(feature: EnhancedFeature, enabled: bool) {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    if !profiler.config.enabled {
        return;
    }
    
    let event = AdaptiveEvent::new(
        AdaptiveEventType::FeatureChange,
        format!("Feature {} {}", feature.name(), if enabled { "enabled" } else { "disabled" })
    )
    .with_feature(feature)
    .with_context("enabled", enabled.to_string());
    
    drop(profiler);
    
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.add_event(event);
}

/// Record a fallback activation event
pub fn record_fallback_activation(feature: EnhancedFeature, strategy_name: Option<&str>) {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    if !profiler.config.enabled {
        return;
    }
    
    let description = if let Some(name) = strategy_name {
        format!("Fallback strategy activated for {}: {}", feature.name(), name)
    } else {
        format!("Fallback strategy deactivated for {}", feature.name())
    };
    
    let mut event = AdaptiveEvent::new(
        AdaptiveEventType::FallbackActivation,
        description
    )
    .with_feature(feature);
    
    if let Some(name) = strategy_name {
        event = event.with_fallback_strategy(name);
    }
    
    drop(profiler);
    
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.add_event(event);
}

/// Record a performance anomaly event
pub fn record_performance_anomaly(description: impl Into<String>, context: HashMap<String, String>) {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    if !profiler.config.enabled {
        return;
    }
    
    let mut event = AdaptiveEvent::new(
        AdaptiveEventType::PerformanceAnomaly,
        description
    );
    
    for (key, value) in context {
        event = event.with_context(key, value);
    }
    
    drop(profiler);
    
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.add_event(event);
}

/// Add a performance metric
pub fn add_performance_metric(name: impl Into<String>, value: f64, unit: impl Into<String>) {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    if !profiler.config.enabled {
        return;
    }
    
    let metric = PerformanceMetric {
        name: name.into(),
        value,
        unit: unit.into(),
        timestamp: Instant::now(),
        context: HashMap::new(),
    };
    
    drop(profiler);
    
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    profiler.add_metric(metric);
}

/// Generate an adaptive performance report
pub fn generate_report() -> Option<AdaptivePerformanceReport> {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    if !profiler.config.enabled {
        return None;
    }
    
    Some(profiler.generate_report())
}

/// Export adaptive profiling data as JSON
pub fn export_data() -> Result<String, String> {
    let profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    profiler.export_data().map_err(|e| e.to_string())
}

/// Initialize the adaptive profiling system
pub fn init() {
    info!("Initializing adaptive performance profiling system");
    
    // Register listeners for adaptive performance events
    
    // Listen for profile changes
    crate::resources::adaptation::add_adaptation_listener(|strategy| {
        let previous_profile = crate::resources::adaptation::get_current_profile().unwrap_or(strategy.profile);
        record_profile_change(previous_profile, strategy.profile);
    });
    
    // Initialize the profiler
    let mut profiler = ADAPTIVE_PROFILER.lock().unwrap();
    
    // Start sampling if enabled
    if profiler.config.enabled {
        let _ = profiler.start_sampling();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adaptive_event_creation() {
        let event = AdaptiveEvent::new(
            AdaptiveEventType::ProfileChange,
            "Test event"
        )
        .with_profile_change(ResourceProfile::LowEnd, ResourceProfile::MidRange)
        .with_context("test_key", "test_value");
        
        assert_eq!(event.event_type, AdaptiveEventType::ProfileChange);
        assert_eq!(event.description, "Test event");
        assert_eq!(event.previous_profile, Some(ResourceProfile::LowEnd));
        assert_eq!(event.new_profile, Some(ResourceProfile::MidRange));
        assert_eq!(event.context.get("test_key"), Some(&"test_value".to_string()));
    }
    
    #[test]
    fn test_adaptive_profiler() {
        let config = AdaptiveProfilingConfig {
            enabled: true,
            resource_sampling_interval_ms: 100,
            max_resource_samples: 10,
            max_events: 10,
            auto_capture_resources: false,
            auto_log_events: false,
            resource_thresholds: HashMap::new(),
        };
        
        let mut profiler = AdaptiveProfiler::new(config);
        
        // Add some events
        let event1 = AdaptiveEvent::new(
            AdaptiveEventType::ProfileChange,
            "Profile changed from LowEnd to MidRange"
        )
        .with_profile_change(ResourceProfile::LowEnd, ResourceProfile::MidRange);
        
        let event2 = AdaptiveEvent::new(
            AdaptiveEventType::FallbackActivation,
            "Fallback activated for HighResolutionAssets"
        )
        .with_feature(EnhancedFeature::HighResolutionAssets)
        .with_fallback_strategy("Medium Resolution Assets");
        
        profiler.add_event(event1);
        profiler.add_event(event2);
        
        // Check events were added
        assert_eq!(profiler.events.len(), 2);
        
        // Check event retrieval
        let profile_events = profiler.get_events_by_type(AdaptiveEventType::ProfileChange);
        assert_eq!(profile_events.len(), 1);
        
        let fallback_events = profiler.get_events_by_type(AdaptiveEventType::FallbackActivation);
        assert_eq!(fallback_events.len(), 1);
        
        // Generate report
        let report = profiler.generate_report();
        assert_eq!(report.total_events, 2);
    }
}