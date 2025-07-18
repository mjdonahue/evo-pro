//! Actor mocking utilities
//!
//! This module provides utilities for mocking actors and messages in tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kameo::prelude::*;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::error::{AppError, Result};
use super::{TestEnv, TestEvent, TestEventType};

/// Mock actor that can be configured to respond in specific ways
#[derive(Actor, Clone)]
pub struct MockActor {
    /// Name of the mock actor
    name: String,
    /// Response handlers for different message types
    response_handlers: Arc<Mutex<HashMap<String, Box<dyn Fn() -> Box<dyn Any + Send> + Send + Sync>>>>,
    /// Expected messages
    expected_messages: Arc<Mutex<Vec<ExpectedMessage>>>,
    /// Received messages
    received_messages: Arc<Mutex<Vec<ReceivedMessage>>>,
    /// Test environment
    test_env: Option<Arc<TestEnv>>,
}

/// Expected message
#[derive(Debug, Clone)]
pub struct ExpectedMessage {
    /// Message type
    pub message_type: String,
    /// Number of times the message is expected
    pub count: usize,
    /// Whether the order matters
    pub ordered: bool,
    /// Position in the order (if ordered)
    pub position: Option<usize>,
}

/// Received message
#[derive(Debug)]
pub struct ReceivedMessage {
    /// Message type
    pub message_type: String,
    /// Timestamp when the message was received
    pub timestamp: std::time::SystemTime,
    /// Message payload
    pub payload: Box<dyn Any + Send>,
}

impl MockActor {
    /// Create a new mock actor
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            response_handlers: Arc::new(Mutex::new(HashMap::new())),
            expected_messages: Arc::new(Mutex::new(Vec::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
            test_env: None,
        }
    }

    /// Set the test environment
    pub fn with_test_env(mut self, test_env: Arc<TestEnv>) -> Self {
        self.test_env = Some(test_env);
        self
    }

    /// Register a response handler for a specific message type
    pub fn on_message<M, R>(&self, handler: impl Fn() -> R + Send + Sync + 'static)
    where
        M: Message<M> + 'static,
        R: Into<M::Reply> + 'static,
    {
        let message_type = std::any::type_name::<M>().to_string();
        let mut handlers = self.response_handlers.lock().unwrap();
        
        handlers.insert(
            message_type,
            Box::new(move || Box::new(handler())),
        );
    }

    /// Expect a message of a specific type
    pub fn expect_message<M>(&self, count: usize, ordered: bool)
    where
        M: Message<M> + 'static,
    {
        let message_type = std::any::type_name::<M>().to_string();
        let mut expected = self.expected_messages.lock().unwrap();
        
        let position = if ordered {
            Some(expected.len())
        } else {
            None
        };
        
        expected.push(ExpectedMessage {
            message_type,
            count,
            ordered,
            position,
        });
    }

    /// Record a received message
    fn record_message<M>(&self, message: &M)
    where
        M: Message<M> + 'static,
    {
        let message_type = std::any::type_name::<M>().to_string();
        let mut received = self.received_messages.lock().unwrap();
        
        received.push(ReceivedMessage {
            message_type: message_type.clone(),
            timestamp: std::time::SystemTime::now(),
            payload: Box::new(()),  // We don't store the actual message for now
        });
        
        // Capture event if test environment is set
        if let Some(ref test_env) = self.test_env {
            test_env.capture_event(
                TestEvent::new(TestEventType::MessageReceived)
                    .with_actor_id(ActorID::new())  // We don't have the actual ID here
                    .with_message_type(message_type)
            );
        }
    }

    /// Verify that all expected messages were received
    pub fn verify(&self) -> Result<()> {
        let expected = self.expected_messages.lock().unwrap();
        let received = self.received_messages.lock().unwrap();
        
        // Check that all expected messages were received
        for exp in expected.iter() {
            let count = received.iter()
                .filter(|r| r.message_type == exp.message_type)
                .count();
            
            if count != exp.count {
                return Err(AppError::ValidationError(format!(
                    "Expected {} messages of type {}, but received {}",
                    exp.count, exp.message_type, count
                )));
            }
            
            // Check order if required
            if exp.ordered {
                if let Some(position) = exp.position {
                    // Find all messages of this type
                    let positions: Vec<usize> = received.iter()
                        .enumerate()
                        .filter(|(_, r)| r.message_type == exp.message_type)
                        .map(|(i, _)| i)
                        .collect();
                    
                    // Check that they are in the correct order
                    for (i, pos) in positions.iter().enumerate() {
                        if *pos < position {
                            return Err(AppError::ValidationError(format!(
                                "Message of type {} received out of order",
                                exp.message_type
                            )));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Generic message handler for MockActor
impl<M: Message<M> + 'static> Message<M> for MockActor
where
    M::Reply: Send + 'static,
{
    type Reply = M::Reply;

    async fn handle(
        &mut self,
        msg: M,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Record the message
        self.record_message(&msg);
        
        // Get the message type
        let message_type = std::any::type_name::<M>().to_string();
        
        // Look up the response handler
        let handlers = self.response_handlers.lock().unwrap();
        
        if let Some(handler) = handlers.get(&message_type) {
            // Call the handler to get the response
            let response = handler();
            
            // Try to downcast to the expected reply type
            if let Some(reply) = response.downcast_ref::<M::Reply>() {
                // Clone the reply
                let reply_clone = reply.clone();
                return reply_clone;
            }
        }
        
        // If no handler is found or downcasting fails, return a default value
        // This is a simplification; in a real implementation, we would need to handle this better
        panic!("No response handler found for message type: {}", message_type);
    }
}

/// Create a mock actor
pub fn create_mock_actor(name: impl Into<String>) -> ActorRef<MockActor> {
    MockActor::spawn(MockActor::new(name))
}

/// Create a mock actor with a test environment
pub fn create_mock_actor_with_env(name: impl Into<String>, test_env: Arc<TestEnv>) -> ActorRef<MockActor> {
    MockActor::spawn(MockActor::new(name).with_test_env(test_env))
}

/// Extension trait for ActorRef to add mocking capabilities
pub trait MockExt<A: Actor + 'static> {
    /// Mock a response for a specific message type
    fn mock_response<M, R>(&self, response: R) -> Result<()>
    where
        M: Message<M, Reply = R> + 'static,
        R: Clone + Send + Sync + 'static;
    
    /// Expect a message of a specific type
    fn expect_message<M>(&self, count: usize, ordered: bool) -> Result<()>
    where
        M: Message<M> + 'static;
    
    /// Verify that all expected messages were received
    fn verify(&self) -> Result<()>;
}

impl MockExt<MockActor> for ActorRef<MockActor> {
    fn mock_response<M, R>(&self, response: R) -> Result<()>
    where
        M: Message<M, Reply = R> + 'static,
        R: Clone + Send + Sync + 'static,
    {
        let response_clone = response.clone();
        self.tell(&MockResponse::<M, R> {
            handler: Box::new(move || response_clone.clone()),
            _phantom: std::marker::PhantomData,
        }).await?;
        
        Ok(())
    }
    
    fn expect_message<M>(&self, count: usize, ordered: bool) -> Result<()>
    where
        M: Message<M> + 'static,
    {
        self.tell(&ExpectMessage::<M> {
            count,
            ordered,
            _phantom: std::marker::PhantomData,
        }).await?;
        
        Ok(())
    }
    
    fn verify(&self) -> Result<()> {
        self.ask(&VerifyMock).await
    }
}

/// Message to mock a response
pub struct MockResponse<M, R>
where
    M: Message<M, Reply = R> + 'static,
    R: Clone + Send + Sync + 'static,
{
    /// Response handler
    pub handler: Box<dyn Fn() -> R + Send + Sync>,
    /// Phantom data for message type
    pub _phantom: std::marker::PhantomData<M>,
}

impl<M, R> Message<MockResponse<M, R>> for MockActor
where
    M: Message<M, Reply = R> + 'static,
    R: Clone + Send + Sync + 'static,
{
    type Reply = ();

    async fn handle(
        &mut self,
        msg: MockResponse<M, R>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let message_type = std::any::type_name::<M>().to_string();
        let mut handlers = self.response_handlers.lock().unwrap();
        
        handlers.insert(
            message_type,
            Box::new(move || Box::new(msg.handler())),
        );
    }
}

/// Message to expect a message
pub struct ExpectMessage<M>
where
    M: Message<M> + 'static,
{
    /// Number of times the message is expected
    pub count: usize,
    /// Whether the order matters
    pub ordered: bool,
    /// Phantom data for message type
    pub _phantom: std::marker::PhantomData<M>,
}

impl<M> Message<ExpectMessage<M>> for MockActor
where
    M: Message<M> + 'static,
{
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ExpectMessage<M>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let message_type = std::any::type_name::<M>().to_string();
        let mut expected = self.expected_messages.lock().unwrap();
        
        let position = if msg.ordered {
            Some(expected.len())
        } else {
            None
        };
        
        expected.push(ExpectedMessage {
            message_type,
            count: msg.count,
            ordered: msg.ordered,
            position,
        });
    }
}

/// Message to verify mock expectations
#[derive(Debug, Clone)]
pub struct VerifyMock;

impl Message<VerifyMock> for MockActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        _msg: VerifyMock,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.verify()
    }
}