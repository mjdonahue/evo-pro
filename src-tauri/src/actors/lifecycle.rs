use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kameo::prelude::*;
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::logging;

/// Actor lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorLifecycleState {
    /// Actor is initializing
    Initializing,
    /// Actor is running normally
    Running,
    /// Actor is shutting down
    ShuttingDown,
    /// Actor has stopped
    Stopped,
}

/// Actor health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorHealthStatus {
    /// Actor is healthy
    Healthy,
    /// Actor is degraded but still functioning
    Degraded,
    /// Actor is unhealthy and should be restarted
    Unhealthy,
}

/// Actor lifecycle events
#[derive(Debug, Clone)]
pub enum ActorLifecycleEvent {
    /// Actor has started
    Started { actor_id: ActorID },
    /// Actor is stopping
    Stopping { actor_id: ActorID },
    /// Actor has stopped
    Stopped { actor_id: ActorID, reason: ActorStopReason },
    /// Actor health status has changed
    HealthChanged { actor_id: ActorID, status: ActorHealthStatus },
}

/// Trait for actors that support lifecycle management
pub trait LifecycleAware: Actor {
    /// Called when the actor is starting
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Default implementation does nothing
        let actor_id = ctx.actor_ref().id();
        debug!("Actor {} starting", actor_id);
    }

    /// Called when the actor is stopping
    fn on_stop(&mut self, ctx: &mut Context<Self, ()>, reason: ActorStopReason) {
        // Default implementation does nothing
        let actor_id = ctx.actor_ref().id();
        debug!("Actor {} stopping: {:?}", actor_id, reason);
    }

    /// Get the current health status of the actor
    fn health_status(&self) -> ActorHealthStatus {
        // Default implementation returns healthy
        ActorHealthStatus::Healthy
    }

    /// Check the health of the actor
    fn check_health(&mut self, ctx: &mut Context<Self, ()>) -> ActorHealthStatus {
        // Default implementation just returns the current status
        self.health_status()
    }
}

/// Actor that manages lifecycle events for the system
#[derive(Actor)]
pub struct LifecycleManagerActor {
    /// Subscribers to lifecycle events
    subscribers: Vec<mpsc::Sender<ActorLifecycleEvent>>,
    /// Actors being monitored for health
    monitored_actors: HashMap<ActorID, MonitoredActor>,
    /// Health check interval
    health_check_interval: Duration,
}

/// Information about a monitored actor
struct MonitoredActor {
    /// Actor reference
    actor_ref: ActorRef<dyn Any>,
    /// Last known health status
    health_status: ActorHealthStatus,
    /// Last health check time
    last_check: Instant,
    /// Health check interval for this actor
    check_interval: Duration,
}

impl LifecycleManagerActor {
    /// Create a new lifecycle manager actor
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            monitored_actors: HashMap::new(),
            health_check_interval: Duration::from_secs(60), // Default to 60 seconds
        }
    }

    /// Set the health check interval
    pub fn with_health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Start the health check loop
    async fn start_health_check_loop(&self, ctx: &mut Context<Self, ()>) {
        let actor_ref = ctx.actor_ref();
        
        // Spawn a task to periodically check actor health
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(1));
            
            loop {
                check_interval.tick().await;
                
                // Send a message to check health
                if let Err(e) = actor_ref.tell(&CheckHealth).await {
                    error!("Failed to send health check message: {}", e);
                    break;
                }
            }
        });
    }

    /// Subscribe to lifecycle events
    pub async fn subscribe(&mut self) -> mpsc::Receiver<ActorLifecycleEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.push(tx);
        rx
    }

    /// Publish a lifecycle event to all subscribers
    async fn publish_event(&mut self, event: ActorLifecycleEvent) {
        // Remove closed channels
        self.subscribers.retain(|tx| !tx.is_closed());
        
        // Send the event to all subscribers
        for tx in &self.subscribers {
            if let Err(e) = tx.send(event.clone()).await {
                warn!("Failed to send lifecycle event: {}", e);
            }
        }
    }

    /// Monitor an actor for health
    pub async fn monitor_actor<A: Actor + LifecycleAware + 'static>(
        &mut self,
        actor_ref: ActorRef<A>,
        check_interval: Option<Duration>,
    ) -> Result<()> {
        let actor_id = actor_ref.id();
        
        // Link to the actor to receive death notifications
        let self_ref = actor_ref.ctx().registry().get_by_id(actor_id).unwrap();
        self_ref.link(&actor_ref).await?;
        
        // Add to monitored actors
        self.monitored_actors.insert(
            actor_id,
            MonitoredActor {
                actor_ref: actor_ref.into_any(),
                health_status: ActorHealthStatus::Healthy,
                last_check: Instant::now(),
                check_interval: check_interval.unwrap_or(self.health_check_interval),
            },
        );
        
        info!("Now monitoring actor {}", actor_id);
        
        Ok(())
    }

    /// Check the health of all monitored actors
    async fn check_health(&mut self, ctx: &mut Context<Self, ()>) {
        let now = Instant::now();
        let mut to_check = Vec::new();
        
        // Identify actors that need health checks
        for (actor_id, monitored) in &self.monitored_actors {
            if now.duration_since(monitored.last_check) >= monitored.check_interval {
                to_check.push(*actor_id);
            }
        }
        
        // Check each actor's health
        for actor_id in to_check {
            if let Some(monitored) = self.monitored_actors.get_mut(&actor_id) {
                // Update last check time
                monitored.last_check = now;
                
                // Send health check message
                // In a real implementation, we would use the actor's check_health method
                // For now, we'll just assume the actor is healthy
                let new_status = ActorHealthStatus::Healthy;
                
                // If health status changed, publish an event
                if new_status != monitored.health_status {
                    monitored.health_status = new_status;
                    
                    self.publish_event(ActorLifecycleEvent::HealthChanged {
                        actor_id,
                        status: new_status,
                    }).await;
                    
                    info!(
                        actor_id = %actor_id,
                        status = ?new_status,
                        "Actor health status changed"
                    );
                }
            }
        }
    }
}

/// Message to check actor health
#[derive(Debug, Clone)]
pub struct CheckHealth;

impl Message<CheckHealth> for LifecycleManagerActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: CheckHealth,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.check_health(ctx).await;
    }
}

/// Message to start an actor with lifecycle management
#[derive(Debug, Clone)]
pub struct StartActor<A: Actor + LifecycleAware + 'static> {
    pub actor: A,
    pub health_check_interval: Option<Duration>,
}

impl<A: Actor + LifecycleAware + 'static> Message<StartActor<A>> for LifecycleManagerActor {
    type Reply = Result<ActorRef<A>>;

    async fn handle(
        &mut self,
        msg: StartActor<A>,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Spawn the actor
        let actor_ref = A::spawn(msg.actor);
        let actor_id = actor_ref.id();
        
        // Monitor the actor
        self.monitor_actor(actor_ref.clone(), msg.health_check_interval).await?;
        
        // Publish started event
        self.publish_event(ActorLifecycleEvent::Started { actor_id }).await;
        
        Ok(actor_ref)
    }
}

/// Message to stop an actor gracefully
#[derive(Debug, Clone)]
pub struct StopActor {
    pub actor_id: ActorID,
    pub timeout: Option<Duration>,
}

impl Message<StopActor> for LifecycleManagerActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: StopActor,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Check if we're monitoring this actor
        if let Some(monitored) = self.monitored_actors.get(&msg.actor_id) {
            // Publish stopping event
            self.publish_event(ActorLifecycleEvent::Stopping { actor_id: msg.actor_id }).await;
            
            // Stop the actor
            // In a real implementation, we would send a graceful shutdown message
            // and wait for the actor to stop
            // For now, we'll just kill the actor
            monitored.actor_ref.kill().await?;
            
            // Remove from monitored actors
            self.monitored_actors.remove(&msg.actor_id);
            
            // Publish stopped event
            self.publish_event(ActorLifecycleEvent::Stopped {
                actor_id: msg.actor_id,
                reason: ActorStopReason::Normal,
            }).await;
            
            Ok(())
        } else {
            Err(AppError::NotFoundError(format!(
                "Actor with ID {} not being monitored",
                msg.actor_id
            )))
        }
    }
}

impl Actor for LifecycleManagerActor {
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Start the health check loop
        self.start_health_check_loop(ctx);
    }

    fn on_link_died(&mut self, actor_id: ActorID, reason: ActorStopReason, ctx: &mut Context<Self, ()>) {
        // Check if this is one of our monitored actors
        if self.monitored_actors.remove(&actor_id).is_some() {
            // Publish stopped event
            tokio::spawn({
                let mut this = self.clone();
                async move {
                    this.publish_event(ActorLifecycleEvent::Stopped {
                        actor_id,
                        reason,
                    }).await;
                }
            });
            
            info!(
                actor_id = %actor_id,
                reason = ?reason,
                "Monitored actor died"
            );
        }
    }
}

/// Extension trait for ActorRef to add lifecycle management capabilities
pub trait LifecycleExt<A: Actor + LifecycleAware + 'static> {
    /// Start the actor with lifecycle management
    async fn with_lifecycle_management(
        self,
        lifecycle_manager: &ActorRef<LifecycleManagerActor>,
        health_check_interval: Option<Duration>,
    ) -> Result<Self>
    where
        Self: Sized;
    
    /// Stop the actor gracefully
    async fn stop_gracefully(
        &self,
        lifecycle_manager: &ActorRef<LifecycleManagerActor>,
        timeout: Option<Duration>,
    ) -> Result<()>;
}

impl<A: Actor + LifecycleAware + 'static> LifecycleExt<A> for ActorRef<A> {
    async fn with_lifecycle_management(
        self,
        lifecycle_manager: &ActorRef<LifecycleManagerActor>,
        health_check_interval: Option<Duration>,
    ) -> Result<Self> {
        // Get the actor state
        let actor_state = self.get_state().await?;
        
        // Start with lifecycle management
        let managed_ref = lifecycle_manager
            .ask(&StartActor {
                actor: actor_state,
                health_check_interval,
            })
            .await?;
        
        Ok(managed_ref)
    }
    
    async fn stop_gracefully(
        &self,
        lifecycle_manager: &ActorRef<LifecycleManagerActor>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        lifecycle_manager
            .ask(&StopActor {
                actor_id: self.id(),
                timeout,
            })
            .await
    }
}

/// Create a lifecycle manager actor
pub fn create_lifecycle_manager(health_check_interval: Duration) -> ActorRef<LifecycleManagerActor> {
    LifecycleManagerActor::spawn(
        LifecycleManagerActor::new().with_health_check_interval(health_check_interval)
    )
}

/// Subscribe to lifecycle events
pub async fn subscribe_to_lifecycle_events(
    lifecycle_manager: &ActorRef<LifecycleManagerActor>,
) -> Result<mpsc::Receiver<ActorLifecycleEvent>> {
    Ok(lifecycle_manager.ask(&SubscribeToEvents).await?)
}

/// Message to subscribe to lifecycle events
#[derive(Debug, Clone)]
pub struct SubscribeToEvents;

impl Message<SubscribeToEvents> for LifecycleManagerActor {
    type Reply = mpsc::Receiver<ActorLifecycleEvent>;

    async fn handle(
        &mut self,
        _msg: SubscribeToEvents,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.subscribe().await
    }
}