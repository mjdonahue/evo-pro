use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::logging;

/// Supervision strategy for handling actor failures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionStrategy {
    /// Stop the actor and don't restart it
    Stop,
    /// Restart the actor immediately
    Restart,
    /// Restart the actor with a delay
    RestartWithDelay(Duration),
    /// Escalate the failure to the parent supervisor
    Escalate,
    /// Apply different strategies based on failure type
    FailureSpecific {
        /// Strategy for panics
        panic: Box<SupervisionStrategy>,
        /// Strategy for peer disconnections
        peer_disconnected: Box<SupervisionStrategy>,
        /// Strategy for killed actors
        killed: Box<SupervisionStrategy>,
    },
}

/// Default supervision strategy
impl Default for SupervisionStrategy {
    fn default() -> Self {
        SupervisionStrategy::Restart
    }
}

/// Supervisor actor that manages the lifecycle of child actors
#[derive(Actor)]
pub struct SupervisorActor<A: Actor + Clone + 'static> {
    /// Name of the supervisor for logging
    pub name: String,
    /// Child actors being supervised
    pub children: HashMap<ActorID, SupervisedActor<A>>,
    /// Default supervision strategy
    pub default_strategy: SupervisionStrategy,
    /// Maximum number of restarts allowed in a time window
    pub max_restarts: usize,
    /// Time window for counting restarts
    pub restart_window: Duration,
    /// Custom strategy function
    pub strategy_fn: Option<Arc<dyn Fn(&A, &ActorStopReason) -> SupervisionStrategy + Send + Sync>>,
}

/// Information about a supervised actor
pub struct SupervisedActor<A: Actor + 'static> {
    /// Actor reference
    pub actor_ref: ActorRef<A>,
    /// Actor state for restarts
    pub actor_state: A,
    /// Restart history
    pub restart_history: Vec<std::time::Instant>,
    /// Custom supervision strategy for this actor
    pub strategy: Option<SupervisionStrategy>,
}

impl<A: Actor + Clone + 'static> SupervisorActor<A> {
    /// Create a new supervisor actor
    pub fn new(name: impl Into<String>, default_strategy: SupervisionStrategy) -> Self {
        Self {
            name: name.into(),
            children: HashMap::new(),
            default_strategy,
            max_restarts: 10,
            restart_window: Duration::from_secs(60),
            strategy_fn: None,
        }
    }

    /// Set the maximum number of restarts allowed in a time window
    pub fn with_max_restarts(mut self, max_restarts: usize, window: Duration) -> Self {
        self.max_restarts = max_restarts;
        self.restart_window = window;
        self
    }

    /// Set a custom strategy function
    pub fn with_strategy_fn(
        mut self,
        f: impl Fn(&A, &ActorStopReason) -> SupervisionStrategy + Send + Sync + 'static,
    ) -> Self {
        self.strategy_fn = Some(Arc::new(f));
        self
    }

    /// Supervise a new actor
    pub async fn supervise(
        &mut self,
        ctx: &mut Context<Self, ()>,
        actor: A,
        strategy: Option<SupervisionStrategy>,
    ) -> Result<ActorRef<A>> {
        // Spawn the actor
        let actor_ref = A::spawn(actor.clone());

        // Link the actor to the supervisor
        ctx.actor_ref().link(&actor_ref).await?;

        // Store the actor in the children map
        let actor_id = actor_ref.id();
        self.children.insert(
            actor_id,
            SupervisedActor {
                actor_ref: actor_ref.clone(),
                actor_state: actor,
                restart_history: Vec::new(),
                strategy,
            },
        );

        info!(
            supervisor = %self.name,
            actor_id = %actor_id,
            "Actor is now supervised"
        );

        Ok(actor_ref)
    }

    /// Handle the death of a linked actor
    fn handle_actor_death(
        &mut self,
        ctx: &mut Context<Self, ()>,
        actor_id: ActorID,
        reason: ActorStopReason,
    ) {
        // Check if this is one of our supervised children
        if let Some(child) = self.children.get_mut(&actor_id) {
            // Log the actor death
            let log_context = logging::LogContext::new()
                .with_operation("actor_supervision")
                .with_entity_type("actor")
                .with_entity_id(actor_id.to_string())
                .with_context("supervisor", &self.name)
                .with_context("reason", format!("{:?}", reason));

            match reason {
                ActorStopReason::Normal => {
                    info_with_context(
                        &format!("Actor {} stopped normally", actor_id),
                        &log_context
                    );
                    // Remove the actor from our children
                    self.children.remove(&actor_id);
                    return;
                }
                ActorStopReason::Panic(ref err) => {
                    error_with_context(
                        &format!("Actor {} panicked: {}", actor_id, err),
                        &log_context
                    );
                }
                ActorStopReason::PeerDisconnected => {
                    warn_with_context(
                        &format!("Actor {} stopped due to peer disconnection", actor_id),
                        &log_context
                    );
                }
                ActorStopReason::Killed => {
                    warn_with_context(
                        &format!("Actor {} was killed", actor_id),
                        &log_context
                    );
                }
            }

            // Determine the supervision strategy
            let strategy = if let Some(strategy) = child.strategy {
                strategy
            } else if let Some(ref strategy_fn) = self.strategy_fn {
                strategy_fn(&child.actor_state, &reason)
            } else {
                self.default_strategy
            };

            // Apply the supervision strategy
            match strategy {
                SupervisionStrategy::Stop => {
                    info_with_context(
                        &format!("Stopping actor {} without restart", actor_id),
                        &log_context
                    );
                    // Remove the actor from our children
                    self.children.remove(&actor_id);
                }
                SupervisionStrategy::Restart => {
                    // Check restart limits
                    child.restart_history.push(std::time::Instant::now());
                    // Remove restarts outside the window
                    child.restart_history.retain(|t| {
                        t.elapsed() < self.restart_window
                    });

                    if child.restart_history.len() > self.max_restarts {
                        error_with_context(
                            &format!(
                                "Actor {} exceeded maximum restarts ({} in {:?}), stopping",
                                actor_id, self.max_restarts, self.restart_window
                            ),
                            &log_context
                        );
                        // Remove the actor from our children
                        self.children.remove(&actor_id);
                        return;
                    }

                    // Restart the actor
                    info_with_context(
                        &format!("Restarting actor {}", actor_id),
                        &log_context
                    );

                    let actor_state = child.actor_state.clone();
                    let actor_ref = A::spawn(actor_state.clone());

                    // Update the child entry
                    child.actor_ref = actor_ref.clone();

                    // Link the new actor
                    let supervisor_ref = ctx.actor_ref();
                    tokio::spawn(async move {
                        if let Err(e) = supervisor_ref.link(&actor_ref).await {
                            error!("Failed to link restarted actor: {}", e);
                        }
                    });
                }
                SupervisionStrategy::RestartWithDelay(delay) => {
                    // Check restart limits
                    child.restart_history.push(std::time::Instant::now());
                    // Remove restarts outside the window
                    child.restart_history.retain(|t| {
                        t.elapsed() < self.restart_window
                    });

                    if child.restart_history.len() > self.max_restarts {
                        error_with_context(
                            &format!(
                                "Actor {} exceeded maximum restarts ({} in {:?}), stopping",
                                actor_id, self.max_restarts, self.restart_window
                            ),
                            &log_context
                        );
                        // Remove the actor from our children
                        self.children.remove(&actor_id);
                        return;
                    }

                    // Restart the actor with delay
                    info_with_context(
                        &format!("Restarting actor {} after delay of {:?}", actor_id, delay),
                        &log_context
                    );

                    let actor_state = child.actor_state.clone();
                    let supervisor_ref = ctx.actor_ref();
                    let actor_id = actor_id;

                    // Spawn a task to restart the actor after the delay
                    tokio::spawn(async move {
                        tokio::time::sleep(delay).await;

                        // Send a message to the supervisor to restart the actor
                        if let Err(e) = supervisor_ref.tell(&RestartActor { actor_id }).await {
                            error!("Failed to send restart message: {}", e);
                        }
                    });
                }
                SupervisionStrategy::Escalate => {
                    warn_with_context(
                        &format!("Escalating failure of actor {}", actor_id),
                        &log_context
                    );

                    // Remove the actor from our children
                    self.children.remove(&actor_id);

                    // Escalate by panicking with a special error that can be caught by parent supervisors
                    // This will trigger the on_link_died handler in the parent
                    let escalation_error = format!(
                        "SUPERVISOR_ESCALATION: Actor {} failed with reason {:?} in supervisor {}",
                        actor_id, reason, self.name
                    );

                    // Spawn a task to panic after a short delay to allow this method to complete
                    let ctx_ref = ctx.actor_ref();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        ctx_ref.kill().await.expect("Failed to kill supervisor for escalation");
                    });
                }
                SupervisionStrategy::FailureSpecific { panic, peer_disconnected, killed } => {
                    // Apply different strategies based on the failure type
                    let specific_strategy = match reason {
                        ActorStopReason::Panic(_) => *panic,
                        ActorStopReason::PeerDisconnected => *peer_disconnected,
                        ActorStopReason::Killed => *killed,
                        ActorStopReason::Normal => {
                            // This shouldn't happen as we handle normal stops earlier
                            SupervisionStrategy::Stop
                        }
                    };

                    info_with_context(
                        &format!(
                            "Applying failure-specific strategy for actor {} based on reason {:?}",
                            actor_id, reason
                        ),
                        &log_context
                    );

                    // Recursively apply the specific strategy
                    // We need to temporarily remove the child from our map to avoid
                    // potential infinite recursion with certain strategies
                    let child_data = self.children.remove(&actor_id).unwrap();

                    // Apply the specific strategy by calling ourselves recursively
                    // with the child data reinserted
                    self.children.insert(actor_id, child_data);

                    // Override the strategy for this specific call
                    if let Some(child) = self.children.get_mut(&actor_id) {
                        child.strategy = Some(specific_strategy);
                    }

                    // Call handle_actor_death again with the same parameters
                    // The strategy will now be the specific one for this failure type
                    self.handle_actor_death(ctx, actor_id, reason);
                }
            }
        }
    }
}

/// Message to restart an actor
#[derive(Debug, Clone)]
pub struct RestartActor {
    pub actor_id: ActorID,
}

impl<A: Actor + Clone + 'static> Message<RestartActor> for SupervisorActor<A> {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: RestartActor,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Some(child) = self.children.get_mut(&msg.actor_id) {
            // Restart the actor
            let actor_state = child.actor_state.clone();
            let actor_ref = A::spawn(actor_state.clone());

            // Update the child entry
            child.actor_ref = actor_ref.clone();

            // Link the new actor
            ctx.actor_ref().link(&actor_ref).await?;

            info!(
                supervisor = %self.name,
                actor_id = %msg.actor_id,
                "Actor restarted successfully"
            );
        } else {
            return Err(AppError::NotFoundError(format!(
                "Actor with ID {} not found in supervisor {}",
                msg.actor_id, self.name
            )));
        }

        Ok(())
    }
}

impl<A: Actor + Clone + 'static> Actor for SupervisorActor<A> {
    fn on_link_died(&mut self, actor_id: ActorID, reason: ActorStopReason, ctx: &mut Context<Self, ()>) {
        self.handle_actor_death(ctx, actor_id, reason);
    }
}

/// Extension trait for ActorRef to add supervision capabilities
pub trait SupervisionExt<A: Actor + Clone + 'static> {
    /// Create a supervisor for this actor type
    fn create_supervisor(
        name: impl Into<String>,
        strategy: SupervisionStrategy,
    ) -> ActorRef<SupervisorActor<A>>;

    /// Supervise this actor with the given supervisor
    async fn with_supervisor(
        self,
        supervisor: &ActorRef<SupervisorActor<A>>,
        strategy: Option<SupervisionStrategy>,
    ) -> Result<Self>
    where
        Self: Sized;
}

impl<A: Actor + Clone + 'static> SupervisionExt<A> for ActorRef<A> {
    fn create_supervisor(
        name: impl Into<String>,
        strategy: SupervisionStrategy,
    ) -> ActorRef<SupervisorActor<A>> {
        SupervisorActor::spawn(SupervisorActor::new(name, strategy))
    }

    async fn with_supervisor(
        self,
        supervisor: &ActorRef<SupervisorActor<A>>,
        strategy: Option<SupervisionStrategy>,
    ) -> Result<Self> {
        // Get the actor state
        let actor_state = self.get_state().await?;

        // Supervise the actor
        let supervised_ref = supervisor
            .ask(&SuperviseActor {
                actor: actor_state,
                strategy,
            })
            .await?;

        Ok(supervised_ref)
    }
}

/// Message to supervise an actor
#[derive(Clone)]
pub struct SuperviseActor<A: Actor + Clone + 'static> {
    pub actor: A,
    pub strategy: Option<SupervisionStrategy>,
}

impl<A: Actor + Clone + 'static> Message<SuperviseActor<A>> for SupervisorActor<A> {
    type Reply = Result<ActorRef<A>>;

    async fn handle(
        &mut self,
        msg: SuperviseActor<A>,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.supervise(ctx, msg.actor, msg.strategy).await
    }
}

/// Helper function to create a logging context for supervision events
fn info_with_context(message: &str, context: &logging::LogContext) {
    logging::info_with_context(message, context);
}

/// Helper function to create a warning logging context for supervision events
fn warn_with_context(message: &str, context: &logging::LogContext) {
    logging::warn_with_context(message, context);
}

/// Helper function to create an error logging context for supervision events
fn error_with_context(message: &str, context: &logging::LogContext) {
    logging::error_with_context(message, context);
}

/// Create a supervisor with integrated fault detection
pub fn create_fault_tolerant_supervisor<A: Actor + Clone + 'static>(
    name: impl Into<String>,
    strategy: SupervisionStrategy,
    heartbeat_monitor: Option<&ActorRef<crate::actors::fault_detection::HeartbeatMonitorActor>>,
    circuit_breaker: Option<&ActorRef<crate::actors::fault_detection::CircuitBreakerActor>>,
    lifecycle_manager: Option<&ActorRef<crate::actors::lifecycle::LifecycleManagerActor>>,
) -> ActorRef<SupervisorActor<A>> {
    use crate::actors::fault_detection::{HeartbeatEvent, HeartbeatExt};
    use tokio::sync::mpsc;

    let name = name.into();
    let supervisor = SupervisorActor::spawn(
        SupervisorActor::new(name.clone(), strategy)
            .with_max_restarts(10, Duration::from_secs(60))
    );

    // If we have a heartbeat monitor, subscribe to heartbeat events
    if let Some(monitor) = heartbeat_monitor {
        let supervisor_ref = supervisor.clone();

        // Spawn a task to handle heartbeat events
        tokio::spawn(async move {
            // Subscribe to heartbeat events
            if let Ok(mut rx) = monitor.ask(&crate::actors::fault_detection::SubscribeToHeartbeatEvents).await {
                while let Some(event) = rx.recv().await {
                    match event {
                        HeartbeatEvent::ActorDead { actor_id } => {
                            // If an actor is considered dead, try to restart it
                            if let Err(e) = supervisor_ref.tell(&RestartActor { actor_id }).await {
                                error!("Failed to send restart message for dead actor: {}", e);
                            }
                        },
                        _ => {
                            // Ignore other heartbeat events
                        }
                    }
                }
            }
        });
    }

    // If we have a lifecycle manager, register the supervisor
    if let Some(lifecycle_manager) = lifecycle_manager {
        let supervisor_ref = supervisor.clone();

        // Spawn a task to register with lifecycle manager
        tokio::spawn(async move {
            if let Err(e) = lifecycle_manager
                .ask(&crate::actors::lifecycle::MonitorActorHeartbeat {
                    actor_ref: supervisor_ref.into_any(),
                })
                .await
            {
                error!("Failed to register supervisor with lifecycle manager: {}", e);
            }
        });
    }

    supervisor
}
