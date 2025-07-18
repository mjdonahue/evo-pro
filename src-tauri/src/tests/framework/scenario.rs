//! Test scenarios for actor testing
//!
//! This module provides utilities for defining and running test scenarios.
//! A test scenario is a sequence of steps that simulate interactions with
//! actors and verify their behavior.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, Result};
use super::{TestEnv, TestEvent, TestEventType, TestConfig, run_with_timeout};
use super::mock::{MockActor, create_mock_actor_with_env, MockExt};
use super::harness::{ActorTestHarness, SupervisedActorTestHarness, LifecycleTestHarness};

/// A step in a test scenario
pub struct ScenarioStep {
    /// Step name
    pub name: String,
    /// Step function
    pub func: Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send>,
    /// Timeout for this step
    pub timeout: Option<Duration>,
}

impl ScenarioStep {
    /// Create a new scenario step
    pub fn new<F>(name: impl Into<String>, func: F) -> Self
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        Self {
            name: name.into(),
            func: Box::new(func),
            timeout: None,
        }
    }

    /// Set a timeout for this step
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Execute the step
    pub async fn execute(self) -> Result<()> {
        if let Some(timeout) = self.timeout {
            run_with_timeout(timeout, async {
                (self.func)().await
            }).await
        } else {
            (self.func)().await
        }
    }
}

/// A test scenario
pub struct TestScenario {
    /// Scenario name
    pub name: String,
    /// Steps in the scenario
    pub steps: Vec<ScenarioStep>,
    /// Setup function
    pub setup: Option<Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send>>,
    /// Teardown function
    pub teardown: Option<Box<dyn FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send>>,
    /// Default timeout for steps
    pub default_timeout: Duration,
}

impl TestScenario {
    /// Create a new test scenario
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
            setup: None,
            teardown: None,
            default_timeout: Duration::from_secs(5),
        }
    }

    /// Add a step to the scenario
    pub fn add_step<F>(mut self, name: impl Into<String>, func: F) -> Self
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        let step = ScenarioStep::new(name, func);
        self.steps.push(step);
        self
    }

    /// Add a step with a timeout to the scenario
    pub fn add_step_with_timeout<F>(mut self, name: impl Into<String>, timeout: Duration, func: F) -> Self
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        let step = ScenarioStep::new(name, func).with_timeout(timeout);
        self.steps.push(step);
        self
    }

    /// Set the setup function
    pub fn with_setup<F>(mut self, func: F) -> Self
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        self.setup = Some(Box::new(func));
        self
    }

    /// Set the teardown function
    pub fn with_teardown<F>(mut self, func: F) -> Self
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        self.teardown = Some(Box::new(func));
        self
    }

    /// Set the default timeout for steps
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Run the scenario
    pub async fn run(self) -> Result<()> {
        info!("Starting scenario: {}", self.name);

        // Run setup if provided
        if let Some(setup) = self.setup {
            info!("Running setup for scenario: {}", self.name);
            setup().await?;
        }

        // Run each step
        for (i, step) in self.steps.into_iter().enumerate() {
            info!("Running step {} of {}: {}", i + 1, self.name, step.name);
            
            // Apply default timeout if none is specified
            let step = if step.timeout.is_none() {
                ScenarioStep {
                    name: step.name,
                    func: step.func,
                    timeout: Some(self.default_timeout),
                }
            } else {
                step
            };
            
            // Execute the step
            match step.execute().await {
                Ok(_) => {
                    info!("Step {} completed successfully", step.name);
                }
                Err(e) => {
                    error!("Step {} failed: {}", step.name, e);
                    
                    // Run teardown even if a step fails
                    if let Some(teardown) = self.teardown {
                        info!("Running teardown for scenario: {}", self.name);
                        let _ = teardown().await;
                    }
                    
                    return Err(e);
                }
            }
        }

        // Run teardown if provided
        if let Some(teardown) = self.teardown {
            info!("Running teardown for scenario: {}", self.name);
            teardown().await?;
        }

        info!("Scenario {} completed successfully", self.name);
        Ok(())
    }
}

/// Builder for actor test scenarios
pub struct ActorScenarioBuilder<A: Actor + 'static> {
    /// Test harness
    pub harness: ActorTestHarness<A>,
    /// Scenario being built
    pub scenario: TestScenario,
}

impl<A: Actor + 'static> ActorScenarioBuilder<A> {
    /// Create a new actor scenario builder
    pub fn new(name: impl Into<String>, harness: ActorTestHarness<A>) -> Self {
        Self {
            harness,
            scenario: TestScenario::new(name),
        }
    }

    /// Add a step to the scenario
    pub fn add_step<F>(mut self, name: impl Into<String>, func: F) -> Self
    where
        F: FnOnce(&ActorTestHarness<A>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        let harness = self.harness.clone();
        let step_func = move || {
            let harness_ref = &harness;
            Box::pin(async move {
                func(harness_ref).await
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        };
        
        self.scenario = self.scenario.add_step(name, step_func);
        self
    }

    /// Add a step with a timeout to the scenario
    pub fn add_step_with_timeout<F>(mut self, name: impl Into<String>, timeout: Duration, func: F) -> Self
    where
        F: FnOnce(&ActorTestHarness<A>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + 'static,
    {
        let harness = self.harness.clone();
        let step_func = move || {
            let harness_ref = &harness;
            Box::pin(async move {
                func(harness_ref).await
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        };
        
        self.scenario = self.scenario.add_step_with_timeout(name, timeout, step_func);
        self
    }

    /// Set the default timeout for steps
    pub fn with_default_timeout(mut self, timeout: Duration) -> Self {
        self.scenario = self.scenario.with_default_timeout(timeout);
        self
    }

    /// Build and run the scenario
    pub async fn run(mut self) -> Result<()> {
        // Add setup to initialize the harness
        let harness = self.harness.clone();
        self.scenario = self.scenario.with_setup(move || {
            let harness_ref = &harness;
            Box::pin(async move {
                // Setup code here if needed
                Ok(())
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        });
        
        // Add teardown to clean up the harness
        let harness = self.harness.clone();
        self.scenario = self.scenario.with_teardown(move || {
            let harness_ref = &harness;
            Box::pin(async move {
                harness_ref.cleanup().await
            }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        });
        
        // Run the scenario
        self.scenario.run().await
    }
}

/// Create a new actor scenario builder
pub fn create_actor_scenario<A: Actor + 'static>(
    name: impl Into<String>,
    actor: ActorRef<A>,
) -> ActorScenarioBuilder<A> {
    let harness = super::harness::create_actor_test_harness()
        .with_actor(actor);
    
    ActorScenarioBuilder::new(name, harness)
}

/// Create a new supervised actor scenario builder
pub fn create_supervised_scenario<A: Actor + Clone + 'static>(
    name: impl Into<String>,
    actor: ActorRef<A>,
    strategy: crate::actors::supervision::SupervisionStrategy,
) -> Result<ActorScenarioBuilder<A>> {
    let mut harness = super::harness::create_supervised_test_harness()
        .with_actor(actor)
        .with_strategy(strategy);
    
    // Setup supervision
    harness.setup_supervision().await?;
    
    // Get the base harness with the supervised actor
    let base_harness = harness.base;
    
    Ok(ActorScenarioBuilder::new(name, base_harness))
}

/// Create a new lifecycle actor scenario builder
pub fn create_lifecycle_scenario<A: Actor + crate::actors::lifecycle::LifecycleAware + 'static>(
    name: impl Into<String>,
    actor: ActorRef<A>,
) -> Result<ActorScenarioBuilder<A>> {
    let mut harness = super::harness::create_lifecycle_test_harness()
        .with_actor(actor);
    
    // Setup lifecycle management
    harness.setup_lifecycle().await?;
    
    // Get the base harness with the managed actor
    let base_harness = harness.base;
    
    Ok(ActorScenarioBuilder::new(name, base_harness))
}