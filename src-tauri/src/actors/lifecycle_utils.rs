//! Utilities for actor lifecycle management
//!
//! This module provides utilities for managing the lifecycle of actors, including
//! initialization, dependency management, state persistence, and recovery.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::actors::lifecycle::{
    ActorLifecycleState, ActorHealthStatus, ActorLifecycleEvent,
    LifecycleAware, LifecycleManagerActor, LifecycleExt,
};
use crate::actors::supervision::{SupervisionStrategy, SupervisorActor, SupervisionExt};

/// Actor initialization options
#[derive(Debug, Clone)]
pub struct ActorInitOptions {
    /// Initial state for the actor
    pub initial_state: Option<Vec<u8>>,
    /// Dependencies that must be available before the actor can start
    pub dependencies: Vec<ActorID>,
    /// Timeout for initialization
    pub init_timeout: Duration,
    /// Whether to recover state from persistence
    pub recover_state: bool,
    /// Supervision strategy for the actor
    pub supervision_strategy: Option<SupervisionStrategy>,
    /// Health check interval
    pub health_check_interval: Option<Duration>,
}

impl Default for ActorInitOptions {
    fn default() -> Self {
        Self {
            initial_state: None,
            dependencies: Vec::new(),
            init_timeout: Duration::from_secs(30),
            recover_state: false,
            supervision_strategy: None,
            health_check_interval: None,
        }
    }
}

/// Actor dependency graph for managing actor initialization order
#[derive(Actor)]
pub struct ActorDependencyManager {
    /// Dependencies between actors
    dependencies: HashMap<ActorID, HashSet<ActorID>>,
    /// Actors that are ready to start
    ready_actors: HashSet<ActorID>,
    /// Actors that are waiting for dependencies
    waiting_actors: HashMap<ActorID, HashSet<ActorID>>,
    /// Callbacks to notify when actors are ready
    callbacks: HashMap<ActorID, Vec<oneshot::Sender<()>>>,
    /// Lifecycle manager
    lifecycle_manager: ActorRef<LifecycleManagerActor>,
}

impl ActorDependencyManager {
    /// Create a new actor dependency manager
    pub fn new(lifecycle_manager: ActorRef<LifecycleManagerActor>) -> Self {
        Self {
            dependencies: HashMap::new(),
            ready_actors: HashSet::new(),
            waiting_actors: HashMap::new(),
            callbacks: HashMap::new(),
            lifecycle_manager,
        }
    }

    /// Register an actor with its dependencies
    pub fn register_actor(&mut self, actor_id: ActorID, dependencies: Vec<ActorID>) {
        // Store the dependencies
        let deps = self.dependencies.entry(actor_id).or_insert_with(HashSet::new);
        deps.extend(dependencies.iter());
        
        // Check if all dependencies are ready
        let mut all_ready = true;
        let mut waiting_for = HashSet::new();
        
        for dep_id in dependencies {
            if !self.ready_actors.contains(&dep_id) {
                all_ready = false;
                waiting_for.insert(dep_id);
            }
        }
        
        if all_ready {
            // Actor is ready to start
            self.ready_actors.insert(actor_id);
            
            // Notify any actors waiting for this one
            self.notify_dependent_actors(actor_id);
        } else {
            // Actor is waiting for dependencies
            self.waiting_actors.insert(actor_id, waiting_for);
        }
    }

    /// Mark an actor as ready
    pub fn mark_actor_ready(&mut self, actor_id: ActorID) {
        // Add to ready actors
        self.ready_actors.insert(actor_id);
        
        // Remove from waiting actors
        self.waiting_actors.remove(&actor_id);
        
        // Notify any actors waiting for this one
        self.notify_dependent_actors(actor_id);
        
        // Notify any callbacks waiting for this actor
        if let Some(callbacks) = self.callbacks.remove(&actor_id) {
            for callback in callbacks {
                let _ = callback.send(());
            }
        }
    }

    /// Notify actors that depend on the given actor
    fn notify_dependent_actors(&mut self, actor_id: ActorID) {
        // Find actors that are waiting for this one
        let mut ready_actors = Vec::new();
        
        for (waiting_id, waiting_for) in &mut self.waiting_actors {
            waiting_for.remove(&actor_id);
            
            if waiting_for.is_empty() {
                // All dependencies are ready
                ready_actors.push(*waiting_id);
            }
        }
        
        // Mark actors as ready
        for ready_id in ready_actors {
            self.waiting_actors.remove(&ready_id);
            self.ready_actors.insert(ready_id);
            
            // Notify any callbacks waiting for this actor
            if let Some(callbacks) = self.callbacks.remove(&ready_id) {
                for callback in callbacks {
                    let _ = callback.send(());
                }
            }
        }
    }

    /// Wait for an actor to be ready
    pub async fn wait_for_actor(&mut self, actor_id: ActorID) -> Result<()> {
        if self.ready_actors.contains(&actor_id) {
            // Actor is already ready
            return Ok(());
        }
        
        // Create a channel to wait for the actor
        let (tx, rx) = oneshot::channel();
        
        // Add the callback
        let callbacks = self.callbacks.entry(actor_id).or_insert_with(Vec::new);
        callbacks.push(tx);
        
        // Wait for the actor to be ready
        rx.await.map_err(|_| {
            AppError::TimeoutError(format!("Timed out waiting for actor {}", actor_id))
        })?;
        
        Ok(())
    }
}

/// Message to register an actor with the dependency manager
#[derive(Debug, Clone)]
pub struct RegisterActorDependencies {
    pub actor_id: ActorID,
    pub dependencies: Vec<ActorID>,
}

impl Message<RegisterActorDependencies> for ActorDependencyManager {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RegisterActorDependencies,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.register_actor(msg.actor_id, msg.dependencies);
    }
}

/// Message to mark an actor as ready
#[derive(Debug, Clone)]
pub struct MarkActorReady {
    pub actor_id: ActorID,
}

impl Message<MarkActorReady> for ActorDependencyManager {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: MarkActorReady,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.mark_actor_ready(msg.actor_id);
    }
}

/// Message to wait for an actor to be ready
#[derive(Debug, Clone)]
pub struct WaitForActor {
    pub actor_id: ActorID,
}

impl Message<WaitForActor> for ActorDependencyManager {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: WaitForActor,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.wait_for_actor(msg.actor_id).await
    }
}

/// Actor state persistence manager
#[derive(Actor)]
pub struct ActorStateManager {
    /// Persisted actor states
    states: HashMap<ActorID, Vec<u8>>,
    /// State persistence handlers
    persistence_handlers: HashMap<String, Box<dyn Fn(ActorID, &[u8]) -> Result<()> + Send + Sync>>,
    /// State recovery handlers
    recovery_handlers: HashMap<String, Box<dyn Fn(ActorID) -> Result<Vec<u8>> + Send + Sync>>,
}

impl ActorStateManager {
    /// Create a new actor state manager
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            persistence_handlers: HashMap::new(),
            recovery_handlers: HashMap::new(),
        }
    }

    /// Register a persistence handler for a specific actor type
    pub fn register_persistence_handler<F>(&mut self, actor_type: impl Into<String>, handler: F)
    where
        F: Fn(ActorID, &[u8]) -> Result<()> + Send + Sync + 'static,
    {
        self.persistence_handlers.insert(actor_type.into(), Box::new(handler));
    }

    /// Register a recovery handler for a specific actor type
    pub fn register_recovery_handler<F>(&mut self, actor_type: impl Into<String>, handler: F)
    where
        F: Fn(ActorID) -> Result<Vec<u8>> + Send + Sync + 'static,
    {
        self.recovery_handlers.insert(actor_type.into(), Box::new(handler));
    }

    /// Persist actor state
    pub fn persist_state(&mut self, actor_id: ActorID, actor_type: &str, state: Vec<u8>) -> Result<()> {
        // Store the state in memory
        self.states.insert(actor_id, state.clone());
        
        // Use the persistence handler if available
        if let Some(handler) = self.persistence_handlers.get(actor_type) {
            handler(actor_id, &state)?;
        }
        
        Ok(())
    }

    /// Recover actor state
    pub fn recover_state(&self, actor_id: ActorID, actor_type: &str) -> Result<Option<Vec<u8>>> {
        // Check if we have the state in memory
        if let Some(state) = self.states.get(&actor_id) {
            return Ok(Some(state.clone()));
        }
        
        // Use the recovery handler if available
        if let Some(handler) = self.recovery_handlers.get(actor_type) {
            match handler(actor_id) {
                Ok(state) => return Ok(Some(state)),
                Err(e) => {
                    warn!("Failed to recover state for actor {}: {}", actor_id, e);
                    return Ok(None);
                }
            }
        }
        
        Ok(None)
    }
}

/// Message to persist actor state
#[derive(Debug, Clone)]
pub struct PersistActorState {
    pub actor_id: ActorID,
    pub actor_type: String,
    pub state: Vec<u8>,
}

impl Message<PersistActorState> for ActorStateManager {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: PersistActorState,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.persist_state(msg.actor_id, &msg.actor_type, msg.state)
    }
}

/// Message to recover actor state
#[derive(Debug, Clone)]
pub struct RecoverActorState {
    pub actor_id: ActorID,
    pub actor_type: String,
}

impl Message<RecoverActorState> for ActorStateManager {
    type Reply = Result<Option<Vec<u8>>>;

    async fn handle(
        &mut self,
        msg: RecoverActorState,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.recover_state(msg.actor_id, &msg.actor_type)
    }
}

/// Message to register a persistence handler
pub struct RegisterPersistenceHandler {
    pub actor_type: String,
    pub handler: Box<dyn Fn(ActorID, &[u8]) -> Result<()> + Send + Sync + 'static>,
}

impl Message<RegisterPersistenceHandler> for ActorStateManager {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RegisterPersistenceHandler,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.persistence_handlers.insert(msg.actor_type, msg.handler);
    }
}

/// Message to register a recovery handler
pub struct RegisterRecoveryHandler {
    pub actor_type: String,
    pub handler: Box<dyn Fn(ActorID) -> Result<Vec<u8>> + Send + Sync + 'static>,
}

impl Message<RegisterRecoveryHandler> for ActorStateManager {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RegisterRecoveryHandler,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.recovery_handlers.insert(msg.actor_type, msg.handler);
    }
}

/// Enhanced lifecycle-aware trait with initialization and state management
pub trait EnhancedLifecycleAware: LifecycleAware {
    /// Initialize the actor
    async fn initialize(&mut self, ctx: &mut Context<Self, ()>, options: ActorInitOptions) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Save the actor's state
    async fn save_state(&self) -> Result<Vec<u8>> {
        // Default implementation returns empty state
        Ok(Vec::new())
    }
    
    /// Restore the actor's state
    async fn restore_state(&mut self, state: &[u8]) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Perform a graceful shutdown
    async fn shutdown(&mut self, ctx: &mut Context<Self, ()>) -> Result<()> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Actor lifecycle coordinator that manages the complete lifecycle
#[derive(Actor)]
pub struct ActorLifecycleCoordinator {
    /// Dependency manager
    dependency_manager: ActorRef<ActorDependencyManager>,
    /// State manager
    state_manager: ActorRef<ActorStateManager>,
    /// Lifecycle manager
    lifecycle_manager: ActorRef<LifecycleManagerActor>,
    /// Supervisor
    supervisor: Option<ActorRef<SupervisorActor<dyn Any>>>,
    /// Actors being managed
    managed_actors: HashMap<ActorID, ManagedActorInfo>,
}

/// Information about a managed actor
struct ManagedActorInfo {
    /// Actor reference
    actor_ref: ActorRef<dyn Any>,
    /// Actor type
    actor_type: String,
    /// Initialization options
    init_options: ActorInitOptions,
    /// Current lifecycle state
    state: ActorLifecycleState,
}

impl ActorLifecycleCoordinator {
    /// Create a new actor lifecycle coordinator
    pub fn new(
        dependency_manager: ActorRef<ActorDependencyManager>,
        state_manager: ActorRef<ActorStateManager>,
        lifecycle_manager: ActorRef<LifecycleManagerActor>,
        supervisor: Option<ActorRef<SupervisorActor<dyn Any>>>,
    ) -> Self {
        Self {
            dependency_manager,
            state_manager,
            lifecycle_manager,
            supervisor,
            managed_actors: HashMap::new(),
        }
    }

    /// Start an actor with lifecycle management
    pub async fn start_actor<A: Actor + EnhancedLifecycleAware + 'static>(
        &mut self,
        actor: A,
        options: ActorInitOptions,
    ) -> Result<ActorRef<A>> {
        let actor_type = std::any::type_name::<A>().to_string();
        
        // Spawn the actor
        let actor_ref = A::spawn(actor);
        let actor_id = actor_ref.id();
        
        // Register with dependency manager
        self.dependency_manager.tell(&RegisterActorDependencies {
            actor_id,
            dependencies: options.dependencies.clone(),
        }).await?;
        
        // Store actor info
        self.managed_actors.insert(
            actor_id,
            ManagedActorInfo {
                actor_ref: actor_ref.clone().into_any(),
                actor_type: actor_type.clone(),
                init_options: options.clone(),
                state: ActorLifecycleState::Initializing,
            },
        );
        
        // Wait for dependencies to be ready
        for dep_id in &options.dependencies {
            self.dependency_manager.ask(&WaitForActor { actor_id: *dep_id }).await?;
        }
        
        // Recover state if needed
        if options.recover_state {
            if let Some(state) = self.state_manager.ask(&RecoverActorState {
                actor_id,
                actor_type: actor_type.clone(),
            }).await? {
                // Restore the actor's state
                actor_ref.tell(&RestoreActorState { state }).await?;
            }
        }
        
        // Initialize the actor
        actor_ref.tell(&InitializeActor { options: options.clone() }).await?;
        
        // Apply supervision if needed
        if let Some(strategy) = options.supervision_strategy {
            if let Some(ref supervisor) = self.supervisor {
                // TODO: This is a simplification, as we can't easily cast between different actor types
                // In a real implementation, we would need a more sophisticated approach
            }
        }
        
        // Register with lifecycle manager
        actor_ref.with_lifecycle_management(&self.lifecycle_manager, options.health_check_interval).await?;
        
        // Mark actor as ready
        self.dependency_manager.tell(&MarkActorReady { actor_id }).await?;
        
        // Update state
        if let Some(info) = self.managed_actors.get_mut(&actor_id) {
            info.state = ActorLifecycleState::Running;
        }
        
        Ok(actor_ref)
    }

    /// Stop an actor gracefully
    pub async fn stop_actor(&mut self, actor_id: ActorID, timeout: Option<Duration>) -> Result<()> {
        // Get actor info
        let info = self.managed_actors.get(&actor_id).ok_or_else(|| {
            AppError::NotFoundError(format!("Actor with ID {} not being managed", actor_id))
        })?;
        
        // Update state
        if let Some(info) = self.managed_actors.get_mut(&actor_id) {
            info.state = ActorLifecycleState::ShuttingDown;
        }
        
        // Send shutdown message
        info.actor_ref.tell(&ShutdownActor).await?;
        
        // Wait for shutdown to complete
        let timeout = timeout.unwrap_or(Duration::from_secs(30));
        tokio::time::timeout(timeout, async {
            // TODO: Wait for shutdown to complete
            // In a real implementation, we would wait for a signal from the actor
            tokio::time::sleep(Duration::from_millis(100)).await;
        }).await.map_err(|_| {
            AppError::TimeoutError(format!("Timed out waiting for actor {} to shut down", actor_id))
        })?;
        
        // Stop the actor with the lifecycle manager
        self.lifecycle_manager.ask(&crate::actors::lifecycle::StopActor {
            actor_id,
            timeout: Some(timeout),
        }).await?;
        
        // Remove from managed actors
        self.managed_actors.remove(&actor_id);
        
        Ok(())
    }

    /// Save actor state
    pub async fn save_actor_state(&self, actor_id: ActorID) -> Result<()> {
        // Get actor info
        let info = self.managed_actors.get(&actor_id).ok_or_else(|| {
            AppError::NotFoundError(format!("Actor with ID {} not being managed", actor_id))
        })?;
        
        // Request state from the actor
        let state = info.actor_ref.ask(&SaveActorState).await?;
        
        // Persist the state
        self.state_manager.ask(&PersistActorState {
            actor_id,
            actor_type: info.actor_type.clone(),
            state,
        }).await?;
        
        Ok(())
    }
}

/// Message to initialize an actor
#[derive(Debug, Clone)]
pub struct InitializeActor {
    pub options: ActorInitOptions,
}

impl<A: Actor + EnhancedLifecycleAware + 'static> Message<InitializeActor> for A {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: InitializeActor,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.initialize(ctx, msg.options).await
    }
}

/// Message to restore actor state
#[derive(Debug, Clone)]
pub struct RestoreActorState {
    pub state: Vec<u8>,
}

impl<A: Actor + EnhancedLifecycleAware + 'static> Message<RestoreActorState> for A {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: RestoreActorState,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.restore_state(&msg.state).await
    }
}

/// Message to save actor state
#[derive(Debug, Clone)]
pub struct SaveActorState;

impl<A: Actor + EnhancedLifecycleAware + 'static> Message<SaveActorState> for A {
    type Reply = Result<Vec<u8>>;

    async fn handle(
        &mut self,
        _msg: SaveActorState,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.save_state().await
    }
}

/// Message to shut down an actor gracefully
#[derive(Debug, Clone)]
pub struct ShutdownActor;

impl<A: Actor + EnhancedLifecycleAware + 'static> Message<ShutdownActor> for A {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        _msg: ShutdownActor,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.shutdown(ctx).await
    }
}

/// Message to start an actor with the coordinator
#[derive(Debug, Clone)]
pub struct StartActorWithCoordinator<A: Actor + EnhancedLifecycleAware + 'static> {
    pub actor: A,
    pub options: ActorInitOptions,
}

impl<A: Actor + EnhancedLifecycleAware + 'static> Message<StartActorWithCoordinator<A>> for ActorLifecycleCoordinator {
    type Reply = Result<ActorRef<A>>;

    async fn handle(
        &mut self,
        msg: StartActorWithCoordinator<A>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.start_actor(msg.actor, msg.options).await
    }
}

/// Message to stop an actor with the coordinator
#[derive(Debug, Clone)]
pub struct StopActorWithCoordinator {
    pub actor_id: ActorID,
    pub timeout: Option<Duration>,
}

impl Message<StopActorWithCoordinator> for ActorLifecycleCoordinator {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: StopActorWithCoordinator,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.stop_actor(msg.actor_id, msg.timeout).await
    }
}

/// Message to save actor state with the coordinator
#[derive(Debug, Clone)]
pub struct SaveActorStateWithCoordinator {
    pub actor_id: ActorID,
}

impl Message<SaveActorStateWithCoordinator> for ActorLifecycleCoordinator {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: SaveActorStateWithCoordinator,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.save_actor_state(msg.actor_id).await
    }
}

/// Create an actor dependency manager
pub fn create_actor_dependency_manager(
    lifecycle_manager: ActorRef<LifecycleManagerActor>,
) -> ActorRef<ActorDependencyManager> {
    ActorDependencyManager::spawn(ActorDependencyManager::new(lifecycle_manager))
}

/// Create an actor state manager
pub fn create_actor_state_manager() -> ActorRef<ActorStateManager> {
    ActorStateManager::spawn(ActorStateManager::new())
}

/// Create an actor lifecycle coordinator
pub fn create_actor_lifecycle_coordinator(
    dependency_manager: ActorRef<ActorDependencyManager>,
    state_manager: ActorRef<ActorStateManager>,
    lifecycle_manager: ActorRef<LifecycleManagerActor>,
    supervisor: Option<ActorRef<SupervisorActor<dyn Any>>>,
) -> ActorRef<ActorLifecycleCoordinator> {
    ActorLifecycleCoordinator::spawn(
        ActorLifecycleCoordinator::new(
            dependency_manager,
            state_manager,
            lifecycle_manager,
            supervisor,
        )
    )
}

/// Extension trait for ActorRef to add enhanced lifecycle management capabilities
pub trait EnhancedLifecycleExt<A: Actor + EnhancedLifecycleAware + 'static> {
    /// Start the actor with enhanced lifecycle management
    async fn with_enhanced_lifecycle(
        self,
        coordinator: &ActorRef<ActorLifecycleCoordinator>,
        options: ActorInitOptions,
    ) -> Result<Self>
    where
        Self: Sized;
    
    /// Stop the actor gracefully
    async fn stop_gracefully_enhanced(
        &self,
        coordinator: &ActorRef<ActorLifecycleCoordinator>,
        timeout: Option<Duration>,
    ) -> Result<()>;
    
    /// Save the actor's state
    async fn save_state_enhanced(
        &self,
        coordinator: &ActorRef<ActorLifecycleCoordinator>,
    ) -> Result<()>;
}

impl<A: Actor + EnhancedLifecycleAware + 'static> EnhancedLifecycleExt<A> for ActorRef<A> {
    async fn with_enhanced_lifecycle(
        self,
        coordinator: &ActorRef<ActorLifecycleCoordinator>,
        options: ActorInitOptions,
    ) -> Result<Self> {
        // Get the actor state
        let actor_state = self.get_state().await?;
        
        // Start with enhanced lifecycle management
        coordinator
            .ask(&StartActorWithCoordinator {
                actor: actor_state,
                options,
            })
            .await
    }
    
    async fn stop_gracefully_enhanced(
        &self,
        coordinator: &ActorRef<ActorLifecycleCoordinator>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        coordinator
            .ask(&StopActorWithCoordinator {
                actor_id: self.id(),
                timeout,
            })
            .await
    }
    
    async fn save_state_enhanced(
        &self,
        coordinator: &ActorRef<ActorLifecycleCoordinator>,
    ) -> Result<()> {
        coordinator
            .ask(&SaveActorStateWithCoordinator {
                actor_id: self.id(),
            })
            .await
    }
}