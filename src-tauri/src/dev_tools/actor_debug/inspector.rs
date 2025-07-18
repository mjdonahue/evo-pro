//! Actor System Inspector
//!
//! This module provides interactive inspection capabilities for the Kameo actor system.
//! It allows developers to inspect actor state, message flows, and performance metrics
//! in real-time.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kameo::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::actors::metrics::{MetricType, MetricValue};
use crate::logging;
use super::{visualization, performance, logging as actor_logging};

/// Actor inspection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InspectionRequest {
    /// Get actor hierarchy
    GetActorHierarchy,
    /// Get actor details
    GetActorDetails { actor_id: ActorID },
    /// Get message flows for an actor
    GetActorMessageFlows { actor_id: ActorID },
    /// Get performance metrics for an actor
    GetActorPerformanceMetrics { actor_id: ActorID, metric_type: Option<performance::PerformanceMetricType> },
    /// Get all active actors
    GetActiveActors,
    /// Get system-wide message flow
    GetSystemMessageFlow,
    /// Get system-wide performance metrics
    GetSystemPerformanceMetrics,
    /// Set log level for an actor
    SetActorLogLevel { actor_id: ActorID, level: logging::LogLevel },
    /// Enable/disable tracing for an actor
    SetActorTracing { actor_id: ActorID, enabled: bool },
}

/// Actor inspection response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InspectionResponse {
    /// Actor hierarchy
    ActorHierarchy { hierarchy: Vec<(ActorID, Vec<ActorID>)> },
    /// Actor details
    ActorDetails { actor: visualization::ActorNode },
    /// Actor message flows
    ActorMessageFlows { flows: Vec<visualization::MessageFlow> },
    /// Actor performance metrics
    ActorPerformanceMetrics { metrics: HashMap<String, performance::MetricStats> },
    /// Active actors
    ActiveActors { actors: Vec<visualization::ActorNode> },
    /// System message flow
    SystemMessageFlow { flows: Vec<visualization::MessageFlow> },
    /// System performance metrics
    SystemPerformanceMetrics { metrics: HashMap<String, performance::MetricStats> },
    /// Operation success
    Success,
    /// Operation error
    Error { message: String },
}

/// Actor inspector state
#[derive(Debug, Default)]
pub struct ActorInspector {
    /// Actors being traced
    traced_actors: HashMap<ActorID, logging::LogLevel>,
    /// Last inspection time by actor ID
    last_inspection: HashMap<ActorID, Instant>,
}

impl ActorInspector {
    /// Create a new actor inspector
    pub fn new() -> Self {
        Self {
            traced_actors: HashMap::new(),
            last_inspection: HashMap::new(),
        }
    }
    
    /// Process an inspection request
    pub fn process_request(&mut self, request: InspectionRequest) -> InspectionResponse {
        match request {
            InspectionRequest::GetActorHierarchy => self.get_actor_hierarchy(),
            InspectionRequest::GetActorDetails { actor_id } => self.get_actor_details(actor_id),
            InspectionRequest::GetActorMessageFlows { actor_id } => self.get_actor_message_flows(actor_id),
            InspectionRequest::GetActorPerformanceMetrics { actor_id, metric_type } => 
                self.get_actor_performance_metrics(actor_id, metric_type),
            InspectionRequest::GetActiveActors => self.get_active_actors(),
            InspectionRequest::GetSystemMessageFlow => self.get_system_message_flow(),
            InspectionRequest::GetSystemPerformanceMetrics => self.get_system_performance_metrics(),
            InspectionRequest::SetActorLogLevel { actor_id, level } => self.set_actor_log_level(actor_id, level),
            InspectionRequest::SetActorTracing { actor_id, enabled } => self.set_actor_tracing(actor_id, enabled),
        }
    }
    
    /// Get the actor hierarchy
    fn get_actor_hierarchy(&mut self) -> InspectionResponse {
        let visualization = visualization::get_actor_visualization();
        let vis = visualization.lock().unwrap();
        
        let hierarchy = vis.get_actor_hierarchy();
        let result: Vec<(ActorID, Vec<ActorID>)> = hierarchy
            .into_iter()
            .map(|(id, children)| (id.clone(), children.into_iter().cloned().collect()))
            .collect();
        
        InspectionResponse::ActorHierarchy { hierarchy: result }
    }
    
    /// Get details for a specific actor
    fn get_actor_details(&mut self, actor_id: ActorID) -> InspectionResponse {
        let visualization = visualization::get_actor_visualization();
        let vis = visualization.lock().unwrap();
        
        if let Some(actor) = vis.actors.get(&actor_id) {
            // Record this inspection
            self.last_inspection.insert(actor_id.clone(), Instant::now());
            
            InspectionResponse::ActorDetails { actor: actor.clone() }
        } else {
            InspectionResponse::Error { message: format!("Actor not found: {}", actor_id) }
        }
    }
    
    /// Get message flows for a specific actor
    fn get_actor_message_flows(&mut self, actor_id: ActorID) -> InspectionResponse {
        let visualization = visualization::get_actor_visualization();
        let vis = visualization.lock().unwrap();
        
        let flows = vis.get_actor_message_flows(&actor_id);
        let result = flows.into_iter().cloned().collect();
        
        InspectionResponse::ActorMessageFlows { flows: result }
    }
    
    /// Get performance metrics for a specific actor
    fn get_actor_performance_metrics(
        &mut self, 
        actor_id: ActorID, 
        metric_type: Option<performance::PerformanceMetricType>
    ) -> InspectionResponse {
        let monitor = performance::get_performance_monitor();
        let mon = monitor.lock().unwrap();
        
        if let Some(history) = mon.get_actor_history(&actor_id) {
            let mut metrics = HashMap::new();
            
            if let Some(metric_type) = metric_type {
                // Get stats for a specific metric type
                if let Some(stats) = history.calculate_metric_stats(metric_type) {
                    metrics.insert(format!("{:?}", metric_type), stats);
                }
            } else {
                // Get stats for all metric types
                for metric_type in [
                    performance::PerformanceMetricType::MessageProcessingTime,
                    performance::PerformanceMetricType::MessageThroughput,
                    performance::PerformanceMetricType::MemoryUsage,
                    performance::PerformanceMetricType::CpuUsage,
                    performance::PerformanceMetricType::QueueLength,
                    performance::PerformanceMetricType::ErrorRate,
                ].iter() {
                    if let Some(stats) = history.calculate_metric_stats(*metric_type) {
                        metrics.insert(format!("{:?}", metric_type), stats);
                    }
                }
            }
            
            InspectionResponse::ActorPerformanceMetrics { metrics }
        } else {
            InspectionResponse::Error { message: format!("Actor not found: {}", actor_id) }
        }
    }
    
    /// Get all active actors
    fn get_active_actors(&mut self) -> InspectionResponse {
        let visualization = visualization::get_actor_visualization();
        let vis = visualization.lock().unwrap();
        
        let actors = vis.actors
            .values()
            .filter(|actor| actor.stopped_at.is_none())
            .cloned()
            .collect();
        
        InspectionResponse::ActiveActors { actors }
    }
    
    /// Get system-wide message flow
    fn get_system_message_flow(&mut self) -> InspectionResponse {
        let visualization = visualization::get_actor_visualization();
        let vis = visualization.lock().unwrap();
        
        let flows = vis.message_flows
            .values()
            .cloned()
            .collect();
        
        InspectionResponse::SystemMessageFlow { flows }
    }
    
    /// Get system-wide performance metrics
    fn get_system_performance_metrics(&mut self) -> InspectionResponse {
        let monitor = performance::get_performance_monitor();
        let mon = monitor.lock().unwrap();
        
        let mut metrics = HashMap::new();
        
        // Get global metrics
        for name in ["system_cpu", "system_memory", "message_rate", "error_rate"] {
            if let Some(stats) = mon.calculate_global_metric_stats(name) {
                metrics.insert(name.to_string(), stats);
            }
        }
        
        InspectionResponse::SystemPerformanceMetrics { metrics }
    }
    
    /// Set log level for an actor
    fn set_actor_log_level(&mut self, actor_id: ActorID, level: logging::LogLevel) -> InspectionResponse {
        // In a real implementation, this would communicate with the actor
        // to change its log level. For now, we'll just store it locally.
        self.traced_actors.insert(actor_id.clone(), level);
        
        // Log this change
        tracing::info!("Set log level for actor {} to {:?}", actor_id, level);
        
        InspectionResponse::Success
    }
    
    /// Enable/disable tracing for an actor
    fn set_actor_tracing(&mut self, actor_id: ActorID, enabled: bool) -> InspectionResponse {
        if enabled {
            // Enable tracing at INFO level by default if not already set
            if !self.traced_actors.contains_key(&actor_id) {
                self.traced_actors.insert(actor_id.clone(), logging::LogLevel::Info);
            }
        } else {
            // Disable tracing
            self.traced_actors.remove(&actor_id);
        }
        
        // Log this change
        tracing::info!("Set tracing for actor {} to {}", actor_id, enabled);
        
        InspectionResponse::Success
    }
}

/// Global actor inspector instance
lazy_static::lazy_static! {
    static ref ACTOR_INSPECTOR: Arc<Mutex<ActorInspector>> = Arc::new(Mutex::new(ActorInspector::new()));
}

/// Get the global actor inspector instance
pub fn get_actor_inspector() -> Arc<Mutex<ActorInspector>> {
    ACTOR_INSPECTOR.clone()
}

/// Process an inspection request
pub fn process_inspection_request(request: InspectionRequest) -> InspectionResponse {
    let mut inspector = ACTOR_INSPECTOR.lock().unwrap();
    inspector.process_request(request)
}

/// Tauri command to process an inspection request
#[tauri::command]
pub fn inspect_actor_system(request_json: String) -> Result<String, String> {
    // Parse the request JSON
    let request: InspectionRequest = serde_json::from_str(&request_json)
        .map_err(|e| format!("Failed to parse request: {}", e))?;
    
    // Process the request
    let response = process_inspection_request(request);
    
    // Serialize the response to JSON
    serde_json::to_string(&response)
        .map_err(|e| format!("Failed to serialize response: {}", e))
}

/// Initialize the actor inspector
pub fn init() {
    tracing::info!("Initializing actor system inspector");
}