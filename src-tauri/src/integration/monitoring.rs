//! Monitoring and alerting for external services
//!
//! This module provides mechanisms to monitor the health and performance
//! of external system integrations and generate alerts when issues occur.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use tokio::time::sleep;
use crate::error::{Error, ErrorKind, Result};
use crate::integration::interfaces::ServiceStatus;

/// Severity level for alerts
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alerts that don't require action
    Info,
    
    /// Warning alerts that may require attention
    Warning,
    
    /// Error alerts that require action
    Error,
    
    /// Critical alerts that require immediate action
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Error => write!(f, "ERROR"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Alert generated for an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Unique identifier for the alert
    pub id: String,
    
    /// Service ID that the alert is for
    pub service_id: String,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Alert title
    pub title: String,
    
    /// Alert description
    pub description: String,
    
    /// When the alert was created
    pub created_at: DateTime<Utc>,
    
    /// When the alert was last updated
    pub updated_at: DateTime<Utc>,
    
    /// When the alert was resolved (if it has been)
    pub resolved_at: Option<DateTime<Utc>>,
    
    /// Whether the alert has been acknowledged
    pub acknowledged: bool,
    
    /// Additional context for the alert
    pub context: HashMap<String, String>,
}

impl Alert {
    /// Create a new alert
    pub fn new(
        service_id: &str,
        severity: AlertSeverity,
        title: &str,
        description: &str,
    ) -> Self {
        let now = Utc::now();
        let id = format!("alert-{}-{}", service_id, now.timestamp());
        
        Self {
            id,
            service_id: service_id.to_string(),
            severity,
            title: title.to_string(),
            description: description.to_string(),
            created_at: now,
            updated_at: now,
            resolved_at: None,
            acknowledged: false,
            context: HashMap::new(),
        }
    }
    
    /// Check if the alert is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolved_at.is_some()
    }
    
    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Acknowledge the alert
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
        self.updated_at = Utc::now();
    }
    
    /// Add context to the alert
    pub fn add_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
        self.updated_at = Utc::now();
    }
}

/// Health check configuration for an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// How often to check the service health (in seconds)
    pub check_interval_seconds: u32,
    
    /// Timeout for health checks (in seconds)
    pub timeout_seconds: u32,
    
    /// Number of consecutive failures before considering the service unhealthy
    pub failure_threshold: u32,
    
    /// Number of consecutive successes before considering the service healthy again
    pub success_threshold: u32,
    
    /// Whether to automatically create alerts for health check failures
    pub create_alerts: bool,
    
    /// Severity of alerts created for health check failures
    pub alert_severity: AlertSeverity,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 60,
            timeout_seconds: 10,
            failure_threshold: 3,
            success_threshold: 2,
            create_alerts: true,
            alert_severity: AlertSeverity::Warning,
        }
    }
}

/// Result of a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Service ID that was checked
    pub service_id: String,
    
    /// When the check was performed
    pub check_time: DateTime<Utc>,
    
    /// Whether the check was successful
    pub success: bool,
    
    /// Status of the service
    pub status: ServiceStatus,
    
    /// Response time in milliseconds (if successful)
    pub response_time_ms: Option<u64>,
    
    /// Error message (if unsuccessful)
    pub error_message: Option<String>,
    
    /// Additional details about the check
    pub details: HashMap<String, String>,
}

/// Health check history for a service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckHistory {
    /// Service ID
    pub service_id: String,
    
    /// Recent health check results
    pub recent_results: VecDeque<HealthCheckResult>,
    
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    
    /// Number of consecutive successes
    pub consecutive_successes: u32,
    
    /// Whether the service is currently considered healthy
    pub is_healthy: bool,
    
    /// When the service was last checked
    pub last_check_time: Option<DateTime<Utc>>,
    
    /// When the service status last changed
    pub last_status_change: Option<DateTime<Utc>>,
}

impl HealthCheckHistory {
    /// Create a new health check history
    pub fn new(service_id: &str) -> Self {
        Self {
            service_id: service_id.to_string(),
            recent_results: VecDeque::with_capacity(10),
            consecutive_failures: 0,
            consecutive_successes: 0,
            is_healthy: true, // Assume healthy initially
            last_check_time: None,
            last_status_change: None,
        }
    }
    
    /// Add a health check result
    pub fn add_result(&mut self, result: HealthCheckResult, config: &HealthCheckConfig) -> bool {
        let status_changed = if result.success {
            self.consecutive_failures = 0;
            self.consecutive_successes += 1;
            
            if !self.is_healthy && self.consecutive_successes >= config.success_threshold {
                self.is_healthy = true;
                true
            } else {
                false
            }
        } else {
            self.consecutive_successes = 0;
            self.consecutive_failures += 1;
            
            if self.is_healthy && self.consecutive_failures >= config.failure_threshold {
                self.is_healthy = false;
                true
            } else {
                false
            }
        };
        
        if status_changed {
            self.last_status_change = Some(result.check_time);
        }
        
        self.last_check_time = Some(result.check_time);
        
        // Keep only the most recent results
        if self.recent_results.len() >= 10 {
            self.recent_results.pop_front();
        }
        self.recent_results.push_back(result);
        
        status_changed
    }
}

/// Trait for health check strategies
#[async_trait]
pub trait HealthCheckStrategy: Send + Sync {
    /// Perform a health check for a service
    async fn check_health(&self, service_id: &str) -> Result<HealthCheckResult>;
    
    /// Get the health check configuration for a service
    fn get_config(&self, service_id: &str) -> Result<HealthCheckConfig>;
    
    /// Set the health check configuration for a service
    fn set_config(&self, service_id: &str, config: HealthCheckConfig) -> Result<()>;
    
    /// Get the health check history for a service
    fn get_history(&self, service_id: &str) -> Result<HealthCheckHistory>;
}

/// HTTP health check strategy
pub struct HttpHealthCheck {
    /// Base URLs for services
    base_urls: RwLock<HashMap<String, String>>,
    
    /// Health check configurations by service ID
    configs: RwLock<HashMap<String, HealthCheckConfig>>,
    
    /// Health check histories by service ID
    histories: RwLock<HashMap<String, HealthCheckHistory>>,
}

impl HttpHealthCheck {
    /// Create a new HTTP health check strategy
    pub fn new() -> Self {
        Self {
            base_urls: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
            histories: RwLock::new(HashMap::new()),
        }
    }
    
    /// Initialize a service with a base URL and health check configuration
    pub fn init_service(
        &self,
        service_id: &str,
        base_url: &str,
        config: HealthCheckConfig,
    ) -> Result<()> {
        let mut base_urls = self.base_urls.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on base URLs"
            )
        })?;
        
        let mut configs = self.configs.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on health check configs"
            )
        })?;
        
        let mut histories = self.histories.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on health check histories"
            )
        })?;
        
        base_urls.insert(service_id.to_string(), base_url.to_string());
        configs.insert(service_id.to_string(), config);
        histories.insert(service_id.to_string(), HealthCheckHistory::new(service_id));
        
        Ok(())
    }
}

#[async_trait]
impl HealthCheckStrategy for HttpHealthCheck {
    async fn check_health(&self, service_id: &str) -> Result<HealthCheckResult> {
        let base_urls = self.base_urls.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on base URLs"
            )
        })?;
        
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on health check configs"
            )
        })?;
        
        let base_url = base_urls.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Base URL not found for service: {}", service_id)
            )
        })?;
        
        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Health check configuration not found for service: {}", service_id)
            )
        })?;
        
        let start_time = std::time::Instant::now();
        let check_time = Utc::now();
        
        // Create a client with a timeout
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds as u64))
            .build()
            .map_err(|e| {
                Error::new(
                    ErrorKind::Internal,
                    &format!("Failed to create HTTP client: {}", e)
                )
            })?;
        
        // Perform the health check
        let result = client.get(base_url).send().await;
        
        let elapsed = start_time.elapsed();
        let response_time_ms = elapsed.as_millis() as u64;
        
        let (success, status, error_message) = match result {
            Ok(response) => {
                let status_code = response.status();
                let success = status_code.is_success();
                let service_status = if success {
                    ServiceStatus::Available
                } else if status_code.is_server_error() {
                    ServiceStatus::Unavailable
                } else {
                    ServiceStatus::Degraded
                };
                
                let error_message = if !success {
                    Some(format!("HTTP status code: {}", status_code))
                } else {
                    None
                };
                
                (success, service_status, error_message)
            },
            Err(e) => {
                let error_message = format!("HTTP request failed: {}", e);
                (false, ServiceStatus::Unavailable, Some(error_message))
            }
        };
        
        let mut details = HashMap::new();
        details.insert("url".to_string(), base_url.to_string());
        
        let result = HealthCheckResult {
            service_id: service_id.to_string(),
            check_time,
            success,
            status,
            response_time_ms: Some(response_time_ms),
            error_message,
            details,
        };
        
        // Update the history
        let mut histories = self.histories.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on health check histories"
            )
        })?;
        
        let history = histories.entry(service_id.to_string()).or_insert_with(|| HealthCheckHistory::new(service_id));
        history.add_result(result.clone(), config);
        
        Ok(result)
    }
    
    fn get_config(&self, service_id: &str) -> Result<HealthCheckConfig> {
        let configs = self.configs.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on health check configs"
            )
        })?;
        
        let config = configs.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Health check configuration not found for service: {}", service_id)
            )
        })?;
        
        Ok(config.clone())
    }
    
    fn set_config(&self, service_id: &str, config: HealthCheckConfig) -> Result<()> {
        let mut configs = self.configs.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on health check configs"
            )
        })?;
        
        configs.insert(service_id.to_string(), config);
        
        Ok(())
    }
    
    fn get_history(&self, service_id: &str) -> Result<HealthCheckHistory> {
        let histories = self.histories.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on health check histories"
            )
        })?;
        
        let history = histories.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Health check history not found for service: {}", service_id)
            )
        })?;
        
        Ok(history.clone())
    }
}

/// Trait for alert handlers
#[async_trait]
pub trait AlertHandler: Send + Sync {
    /// Handle a new alert
    async fn handle_alert(&self, alert: &Alert) -> Result<()>;
    
    /// Handle a resolved alert
    async fn handle_resolved_alert(&self, alert: &Alert) -> Result<()>;
}

/// Console alert handler (logs alerts to the console)
pub struct ConsoleAlertHandler;

#[async_trait]
impl AlertHandler for ConsoleAlertHandler {
    async fn handle_alert(&self, alert: &Alert) -> Result<()> {
        tracing::warn!(
            "ALERT [{}] {}: {} (Service: {})",
            alert.severity,
            alert.title,
            alert.description,
            alert.service_id
        );
        
        Ok(())
    }
    
    async fn handle_resolved_alert(&self, alert: &Alert) -> Result<()> {
        tracing::info!(
            "RESOLVED ALERT [{}] {}: {} (Service: {})",
            alert.severity,
            alert.title,
            alert.description,
            alert.service_id
        );
        
        Ok(())
    }
}

/// Monitoring service for external integrations
pub struct MonitoringService {
    /// Health check strategy
    health_checker: Arc<dyn HealthCheckStrategy>,
    
    /// Alert handlers
    alert_handlers: Vec<Arc<dyn AlertHandler>>,
    
    /// Active alerts by ID
    active_alerts: RwLock<HashMap<String, Alert>>,
    
    /// Alert history
    alert_history: RwLock<VecDeque<Alert>>,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(health_checker: Arc<dyn HealthCheckStrategy>) -> Self {
        Self {
            health_checker,
            alert_handlers: Vec::new(),
            active_alerts: RwLock::new(HashMap::new()),
            alert_history: RwLock::new(VecDeque::with_capacity(100)),
        }
    }
    
    /// Add an alert handler
    pub fn add_alert_handler(&mut self, handler: Arc<dyn AlertHandler>) {
        self.alert_handlers.push(handler);
    }
    
    /// Initialize a service for monitoring
    pub fn init_service(
        &self,
        service_id: &str,
        base_url: &str,
        config: HealthCheckConfig,
    ) -> Result<()> {
        if let Some(http_checker) = Arc::get_mut(&mut self.health_checker.clone()) {
            if let Some(http) = http_checker.downcast_mut::<HttpHealthCheck>() {
                http.init_service(service_id, base_url, config)?;
            }
        }
        
        Ok(())
    }
    
    /// Check the health of a service
    pub async fn check_service_health(&self, service_id: &str) -> Result<HealthCheckResult> {
        let result = self.health_checker.check_health(service_id).await?;
        
        // Get the service's health check history
        let history = self.health_checker.get_history(service_id)?;
        
        // Get the service's health check configuration
        let config = self.health_checker.get_config(service_id)?;
        
        // Check if we need to create or resolve alerts
        if config.create_alerts {
            if !history.is_healthy {
                // Service is unhealthy, create an alert if one doesn't exist
                let mut active_alerts = self.active_alerts.write().map_err(|_| {
                    Error::new(
                        ErrorKind::Internal,
                        "Failed to acquire write lock on active alerts"
                    )
                })?;
                
                let alert_id = format!("health-{}", service_id);
                
                if !active_alerts.contains_key(&alert_id) {
                    let alert = Alert::new(
                        service_id,
                        config.alert_severity,
                        &format!("Service {} is unhealthy", service_id),
                        &format!(
                            "Service {} has failed {} consecutive health checks. Last error: {}",
                            service_id,
                            history.consecutive_failures,
                            result.error_message.as_deref().unwrap_or("Unknown error")
                        ),
                    );
                    
                    // Add the alert to active alerts
                    active_alerts.insert(alert_id, alert.clone());
                    
                    // Add the alert to history
                    let mut alert_history = self.alert_history.write().map_err(|_| {
                        Error::new(
                            ErrorKind::Internal,
                            "Failed to acquire write lock on alert history"
                        )
                    })?;
                    
                    if alert_history.len() >= 100 {
                        alert_history.pop_front();
                    }
                    alert_history.push_back(alert.clone());
                    
                    // Notify alert handlers
                    for handler in &self.alert_handlers {
                        handler.handle_alert(&alert).await?;
                    }
                }
            } else if history.consecutive_successes >= config.success_threshold {
                // Service is healthy, resolve any existing alerts
                let mut active_alerts = self.active_alerts.write().map_err(|_| {
                    Error::new(
                        ErrorKind::Internal,
                        "Failed to acquire write lock on active alerts"
                    )
                })?;
                
                let alert_id = format!("health-{}", service_id);
                
                if let Some(mut alert) = active_alerts.remove(&alert_id) {
                    // Resolve the alert
                    alert.resolve();
                    
                    // Add the resolved alert to history
                    let mut alert_history = self.alert_history.write().map_err(|_| {
                        Error::new(
                            ErrorKind::Internal,
                            "Failed to acquire write lock on alert history"
                        )
                    })?;
                    
                    if alert_history.len() >= 100 {
                        alert_history.pop_front();
                    }
                    alert_history.push_back(alert.clone());
                    
                    // Notify alert handlers
                    for handler in &self.alert_handlers {
                        handler.handle_resolved_alert(&alert).await?;
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// Create a custom alert
    pub async fn create_alert(
        &self,
        service_id: &str,
        severity: AlertSeverity,
        title: &str,
        description: &str,
    ) -> Result<Alert> {
        let alert = Alert::new(service_id, severity, title, description);
        
        // Add the alert to active alerts
        let mut active_alerts = self.active_alerts.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on active alerts"
            )
        })?;
        
        active_alerts.insert(alert.id.clone(), alert.clone());
        
        // Add the alert to history
        let mut alert_history = self.alert_history.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on alert history"
            )
        })?;
        
        if alert_history.len() >= 100 {
            alert_history.pop_front();
        }
        alert_history.push_back(alert.clone());
        
        // Notify alert handlers
        for handler in &self.alert_handlers {
            handler.handle_alert(&alert).await?;
        }
        
        Ok(alert)
    }
    
    /// Resolve an alert
    pub async fn resolve_alert(&self, alert_id: &str) -> Result<Alert> {
        let mut active_alerts = self.active_alerts.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on active alerts"
            )
        })?;
        
        let mut alert = active_alerts.remove(alert_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Alert with ID {} not found", alert_id)
            )
        })?;
        
        // Resolve the alert
        alert.resolve();
        
        // Add the resolved alert to history
        let mut alert_history = self.alert_history.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on alert history"
            )
        })?;
        
        if alert_history.len() >= 100 {
            alert_history.pop_front();
        }
        alert_history.push_back(alert.clone());
        
        // Notify alert handlers
        for handler in &self.alert_handlers {
            handler.handle_resolved_alert(&alert).await?;
        }
        
        Ok(alert)
    }
    
    /// Acknowledge an alert
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<Alert> {
        let mut active_alerts = self.active_alerts.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on active alerts"
            )
        })?;
        
        let alert = active_alerts.get_mut(alert_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Alert with ID {} not found", alert_id)
            )
        })?;
        
        // Acknowledge the alert
        alert.acknowledge();
        
        Ok(alert.clone())
    }
    
    /// Get all active alerts
    pub fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let active_alerts = self.active_alerts.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on active alerts"
            )
        })?;
        
        Ok(active_alerts.values().cloned().collect())
    }
    
    /// Get active alerts for a service
    pub fn get_active_alerts_for_service(&self, service_id: &str) -> Result<Vec<Alert>> {
        let active_alerts = self.active_alerts.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on active alerts"
            )
        })?;
        
        Ok(active_alerts.values()
            .filter(|a| a.service_id == service_id)
            .cloned()
            .collect())
    }
    
    /// Get alert history
    pub fn get_alert_history(&self) -> Result<Vec<Alert>> {
        let alert_history = self.alert_history.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on alert history"
            )
        })?;
        
        Ok(alert_history.iter().cloned().collect())
    }
    
    /// Get alert history for a service
    pub fn get_alert_history_for_service(&self, service_id: &str) -> Result<Vec<Alert>> {
        let alert_history = self.alert_history.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on alert history"
            )
        })?;
        
        Ok(alert_history.iter()
            .filter(|a| a.service_id == service_id)
            .cloned()
            .collect())
    }
    
    /// Start the monitoring service
    pub async fn start(&self) -> Result<()> {
        // This would typically start background tasks to periodically check services
        // For simplicity, we're not implementing the full background monitoring here
        tracing::info!("Monitoring service started");
        Ok(())
    }
    
    /// Stop the monitoring service
    pub async fn stop(&self) -> Result<()> {
        // This would typically stop background tasks
        tracing::info!("Monitoring service stopped");
        Ok(())
    }
}

/// Metrics for an external service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Service ID
    pub service_id: String,
    
    /// When the metrics were collected
    pub collected_at: DateTime<Utc>,
    
    /// Total number of requests made to the service
    pub total_requests: u64,
    
    /// Number of successful requests
    pub successful_requests: u64,
    
    /// Number of failed requests
    pub failed_requests: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// 95th percentile response time in milliseconds
    pub p95_response_time_ms: f64,
    
    /// 99th percentile response time in milliseconds
    pub p99_response_time_ms: f64,
    
    /// Maximum response time in milliseconds
    pub max_response_time_ms: f64,
    
    /// Minimum response time in milliseconds
    pub min_response_time_ms: f64,
    
    /// Number of rate limit hits
    pub rate_limit_hits: u64,
    
    /// Number of quota exceeded events
    pub quota_exceeded: u64,
    
    /// Additional metrics as key-value pairs
    pub additional_metrics: HashMap<String, f64>,
}

impl ServiceMetrics {
    /// Create new empty service metrics
    pub fn new(service_id: &str) -> Self {
        Self {
            service_id: service_id.to_string(),
            collected_at: Utc::now(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time_ms: 0.0,
            p95_response_time_ms: 0.0,
            p99_response_time_ms: 0.0,
            max_response_time_ms: 0.0,
            min_response_time_ms: 0.0,
            rate_limit_hits: 0,
            quota_exceeded: 0,
            additional_metrics: HashMap::new(),
        }
    }
}

/// Trait for metrics collectors
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Record a request to an external service
    async fn record_request(
        &self,
        service_id: &str,
        success: bool,
        response_time_ms: u64,
        details: HashMap<String, String>,
    ) -> Result<()>;
    
    /// Record a rate limit hit
    async fn record_rate_limit_hit(&self, service_id: &str) -> Result<()>;
    
    /// Record a quota exceeded event
    async fn record_quota_exceeded(&self, service_id: &str) -> Result<()>;
    
    /// Record a custom metric
    async fn record_custom_metric(&self, service_id: &str, name: &str, value: f64) -> Result<()>;
    
    /// Get metrics for a service
    async fn get_metrics(&self, service_id: &str) -> Result<ServiceMetrics>;
    
    /// Reset metrics for a service
    async fn reset_metrics(&self, service_id: &str) -> Result<()>;
}

/// In-memory metrics collector
pub struct InMemoryMetricsCollector {
    /// Metrics by service ID
    metrics: RwLock<HashMap<String, ServiceMetrics>>,
    
    /// Response times by service ID for percentile calculations
    response_times: RwLock<HashMap<String, Vec<u64>>>,
}

impl InMemoryMetricsCollector {
    /// Create a new in-memory metrics collector
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(HashMap::new()),
            response_times: RwLock::new(HashMap::new()),
        }
    }
    
    /// Initialize metrics for a service
    pub fn init_service(&self, service_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let mut response_times = self.response_times.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on response times"
            )
        })?;
        
        metrics.insert(service_id.to_string(), ServiceMetrics::new(service_id));
        response_times.insert(service_id.to_string(), Vec::new());
        
        Ok(())
    }
    
    /// Calculate percentiles for response times
    fn calculate_percentiles(&self, service_id: &str) -> Result<(f64, f64, f64, f64)> {
        let response_times = self.response_times.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on response times"
            )
        })?;
        
        let times = response_times.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Response times not found for service: {}", service_id)
            )
        })?;
        
        if times.is_empty() {
            return Ok((0.0, 0.0, 0.0, 0.0));
        }
        
        let mut sorted_times = times.clone();
        sorted_times.sort();
        
        let len = sorted_times.len();
        let avg = sorted_times.iter().sum::<u64>() as f64 / len as f64;
        let min = *sorted_times.first().unwrap() as f64;
        let max = *sorted_times.last().unwrap() as f64;
        
        let p95_idx = (len as f64 * 0.95) as usize;
        let p99_idx = (len as f64 * 0.99) as usize;
        
        let p95 = sorted_times[p95_idx.min(len - 1)] as f64;
        let p99 = sorted_times[p99_idx.min(len - 1)] as f64;
        
        Ok((avg, p95, p99, min))
    }
    
    /// Update metrics with calculated percentiles
    fn update_metrics(&self, service_id: &str) -> Result<()> {
        let (avg, p95, p99, min) = self.calculate_percentiles(service_id)?;
        
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let metric = metrics.get_mut(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Metrics not found for service: {}", service_id)
            )
        })?;
        
        metric.avg_response_time_ms = avg;
        metric.p95_response_time_ms = p95;
        metric.p99_response_time_ms = p99;
        metric.min_response_time_ms = min;
        
        Ok(())
    }
}

#[async_trait]
impl MetricsCollector for InMemoryMetricsCollector {
    async fn record_request(
        &self,
        service_id: &str,
        success: bool,
        response_time_ms: u64,
        _details: HashMap<String, String>,
    ) -> Result<()> {
        // Update metrics
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let metric = metrics.entry(service_id.to_string()).or_insert_with(|| ServiceMetrics::new(service_id));
        metric.total_requests += 1;
        
        if success {
            metric.successful_requests += 1;
        } else {
            metric.failed_requests += 1;
        }
        
        metric.max_response_time_ms = metric.max_response_time_ms.max(response_time_ms as f64);
        metric.collected_at = Utc::now();
        
        // Update response times
        let mut response_times = self.response_times.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on response times"
            )
        })?;
        
        let times = response_times.entry(service_id.to_string()).or_insert_with(Vec::new);
        times.push(response_time_ms);
        
        // Keep only the most recent 1000 response times
        if times.len() > 1000 {
            times.remove(0);
        }
        
        // Update percentiles
        drop(metrics);
        drop(response_times);
        self.update_metrics(service_id)?;
        
        Ok(())
    }
    
    async fn record_rate_limit_hit(&self, service_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let metric = metrics.entry(service_id.to_string()).or_insert_with(|| ServiceMetrics::new(service_id));
        metric.rate_limit_hits += 1;
        metric.collected_at = Utc::now();
        
        Ok(())
    }
    
    async fn record_quota_exceeded(&self, service_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let metric = metrics.entry(service_id.to_string()).or_insert_with(|| ServiceMetrics::new(service_id));
        metric.quota_exceeded += 1;
        metric.collected_at = Utc::now();
        
        Ok(())
    }
    
    async fn record_custom_metric(&self, service_id: &str, name: &str, value: f64) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let metric = metrics.entry(service_id.to_string()).or_insert_with(|| ServiceMetrics::new(service_id));
        metric.additional_metrics.insert(name.to_string(), value);
        metric.collected_at = Utc::now();
        
        Ok(())
    }
    
    async fn get_metrics(&self, service_id: &str) -> Result<ServiceMetrics> {
        let metrics = self.metrics.read().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire read lock on metrics"
            )
        })?;
        
        let metric = metrics.get(service_id).ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                &format!("Metrics not found for service: {}", service_id)
            )
        })?;
        
        Ok(metric.clone())
    }
    
    async fn reset_metrics(&self, service_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on metrics"
            )
        })?;
        
        let mut response_times = self.response_times.write().map_err(|_| {
            Error::new(
                ErrorKind::Internal,
                "Failed to acquire write lock on response times"
            )
        })?;
        
        metrics.insert(service_id.to_string(), ServiceMetrics::new(service_id));
        response_times.insert(service_id.to_string(), Vec::new());
        
        Ok(())
    }
}