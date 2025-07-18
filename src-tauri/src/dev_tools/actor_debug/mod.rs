//! Actor System Debugging Tools
//!
//! This module provides specialized debugging tools for the Kameo actor system.
//! It integrates visualization, logging, and performance monitoring into a cohesive
//! debugging experience for developers.

use std::sync::{Arc, Mutex};
use kameo::prelude::*;
use serde::{Serialize, Deserialize};

pub mod visualization;
pub mod logging;
pub mod performance;
pub mod inspector;

/// Re-export commonly used types and functions
pub use visualization::{
    ActorNode, MessageFlow, ActorVisualization,
    register_actor as register_actor_for_visualization,
    record_message_sent, record_message_received, record_message_processed,
    export_visualization_json,
};

pub use logging::{
    ActorLogContext, ActorOperationLogger,
    log_with_actor_context, trace_with_actor_context, debug_with_actor_context,
    info_with_actor_context, warn_with_actor_context, error_with_actor_context,
};

pub use performance::{
    PerformanceMetricType, PerformanceMetricValue, PerformanceMetric,
    ActorPerformanceSnapshot, ActorPerformanceHistory, PerformanceMonitor,
    register_actor as register_actor_for_performance,
    record_message_processing_time, record_message_throughput,
    record_memory_usage, record_cpu_usage, record_queue_length, record_error_rate,
    export_performance_json,
};

/// Actor system debugging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorDebugConfig {
    /// Whether visualization is enabled
    pub enable_visualization: bool,
    /// Whether performance monitoring is enabled
    pub enable_performance: bool,
    /// Whether enhanced logging is enabled
    pub enable_logging: bool,
    /// Whether inspector is enabled
    pub enable_inspector: bool,
    /// Maximum history length for metrics
    pub max_history_len: usize,
}

impl Default for ActorDebugConfig {
    fn default() -> Self {
        Self {
            enable_visualization: true,
            enable_performance: true,
            enable_logging: true,
            enable_inspector: true,
            max_history_len: 100,
        }
    }
}

/// Global actor debug configuration
lazy_static::lazy_static! {
    static ref ACTOR_DEBUG_CONFIG: Arc<Mutex<ActorDebugConfig>> = Arc::new(Mutex::new(ActorDebugConfig::default()));
}

/// Get the global actor debug configuration
pub fn get_debug_config() -> Arc<Mutex<ActorDebugConfig>> {
    ACTOR_DEBUG_CONFIG.clone()
}

/// Configure the actor debugging system
pub fn configure(config: ActorDebugConfig) {
    let mut current_config = ACTOR_DEBUG_CONFIG.lock().unwrap();
    *current_config = config;
}

/// Register an actor with all debugging systems
pub fn register_actor(actor_id: ActorID, actor_type: impl Into<String> + Clone, parent_id: Option<ActorID>) {
    let config = ACTOR_DEBUG_CONFIG.lock().unwrap();
    
    if config.enable_visualization {
        visualization::register_actor(actor_id.clone(), actor_type.clone(), parent_id);
    }
    
    if config.enable_performance {
        performance::register_actor(actor_id.clone(), actor_type.clone());
    }
}

/// Export all debugging data as JSON
pub fn export_debug_data() -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    struct DebugData {
        visualization: Option<String>,
        performance: Option<String>,
    }
    
    let config = ACTOR_DEBUG_CONFIG.lock().unwrap();
    
    let visualization = if config.enable_visualization {
        match visualization::export_visualization_json() {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    } else {
        None
    };
    
    let performance = if config.enable_performance {
        match performance::export_performance_json() {
            Ok(json) => Some(json),
            Err(_) => None,
        }
    } else {
        None
    };
    
    let debug_data = DebugData {
        visualization,
        performance,
    };
    
    serde_json::to_string_pretty(&debug_data)
}

/// Initialize the actor debugging system
pub fn init() {
    // This function would be called at application startup
    // to initialize the debugging system
    tracing::info!("Initializing actor system debugging tools");
}