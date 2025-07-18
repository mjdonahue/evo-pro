//! Test harnesses for actor testing
//!
//! This module provides test harnesses for different types of actors and testing scenarios.
//! A test harness encapsulates the setup, execution, and teardown of a test, making it
//! easier to write consistent and reliable tests.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, Result};
use super::{TestEnv, TestEvent, TestEventType, TestConfig, run_with_timeout};
use super::mock::{MockActor, create_mock_actor_with_env, MockExt};

/// Base test harness for actor testing
pub struct ActorTestHarness<A: Actor + 'static> {
    /// Test environment
    pub test_env: Arc<TestEnv>,
    /// Actor under test
    pub actor: Option<ActorRef<A>>,
    /// Mock actors
    pub mocks: HashMap<String, ActorRef<MockActor>>,
    /// Test timeout
    pub timeout: Duration,
}

impl<A: Actor + 'static> ActorTestHarness<A> {
    /// Create a new actor test harness
    pub fn new(test_env: Arc<TestEnv>) -> Self {
        Self {
            test_env,
            actor: None,
            mocks: HashMap::new(),
            timeout: Duration::from_secs(5),
        }
    }

    /// Set the actor under test
    pub fn with_actor(mut self, actor: ActorRef<A>) -> Self {
        self.actor = Some(actor);
        self
    }

    /// Set the test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a mock actor
    pub fn with_mock(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        let mock = create_mock_actor_with_env(name.clone(), self.test_env.clone());
        self.test_env.register_mock(name.clone(), mock.clone());
        self.mocks.insert(name, mock);
        self
    }

    /// Get a mock actor by name
    pub fn get_mock(&self, name: &str) -> Option<&ActorRef<MockActor>> {
        self.mocks.get(name)
    }

    /// Run a test function with the harness
    pub async fn run<F, T>(&self, test_fn: F) -> Result<T>
    where
        F: FnOnce(&Self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send + 'static,
        T: Send + 'static,
    {
        run_with_timeout(self.timeout, async {
            test_fn(self).await
        }).await
    }

    /// Verify all mock expectations
    pub async fn verify_mocks(&self) -> Result<()> {
        for (name, mock) in &self.mocks {
            mock.verify().await?;
        }
        Ok(())
    }

    /// Clean up the harness
    pub async fn cleanup(&self) -> Result<()> {
        // Stop the actor under test
        if let Some(ref actor) = self.actor {
            let _ = actor.kill().await;
        }
        
        // Stop all mock actors
        for (_, mock) in &self.mocks {
            let _ = mock.kill().await;
        }
        
        // Reset the test environment
        self.test_env.reset();
        
        Ok(())
    }
}

/// Test harness for supervised actors
pub struct SupervisedActorTestHarness<A: Actor + Clone + 'static> {
    /// Base test harness
    pub base: ActorTestHarness<A>,
    /// Supervisor actor
    pub supervisor: Option<ActorRef<crate::actors::supervision::SupervisorActor<A>>>,
    /// Supervision strategy
    pub strategy: crate::actors::supervision::SupervisionStrategy,
}

impl<A: Actor + Clone + 'static> SupervisedActorTestHarness<A> {
    /// Create a new supervised actor test harness
    pub fn new(test_env: Arc<TestEnv>) -> Self {
        Self {
            base: ActorTestHarness::new(test_env),
            supervisor: None,
            strategy: crate::actors::supervision::SupervisionStrategy::Restart,
        }
    }

    /// Set the actor under test
    pub fn with_actor(mut self, actor: ActorRef<A>) -> Self {
        self.base = self.base.with_actor(actor);
        self
    }

    /// Set the test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.base = self.base.with_timeout(timeout);
        self
    }

    /// Add a mock actor
    pub fn with_mock(mut self, name: impl Into<String>) -> Self {
        self.base = self.base.with_mock(name);
        self
    }

    /// Set the supervisor
    pub fn with_supervisor(mut self, supervisor: ActorRef<crate::actors::supervision::SupervisorActor<A>>) -> Self {
        self.supervisor = Some(supervisor);
        self
    }

    /// Set the supervision strategy
    pub fn with_strategy(mut self, strategy: crate::actors::supervision::SupervisionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Create a supervisor and supervise the actor
    pub async fn setup_supervision(&mut self) -> Result<()> {
        if self.base.actor.is_none() {
            return Err(AppError::InvalidStateError("Actor not set".to_string()));
        }
        
        if self.supervisor.is_none() {
            // Create a supervisor
            let supervisor = crate::actors::supervision::SupervisorActor::spawn(
                crate::actors::supervision::SupervisorActor::new("test-supervisor", self.strategy)
            );
            
            self.supervisor = Some(supervisor);
        }
        
        // Supervise the actor
        let actor = self.base.actor.take().unwrap();
        let actor_state = actor.get_state().await?;
        
        let supervised_actor = self.supervisor.as_ref().unwrap()
            .ask(&crate::actors::supervision::SuperviseActor {
                actor: actor_state,
                strategy: Some(self.strategy),
            })
            .await?;
        
        self.base.actor = Some(supervised_actor);
        
        Ok(())
    }

    /// Run a test function with the harness
    pub async fn run<F, T>(&self, test_fn: F) -> Result<T>
    where
        F: FnOnce(&Self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send + 'static,
        T: Send + 'static,
    {
        run_with_timeout(self.base.timeout, async {
            test_fn(self).await
        }).await
    }

    /// Verify all mock expectations
    pub async fn verify_mocks(&self) -> Result<()> {
        self.base.verify_mocks().await
    }

    /// Clean up the harness
    pub async fn cleanup(&self) -> Result<()> {
        // Stop the supervisor
        if let Some(ref supervisor) = self.supervisor {
            let _ = supervisor.kill().await;
        }
        
        // Clean up the base harness
        self.base.cleanup().await
    }
}

/// Test harness for lifecycle-aware actors
pub struct LifecycleTestHarness<A: Actor + crate::actors::lifecycle::LifecycleAware + 'static> {
    /// Base test harness
    pub base: ActorTestHarness<A>,
    /// Lifecycle manager
    pub lifecycle_manager: Option<ActorRef<crate::actors::lifecycle::LifecycleManagerActor>>,
    /// Health check interval
    pub health_check_interval: Option<Duration>,
}

impl<A: Actor + crate::actors::lifecycle::LifecycleAware + 'static> LifecycleTestHarness<A> {
    /// Create a new lifecycle test harness
    pub fn new(test_env: Arc<TestEnv>) -> Self {
        Self {
            base: ActorTestHarness::new(test_env),
            lifecycle_manager: None,
            health_check_interval: None,
        }
    }

    /// Set the actor under test
    pub fn with_actor(mut self, actor: ActorRef<A>) -> Self {
        self.base = self.base.with_actor(actor);
        self
    }

    /// Set the test timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.base = self.base.with_timeout(timeout);
        self
    }

    /// Add a mock actor
    pub fn with_mock(mut self, name: impl Into<String>) -> Self {
        self.base = self.base.with_mock(name);
        self
    }

    /// Set the lifecycle manager
    pub fn with_lifecycle_manager(mut self, lifecycle_manager: ActorRef<crate::actors::lifecycle::LifecycleManagerActor>) -> Self {
        self.lifecycle_manager = Some(lifecycle_manager);
        self
    }

    /// Set the health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = Some(interval);
        self
    }

    /// Setup lifecycle management
    pub async fn setup_lifecycle(&mut self) -> Result<()> {
        if self.base.actor.is_none() {
            return Err(AppError::InvalidStateError("Actor not set".to_string()));
        }
        
        if self.lifecycle_manager.is_none() {
            // Create a lifecycle manager
            let lifecycle_manager = crate::actors::lifecycle::create_lifecycle_manager(
                self.health_check_interval.unwrap_or(Duration::from_secs(1))
            );
            
            self.lifecycle_manager = Some(lifecycle_manager);
        }
        
        // Register the actor with the lifecycle manager
        let actor = self.base.actor.take().unwrap();
        let managed_actor = actor.with_lifecycle_management(
            self.lifecycle_manager.as_ref().unwrap(),
            self.health_check_interval,
        ).await?;
        
        self.base.actor = Some(managed_actor);
        
        Ok(())
    }

    /// Run a test function with the harness
    pub async fn run<F, T>(&self, test_fn: F) -> Result<T>
    where
        F: FnOnce(&Self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>> + Send + 'static,
        T: Send + 'static,
    {
        run_with_timeout(self.base.timeout, async {
            test_fn(self).await
        }).await
    }

    /// Verify all mock expectations
    pub async fn verify_mocks(&self) -> Result<()> {
        self.base.verify_mocks().await
    }

    /// Clean up the harness
    pub async fn cleanup(&self) -> Result<()> {
        // Stop the lifecycle manager
        if let Some(ref lifecycle_manager) = self.lifecycle_manager {
            let _ = lifecycle_manager.kill().await;
        }
        
        // Clean up the base harness
        self.base.cleanup().await
    }
}

/// Create a basic actor test harness
pub fn create_actor_test_harness<A: Actor + 'static>() -> ActorTestHarness<A> {
    let test_env = super::create_test_env();
    ActorTestHarness::new(test_env)
}

/// Create a supervised actor test harness
pub fn create_supervised_test_harness<A: Actor + Clone + 'static>() -> SupervisedActorTestHarness<A> {
    let test_env = super::create_test_env();
    SupervisedActorTestHarness::new(test_env)
}

/// Create a lifecycle test harness
pub fn create_lifecycle_test_harness<A: Actor + crate::actors::lifecycle::LifecycleAware + 'static>() -> LifecycleTestHarness<A> {
    let test_env = super::create_test_env();
    LifecycleTestHarness::new(test_env)
}