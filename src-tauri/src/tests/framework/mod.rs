//! Actor testing framework
//!
//! This module provides a framework for testing actors in isolation and in
//! integration scenarios. It includes utilities for mocking actors, capturing
//! events, and verifying actor behavior.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kameo::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, Result};

pub mod mock;
pub mod harness;
pub mod scenario;
pub mod verification;

/// Configuration for the test environment
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Whether to capture events
    pub capture_events: bool,
    /// Maximum number of events to capture
    pub max_events: usize,
    /// Timeout for test operations
    pub timeout: Duration,
    /// Whether to fail fast on errors
    pub fail_fast: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            capture_events: true,
            max_events: 1000,
            timeout: Duration::from_secs(5),
            fail_fast: true,
        }
    }
}

/// Test event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestEventType {
    /// Actor started
    ActorStarted,
    /// Actor stopped
    ActorStopped,
    /// Message sent
    MessageSent,
    /// Message received
    MessageReceived,
    /// Error occurred
    ErrorOccurred,
    /// Custom event
    Custom(String),
}

/// Test event
#[derive(Debug, Clone)]
pub struct TestEvent {
    /// Event type
    pub event_type: TestEventType,
    /// Timestamp when the event occurred
    pub timestamp: Instant,
    /// Actor ID associated with the event
    pub actor_id: Option<ActorID>,
    /// Message type associated with the event
    pub message_type: Option<String>,
    /// Error message associated with the event
    pub error_message: Option<String>,
    /// Custom data associated with the event
    pub custom_data: Option<HashMap<String, String>>,
}

impl TestEvent {
    /// Create a new test event
    pub fn new(event_type: TestEventType) -> Self {
        Self {
            event_type,
            timestamp: Instant::now(),
            actor_id: None,
            message_type: None,
            error_message: None,
            custom_data: None,
        }
    }

    /// Set the actor ID
    pub fn with_actor_id(mut self, actor_id: ActorID) -> Self {
        self.actor_id = Some(actor_id);
        self
    }

    /// Set the message type
    pub fn with_message_type(mut self, message_type: impl Into<String>) -> Self {
        self.message_type = Some(message_type.into());
        self
    }

    /// Set the error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_message = Some(error.into());
        self
    }

    /// Add custom data
    pub fn with_custom_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.custom_data.is_none() {
            self.custom_data = Some(HashMap::new());
        }
        
        if let Some(ref mut data) = self.custom_data {
            data.insert(key.into(), value.into());
        }
        
        self
    }
}

/// Test environment for actor testing
pub struct TestEnv {
    /// Test configuration
    config: TestConfig,
    /// Captured events
    events: Mutex<Vec<TestEvent>>,
    /// Event subscribers
    subscribers: Mutex<Vec<mpsc::Sender<TestEvent>>>,
    /// Mock registry
    mocks: Mutex<HashMap<String, ActorRef<dyn Any>>>,
}

impl TestEnv {
    /// Create a new test environment
    pub fn new(config: TestConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            events: Mutex::new(Vec::new()),
            subscribers: Mutex::new(Vec::new()),
            mocks: Mutex::new(HashMap::new()),
        })
    }

    /// Capture an event
    pub fn capture_event(&self, event: TestEvent) {
        if !self.config.capture_events {
            return;
        }
        
        // Add to events
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        
        // Trim events if needed
        if events.len() > self.config.max_events {
            events.remove(0);
        }
        
        // Notify subscribers
        let subscribers = self.subscribers.lock().unwrap();
        for subscriber in subscribers.iter() {
            let _ = subscriber.try_send(event.clone());
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> mpsc::Receiver<TestEvent> {
        let (tx, rx) = mpsc::channel(100);
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.push(tx);
        rx
    }

    /// Get all captured events
    pub fn get_events(&self) -> Vec<TestEvent> {
        let events = self.events.lock().unwrap();
        events.clone()
    }

    /// Get events of a specific type
    pub fn get_events_by_type(&self, event_type: TestEventType) -> Vec<TestEvent> {
        let events = self.events.lock().unwrap();
        events.iter()
            .filter(|e| e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get events for a specific actor
    pub fn get_events_for_actor(&self, actor_id: ActorID) -> Vec<TestEvent> {
        let events = self.events.lock().unwrap();
        events.iter()
            .filter(|e| e.actor_id == Some(actor_id))
            .cloned()
            .collect()
    }

    /// Register a mock actor
    pub fn register_mock<A: Actor + 'static>(&self, name: impl Into<String>, mock: ActorRef<A>) {
        let mut mocks = self.mocks.lock().unwrap();
        mocks.insert(name.into(), mock.into_any());
    }

    /// Get a mock actor by name
    pub fn get_mock<A: Actor + 'static>(&self, name: &str) -> Option<ActorRef<A>> {
        let mocks = self.mocks.lock().unwrap();
        mocks.get(name).and_then(|mock| {
            mock.clone().try_into().ok()
        })
    }

    /// Clear all captured events
    pub fn clear_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }

    /// Clear all registered mocks
    pub fn clear_mocks(&self) {
        let mut mocks = self.mocks.lock().unwrap();
        mocks.clear();
    }

    /// Reset the test environment
    pub fn reset(&self) {
        self.clear_events();
        self.clear_mocks();
    }
}

/// Create a default test environment
pub fn create_test_env() -> Arc<TestEnv> {
    TestEnv::new(TestConfig::default())
}

/// Create a test environment with custom configuration
pub fn create_test_env_with_config(config: TestConfig) -> Arc<TestEnv> {
    TestEnv::new(config)
}

/// Run a test with timeout
pub async fn run_with_timeout<F, T>(timeout: Duration, test: F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    tokio::time::timeout(timeout, test)
        .await
        .map_err(|_| AppError::TimeoutError(format!("Test timed out after {:?}", timeout)))?
}