//! Performance Monitoring and Alerting
//!
//! This module provides tools for monitoring application performance and
//! generating alerts when performance issues are detected.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn, error};

use crate::resources::{ResourceProfile, SystemResources};
use crate::dev_tools::adaptive_profiling::{
    AdaptiveEventType, AdaptiveEvent, ResourceSample, 
    get_adaptive_profiler, record_performance_anomaly
};

/// Performance alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
}

/// Performance alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertType {
    /// High resource usage
    HighResourceUsage,
    /// Frequent profile changes
    FrequentProfileChanges,
    /// Excessive fallback activations
    ExcessiveFallbacks,
    /// Performance degradation
    PerformanceDegradation,
    /// Resource threshold crossed
    ResourceThreshold,
    /// Custom alert
    Custom(u32),
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    /// Alert ID
    pub id: String,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Timestamp
    pub timestamp: Instant,
    /// Related resource profile
    pub profile: Option<ResourceProfile>,
    /// System resources at the time of the alert
    pub resources: Option<SystemResources>,
    /// Additional context
    pub context: HashMap<String, String>,
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    /// Whether the alert has been resolved
    pub resolved: bool,
}

impl PerformanceAlert {
    /// Create a new performance alert
    pub fn new(
        alert_type: AlertType,
        severity: AlertSeverity,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            alert_type,
            severity,
            title: title.into(),
            description: description.into(),
            timestamp: Instant::now(),
            profile: None,
            resources: None,
            context: HashMap::new(),
            acknowledged: false,
            resolved: false,
        }
    }
    
    /// Set the resource profile
    pub fn with_profile(mut self, profile: ResourceProfile) -> Self {
        self.profile = Some(profile);
        self
    }
    
    /// Set the system resources
    pub fn with_resources(mut self, resources: SystemResources) -> Self {
        self.resources = Some(resources);
        self
    }
    
    /// Add context to the alert
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
    
    /// Mark the alert as acknowledged
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }
    
    /// Mark the alert as resolved
    pub fn resolve(&mut self) {
        self.resolved = true;
    }
}

/// Alert handler function type
pub type AlertHandler = Box<dyn Fn(&PerformanceAlert) + Send + Sync>;

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Whether performance monitoring is enabled
    pub enabled: bool,
    /// Monitoring interval in milliseconds
    pub monitoring_interval_ms: u64,
    /// Maximum number of alerts to keep
    pub max_alerts: usize,
    /// CPU usage threshold for generating alerts (percentage)
    pub cpu_usage_threshold: f64,
    /// Memory usage threshold for generating alerts (percentage)
    pub memory_usage_threshold: f64,
    /// Disk usage threshold for generating alerts (percentage)
    pub disk_usage_threshold: f64,
    /// Profile change threshold (changes per minute)
    pub profile_change_threshold: f64,
    /// Fallback activation threshold (activations per minute)
    pub fallback_activation_threshold: f64,
    /// Whether to automatically capture system resources with alerts
    pub auto_capture_resources: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            monitoring_interval_ms: 5000, // 5 seconds
            max_alerts: 1000,
            cpu_usage_threshold: 90.0, // 90% CPU usage
            memory_usage_threshold: 85.0, // 85% memory usage
            disk_usage_threshold: 95.0, // 95% disk usage
            profile_change_threshold: 6.0, // 6 changes per minute
            fallback_activation_threshold: 10.0, // 10 activations per minute
            auto_capture_resources: true,
        }
    }
}

/// Performance monitor for detecting performance issues and generating alerts
pub struct PerformanceMonitor {
    /// Configuration
    config: MonitoringConfig,
    /// Performance alerts
    alerts: VecDeque<PerformanceAlert>,
    /// Alert handlers
    alert_handlers: Vec<AlertHandler>,
    /// Last monitoring time
    last_monitoring_time: Instant,
    /// Background monitoring thread handle
    monitoring_thread: Option<std::thread::JoinHandle<()>>,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: MonitoringConfig) -> Self {
        Self {
            config,
            alerts: VecDeque::new(),
            alert_handlers: Vec::new(),
            last_monitoring_time: Instant::now(),
            monitoring_thread: None,
        }
    }
    
    /// Start performance monitoring
    pub fn start_monitoring(&mut self) -> Result<(), String> {
        if self.monitoring_thread.is_some() {
            return Err("Monitoring already started".to_string());
        }
        
        let config = self.config.clone();
        let monitor = Arc::new(Mutex::new(self.clone()));
        
        let handle = std::thread::spawn(move || {
            info!("Starting performance monitoring");
            
            loop {
                std::thread::sleep(Duration::from_millis(config.monitoring_interval_ms));
                
                if let Ok(mut monitor) = monitor.lock() {
                    // Check for performance issues
                    monitor.check_performance_issues();
                }
            }
        });
        
        self.monitoring_thread = Some(handle);
        Ok(())
    }
    
    /// Stop performance monitoring
    pub fn stop_monitoring(&mut self) -> Result<(), String> {
        if let Some(handle) = self.monitoring_thread.take() {
            // We can't actually join the thread here because it runs indefinitely
            // In a real implementation, we would use a channel to signal the thread to stop
            info!("Stopping performance monitoring");
        }
        
        Ok(())
    }
    
    /// Add an alert handler
    pub fn add_alert_handler<F>(&mut self, handler: F)
    where
        F: Fn(&PerformanceAlert) + Send + Sync + 'static,
    {
        self.alert_handlers.push(Box::new(handler));
    }
    
    /// Add a performance alert
    pub fn add_alert(&mut self, mut alert: PerformanceAlert) {
        // Automatically capture system resources if configured
        if self.config.auto_capture_resources && alert.resources.is_none() {
            if let Ok(resources) = crate::resources::detection::get_system_resources() {
                alert.resources = Some(resources);
            }
        }
        
        // Log the alert
        match alert.severity {
            AlertSeverity::Info => info!("Performance alert: {} - {}", alert.title, alert.description),
            AlertSeverity::Warning => warn!("Performance alert: {} - {}", alert.title, alert.description),
            AlertSeverity::Error => error!("Performance alert: {} - {}", alert.title, alert.description),
            AlertSeverity::Critical => error!("CRITICAL performance alert: {} - {}", alert.title, alert.description),
        }
        
        // Notify handlers
        for handler in &self.alert_handlers {
            handler(&alert);
        }
        
        // Record as performance anomaly in adaptive profiling
        let mut context = HashMap::new();
        context.insert("alert_id".to_string(), alert.id.clone());
        context.insert("alert_type".to_string(), format!("{:?}", alert.alert_type));
        context.insert("severity".to_string(), format!("{:?}", alert.severity));
        
        record_performance_anomaly(alert.description.clone(), context);
        
        // Add to alerts queue
        self.alerts.push_back(alert);
        
        // Trim alerts if needed
        while self.alerts.len() > self.config.max_alerts {
            self.alerts.pop_front();
        }
    }
    
    /// Check for performance issues
    fn check_performance_issues(&mut self) {
        self.last_monitoring_time = Instant::now();
        
        // Get the adaptive profiler
        let profiler = get_adaptive_profiler();
        if let Ok(profiler) = profiler.lock() {
            // Check resource usage
            self.check_resource_usage(&profiler);
            
            // Check for frequent profile changes
            self.check_profile_changes(&profiler);
            
            // Check for excessive fallback activations
            self.check_fallback_activations(&profiler);
            
            // Check for performance anomalies
            self.check_performance_anomalies(&profiler);
        }
    }
    
    /// Check resource usage
    fn check_resource_usage(&mut self, profiler: &impl std::ops::Deref<Target = crate::dev_tools::adaptive_profiling::AdaptiveProfiler>) {
        // Get the most recent resource sample
        let samples = profiler.get_resource_samples(Some(Duration::from_secs(60)));
        if let Some(sample) = samples.last() {
            let resources = &sample.resources;
            
            // Check CPU usage
            let cpu_usage = resources.cpu.usage_percent;
            if cpu_usage > self.config.cpu_usage_threshold {
                let alert = PerformanceAlert::new(
                    AlertType::HighResourceUsage,
                    AlertSeverity::Warning,
                    "High CPU Usage",
                    format!("CPU usage is at {:.1}%, which exceeds the threshold of {:.1}%", 
                            cpu_usage, self.config.cpu_usage_threshold)
                )
                .with_profile(sample.profile)
                .with_resources(resources.clone())
                .with_context("resource_type", "cpu")
                .with_context("threshold", self.config.cpu_usage_threshold.to_string())
                .with_context("actual_value", cpu_usage.to_string());
                
                self.add_alert(alert);
            }
            
            // Check memory usage
            let memory_total = resources.memory.total as f64;
            let memory_used = resources.memory.used as f64;
            let memory_usage = (memory_used / memory_total) * 100.0;
            
            if memory_usage > self.config.memory_usage_threshold {
                let alert = PerformanceAlert::new(
                    AlertType::HighResourceUsage,
                    AlertSeverity::Warning,
                    "High Memory Usage",
                    format!("Memory usage is at {:.1}%, which exceeds the threshold of {:.1}%", 
                            memory_usage, self.config.memory_usage_threshold)
                )
                .with_profile(sample.profile)
                .with_resources(resources.clone())
                .with_context("resource_type", "memory")
                .with_context("threshold", self.config.memory_usage_threshold.to_string())
                .with_context("actual_value", memory_usage.to_string());
                
                self.add_alert(alert);
            }
            
            // Check disk usage
            for disk in &resources.disks {
                let disk_total = disk.total_space as f64;
                let disk_used = disk.used_space as f64;
                let disk_usage = (disk_used / disk_total) * 100.0;
                
                if disk_usage > self.config.disk_usage_threshold {
                    let alert = PerformanceAlert::new(
                        AlertType::HighResourceUsage,
                        AlertSeverity::Warning,
                        format!("High Disk Usage on {}", disk.name),
                        format!("Disk usage on {} is at {:.1}%, which exceeds the threshold of {:.1}%", 
                                disk.name, disk_usage, self.config.disk_usage_threshold)
                    )
                    .with_profile(sample.profile)
                    .with_resources(resources.clone())
                    .with_context("resource_type", "disk")
                    .with_context("disk_name", disk.name.clone())
                    .with_context("threshold", self.config.disk_usage_threshold.to_string())
                    .with_context("actual_value", disk_usage.to_string());
                    
                    self.add_alert(alert);
                }
            }
        }
    }
    
    /// Check for frequent profile changes
    fn check_profile_changes(&mut self, profiler: &impl std::ops::Deref<Target = crate::dev_tools::adaptive_profiling::AdaptiveProfiler>) {
        // Get profile change events in the last minute
        let events = profiler.get_events(Some(Duration::from_secs(60)));
        let profile_changes = events.iter()
            .filter(|e| e.event_type == AdaptiveEventType::ProfileChange)
            .count();
        
        // Calculate changes per minute
        let changes_per_minute = profile_changes as f64;
        
        if changes_per_minute > self.config.profile_change_threshold {
            let alert = PerformanceAlert::new(
                AlertType::FrequentProfileChanges,
                AlertSeverity::Warning,
                "Frequent Resource Profile Changes",
                format!("Resource profile is changing frequently ({} changes in the last minute)", 
                        profile_changes)
            )
            .with_context("changes_per_minute", changes_per_minute.to_string())
            .with_context("threshold", self.config.profile_change_threshold.to_string());
            
            self.add_alert(alert);
        }
    }
    
    /// Check for excessive fallback activations
    fn check_fallback_activations(&mut self, profiler: &impl std::ops::Deref<Target = crate::dev_tools::adaptive_profiling::AdaptiveProfiler>) {
        // Get fallback activation events in the last minute
        let events = profiler.get_events(Some(Duration::from_secs(60)));
        let fallback_activations = events.iter()
            .filter(|e| e.event_type == AdaptiveEventType::FallbackActivation)
            .count();
        
        // Calculate activations per minute
        let activations_per_minute = fallback_activations as f64;
        
        if activations_per_minute > self.config.fallback_activation_threshold {
            let alert = PerformanceAlert::new(
                AlertType::ExcessiveFallbacks,
                AlertSeverity::Warning,
                "Excessive Fallback Strategy Activations",
                format!("Fallback strategies are being activated frequently ({} activations in the last minute)", 
                        fallback_activations)
            )
            .with_context("activations_per_minute", activations_per_minute.to_string())
            .with_context("threshold", self.config.fallback_activation_threshold.to_string());
            
            self.add_alert(alert);
        }
    }
    
    /// Check for performance anomalies
    fn check_performance_anomalies(&mut self, profiler: &impl std::ops::Deref<Target = crate::dev_tools::adaptive_profiling::AdaptiveProfiler>) {
        // Get performance anomaly events in the last minute
        let events = profiler.get_events(Some(Duration::from_secs(60)));
        let anomalies = events.iter()
            .filter(|e| e.event_type == AdaptiveEventType::PerformanceAnomaly)
            .count();
        
        if anomalies > 0 {
            let alert = PerformanceAlert::new(
                AlertType::PerformanceDegradation,
                AlertSeverity::Warning,
                "Performance Anomalies Detected",
                format!("{} performance anomalies detected in the last minute", anomalies)
            )
            .with_context("anomalies_count", anomalies.to_string());
            
            self.add_alert(alert);
        }
    }
    
    /// Get all alerts
    pub fn get_alerts(&self) -> Vec<&PerformanceAlert> {
        self.alerts.iter().collect()
    }
    
    /// Get unacknowledged alerts
    pub fn get_unacknowledged_alerts(&self) -> Vec<&PerformanceAlert> {
        self.alerts.iter()
            .filter(|a| !a.acknowledged)
            .collect()
    }
    
    /// Get unresolved alerts
    pub fn get_unresolved_alerts(&self) -> Vec<&PerformanceAlert> {
        self.alerts.iter()
            .filter(|a| !a.resolved)
            .collect()
    }
    
    /// Get alerts by type
    pub fn get_alerts_by_type(&self, alert_type: AlertType) -> Vec<&PerformanceAlert> {
        self.alerts.iter()
            .filter(|a| a.alert_type == alert_type)
            .collect()
    }
    
    /// Get alerts by severity
    pub fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Vec<&PerformanceAlert> {
        self.alerts.iter()
            .filter(|a| a.severity == severity)
            .collect()
    }
    
    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        for alert in self.alerts.iter_mut() {
            if alert.id == alert_id {
                alert.acknowledge();
                return true;
            }
        }
        false
    }
    
    /// Resolve an alert
    pub fn resolve_alert(&mut self, alert_id: &str) -> bool {
        for alert in self.alerts.iter_mut() {
            if alert.id == alert_id {
                alert.resolve();
                return true;
            }
        }
        false
    }
    
    /// Clear all alerts
    pub fn clear_alerts(&mut self) {
        self.alerts.clear();
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            alerts: self.alerts.clone(),
            alert_handlers: Vec::new(), // Don't clone handlers
            last_monitoring_time: self.last_monitoring_time,
            monitoring_thread: None, // Don't clone the thread handle
        }
    }
}

/// Global performance monitor instance
lazy_static::lazy_static! {
    static ref PERFORMANCE_MONITOR: Arc<Mutex<PerformanceMonitor>> = Arc::new(Mutex::new(
        PerformanceMonitor::new(MonitoringConfig::default())
    ));
}

/// Get the global performance monitor instance
pub fn get_performance_monitor() -> Arc<Mutex<PerformanceMonitor>> {
    PERFORMANCE_MONITOR.clone()
}

/// Configure performance monitoring
pub fn configure(config: MonitoringConfig) {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    
    // Stop monitoring if it's running
    let _ = monitor.stop_monitoring();
    
    // Update configuration
    monitor.config = config.clone();
    
    // Restart monitoring if enabled
    if config.enabled {
        let _ = monitor.start_monitoring();
        info!("Performance monitoring enabled with interval of {}ms", 
              config.monitoring_interval_ms);
    } else {
        info!("Performance monitoring disabled");
    }
}

/// Enable performance monitoring
pub fn enable() {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.config.enabled = true;
    let _ = monitor.start_monitoring();
    info!("Performance monitoring enabled");
}

/// Disable performance monitoring
pub fn disable() {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.config.enabled = false;
    let _ = monitor.stop_monitoring();
    info!("Performance monitoring disabled");
}

/// Add a performance alert
pub fn add_alert(alert: PerformanceAlert) {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    
    if !monitor.config.enabled {
        return;
    }
    
    monitor.add_alert(alert);
}

/// Add an alert handler
pub fn add_alert_handler<F>(handler: F)
where
    F: Fn(&PerformanceAlert) + Send + Sync + 'static,
{
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.add_alert_handler(handler);
}

/// Get all alerts
pub fn get_alerts() -> Vec<PerformanceAlert> {
    let monitor = PERFORMANCE_MONITOR.lock().unwrap();
    
    monitor.get_alerts()
        .iter()
        .map(|a| (*a).clone())
        .collect()
}

/// Get unacknowledged alerts
pub fn get_unacknowledged_alerts() -> Vec<PerformanceAlert> {
    let monitor = PERFORMANCE_MONITOR.lock().unwrap();
    
    monitor.get_unacknowledged_alerts()
        .iter()
        .map(|a| (*a).clone())
        .collect()
}

/// Get unresolved alerts
pub fn get_unresolved_alerts() -> Vec<PerformanceAlert> {
    let monitor = PERFORMANCE_MONITOR.lock().unwrap();
    
    monitor.get_unresolved_alerts()
        .iter()
        .map(|a| (*a).clone())
        .collect()
}

/// Acknowledge an alert
pub fn acknowledge_alert(alert_id: &str) -> bool {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.acknowledge_alert(alert_id)
}

/// Resolve an alert
pub fn resolve_alert(alert_id: &str) -> bool {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.resolve_alert(alert_id)
}

/// Clear all alerts
pub fn clear_alerts() {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.clear_alerts();
}

/// Initialize the performance monitoring system
pub fn init() {
    info!("Initializing performance monitoring system");
    
    // Register default alert handlers
    
    // Log alerts to the console
    add_alert_handler(|alert| {
        match alert.severity {
            AlertSeverity::Info => info!("Performance alert: {} - {}", alert.title, alert.description),
            AlertSeverity::Warning => warn!("Performance alert: {} - {}", alert.title, alert.description),
            AlertSeverity::Error => error!("Performance alert: {} - {}", alert.title, alert.description),
            AlertSeverity::Critical => error!("CRITICAL performance alert: {} - {}", alert.title, alert.description),
        }
    });
    
    // Initialize the monitor
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    
    // Start monitoring if enabled
    if monitor.config.enabled {
        let _ = monitor.start_monitoring();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_performance_alert_creation() {
        let alert = PerformanceAlert::new(
            AlertType::HighResourceUsage,
            AlertSeverity::Warning,
            "Test Alert",
            "This is a test alert"
        )
        .with_context("test_key", "test_value");
        
        assert_eq!(alert.alert_type, AlertType::HighResourceUsage);
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.title, "Test Alert");
        assert_eq!(alert.description, "This is a test alert");
        assert_eq!(alert.context.get("test_key"), Some(&"test_value".to_string()));
        assert!(!alert.acknowledged);
        assert!(!alert.resolved);
    }
    
    #[test]
    fn test_performance_monitor() {
        let config = MonitoringConfig {
            enabled: true,
            monitoring_interval_ms: 100,
            max_alerts: 10,
            cpu_usage_threshold: 80.0,
            memory_usage_threshold: 80.0,
            disk_usage_threshold: 80.0,
            profile_change_threshold: 5.0,
            fallback_activation_threshold: 5.0,
            auto_capture_resources: false,
        };
        
        let mut monitor = PerformanceMonitor::new(config);
        
        // Add some alerts
        let alert1 = PerformanceAlert::new(
            AlertType::HighResourceUsage,
            AlertSeverity::Warning,
            "High CPU Usage",
            "CPU usage is at 90%"
        );
        
        let alert2 = PerformanceAlert::new(
            AlertType::FrequentProfileChanges,
            AlertSeverity::Error,
            "Frequent Profile Changes",
            "Profile is changing too frequently"
        );
        
        monitor.add_alert(alert1);
        monitor.add_alert(alert2);
        
        // Check alerts were added
        assert_eq!(monitor.alerts.len(), 2);
        
        // Check alert retrieval
        let high_resource_alerts = monitor.get_alerts_by_type(AlertType::HighResourceUsage);
        assert_eq!(high_resource_alerts.len(), 1);
        
        let error_alerts = monitor.get_alerts_by_severity(AlertSeverity::Error);
        assert_eq!(error_alerts.len(), 1);
        
        // Check alert acknowledgement
        let alert_id = monitor.alerts[0].id.clone();
        assert!(monitor.acknowledge_alert(&alert_id));
        assert!(monitor.alerts[0].acknowledged);
        
        // Check alert resolution
        assert!(monitor.resolve_alert(&alert_id));
        assert!(monitor.alerts[0].resolved);
    }
}