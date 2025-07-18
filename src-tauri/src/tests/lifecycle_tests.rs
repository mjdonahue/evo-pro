use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use tokio::time::sleep;

use crate::actors::lifecycle::{
    ActorLifecycleState, ActorHealthStatus, LifecycleAware, LifecycleManagerActor,
    create_lifecycle_manager,
};
use crate::actors::lifecycle_utils::{
    ActorInitOptions, EnhancedLifecycleAware, EnhancedLifecycleExt,
    ActorDependencyManager, ActorStateManager, ActorLifecycleCoordinator,
    create_actor_dependency_manager, create_actor_state_manager, create_actor_lifecycle_coordinator,
};
use crate::error::Result;

// Test actor that implements EnhancedLifecycleAware
#[derive(Actor, Clone)]
struct TestLifecycleActor {
    name: String,
    state: String,
    initialized: bool,
    shutdown_requested: bool,
    init_count: Arc<AtomicUsize>,
    shutdown_count: Arc<AtomicUsize>,
    save_count: Arc<AtomicUsize>,
    restore_count: Arc<AtomicUsize>,
    health_status: ActorHealthStatus,
}

impl TestLifecycleActor {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: "initial".to_string(),
            initialized: false,
            shutdown_requested: false,
            init_count: Arc::new(AtomicUsize::new(0)),
            shutdown_count: Arc::new(AtomicUsize::new(0)),
            save_count: Arc::new(AtomicUsize::new(0)),
            restore_count: Arc::new(AtomicUsize::new(0)),
            health_status: ActorHealthStatus::Healthy,
        }
    }
    
    fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = state.into();
        self
    }
    
    fn with_health_status(mut self, status: ActorHealthStatus) -> Self {
        self.health_status = status;
        self
    }
}

impl LifecycleAware for TestLifecycleActor {
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Default implementation
        let actor_id = ctx.actor_ref().id();
        tracing::debug!("Actor {} starting", actor_id);
    }
    
    fn on_stop(&mut self, ctx: &mut Context<Self, ()>, reason: ActorStopReason) {
        // Default implementation
        let actor_id = ctx.actor_ref().id();
        tracing::debug!("Actor {} stopping: {:?}", actor_id, reason);
    }
    
    fn health_status(&self) -> ActorHealthStatus {
        self.health_status
    }
    
    fn check_health(&mut self, _ctx: &mut Context<Self, ()>) -> ActorHealthStatus {
        self.health_status
    }
}

impl EnhancedLifecycleAware for TestLifecycleActor {
    async fn initialize(&mut self, _ctx: &mut Context<Self, ()>, options: ActorInitOptions) -> Result<()> {
        self.init_count.fetch_add(1, Ordering::SeqCst);
        self.initialized = true;
        
        // Apply initial state if provided
        if let Some(state_bytes) = options.initial_state {
            if let Ok(state_str) = String::from_utf8(state_bytes) {
                self.state = state_str;
            }
        }
        
        Ok(())
    }
    
    async fn save_state(&self) -> Result<Vec<u8>> {
        self.save_count.fetch_add(1, Ordering::SeqCst);
        Ok(self.state.as_bytes().to_vec())
    }
    
    async fn restore_state(&mut self, state: &[u8]) -> Result<()> {
        self.restore_count.fetch_add(1, Ordering::SeqCst);
        if let Ok(state_str) = String::from_utf8(state.to_vec()) {
            self.state = state_str;
        }
        Ok(())
    }
    
    async fn shutdown(&mut self, _ctx: &mut Context<Self, ()>) -> Result<()> {
        self.shutdown_count.fetch_add(1, Ordering::SeqCst);
        self.shutdown_requested = true;
        Ok(())
    }
}

// Message to get actor state
#[derive(Debug, Clone)]
struct GetActorState;

impl Message<GetActorState> for TestLifecycleActor {
    type Reply = String;
    
    async fn handle(
        &mut self,
        _msg: GetActorState,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.state.clone()
    }
}

// Message to check if actor is initialized
#[derive(Debug, Clone)]
struct IsInitialized;

impl Message<IsInitialized> for TestLifecycleActor {
    type Reply = bool;
    
    async fn handle(
        &mut self,
        _msg: IsInitialized,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.initialized
    }
}

// Message to check if shutdown was requested
#[derive(Debug, Clone)]
struct WasShutdownRequested;

impl Message<WasShutdownRequested> for TestLifecycleActor {
    type Reply = bool;
    
    async fn handle(
        &mut self,
        _msg: WasShutdownRequested,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.shutdown_requested
    }
}

// Message to get counters
#[derive(Debug, Clone)]
struct GetCounters;

impl Message<GetCounters> for TestLifecycleActor {
    type Reply = (usize, usize, usize, usize);
    
    async fn handle(
        &mut self,
        _msg: GetCounters,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        (
            self.init_count.load(Ordering::SeqCst),
            self.shutdown_count.load(Ordering::SeqCst),
            self.save_count.load(Ordering::SeqCst),
            self.restore_count.load(Ordering::SeqCst),
        )
    }
}

// Test dependency actor
#[derive(Actor, Clone)]
struct TestDependencyActor {
    name: String,
    ready: Arc<AtomicBool>,
}

impl TestDependencyActor {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ready: Arc::new(AtomicBool::new(false)),
        }
    }
    
    fn mark_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
    }
}

impl LifecycleAware for TestDependencyActor {}
impl EnhancedLifecycleAware for TestDependencyActor {}

// Message to check if dependency is ready
#[derive(Debug, Clone)]
struct IsReady;

impl Message<IsReady> for TestDependencyActor {
    type Reply = bool;
    
    async fn handle(
        &mut self,
        _msg: IsReady,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.ready.load(Ordering::SeqCst)
    }
}

// Message to mark dependency as ready
#[derive(Debug, Clone)]
struct MarkReady;

impl Message<MarkReady> for TestDependencyActor {
    type Reply = ();
    
    async fn handle(
        &mut self,
        _msg: MarkReady,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.mark_ready();
    }
}

#[tokio::test]
async fn test_actor_initialization() -> Result<()> {
    // Create lifecycle components
    let lifecycle_manager = create_lifecycle_manager(Duration::from_secs(1));
    let dependency_manager = create_actor_dependency_manager(lifecycle_manager.clone());
    let state_manager = create_actor_state_manager();
    let coordinator = create_actor_lifecycle_coordinator(
        dependency_manager.clone(),
        state_manager.clone(),
        lifecycle_manager.clone(),
        None,
    );
    
    // Create a test actor
    let test_actor = TestLifecycleActor::new("test-lifecycle-actor");
    let actor_ref = TestLifecycleActor::spawn(test_actor);
    
    // Initialize the actor
    let options = ActorInitOptions {
        initial_state: Some("initialized".as_bytes().to_vec()),
        ..Default::default()
    };
    
    let managed_ref = actor_ref.with_enhanced_lifecycle(&coordinator, options).await?;
    
    // Check if the actor was initialized
    let initialized = managed_ref.ask(&IsInitialized).await;
    assert!(initialized, "Actor should be initialized");
    
    // Check the actor's state
    let state = managed_ref.ask(&GetActorState).await;
    assert_eq!(state, "initialized", "Actor state should be 'initialized'");
    
    // Check counters
    let (init_count, _, _, _) = managed_ref.ask(&GetCounters).await;
    assert_eq!(init_count, 1, "Actor should have been initialized once");
    
    Ok(())
}

#[tokio::test]
async fn test_actor_dependencies() -> Result<()> {
    // Create lifecycle components
    let lifecycle_manager = create_lifecycle_manager(Duration::from_secs(1));
    let dependency_manager = create_actor_dependency_manager(lifecycle_manager.clone());
    let state_manager = create_actor_state_manager();
    let coordinator = create_actor_lifecycle_coordinator(
        dependency_manager.clone(),
        state_manager.clone(),
        lifecycle_manager.clone(),
        None,
    );
    
    // Create dependency actors
    let dep1 = TestDependencyActor::new("dependency-1");
    let dep2 = TestDependencyActor::new("dependency-2");
    let dep1_ref = TestDependencyActor::spawn(dep1);
    let dep2_ref = TestDependencyActor::spawn(dep2);
    
    // Start dependency actors with lifecycle management
    let dep1_ref = dep1_ref.with_enhanced_lifecycle(&coordinator, ActorInitOptions::default()).await?;
    let dep2_ref = dep2_ref.with_enhanced_lifecycle(&coordinator, ActorInitOptions::default()).await?;
    
    // Create a test actor with dependencies
    let test_actor = TestLifecycleActor::new("test-with-dependencies");
    let actor_ref = TestLifecycleActor::spawn(test_actor);
    
    // Create a flag to track when the actor is started
    let actor_started = Arc::new(AtomicBool::new(false));
    let actor_started_clone = actor_started.clone();
    
    // Start the actor in a separate task
    let coordinator_clone = coordinator.clone();
    let actor_ref_clone = actor_ref.clone();
    tokio::spawn(async move {
        // Initialize the actor with dependencies
        let options = ActorInitOptions {
            dependencies: vec![dep1_ref.id(), dep2_ref.id()],
            ..Default::default()
        };
        
        let _ = actor_ref_clone.with_enhanced_lifecycle(&coordinator_clone, options).await;
        actor_started_clone.store(true, Ordering::SeqCst);
    });
    
    // Wait a bit and check if the actor has started
    sleep(Duration::from_millis(100)).await;
    assert!(!actor_started.load(Ordering::SeqCst), "Actor should not start until dependencies are ready");
    
    // Mark the first dependency as ready
    dep1_ref.tell(&MarkReady).await?;
    
    // Wait a bit and check again
    sleep(Duration::from_millis(100)).await;
    assert!(!actor_started.load(Ordering::SeqCst), "Actor should not start until all dependencies are ready");
    
    // Mark the second dependency as ready
    dep2_ref.tell(&MarkReady).await?;
    
    // Wait for the actor to start
    for _ in 0..10 {
        if actor_started.load(Ordering::SeqCst) {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    
    assert!(actor_started.load(Ordering::SeqCst), "Actor should start after all dependencies are ready");
    
    Ok(())
}

#[tokio::test]
async fn test_actor_state_persistence() -> Result<()> {
    // Create lifecycle components
    let lifecycle_manager = create_lifecycle_manager(Duration::from_secs(1));
    let dependency_manager = create_actor_dependency_manager(lifecycle_manager.clone());
    let state_manager = create_actor_state_manager();
    let coordinator = create_actor_lifecycle_coordinator(
        dependency_manager.clone(),
        state_manager.clone(),
        lifecycle_manager.clone(),
        None,
    );
    
    // Create a test actor with initial state
    let test_actor = TestLifecycleActor::new("test-state-actor").with_state("initial-state");
    let actor_ref = TestLifecycleActor::spawn(test_actor);
    
    // Start the actor with lifecycle management
    let managed_ref = actor_ref.with_enhanced_lifecycle(&coordinator, ActorInitOptions::default()).await?;
    
    // Save the actor's state
    managed_ref.save_state_enhanced(&coordinator).await?;
    
    // Check counters
    let (_, _, save_count, _) = managed_ref.ask(&GetCounters).await;
    assert_eq!(save_count, 1, "Actor state should have been saved once");
    
    // Create a new actor and recover its state
    let new_actor = TestLifecycleActor::new("test-state-actor-2");
    let new_actor_ref = TestLifecycleActor::spawn(new_actor);
    
    // Start the new actor with state recovery
    let options = ActorInitOptions {
        recover_state: true,
        ..Default::default()
    };
    
    let new_managed_ref = new_actor_ref.with_enhanced_lifecycle(&coordinator, options).await?;
    
    // Check counters
    let (_, _, _, restore_count) = new_managed_ref.ask(&GetCounters).await;
    assert_eq!(restore_count, 1, "Actor state should have been restored once");
    
    // Check the actor's state
    let state = new_managed_ref.ask(&GetActorState).await;
    assert_eq!(state, "initial-state", "Actor state should be recovered");
    
    Ok(())
}

#[tokio::test]
async fn test_actor_graceful_shutdown() -> Result<()> {
    // Create lifecycle components
    let lifecycle_manager = create_lifecycle_manager(Duration::from_secs(1));
    let dependency_manager = create_actor_dependency_manager(lifecycle_manager.clone());
    let state_manager = create_actor_state_manager();
    let coordinator = create_actor_lifecycle_coordinator(
        dependency_manager.clone(),
        state_manager.clone(),
        lifecycle_manager.clone(),
        None,
    );
    
    // Create a test actor
    let test_actor = TestLifecycleActor::new("test-shutdown-actor");
    let actor_ref = TestLifecycleActor::spawn(test_actor);
    
    // Start the actor with lifecycle management
    let managed_ref = actor_ref.with_enhanced_lifecycle(&coordinator, ActorInitOptions::default()).await?;
    
    // Stop the actor gracefully
    managed_ref.stop_gracefully_enhanced(&coordinator, None).await?;
    
    // Check if shutdown was requested
    let shutdown_requested = managed_ref.ask(&WasShutdownRequested).await;
    assert!(shutdown_requested, "Actor shutdown should have been requested");
    
    // Check counters
    let (_, shutdown_count, _, _) = managed_ref.ask(&GetCounters).await;
    assert_eq!(shutdown_count, 1, "Actor shutdown should have been called once");
    
    Ok(())
}

#[tokio::test]
async fn test_actor_health_check() -> Result<()> {
    // Create lifecycle components
    let lifecycle_manager = create_lifecycle_manager(Duration::from_secs(1));
    let dependency_manager = create_actor_dependency_manager(lifecycle_manager.clone());
    let state_manager = create_actor_state_manager();
    let coordinator = create_actor_lifecycle_coordinator(
        dependency_manager.clone(),
        state_manager.clone(),
        lifecycle_manager.clone(),
        None,
    );
    
    // Create a test actor with degraded health
    let test_actor = TestLifecycleActor::new("test-health-actor")
        .with_health_status(ActorHealthStatus::Degraded);
    let actor_ref = TestLifecycleActor::spawn(test_actor);
    
    // Start the actor with lifecycle management
    let managed_ref = actor_ref.with_enhanced_lifecycle(
        &coordinator,
        ActorInitOptions {
            health_check_interval: Some(Duration::from_millis(100)),
            ..Default::default()
        },
    ).await?;
    
    // Wait for health check to run
    sleep(Duration::from_millis(200)).await;
    
    // TODO: Verify that health check was performed
    // This would require subscribing to lifecycle events
    
    Ok(())
}