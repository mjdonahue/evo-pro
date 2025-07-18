use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kameo::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::actors::metrics::{MetricType, MetricValue};
use crate::logging;

/// Performance metric types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PerformanceMetricType {
    /// Message processing time
    MessageProcessingTime,
    /// Message throughput (messages per second)
    MessageThroughput,
    /// Actor memory usage
    MemoryUsage,
    /// Actor CPU usage
    CpuUsage,
    /// Message queue length
    QueueLength,
    /// Error rate
    ErrorRate,
    /// Custom metric
    Custom(u32),
}

/// Performance metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMetricValue {
    /// Counter value (always increases)
    Counter(u64),
    /// Gauge value (can go up or down)
    Gauge(i64),
    /// Histogram value (distribution of values)
    Histogram(Vec<f64>),
    /// Timer value (duration)
    Timer(Duration),
    /// Rate value (events per second)
    Rate(f64),
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetric {
    /// Metric ID
    pub id: Uuid,
    /// Actor ID
    pub actor_id: ActorID,
    /// Metric type
    pub metric_type: PerformanceMetricType,
    /// Metric value
    pub value: PerformanceMetricValue,
    /// Timestamp when the metric was recorded
    pub timestamp: Instant,
    /// Labels associated with the metric
    pub labels: HashMap<String, String>,
}

impl PerformanceMetric {
    /// Create a new performance metric
    pub fn new(
        actor_id: ActorID,
        metric_type: PerformanceMetricType,
        value: PerformanceMetricValue,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            actor_id,
            metric_type,
            value,
            timestamp: Instant::now(),
            labels: HashMap::new(),
        }
    }
    
    /// Add a label to the metric
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }
}

/// Performance snapshot for an actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorPerformanceSnapshot {
    /// Actor ID
    pub actor_id: ActorID,
    /// Actor type name
    pub actor_type: String,
    /// Timestamp when the snapshot was taken
    pub timestamp: Instant,
    /// Message processing time (average)
    pub avg_processing_time: Option<Duration>,
    /// Message processing time (percentiles)
    pub processing_time_percentiles: Option<HashMap<String, Duration>>,
    /// Message throughput (messages per second)
    pub throughput: Option<f64>,
    /// Memory usage
    pub memory_usage: Option<usize>,
    /// CPU usage
    pub cpu_usage: Option<f64>,
    /// Message queue length
    pub queue_length: Option<usize>,
    /// Error rate
    pub error_rate: Option<f64>,
    /// Custom metrics
    pub custom_metrics: HashMap<String, PerformanceMetricValue>,
}

impl ActorPerformanceSnapshot {
    /// Create a new performance snapshot
    pub fn new(actor_id: ActorID, actor_type: impl Into<String>) -> Self {
        Self {
            actor_id,
            actor_type: actor_type.into(),
            timestamp: Instant::now(),
            avg_processing_time: None,
            processing_time_percentiles: None,
            throughput: None,
            memory_usage: None,
            cpu_usage: None,
            queue_length: None,
            error_rate: None,
            custom_metrics: HashMap::new(),
        }
    }
    
    /// Set the average processing time
    pub fn with_avg_processing_time(mut self, time: Duration) -> Self {
        self.avg_processing_time = Some(time);
        self
    }
    
    /// Set the processing time percentiles
    pub fn with_processing_time_percentiles(mut self, percentiles: HashMap<String, Duration>) -> Self {
        self.processing_time_percentiles = Some(percentiles);
        self
    }
    
    /// Set the throughput
    pub fn with_throughput(mut self, throughput: f64) -> Self {
        self.throughput = Some(throughput);
        self
    }
    
    /// Set the memory usage
    pub fn with_memory_usage(mut self, memory_usage: usize) -> Self {
        self.memory_usage = Some(memory_usage);
        self
    }
    
    /// Set the CPU usage
    pub fn with_cpu_usage(mut self, cpu_usage: f64) -> Self {
        self.cpu_usage = Some(cpu_usage);
        self
    }
    
    /// Set the queue length
    pub fn with_queue_length(mut self, queue_length: usize) -> Self {
        self.queue_length = Some(queue_length);
        self
    }
    
    /// Set the error rate
    pub fn with_error_rate(mut self, error_rate: f64) -> Self {
        self.error_rate = Some(error_rate);
        self
    }
    
    /// Add a custom metric
    pub fn with_custom_metric(
        mut self,
        name: impl Into<String>,
        value: PerformanceMetricValue,
    ) -> Self {
        self.custom_metrics.insert(name.into(), value);
        self
    }
}

/// Actor performance history
#[derive(Debug, Clone, Default)]
pub struct ActorPerformanceHistory {
    /// Actor ID
    pub actor_id: ActorID,
    /// Actor type name
    pub actor_type: String,
    /// Performance metrics
    pub metrics: HashMap<PerformanceMetricType, VecDeque<PerformanceMetric>>,
    /// Performance snapshots
    pub snapshots: VecDeque<ActorPerformanceSnapshot>,
    /// Maximum history length
    pub max_history_len: usize,
}

impl ActorPerformanceHistory {
    /// Create a new actor performance history
    pub fn new(actor_id: ActorID, actor_type: impl Into<String>) -> Self {
        Self {
            actor_id,
            actor_type: actor_type.into(),
            metrics: HashMap::new(),
            snapshots: VecDeque::new(),
            max_history_len: 100, // Default to 100 historical values
        }
    }
    
    /// Set the maximum history length
    pub fn with_max_history_len(mut self, max_len: usize) -> Self {
        self.max_history_len = max_len;
        self
    }
    
    /// Add a metric to the history
    pub fn add_metric(&mut self, metric: PerformanceMetric) {
        let metrics = self.metrics
            .entry(metric.metric_type)
            .or_insert_with(VecDeque::new);
        
        metrics.push_back(metric);
        
        // Trim history if it exceeds the maximum length
        if metrics.len() > self.max_history_len {
            metrics.pop_front();
        }
    }
    
    /// Add a snapshot to the history
    pub fn add_snapshot(&mut self, snapshot: ActorPerformanceSnapshot) {
        self.snapshots.push_back(snapshot);
        
        // Trim history if it exceeds the maximum length
        if self.snapshots.len() > self.max_history_len {
            self.snapshots.pop_front();
        }
    }
    
    /// Get the latest snapshot
    pub fn latest_snapshot(&self) -> Option<&ActorPerformanceSnapshot> {
        self.snapshots.back()
    }
    
    /// Get metrics of a specific type
    pub fn get_metrics(&self, metric_type: PerformanceMetricType) -> Vec<&PerformanceMetric> {
        self.metrics
            .get(&metric_type)
            .map(|metrics| metrics.iter().collect())
            .unwrap_or_default()
    }
    
    /// Calculate statistics for a specific metric type
    pub fn calculate_metric_stats(
        &self,
        metric_type: PerformanceMetricType,
    ) -> Option<MetricStats> {
        let metrics = self.get_metrics(metric_type);
        
        if metrics.is_empty() {
            return None;
        }
        
        let mut values = Vec::new();
        
        for metric in metrics {
            match &metric.value {
                PerformanceMetricValue::Counter(v) => values.push(*v as f64),
                PerformanceMetricValue::Gauge(v) => values.push(*v as f64),
                PerformanceMetricValue::Rate(v) => values.push(*v),
                PerformanceMetricValue::Timer(d) => values.push(d.as_secs_f64() * 1000.0), // Convert to milliseconds
                PerformanceMetricValue::Histogram(h) => values.extend(h.iter().copied()),
            }
        }
        
        if values.is_empty() {
            return None;
        }
        
        // Sort values for percentile calculations
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let min = values[0];
        let max = values[count - 1];
        let mean = sum / count as f64;
        
        // Calculate percentiles
        let p50 = percentile(&values, 0.5);
        let p90 = percentile(&values, 0.9);
        let p95 = percentile(&values, 0.95);
        let p99 = percentile(&values, 0.99);
        
        Some(MetricStats {
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
}

/// Statistics for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    /// Number of samples
    pub count: usize,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Mean value
    pub mean: f64,
    /// Sum of values
    pub sum: f64,
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
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

/// Performance monitoring system
#[derive(Debug, Clone, Default)]
pub struct PerformanceMonitor {
    /// Actor performance histories by ID
    pub actor_histories: HashMap<ActorID, ActorPerformanceHistory>,
    /// Global performance metrics
    pub global_metrics: HashMap<String, VecDeque<PerformanceMetric>>,
    /// Maximum history length for global metrics
    pub max_global_history_len: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            actor_histories: HashMap::new(),
            global_metrics: HashMap::new(),
            max_global_history_len: 100, // Default to 100 historical values
        }
    }
    
    /// Set the maximum history length for global metrics
    pub fn with_max_global_history_len(mut self, max_len: usize) -> Self {
        self.max_global_history_len = max_len;
        self
    }
    
    /// Register an actor with the performance monitor
    pub fn register_actor(&mut self, actor_id: ActorID, actor_type: impl Into<String>) {
        let history = ActorPerformanceHistory::new(actor_id.clone(), actor_type);
        self.actor_histories.insert(actor_id, history);
    }
    
    /// Record a metric for an actor
    pub fn record_actor_metric(
        &mut self,
        actor_id: ActorID,
        metric_type: PerformanceMetricType,
        value: PerformanceMetricValue,
    ) {
        if let Some(history) = self.actor_histories.get_mut(&actor_id) {
            let metric = PerformanceMetric::new(actor_id, metric_type, value);
            history.add_metric(metric);
        }
    }
    
    /// Record a snapshot for an actor
    pub fn record_actor_snapshot(&mut self, snapshot: ActorPerformanceSnapshot) {
        if let Some(history) = self.actor_histories.get_mut(&snapshot.actor_id) {
            history.add_snapshot(snapshot);
        }
    }
    
    /// Record a global metric
    pub fn record_global_metric(
        &mut self,
        name: impl Into<String>,
        value: PerformanceMetricValue,
    ) {
        let name = name.into();
        let metric = PerformanceMetric::new(
            "global".into(), // Use a special actor ID for global metrics
            PerformanceMetricType::Custom(0),
            value,
        ).with_label("name", name.clone());
        
        let metrics = self.global_metrics
            .entry(name)
            .or_insert_with(VecDeque::new);
        
        metrics.push_back(metric);
        
        // Trim history if it exceeds the maximum length
        if metrics.len() > self.max_global_history_len {
            metrics.pop_front();
        }
    }
    
    /// Get an actor's performance history
    pub fn get_actor_history(&self, actor_id: &ActorID) -> Option<&ActorPerformanceHistory> {
        self.actor_histories.get(actor_id)
    }
    
    /// Get global metrics by name
    pub fn get_global_metrics(&self, name: &str) -> Vec<&PerformanceMetric> {
        self.global_metrics
            .get(name)
            .map(|metrics| metrics.iter().collect())
            .unwrap_or_default()
    }
    
    /// Calculate statistics for a global metric
    pub fn calculate_global_metric_stats(&self, name: &str) -> Option<MetricStats> {
        let metrics = self.get_global_metrics(name);
        
        if metrics.is_empty() {
            return None;
        }
        
        let mut values = Vec::new();
        
        for metric in metrics {
            match &metric.value {
                PerformanceMetricValue::Counter(v) => values.push(*v as f64),
                PerformanceMetricValue::Gauge(v) => values.push(*v as f64),
                PerformanceMetricValue::Rate(v) => values.push(*v),
                PerformanceMetricValue::Timer(d) => values.push(d.as_secs_f64() * 1000.0), // Convert to milliseconds
                PerformanceMetricValue::Histogram(h) => values.extend(h.iter().copied()),
            }
        }
        
        if values.is_empty() {
            return None;
        }
        
        // Sort values for percentile calculations
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let count = values.len();
        let sum: f64 = values.iter().sum();
        let min = values[0];
        let max = values[count - 1];
        let mean = sum / count as f64;
        
        // Calculate percentiles
        let p50 = percentile(&values, 0.5);
        let p90 = percentile(&values, 0.9);
        let p95 = percentile(&values, 0.95);
        let p99 = percentile(&values, 0.99);
        
        Some(MetricStats {
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
    
    /// Export performance data as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ExportData {
            actor_snapshots: HashMap<String, Vec<ActorPerformanceSnapshot>>,
            global_metrics: HashMap<String, Vec<GlobalMetricExport>>,
        }
        
        #[derive(Serialize)]
        struct GlobalMetricExport {
            value: f64,
            timestamp_ms: u64,
        }
        
        let mut actor_snapshots = HashMap::new();
        
        // Export the latest snapshot for each actor
        for (actor_id, history) in &self.actor_histories {
            if let Some(snapshots) = history.snapshots.iter().collect::<Vec<_>>().last() {
                actor_snapshots.insert(actor_id.to_string(), vec![(*snapshots).clone()]);
            }
        }
        
        // Export global metrics
        let mut global_metrics = HashMap::new();
        
        for (name, metrics) in &self.global_metrics {
            let exports: Vec<GlobalMetricExport> = metrics
                .iter()
                .filter_map(|metric| {
                    let value = match &metric.value {
                        PerformanceMetricValue::Counter(v) => Some(*v as f64),
                        PerformanceMetricValue::Gauge(v) => Some(*v as f64),
                        PerformanceMetricValue::Rate(v) => Some(*v),
                        PerformanceMetricValue::Timer(d) => Some(d.as_secs_f64() * 1000.0), // Convert to milliseconds
                        PerformanceMetricValue::Histogram(_) => None, // Skip histograms for simplicity
                    }?;
                    
                    let now = Instant::now();
                    let timestamp_ms = now.duration_since(metric.timestamp).as_millis() as u64;
                    
                    Some(GlobalMetricExport {
                        value,
                        timestamp_ms,
                    })
                })
                .collect();
            
            global_metrics.insert(name.clone(), exports);
        }
        
        let export_data = ExportData {
            actor_snapshots,
            global_metrics,
        };
        
        serde_json::to_string_pretty(&export_data)
    }
}

/// Global performance monitor instance
lazy_static::lazy_static! {
    static ref PERFORMANCE_MONITOR: Arc<Mutex<PerformanceMonitor>> = Arc::new(Mutex::new(PerformanceMonitor::new()));
}

/// Get the global performance monitor instance
pub fn get_performance_monitor() -> Arc<Mutex<PerformanceMonitor>> {
    PERFORMANCE_MONITOR.clone()
}

/// Register an actor with the performance monitor
pub fn register_actor(actor_id: ActorID, actor_type: impl Into<String>) {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.register_actor(actor_id, actor_type);
}

/// Record a metric for an actor
pub fn record_actor_metric(
    actor_id: ActorID,
    metric_type: PerformanceMetricType,
    value: PerformanceMetricValue,
) {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.record_actor_metric(actor_id, metric_type, value);
}

/// Record a message processing time for an actor
pub fn record_message_processing_time(actor_id: ActorID, duration: Duration) {
    record_actor_metric(
        actor_id,
        PerformanceMetricType::MessageProcessingTime,
        PerformanceMetricValue::Timer(duration),
    );
}

/// Record a message throughput for an actor
pub fn record_message_throughput(actor_id: ActorID, messages_per_second: f64) {
    record_actor_metric(
        actor_id,
        PerformanceMetricType::MessageThroughput,
        PerformanceMetricValue::Rate(messages_per_second),
    );
}

/// Record memory usage for an actor
pub fn record_memory_usage(actor_id: ActorID, bytes: usize) {
    record_actor_metric(
        actor_id,
        PerformanceMetricType::MemoryUsage,
        PerformanceMetricValue::Gauge(bytes as i64),
    );
}

/// Record CPU usage for an actor
pub fn record_cpu_usage(actor_id: ActorID, percentage: f64) {
    record_actor_metric(
        actor_id,
        PerformanceMetricType::CpuUsage,
        PerformanceMetricValue::Rate(percentage),
    );
}

/// Record queue length for an actor
pub fn record_queue_length(actor_id: ActorID, length: usize) {
    record_actor_metric(
        actor_id,
        PerformanceMetricType::QueueLength,
        PerformanceMetricValue::Gauge(length as i64),
    );
}

/// Record error rate for an actor
pub fn record_error_rate(actor_id: ActorID, errors_per_second: f64) {
    record_actor_metric(
        actor_id,
        PerformanceMetricType::ErrorRate,
        PerformanceMetricValue::Rate(errors_per_second),
    );
}

/// Record a snapshot for an actor
pub fn record_actor_snapshot(snapshot: ActorPerformanceSnapshot) {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.record_actor_snapshot(snapshot);
}

/// Record a global metric
pub fn record_global_metric(name: impl Into<String>, value: PerformanceMetricValue) {
    let mut monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.record_global_metric(name, value);
}

/// Export performance data as JSON
pub fn export_performance_json() -> Result<String, serde_json::Error> {
    let monitor = PERFORMANCE_MONITOR.lock().unwrap();
    monitor.export_json()
}