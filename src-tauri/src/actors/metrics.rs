use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use kameo::prelude::*;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::logging;

/// Actor metrics types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// Message count
    MessageCount,
    /// Message processing time
    ProcessingTime,
    /// Error count
    ErrorCount,
    /// Memory usage
    MemoryUsage,
    /// CPU usage
    CpuUsage,
    /// Custom metric
    Custom(u32),
}

/// Actor metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value (always increases)
    Counter(u64),
    /// Gauge value (can go up or down)
    Gauge(i64),
    /// Histogram value (distribution of values)
    Histogram(Vec<f64>),
    /// Summary value (percentiles)
    Summary {
        count: u64,
        sum: f64,
        min: f64,
        max: f64,
        p50: f64,
        p90: f64,
        p99: f64,
    },
    /// Timer value (duration)
    Timer(Duration),
}

/// Actor metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric type
    pub metric_type: MetricType,
    /// Metric value
    pub value: MetricValue,
    /// Timestamp when the metric was recorded
    pub timestamp: std::time::SystemTime,
    /// Labels associated with the metric
    pub labels: HashMap<String, String>,
}

/// Actor metrics data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActorMetrics {
    /// Actor ID
    pub actor_id: ActorID,
    /// Actor type name
    pub actor_type: String,
    /// Metrics by type
    pub metrics: HashMap<MetricType, Metric>,
    /// Historical metrics (time series)
    pub history: HashMap<MetricType, VecDeque<Metric>>,
    /// Maximum history length
    pub max_history_len: usize,
}

impl ActorMetrics {
    /// Create new actor metrics
    pub fn new(actor_id: ActorID, actor_type: impl Into<String>) -> Self {
        Self {
            actor_id,
            actor_type: actor_type.into(),
            metrics: HashMap::new(),
            history: HashMap::new(),
            max_history_len: 100, // Default to 100 historical values
        }
    }

    /// Set the maximum history length
    pub fn with_max_history_len(mut self, max_len: usize) -> Self {
        self.max_history_len = max_len;
        self
    }

    /// Record a metric
    pub fn record_metric(&mut self, metric_type: MetricType, value: MetricValue, labels: Option<HashMap<String, String>>) {
        let timestamp = std::time::SystemTime::now();
        let labels = labels.unwrap_or_default();
        
        let metric = Metric {
            metric_type,
            value,
            timestamp,
            labels,
        };
        
        // Update current metric
        self.metrics.insert(metric_type, metric.clone());
        
        // Add to history
        let history = self.history.entry(metric_type).or_insert_with(VecDeque::new);
        history.push_back(metric);
        
        // Trim history if needed
        while history.len() > self.max_history_len {
            history.pop_front();
        }
    }

    /// Get the current value of a metric
    pub fn get_metric(&self, metric_type: MetricType) -> Option<&Metric> {
        self.metrics.get(&metric_type)
    }

    /// Get the history of a metric
    pub fn get_metric_history(&self, metric_type: MetricType) -> Option<&VecDeque<Metric>> {
        self.history.get(&metric_type)
    }

    /// Increment a counter metric
    pub fn increment_counter(&mut self, metric_type: MetricType, amount: u64, labels: Option<HashMap<String, String>>) {
        let current = match self.metrics.get(&metric_type) {
            Some(Metric { value: MetricValue::Counter(count), .. }) => *count,
            _ => 0,
        };
        
        self.record_metric(
            metric_type,
            MetricValue::Counter(current + amount),
            labels,
        );
    }

    /// Set a gauge metric
    pub fn set_gauge(&mut self, metric_type: MetricType, value: i64, labels: Option<HashMap<String, String>>) {
        self.record_metric(
            metric_type,
            MetricValue::Gauge(value),
            labels,
        );
    }

    /// Record a timer metric
    pub fn record_timer(&mut self, metric_type: MetricType, duration: Duration, labels: Option<HashMap<String, String>>) {
        self.record_metric(
            metric_type,
            MetricValue::Timer(duration),
            labels,
        );
    }

    /// Add a value to a histogram metric
    pub fn add_to_histogram(&mut self, metric_type: MetricType, value: f64, labels: Option<HashMap<String, String>>) {
        let values = match self.metrics.get(&metric_type) {
            Some(Metric { value: MetricValue::Histogram(values), .. }) => {
                let mut new_values = values.clone();
                new_values.push(value);
                new_values
            },
            _ => vec![value],
        };
        
        self.record_metric(
            metric_type,
            MetricValue::Histogram(values),
            labels,
        );
    }

    /// Update a summary metric
    pub fn update_summary(&mut self, metric_type: MetricType, value: f64, labels: Option<HashMap<String, String>>) {
        let (count, sum, min, max, values) = match self.metrics.get(&metric_type) {
            Some(Metric { value: MetricValue::Summary { count, sum, min, max, .. }, .. }) => {
                let mut values = match self.history.get(&metric_type) {
                    Some(history) => history.iter()
                        .filter_map(|m| match &m.value {
                            MetricValue::Summary { .. } => Some(value),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    None => Vec::new(),
                };
                values.push(value);
                (*count + 1, *sum + value, (*min).min(value), (*max).max(value), values)
            },
            _ => (1, value, value, value, vec![value]),
        };
        
        // Calculate percentiles
        let mut sorted_values = values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let p50 = percentile(&sorted_values, 0.5).unwrap_or(0.0);
        let p90 = percentile(&sorted_values, 0.9).unwrap_or(0.0);
        let p99 = percentile(&sorted_values, 0.99).unwrap_or(0.0);
        
        self.record_metric(
            metric_type,
            MetricValue::Summary {
                count,
                sum,
                min,
                max,
                p50,
                p90,
                p99,
            },
            labels,
        );
    }
}

/// Calculate a percentile from a sorted list of values
fn percentile(sorted_values: &[f64], p: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    
    let index = (sorted_values.len() as f64 * p).floor() as usize;
    Some(sorted_values[index.min(sorted_values.len() - 1)])
}

/// Actor that collects and manages metrics
#[derive(Actor)]
pub struct MetricsCollectorActor {
    /// Metrics by actor ID
    metrics: HashMap<ActorID, ActorMetrics>,
    /// Subscribers to metric events
    subscribers: Vec<mpsc::Sender<MetricEvent>>,
    /// Collection interval
    collection_interval: Duration,
    /// Retention period for metrics
    retention_period: Duration,
}

/// Metric events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricEvent {
    /// New metric recorded
    MetricRecorded {
        actor_id: ActorID,
        metric_type: MetricType,
        metric: Metric,
    },
    /// Actor metrics snapshot
    MetricsSnapshot {
        actor_id: ActorID,
        metrics: ActorMetrics,
    },
    /// System-wide metrics snapshot
    SystemMetricsSnapshot {
        metrics: HashMap<ActorID, ActorMetrics>,
    },
}

impl MetricsCollectorActor {
    /// Create a new metrics collector actor
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            subscribers: Vec::new(),
            collection_interval: Duration::from_secs(10), // Default to 10 seconds
            retention_period: Duration::from_secs(3600),  // Default to 1 hour
        }
    }

    /// Set the collection interval
    pub fn with_collection_interval(mut self, interval: Duration) -> Self {
        self.collection_interval = interval;
        self
    }

    /// Set the retention period
    pub fn with_retention_period(mut self, period: Duration) -> Self {
        self.retention_period = period;
        self
    }

    /// Start the metrics collection loop
    async fn start_collection_loop(&self, ctx: &mut Context<Self, ()>) {
        let actor_ref = ctx.actor_ref();
        
        // Spawn a task to periodically collect metrics
        tokio::spawn(async move {
            let mut collection_interval = interval(Duration::from_secs(1));
            
            loop {
                collection_interval.tick().await;
                
                // Send a message to collect metrics
                if let Err(e) = actor_ref.tell(&CollectMetrics).await {
                    error!("Failed to send metrics collection message: {}", e);
                    break;
                }
            }
        });
    }

    /// Subscribe to metric events
    pub async fn subscribe(&mut self) -> mpsc::Receiver<MetricEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.push(tx);
        rx
    }

    /// Publish a metric event to all subscribers
    async fn publish_event(&mut self, event: MetricEvent) {
        // Remove closed channels
        self.subscribers.retain(|tx| !tx.is_closed());
        
        // Send the event to all subscribers
        for tx in &self.subscribers {
            if let Err(e) = tx.send(event.clone()).await {
                warn!("Failed to send metric event: {}", e);
            }
        }
    }

    /// Register an actor for metrics collection
    pub async fn register_actor<A: Actor + 'static>(
        &mut self,
        actor_ref: ActorRef<A>,
        actor_type: impl Into<String>,
    ) -> Result<()> {
        let actor_id = actor_ref.id();
        
        // Create metrics for the actor
        let metrics = ActorMetrics::new(actor_id, actor_type);
        
        // Store the metrics
        self.metrics.insert(actor_id, metrics);
        
        info!("Now collecting metrics for actor {}", actor_id);
        
        Ok(())
    }

    /// Record a metric for an actor
    pub fn record_metric(
        &mut self,
        actor_id: ActorID,
        metric_type: MetricType,
        value: MetricValue,
        labels: Option<HashMap<String, String>>,
    ) -> Result<()> {
        if let Some(metrics) = self.metrics.get_mut(&actor_id) {
            metrics.record_metric(metric_type, value, labels);
            
            // Publish metric recorded event
            if let Some(metric) = metrics.get_metric(metric_type) {
                tokio::spawn({
                    let mut this = self.clone();
                    let actor_id = actor_id;
                    let metric_type = metric_type;
                    let metric = metric.clone();
                    async move {
                        this.publish_event(MetricEvent::MetricRecorded {
                            actor_id,
                            metric_type,
                            metric,
                        }).await;
                    }
                });
            }
            
            Ok(())
        } else {
            Err(AppError::NotFoundError(format!(
                "Actor with ID {} not registered for metrics collection",
                actor_id
            )))
        }
    }

    /// Get metrics for an actor
    pub fn get_actor_metrics(&self, actor_id: ActorID) -> Option<&ActorMetrics> {
        self.metrics.get(&actor_id)
    }

    /// Get all metrics
    pub fn get_all_metrics(&self) -> &HashMap<ActorID, ActorMetrics> {
        &self.metrics
    }

    /// Collect metrics from all registered actors
    async fn collect_metrics(&mut self) {
        // In a real implementation, we would query each actor for its metrics
        // For now, we'll just publish a system-wide snapshot
        
        // Publish system-wide metrics snapshot
        self.publish_event(MetricEvent::SystemMetricsSnapshot {
            metrics: self.metrics.clone(),
        }).await;
        
        // Clean up old metrics based on retention period
        self.clean_old_metrics();
    }

    /// Clean up old metrics based on retention period
    fn clean_old_metrics(&mut self) {
        let now = std::time::SystemTime::now();
        let retention_period = self.retention_period;
        
        for (_actor_id, metrics) in &mut self.metrics {
            for (_metric_type, history) in &mut metrics.history {
                history.retain(|metric| {
                    match now.duration_since(metric.timestamp) {
                        Ok(age) => age < retention_period,
                        Err(_) => true, // Keep metrics with invalid timestamps
                    }
                });
            }
        }
    }
}

/// Message to collect metrics
#[derive(Debug, Clone)]
pub struct CollectMetrics;

impl Message<CollectMetrics> for MetricsCollectorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: CollectMetrics,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.collect_metrics().await;
    }
}

/// Message to record a metric
#[derive(Debug, Clone)]
pub struct RecordMetric {
    pub actor_id: ActorID,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: Option<HashMap<String, String>>,
}

impl Message<RecordMetric> for MetricsCollectorActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: RecordMetric,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.record_metric(msg.actor_id, msg.metric_type, msg.value, msg.labels)
    }
}

/// Message to register an actor for metrics collection
#[derive(Debug, Clone)]
pub struct RegisterActorMetrics<A: Actor + 'static> {
    pub actor_ref: ActorRef<A>,
    pub actor_type: String,
}

impl<A: Actor + 'static> Message<RegisterActorMetrics<A>> for MetricsCollectorActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: RegisterActorMetrics<A>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.register_actor(msg.actor_ref, msg.actor_type).await
    }
}

/// Message to get metrics for an actor
#[derive(Debug, Clone)]
pub struct GetActorMetrics {
    pub actor_id: ActorID,
}

impl Message<GetActorMetrics> for MetricsCollectorActor {
    type Reply = Option<ActorMetrics>;

    async fn handle(
        &mut self,
        msg: GetActorMetrics,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.get_actor_metrics(msg.actor_id).cloned()
    }
}

/// Message to get all metrics
#[derive(Debug, Clone)]
pub struct GetAllMetrics;

impl Message<GetAllMetrics> for MetricsCollectorActor {
    type Reply = HashMap<ActorID, ActorMetrics>;

    async fn handle(
        &mut self,
        _msg: GetAllMetrics,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.get_all_metrics().clone()
    }
}

/// Message to subscribe to metric events
#[derive(Debug, Clone)]
pub struct SubscribeToMetricEvents;

impl Message<SubscribeToMetricEvents> for MetricsCollectorActor {
    type Reply = mpsc::Receiver<MetricEvent>;

    async fn handle(
        &mut self,
        _msg: SubscribeToMetricEvents,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.subscribe().await
    }
}

impl Actor for MetricsCollectorActor {
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Start the metrics collection loop
        self.start_collection_loop(ctx);
    }
}

/// Extension trait for ActorRef to add metrics capabilities
pub trait MetricsExt<A: Actor + 'static> {
    /// Register this actor for metrics collection
    async fn with_metrics(
        self,
        metrics_collector: &ActorRef<MetricsCollectorActor>,
        actor_type: impl Into<String>,
    ) -> Result<Self>
    where
        Self: Sized;
    
    /// Record a metric for this actor
    async fn record_metric(
        &self,
        metrics_collector: &ActorRef<MetricsCollectorActor>,
        metric_type: MetricType,
        value: MetricValue,
        labels: Option<HashMap<String, String>>,
    ) -> Result<()>;
    
    /// Get metrics for this actor
    async fn get_metrics(
        &self,
        metrics_collector: &ActorRef<MetricsCollectorActor>,
    ) -> Result<Option<ActorMetrics>>;
}

impl<A: Actor + 'static> MetricsExt<A> for ActorRef<A> {
    async fn with_metrics(
        self,
        metrics_collector: &ActorRef<MetricsCollectorActor>,
        actor_type: impl Into<String>,
    ) -> Result<Self> {
        // Register the actor for metrics collection
        metrics_collector
            .ask(&RegisterActorMetrics {
                actor_ref: self.clone(),
                actor_type: actor_type.into(),
            })
            .await?;
        
        Ok(self)
    }
    
    async fn record_metric(
        &self,
        metrics_collector: &ActorRef<MetricsCollectorActor>,
        metric_type: MetricType,
        value: MetricValue,
        labels: Option<HashMap<String, String>>,
    ) -> Result<()> {
        metrics_collector
            .ask(&RecordMetric {
                actor_id: self.id(),
                metric_type,
                value,
                labels,
            })
            .await
    }
    
    async fn get_metrics(
        &self,
        metrics_collector: &ActorRef<MetricsCollectorActor>,
    ) -> Result<Option<ActorMetrics>> {
        Ok(metrics_collector
            .ask(&GetActorMetrics {
                actor_id: self.id(),
            })
            .await)
    }
}

/// Create a metrics collector actor
pub fn create_metrics_collector(
    collection_interval: Duration,
    retention_period: Duration,
) -> ActorRef<MetricsCollectorActor> {
    MetricsCollectorActor::spawn(
        MetricsCollectorActor::new()
            .with_collection_interval(collection_interval)
            .with_retention_period(retention_period)
    )
}

/// Trait for actors that support metrics
pub trait MetricsAware: Actor {
    /// Get metrics for this actor
    fn get_metrics(&self) -> HashMap<MetricType, MetricValue> {
        HashMap::new()
    }
    
    /// Record a message processing time
    fn record_processing_time(&mut self, message_type: &str, duration: Duration) {
        // Default implementation does nothing
    }
    
    /// Record a message received
    fn record_message_received(&mut self, message_type: &str) {
        // Default implementation does nothing
    }
    
    /// Record an error
    fn record_error(&mut self, error_type: &str) {
        // Default implementation does nothing
    }
}

/// Message to get metrics from an actor
#[derive(Debug, Clone)]
pub struct GetMetrics;

/// Implement the GetMetrics message for all MetricsAware actors
impl<A: MetricsAware + 'static> Message<GetMetrics> for A {
    type Reply = HashMap<MetricType, MetricValue>;

    async fn handle(
        &mut self,
        _msg: GetMetrics,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.get_metrics()
    }
}

/// Timer for measuring message processing time
pub struct MessageTimer {
    /// Start time
    start: Instant,
    /// Message type
    message_type: String,
}

impl MessageTimer {
    /// Create a new message timer
    pub fn new(message_type: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            message_type: message_type.into(),
        }
    }
    
    /// End the timer and record the duration
    pub fn end<A: MetricsAware + 'static>(self, actor: &mut A) {
        let duration = self.start.elapsed();
        actor.record_processing_time(&self.message_type, duration);
    }
}

/// Metrics dashboard for visualizing actor metrics
#[derive(Actor)]
pub struct MetricsDashboardActor {
    /// Metrics collector
    metrics_collector: ActorRef<MetricsCollectorActor>,
    /// Dashboard configuration
    config: MetricsDashboardConfig,
    /// Dashboard data
    data: MetricsDashboardData,
}

/// Metrics dashboard configuration
#[derive(Debug, Clone)]
pub struct MetricsDashboardConfig {
    /// Refresh interval
    pub refresh_interval: Duration,
    /// Metrics to display
    pub displayed_metrics: Vec<MetricType>,
    /// Actor IDs to display
    pub displayed_actors: Option<Vec<ActorID>>,
}

impl Default for MetricsDashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_secs(5),
            displayed_metrics: vec![
                MetricType::MessageCount,
                MetricType::ProcessingTime,
                MetricType::ErrorCount,
            ],
            displayed_actors: None,
        }
    }
}

/// Metrics dashboard data
#[derive(Debug, Clone, Default)]
pub struct MetricsDashboardData {
    /// Last update time
    pub last_update: Option<std::time::SystemTime>,
    /// Actor metrics
    pub actor_metrics: HashMap<ActorID, ActorMetrics>,
    /// System-wide metrics
    pub system_metrics: HashMap<String, MetricValue>,
}

impl MetricsDashboardActor {
    /// Create a new metrics dashboard actor
    pub fn new(metrics_collector: ActorRef<MetricsCollectorActor>) -> Self {
        Self {
            metrics_collector,
            config: MetricsDashboardConfig::default(),
            data: MetricsDashboardData::default(),
        }
    }
    
    /// Set the dashboard configuration
    pub fn with_config(mut self, config: MetricsDashboardConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Start the dashboard refresh loop
    async fn start_refresh_loop(&self, ctx: &mut Context<Self, ()>) {
        let actor_ref = ctx.actor_ref();
        let refresh_interval = self.config.refresh_interval;
        
        // Spawn a task to periodically refresh the dashboard
        tokio::spawn(async move {
            let mut refresh_interval = interval(refresh_interval);
            
            loop {
                refresh_interval.tick().await;
                
                // Send a message to refresh the dashboard
                if let Err(e) = actor_ref.tell(&RefreshDashboard).await {
                    error!("Failed to send dashboard refresh message: {}", e);
                    break;
                }
            }
        });
    }
    
    /// Refresh the dashboard data
    async fn refresh_dashboard(&mut self) -> Result<()> {
        // Get all metrics
        let all_metrics = self.metrics_collector.ask(&GetAllMetrics).await;
        
        // Filter metrics based on configuration
        let filtered_metrics = if let Some(ref actor_ids) = self.config.displayed_actors {
            all_metrics
                .into_iter()
                .filter(|(actor_id, _)| actor_ids.contains(actor_id))
                .collect()
        } else {
            all_metrics
        };
        
        // Update dashboard data
        self.data.last_update = Some(std::time::SystemTime::now());
        self.data.actor_metrics = filtered_metrics;
        
        // Calculate system-wide metrics
        self.calculate_system_metrics();
        
        Ok(())
    }
    
    /// Calculate system-wide metrics
    fn calculate_system_metrics(&mut self) {
        let mut system_metrics = HashMap::new();
        
        // Total number of actors
        system_metrics.insert(
            "total_actors".to_string(),
            MetricValue::Gauge(self.data.actor_metrics.len() as i64),
        );
        
        // Total message count
        let total_messages: u64 = self.data.actor_metrics.values()
            .filter_map(|metrics| {
                metrics.get_metric(MetricType::MessageCount)
                    .and_then(|metric| match &metric.value {
                        MetricValue::Counter(count) => Some(*count),
                        _ => None,
                    })
            })
            .sum();
        
        system_metrics.insert(
            "total_messages".to_string(),
            MetricValue::Counter(total_messages),
        );
        
        // Total error count
        let total_errors: u64 = self.data.actor_metrics.values()
            .filter_map(|metrics| {
                metrics.get_metric(MetricType::ErrorCount)
                    .and_then(|metric| match &metric.value {
                        MetricValue::Counter(count) => Some(*count),
                        _ => None,
                    })
            })
            .sum();
        
        system_metrics.insert(
            "total_errors".to_string(),
            MetricValue::Counter(total_errors),
        );
        
        // Average processing time
        let processing_times: Vec<Duration> = self.data.actor_metrics.values()
            .filter_map(|metrics| {
                metrics.get_metric(MetricType::ProcessingTime)
                    .and_then(|metric| match &metric.value {
                        MetricValue::Timer(duration) => Some(*duration),
                        _ => None,
                    })
            })
            .collect();
        
        if !processing_times.is_empty() {
            let total_nanos: u128 = processing_times.iter()
                .map(|d| d.as_nanos())
                .sum();
            
            let avg_nanos = total_nanos / processing_times.len() as u128;
            let avg_duration = Duration::from_nanos(avg_nanos as u64);
            
            system_metrics.insert(
                "avg_processing_time".to_string(),
                MetricValue::Timer(avg_duration),
            );
        }
        
        self.data.system_metrics = system_metrics;
    }
    
    /// Get the dashboard data
    pub fn get_dashboard_data(&self) -> &MetricsDashboardData {
        &self.data
    }
}

/// Message to refresh the dashboard
#[derive(Debug, Clone)]
pub struct RefreshDashboard;

impl Message<RefreshDashboard> for MetricsDashboardActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        _msg: RefreshDashboard,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.refresh_dashboard().await
    }
}

/// Message to get dashboard data
#[derive(Debug, Clone)]
pub struct GetDashboardData;

impl Message<GetDashboardData> for MetricsDashboardActor {
    type Reply = MetricsDashboardData;

    async fn handle(
        &mut self,
        _msg: GetDashboardData,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.get_dashboard_data().clone()
    }
}

impl Actor for MetricsDashboardActor {
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Start the dashboard refresh loop
        self.start_refresh_loop(ctx);
    }
}

/// Create a metrics dashboard actor
pub fn create_metrics_dashboard(
    metrics_collector: ActorRef<MetricsCollectorActor>,
    config: Option<MetricsDashboardConfig>,
) -> ActorRef<MetricsDashboardActor> {
    let dashboard = MetricsDashboardActor::new(metrics_collector);
    
    if let Some(config) = config {
        MetricsDashboardActor::spawn(dashboard.with_config(config))
    } else {
        MetricsDashboardActor::spawn(dashboard)
    }
}