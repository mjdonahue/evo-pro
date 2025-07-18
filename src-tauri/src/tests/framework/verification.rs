//! Verification utilities for actor testing
//!
//! This module provides utilities for verifying actor behavior in tests.
//! It includes assertion helpers, event verification, and other utilities
//! for checking that actors behave as expected.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kameo::prelude::*;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, Result};
use super::{TestEnv, TestEvent, TestEventType, TestConfig};

/// Assertion result
pub type AssertionResult = std::result::Result<(), String>;

/// Trait for types that can be verified
pub trait Verifiable {
    /// Verify that the condition is true
    fn verify(&self) -> AssertionResult;
}

/// Assertion builder for fluent assertions
pub struct Assert<T> {
    /// Value being asserted on
    value: T,
    /// Description of the assertion
    description: Option<String>,
}

impl<T> Assert<T> {
    /// Create a new assertion
    pub fn new(value: T) -> Self {
        Self {
            value,
            description: None,
        }
    }

    /// Add a description to the assertion
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get the description or a default
    fn get_description(&self) -> String {
        self.description.clone().unwrap_or_else(|| "Assertion failed".to_string())
    }
}

impl<T: PartialEq> Assert<T> {
    /// Assert that the value equals the expected value
    pub fn equals(self, expected: T) -> AssertionResult
    where
        T: std::fmt::Debug,
    {
        if self.value == expected {
            Ok(())
        } else {
            Err(format!("{}: expected {:?}, got {:?}", self.get_description(), expected, self.value))
        }
    }

    /// Assert that the value does not equal the expected value
    pub fn not_equals(self, expected: T) -> AssertionResult
    where
        T: std::fmt::Debug,
    {
        if self.value != expected {
            Ok(())
        } else {
            Err(format!("{}: expected not {:?}, got {:?}", self.get_description(), expected, self.value))
        }
    }
}

impl<T: PartialOrd> Assert<T> {
    /// Assert that the value is greater than the expected value
    pub fn greater_than(self, expected: T) -> AssertionResult
    where
        T: std::fmt::Debug,
    {
        if self.value > expected {
            Ok(())
        } else {
            Err(format!("{}: expected > {:?}, got {:?}", self.get_description(), expected, self.value))
        }
    }

    /// Assert that the value is greater than or equal to the expected value
    pub fn greater_than_or_equal(self, expected: T) -> AssertionResult
    where
        T: std::fmt::Debug,
    {
        if self.value >= expected {
            Ok(())
        } else {
            Err(format!("{}: expected >= {:?}, got {:?}", self.get_description(), expected, self.value))
        }
    }

    /// Assert that the value is less than the expected value
    pub fn less_than(self, expected: T) -> AssertionResult
    where
        T: std::fmt::Debug,
    {
        if self.value < expected {
            Ok(())
        } else {
            Err(format!("{}: expected < {:?}, got {:?}", self.get_description(), expected, self.value))
        }
    }

    /// Assert that the value is less than or equal to the expected value
    pub fn less_than_or_equal(self, expected: T) -> AssertionResult
    where
        T: std::fmt::Debug,
    {
        if self.value <= expected {
            Ok(())
        } else {
            Err(format!("{}: expected <= {:?}, got {:?}", self.get_description(), expected, self.value))
        }
    }
}

impl<T> Assert<Option<T>> {
    /// Assert that the option is Some
    pub fn is_some(self) -> AssertionResult {
        if self.value.is_some() {
            Ok(())
        } else {
            Err(format!("{}: expected Some, got None", self.get_description()))
        }
    }

    /// Assert that the option is None
    pub fn is_none(self) -> AssertionResult {
        if self.value.is_none() {
            Ok(())
        } else {
            Err(format!("{}: expected None, got Some", self.get_description()))
        }
    }

    /// Map the option to a new assertion
    pub fn map<U, F>(self, f: F) -> Assert<Option<U>>
    where
        F: FnOnce(T) -> U,
    {
        Assert {
            value: self.value.map(f),
            description: self.description,
        }
    }

    /// Unwrap the option and continue assertions
    pub fn unwrap(self) -> Result<Assert<T>> {
        match self.value {
            Some(value) => Ok(Assert {
                value,
                description: self.description,
            }),
            None => Err(AppError::ValidationError(format!("{}: expected Some, got None", self.get_description()))),
        }
    }
}

impl<T, E> Assert<std::result::Result<T, E>> {
    /// Assert that the result is Ok
    pub fn is_ok(self) -> AssertionResult
    where
        E: std::fmt::Debug,
    {
        match &self.value {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}: expected Ok, got Err({:?})", self.get_description(), e)),
        }
    }

    /// Assert that the result is Err
    pub fn is_err(self) -> AssertionResult
    where
        E: std::fmt::Debug,
    {
        match &self.value {
            Ok(_) => Err(format!("{}: expected Err, got Ok", self.get_description())),
            Err(_) => Ok(()),
        }
    }

    /// Map the result to a new assertion
    pub fn map<U, F>(self, f: F) -> Assert<std::result::Result<U, E>>
    where
        F: FnOnce(T) -> U,
    {
        Assert {
            value: self.value.map(f),
            description: self.description,
        }
    }

    /// Unwrap the result and continue assertions
    pub fn unwrap(self) -> Result<Assert<T>>
    where
        E: std::fmt::Debug,
    {
        match self.value {
            Ok(value) => Ok(Assert {
                value,
                description: self.description,
            }),
            Err(e) => Err(AppError::ValidationError(format!("{}: expected Ok, got Err({:?})", self.get_description(), e))),
        }
    }
}

impl<T> Assert<Vec<T>> {
    /// Assert that the vector contains the expected value
    pub fn contains(self, expected: &T) -> AssertionResult
    where
        T: PartialEq + std::fmt::Debug,
    {
        if self.value.contains(expected) {
            Ok(())
        } else {
            Err(format!("{}: expected to contain {:?}", self.get_description(), expected))
        }
    }

    /// Assert that the vector does not contain the expected value
    pub fn does_not_contain(self, expected: &T) -> AssertionResult
    where
        T: PartialEq + std::fmt::Debug,
    {
        if !self.value.contains(expected) {
            Ok(())
        } else {
            Err(format!("{}: expected not to contain {:?}", self.get_description(), expected))
        }
    }

    /// Assert that the vector has the expected length
    pub fn has_length(self, expected: usize) -> AssertionResult {
        if self.value.len() == expected {
            Ok(())
        } else {
            Err(format!("{}: expected length {}, got {}", self.get_description(), expected, self.value.len()))
        }
    }

    /// Assert that the vector is empty
    pub fn is_empty(self) -> AssertionResult {
        if self.value.is_empty() {
            Ok(())
        } else {
            Err(format!("{}: expected empty, got length {}", self.get_description(), self.value.len()))
        }
    }

    /// Assert that the vector is not empty
    pub fn is_not_empty(self) -> AssertionResult {
        if !self.value.is_empty() {
            Ok(())
        } else {
            Err(format!("{}: expected not empty", self.get_description()))
        }
    }
}

impl Assert<String> {
    /// Assert that the string contains the expected substring
    pub fn contains(self, expected: &str) -> AssertionResult {
        if self.value.contains(expected) {
            Ok(())
        } else {
            Err(format!("{}: expected to contain '{}', got '{}'", self.get_description(), expected, self.value))
        }
    }

    /// Assert that the string starts with the expected prefix
    pub fn starts_with(self, expected: &str) -> AssertionResult {
        if self.value.starts_with(expected) {
            Ok(())
        } else {
            Err(format!("{}: expected to start with '{}', got '{}'", self.get_description(), expected, self.value))
        }
    }

    /// Assert that the string ends with the expected suffix
    pub fn ends_with(self, expected: &str) -> AssertionResult {
        if self.value.ends_with(expected) {
            Ok(())
        } else {
            Err(format!("{}: expected to end with '{}', got '{}'", self.get_description(), expected, self.value))
        }
    }
}

impl Assert<bool> {
    /// Assert that the value is true
    pub fn is_true(self) -> AssertionResult {
        if self.value {
            Ok(())
        } else {
            Err(format!("{}: expected true, got false", self.get_description()))
        }
    }

    /// Assert that the value is false
    pub fn is_false(self) -> AssertionResult {
        if !self.value {
            Ok(())
        } else {
            Err(format!("{}: expected false, got true", self.get_description()))
        }
    }
}

/// Create a new assertion
pub fn assert_that<T>(value: T) -> Assert<T> {
    Assert::new(value)
}

/// Event verifier for checking test events
pub struct EventVerifier {
    /// Test environment
    test_env: Arc<TestEnv>,
    /// Event receiver
    event_rx: mpsc::Receiver<TestEvent>,
}

impl EventVerifier {
    /// Create a new event verifier
    pub fn new(test_env: Arc<TestEnv>) -> Self {
        let event_rx = test_env.subscribe();
        Self {
            test_env,
            event_rx,
        }
    }

    /// Wait for an event of the specified type
    pub async fn wait_for_event(&mut self, event_type: TestEventType, timeout_duration: Duration) -> Result<TestEvent> {
        let result = timeout(timeout_duration, async {
            while let Some(event) = self.event_rx.recv().await {
                if event.event_type == event_type {
                    return event;
                }
            }
            Err(AppError::TimeoutError("Event stream ended".to_string()))
        }).await;

        match result {
            Ok(Ok(event)) => Ok(event),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AppError::TimeoutError(format!("Timed out waiting for event {:?}", event_type))),
        }
    }

    /// Wait for an event matching a predicate
    pub async fn wait_for_event_matching<F>(&mut self, predicate: F, timeout_duration: Duration) -> Result<TestEvent>
    where
        F: Fn(&TestEvent) -> bool,
    {
        let result = timeout(timeout_duration, async {
            while let Some(event) = self.event_rx.recv().await {
                if predicate(&event) {
                    return event;
                }
            }
            Err(AppError::TimeoutError("Event stream ended".to_string()))
        }).await;

        match result {
            Ok(Ok(event)) => Ok(event),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AppError::TimeoutError("Timed out waiting for matching event".to_string())),
        }
    }

    /// Verify that an event of the specified type occurred
    pub fn verify_event_occurred(&self, event_type: TestEventType) -> AssertionResult {
        let events = self.test_env.get_events_by_type(event_type);
        if events.is_empty() {
            Err(format!("Expected event {:?} to have occurred, but it did not", event_type))
        } else {
            Ok(())
        }
    }

    /// Verify that an event of the specified type did not occur
    pub fn verify_event_did_not_occur(&self, event_type: TestEventType) -> AssertionResult {
        let events = self.test_env.get_events_by_type(event_type);
        if events.is_empty() {
            Ok(())
        } else {
            Err(format!("Expected event {:?} not to have occurred, but it did {} times", event_type, events.len()))
        }
    }

    /// Verify that an event matching a predicate occurred
    pub fn verify_event_matching<F>(&self, predicate: F) -> AssertionResult
    where
        F: Fn(&TestEvent) -> bool,
    {
        let events = self.test_env.get_events();
        if events.iter().any(predicate) {
            Ok(())
        } else {
            Err("Expected matching event to have occurred, but it did not".to_string())
        }
    }

    /// Verify that an event for a specific actor occurred
    pub fn verify_actor_event(&self, actor_id: ActorID, event_type: TestEventType) -> AssertionResult {
        let events = self.test_env.get_events_for_actor(actor_id);
        if events.iter().any(|e| e.event_type == event_type) {
            Ok(())
        } else {
            Err(format!("Expected event {:?} for actor {} to have occurred, but it did not", event_type, actor_id))
        }
    }

    /// Get all events of the specified type
    pub fn get_events_by_type(&self, event_type: TestEventType) -> Vec<TestEvent> {
        self.test_env.get_events_by_type(event_type)
    }

    /// Get all events for the specified actor
    pub fn get_events_for_actor(&self, actor_id: ActorID) -> Vec<TestEvent> {
        self.test_env.get_events_for_actor(actor_id)
    }

    /// Clear all events
    pub fn clear_events(&self) {
        self.test_env.clear_events();
    }
}

/// Create a new event verifier
pub fn create_event_verifier(test_env: Arc<TestEnv>) -> EventVerifier {
    EventVerifier::new(test_env)
}

/// Convert an assertion result to a Result
pub fn to_result(assertion_result: AssertionResult) -> Result<()> {
    assertion_result.map_err(|e| AppError::ValidationError(e))
}

/// Verify that a condition is true
pub fn verify_that(condition: bool, message: impl Into<String>) -> AssertionResult {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

/// Verify that a function returns true
pub fn verify_fn<F>(f: F, message: impl Into<String>) -> AssertionResult
where
    F: FnOnce() -> bool,
{
    verify_that(f(), message)
}

/// Verify that an async function returns true
pub async fn verify_async<F, Fut>(f: F, message: impl Into<String>) -> AssertionResult
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    verify_that(f().await, message)
}