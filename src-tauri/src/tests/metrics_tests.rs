use std::collections::HashMap;
use std::time::Duration;

use kameo::prelude::*;
use tokio::time::sleep;

use crate::actors::metrics::{
    MetricType, MetricValue, MetricsCollectorActor, MetricsDashboardActor,
    MetricsAware, MetricsExt, create_metrics_collector, create_metrics_dashboard,
    MessageTimer, GetMetrics,
};
use crate::error::Result;

// Test actor that implements MetricsAware
#[derive(Actor, Clone)]
struct TestMetricsActor {
    name: String,
    message_count: u64,
    error_count: u64,
    processing_times: HashMap<String, Vec<Duration>>,
}

impl TestMetricsActor {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            message_count: 0,
            error_count: 0,
            processing_times: HashMap::new(),
        }
    }
}

impl MetricsAware for TestMetricsActor {
    fn get_metrics(&self) -> HashMap<MetricType, MetricValue> {
        let mut metrics = HashMap::new();
        
        // Message count
        metrics.insert(
            MetricType::MessageCount,
            MetricValue::Counter(self.message_count),
        );
        
        // Error count
        metrics.insert(
            MetricType::ErrorCount,
            MetricValue::Counter(self.error_count),
        );
        
        // Processing time (average)
        let mut total_processing_time = Duration::from_secs(0);
        let mut count = 0;
        
        for times in self.processing_times.values() {
            for time in times {
                total_processing_time += *time;
                count += 1;
            }
        }
        
        if count > 0 {
            let avg_processing_time = total_processing_time / count as u32;
            metrics.insert(
                MetricType::ProcessingTime,
                MetricValue::Timer(avg_processing_time),
            );
        }
        
        metrics
    }
    
    fn record_processing_time(&mut self, message_type: &str, duration: Duration) {
        self.processing_times
            .entry(message_type.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }
    
    fn record_message_received(&mut self, _message_type: &str) {
        self.message_count += 1;
    }
    
    fn record_error(&mut self, _error_type: &str) {
        self.error_count += 1;
    }
}

// Test message
#[derive(Debug, Clone)]
struct TestMessage {
    data: String,
}

impl Message<TestMessage> for TestMetricsActor {
    type Reply = Result<String>;
    
    async fn handle(
        &mut self,
        msg: TestMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Record message received
        self.record_message_received("TestMessage");
        
        // Create a timer for this message
        let timer = MessageTimer::new("TestMessage");
        
        // Simulate some processing time
        sleep(Duration::from_millis(10)).await;
        
        // End the timer
        timer.end(self);
        
        // Return a response
        Ok(format!("Processed: {}", msg.data))
    }
}

// Test error message
#[derive(Debug, Clone)]
struct TestErrorMessage;

impl Message<TestErrorMessage> for TestMetricsActor {
    type Reply = Result<()>;
    
    async fn handle(
        &mut self,
        _msg: TestErrorMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Record message received
        self.record_message_received("TestErrorMessage");
        
        // Record an error
        self.record_error("TestError");
        
        // Return an error
        Err(crate::error::AppError::GenericError("Test error".to_string()))
    }
}

#[tokio::test]
async fn test_metrics_collection() -> Result<()> {
    // Create a metrics collector
    let metrics_collector = create_metrics_collector(
        Duration::from_secs(1),
        Duration::from_secs(60),
    );
    
    // Create a test actor
    let test_actor = TestMetricsActor::new("test-metrics-actor");
    let actor_ref = TestMetricsActor::spawn(test_actor);
    
    // Register the actor with the metrics collector
    actor_ref
        .with_metrics(&metrics_collector, "TestMetricsActor")
        .await?;
    
    // Send some test messages
    for i in 0..5 {
        let _ = actor_ref
            .ask(&TestMessage { data: format!("test-{}", i) })
            .await;
    }
    
    // Send an error message
    let _ = actor_ref.ask(&TestErrorMessage).await;
    
    // Wait for metrics to be collected
    sleep(Duration::from_secs(2)).await;
    
    // Get metrics for the actor
    let actor_metrics = actor_ref.get_metrics(&metrics_collector).await?;
    
    // Verify metrics
    assert!(actor_metrics.is_some(), "Actor metrics should be available");
    
    if let Some(metrics) = actor_metrics {
        // Check message count
        if let Some(metric) = metrics.get_metric(MetricType::MessageCount) {
            if let MetricValue::Counter(count) = &metric.value {
                assert_eq!(*count, 6, "Message count should be 6");
            } else {
                panic!("MessageCount should be a Counter");
            }
        } else {
            panic!("MessageCount metric should exist");
        }
        
        // Check error count
        if let Some(metric) = metrics.get_metric(MetricType::ErrorCount) {
            if let MetricValue::Counter(count) = &metric.value {
                assert_eq!(*count, 1, "Error count should be 1");
            } else {
                panic!("ErrorCount should be a Counter");
            }
        } else {
            panic!("ErrorCount metric should exist");
        }
        
        // Check processing time
        assert!(metrics.get_metric(MetricType::ProcessingTime).is_some(), "ProcessingTime metric should exist");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_metrics_dashboard() -> Result<()> {
    // Create a metrics collector
    let metrics_collector = create_metrics_collector(
        Duration::from_secs(1),
        Duration::from_secs(60),
    );
    
    // Create a metrics dashboard
    let dashboard = create_metrics_dashboard(
        metrics_collector.clone(),
        None,
    );
    
    // Create multiple test actors
    let mut actor_refs = Vec::new();
    
    for i in 0..3 {
        let test_actor = TestMetricsActor::new(format!("test-actor-{}", i));
        let actor_ref = TestMetricsActor::spawn(test_actor);
        
        // Register the actor with the metrics collector
        actor_ref
            .with_metrics(&metrics_collector, "TestMetricsActor")
            .await?;
        
        // Send some test messages
        for j in 0..3 {
            let _ = actor_ref
                .ask(&TestMessage { data: format!("test-{}-{}", i, j) })
                .await;
        }
        
        // Send an error message if this is the first actor
        if i == 0 {
            let _ = actor_ref.ask(&TestErrorMessage).await;
        }
        
        actor_refs.push(actor_ref);
    }
    
    // Wait for metrics to be collected and dashboard to be updated
    sleep(Duration::from_secs(2)).await;
    
    // Refresh the dashboard
    dashboard.tell(&crate::actors::metrics::RefreshDashboard).await?;
    
    // Get dashboard data
    let dashboard_data = dashboard
        .ask(&crate::actors::metrics::GetDashboardData)
        .await;
    
    // Verify dashboard data
    assert!(dashboard_data.last_update.is_some(), "Dashboard should have been updated");
    assert_eq!(dashboard_data.actor_metrics.len(), 3, "Dashboard should have metrics for 3 actors");
    
    // Check system metrics
    assert!(dashboard_data.system_metrics.contains_key("total_actors"), "System metrics should include total_actors");
    assert!(dashboard_data.system_metrics.contains_key("total_messages"), "System metrics should include total_messages");
    assert!(dashboard_data.system_metrics.contains_key("total_errors"), "System metrics should include total_errors");
    
    // Verify total message count
    if let Some(MetricValue::Counter(count)) = dashboard_data.system_metrics.get("total_messages") {
        assert_eq!(*count, 10, "Total message count should be 10 (3 actors * 3 messages + 1 error message)");
    } else {
        panic!("total_messages should be a Counter");
    }
    
    // Verify total error count
    if let Some(MetricValue::Counter(count)) = dashboard_data.system_metrics.get("total_errors") {
        assert_eq!(*count, 1, "Total error count should be 1");
    } else {
        panic!("total_errors should be a Counter");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_get_metrics_message() -> Result<()> {
    // Create a test actor
    let test_actor = TestMetricsActor::new("test-metrics-actor");
    let actor_ref = TestMetricsActor::spawn(test_actor);
    
    // Send some test messages
    for i in 0..3 {
        let _ = actor_ref
            .ask(&TestMessage { data: format!("test-{}", i) })
            .await;
    }
    
    // Send an error message
    let _ = actor_ref.ask(&TestErrorMessage).await;
    
    // Get metrics directly from the actor
    let metrics = actor_ref.ask(&GetMetrics).await;
    
    // Verify metrics
    assert!(metrics.contains_key(&MetricType::MessageCount), "Metrics should include MessageCount");
    assert!(metrics.contains_key(&MetricType::ErrorCount), "Metrics should include ErrorCount");
    assert!(metrics.contains_key(&MetricType::ProcessingTime), "Metrics should include ProcessingTime");
    
    // Check message count
    if let Some(MetricValue::Counter(count)) = metrics.get(&MetricType::MessageCount) {
        assert_eq!(*count, 4, "Message count should be 4");
    } else {
        panic!("MessageCount should be a Counter");
    }
    
    // Check error count
    if let Some(MetricValue::Counter(count)) = metrics.get(&MetricType::ErrorCount) {
        assert_eq!(*count, 1, "Error count should be 1");
    } else {
        panic!("ErrorCount should be a Counter");
    }
    
    Ok(())
}