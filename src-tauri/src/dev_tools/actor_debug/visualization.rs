use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kameo::prelude::*;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::actors::metrics::{MetricType, MetricValue};
use crate::logging;

/// Represents a message sent between actors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFlow {
    /// Unique ID for this message flow
    pub id: Uuid,
    /// Sender actor ID
    pub sender_id: ActorID,
    /// Receiver actor ID
    pub receiver_id: ActorID,
    /// Message type name
    pub message_type: String,
    /// Time when the message was sent
    pub sent_at: Instant,
    /// Time when the message was received (if known)
    pub received_at: Option<Instant>,
    /// Time when the message was processed (if known)
    pub processed_at: Option<Instant>,
    /// Processing duration (if known)
    pub processing_duration: Option<Duration>,
    /// Whether the message was successful
    pub success: Option<bool>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Correlation ID for tracing related messages
    pub correlation_id: Option<String>,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl MessageFlow {
    /// Create a new message flow
    pub fn new(
        sender_id: ActorID,
        receiver_id: ActorID,
        message_type: impl Into<String>,
    ) -> Self {
        let correlation_id = logging::correlation::get_correlation_id();
        
        Self {
            id: Uuid::new_v4(),
            sender_id,
            receiver_id,
            message_type: message_type.into(),
            sent_at: Instant::now(),
            received_at: None,
            processed_at: None,
            processing_duration: None,
            success: None,
            error: None,
            correlation_id,
            context: HashMap::new(),
        }
    }
    
    /// Mark the message as received
    pub fn mark_received(&mut self) {
        self.received_at = Some(Instant::now());
    }
    
    /// Mark the message as processed
    pub fn mark_processed(&mut self, success: bool, error: Option<String>) {
        let now = Instant::now();
        self.processed_at = Some(now);
        
        if let Some(received_at) = self.received_at {
            self.processing_duration = Some(now.duration_since(received_at));
        }
        
        self.success = Some(success);
        self.error = error;
    }
    
    /// Add context to the message flow
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Represents an actor in the visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorNode {
    /// Actor ID
    pub id: ActorID,
    /// Actor type name
    pub actor_type: String,
    /// Time when the actor was created
    pub created_at: Instant,
    /// Time when the actor was stopped (if applicable)
    pub stopped_at: Option<Instant>,
    /// Parent actor ID (if applicable)
    pub parent_id: Option<ActorID>,
    /// Child actor IDs
    pub child_ids: HashSet<ActorID>,
    /// Number of messages sent
    pub messages_sent: u64,
    /// Number of messages received
    pub messages_received: u64,
    /// Number of messages processed successfully
    pub messages_succeeded: u64,
    /// Number of messages that failed
    pub messages_failed: u64,
    /// Average processing time
    pub avg_processing_time: Option<Duration>,
    /// Additional context
    pub context: HashMap<String, String>,
}

impl ActorNode {
    /// Create a new actor node
    pub fn new(id: ActorID, actor_type: impl Into<String>) -> Self {
        Self {
            id,
            actor_type: actor_type.into(),
            created_at: Instant::now(),
            stopped_at: None,
            parent_id: None,
            child_ids: HashSet::new(),
            messages_sent: 0,
            messages_received: 0,
            messages_succeeded: 0,
            messages_failed: 0,
            avg_processing_time: None,
            context: HashMap::new(),
        }
    }
    
    /// Set the parent actor ID
    pub fn with_parent(mut self, parent_id: ActorID) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
    
    /// Add a child actor ID
    pub fn add_child(&mut self, child_id: ActorID) {
        self.child_ids.insert(child_id);
    }
    
    /// Mark the actor as stopped
    pub fn mark_stopped(&mut self) {
        self.stopped_at = Some(Instant::now());
    }
    
    /// Update message statistics
    pub fn update_stats(
        &mut self,
        messages_sent: u64,
        messages_received: u64,
        messages_succeeded: u64,
        messages_failed: u64,
        avg_processing_time: Option<Duration>,
    ) {
        self.messages_sent = messages_sent;
        self.messages_received = messages_received;
        self.messages_succeeded = messages_succeeded;
        self.messages_failed = messages_failed;
        self.avg_processing_time = avg_processing_time;
    }
    
    /// Add context to the actor node
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Actor system visualization
#[derive(Debug, Clone, Default)]
pub struct ActorVisualization {
    /// Actor nodes by ID
    pub actors: HashMap<ActorID, ActorNode>,
    /// Message flows by ID
    pub message_flows: HashMap<Uuid, MessageFlow>,
    /// Root actor IDs
    pub root_actor_ids: HashSet<ActorID>,
}

impl ActorVisualization {
    /// Create a new actor visualization
    pub fn new() -> Self {
        Self {
            actors: HashMap::new(),
            message_flows: HashMap::new(),
            root_actor_ids: HashSet::new(),
        }
    }
    
    /// Add an actor to the visualization
    pub fn add_actor(&mut self, actor: ActorNode) {
        // If the actor has no parent, add it to the root actors
        if actor.parent_id.is_none() {
            self.root_actor_ids.insert(actor.id.clone());
        } else if let Some(parent_id) = &actor.parent_id {
            // Add this actor as a child of its parent
            if let Some(parent) = self.actors.get_mut(parent_id) {
                parent.add_child(actor.id.clone());
            }
        }
        
        self.actors.insert(actor.id.clone(), actor);
    }
    
    /// Add a message flow to the visualization
    pub fn add_message_flow(&mut self, message_flow: MessageFlow) {
        self.message_flows.insert(message_flow.id, message_flow);
    }
    
    /// Get all message flows for a specific actor (sent or received)
    pub fn get_actor_message_flows(&self, actor_id: &ActorID) -> Vec<&MessageFlow> {
        self.message_flows
            .values()
            .filter(|flow| flow.sender_id == *actor_id || flow.receiver_id == *actor_id)
            .collect()
    }
    
    /// Get all message flows with a specific correlation ID
    pub fn get_correlated_message_flows(&self, correlation_id: &str) -> Vec<&MessageFlow> {
        self.message_flows
            .values()
            .filter(|flow| flow.correlation_id.as_deref() == Some(correlation_id))
            .collect()
    }
    
    /// Get the actor hierarchy as a tree
    pub fn get_actor_hierarchy(&self) -> Vec<(&ActorID, Vec<&ActorID>)> {
        let mut result = Vec::new();
        
        for root_id in &self.root_actor_ids {
            if let Some(root_actor) = self.actors.get(root_id) {
                let children: Vec<&ActorID> = root_actor.child_ids
                    .iter()
                    .filter(|id| self.actors.contains_key(*id))
                    .collect();
                
                result.push((root_id, children));
            }
        }
        
        result
    }
    
    /// Export the visualization as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        #[derive(Serialize)]
        struct ExportData {
            actors: Vec<ActorNode>,
            message_flows: Vec<MessageFlowExport>,
            root_actor_ids: Vec<ActorID>,
        }
        
        #[derive(Serialize)]
        struct MessageFlowExport {
            id: Uuid,
            sender_id: ActorID,
            receiver_id: ActorID,
            message_type: String,
            sent_at_ms: u64,
            received_at_ms: Option<u64>,
            processed_at_ms: Option<u64>,
            processing_duration_ms: Option<u64>,
            success: Option<bool>,
            error: Option<String>,
            correlation_id: Option<String>,
            context: HashMap<String, String>,
        }
        
        // Convert message flows to a serializable format
        let message_flows: Vec<MessageFlowExport> = self.message_flows
            .values()
            .map(|flow| {
                let now = Instant::now();
                let sent_at_ms = now.duration_since(flow.sent_at).as_millis() as u64;
                
                let received_at_ms = flow.received_at.map(|t| now.duration_since(t).as_millis() as u64);
                let processed_at_ms = flow.processed_at.map(|t| now.duration_since(t).as_millis() as u64);
                let processing_duration_ms = flow.processing_duration.map(|d| d.as_millis() as u64);
                
                MessageFlowExport {
                    id: flow.id,
                    sender_id: flow.sender_id.clone(),
                    receiver_id: flow.receiver_id.clone(),
                    message_type: flow.message_type.clone(),
                    sent_at_ms,
                    received_at_ms,
                    processed_at_ms,
                    processing_duration_ms,
                    success: flow.success,
                    error: flow.error.clone(),
                    correlation_id: flow.correlation_id.clone(),
                    context: flow.context.clone(),
                }
            })
            .collect();
        
        let export_data = ExportData {
            actors: self.actors.values().cloned().collect(),
            message_flows,
            root_actor_ids: self.root_actor_ids.iter().cloned().collect(),
        };
        
        serde_json::to_string_pretty(&export_data)
    }
}

/// Global actor visualization instance
lazy_static::lazy_static! {
    static ref ACTOR_VISUALIZATION: Arc<Mutex<ActorVisualization>> = Arc::new(Mutex::new(ActorVisualization::new()));
}

/// Get the global actor visualization instance
pub fn get_actor_visualization() -> Arc<Mutex<ActorVisualization>> {
    ACTOR_VISUALIZATION.clone()
}

/// Register an actor with the visualization system
pub fn register_actor(id: ActorID, actor_type: impl Into<String>, parent_id: Option<ActorID>) {
    let mut actor = ActorNode::new(id.clone(), actor_type);
    
    if let Some(parent_id) = parent_id {
        actor = actor.with_parent(parent_id);
    }
    
    let mut visualization = ACTOR_VISUALIZATION.lock().unwrap();
    visualization.add_actor(actor);
}

/// Record a message being sent between actors
pub fn record_message_sent(
    sender_id: ActorID,
    receiver_id: ActorID,
    message_type: impl Into<String>,
) -> Uuid {
    let message_flow = MessageFlow::new(sender_id, receiver_id, message_type);
    let message_id = message_flow.id;
    
    let mut visualization = ACTOR_VISUALIZATION.lock().unwrap();
    visualization.add_message_flow(message_flow);
    
    message_id
}

/// Record a message being received
pub fn record_message_received(message_id: Uuid) {
    let mut visualization = ACTOR_VISUALIZATION.lock().unwrap();
    
    if let Some(message_flow) = visualization.message_flows.get_mut(&message_id) {
        message_flow.mark_received();
    }
}

/// Record a message being processed
pub fn record_message_processed(message_id: Uuid, success: bool, error: Option<String>) {
    let mut visualization = ACTOR_VISUALIZATION.lock().unwrap();
    
    if let Some(message_flow) = visualization.message_flows.get_mut(&message_id) {
        message_flow.mark_processed(success, error);
    }
}

/// Update actor statistics
pub fn update_actor_stats(
    actor_id: ActorID,
    messages_sent: u64,
    messages_received: u64,
    messages_succeeded: u64,
    messages_failed: u64,
    avg_processing_time: Option<Duration>,
) {
    let mut visualization = ACTOR_VISUALIZATION.lock().unwrap();
    
    if let Some(actor) = visualization.actors.get_mut(&actor_id) {
        actor.update_stats(
            messages_sent,
            messages_received,
            messages_succeeded,
            messages_failed,
            avg_processing_time,
        );
    }
}

/// Mark an actor as stopped
pub fn mark_actor_stopped(actor_id: ActorID) {
    let mut visualization = ACTOR_VISUALIZATION.lock().unwrap();
    
    if let Some(actor) = visualization.actors.get_mut(&actor_id) {
        actor.mark_stopped();
    }
}

/// Export the current actor visualization as JSON
pub fn export_visualization_json() -> Result<String, serde_json::Error> {
    let visualization = ACTOR_VISUALIZATION.lock().unwrap();
    visualization.export_json()
}