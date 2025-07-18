use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use kameo::prelude::*;
use tokio::sync::{mpsc, oneshot};
use tokio::time::{interval, sleep, timeout};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::logging;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed (allowing requests)
    Closed,
    /// Circuit is open (blocking requests)
    Open,
    /// Circuit is half-open (allowing a limited number of test requests)
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening the circuit
    pub failure_threshold: usize,
    /// Success threshold in half-open state before closing the circuit
    pub success_threshold: usize,
    /// Time to wait before transitioning from open to half-open
    pub reset_timeout: Duration,
    /// Time window for counting failures
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            reset_timeout: Duration::from_secs(30),
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    /// Name of the circuit breaker
    name: String,
    /// Current state
    state: CircuitBreakerState,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Failure history
    failures: Vec<Instant>,
    /// Success count in half-open state
    half_open_successes: usize,
    /// Time when the circuit was opened
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            name: name.into(),
            state: CircuitBreakerState::Closed,
            config,
            failures: Vec::new(),
            half_open_successes: 0,
            opened_at: None,
        }
    }

    /// Check if the circuit is closed (allowing requests)
    pub fn is_closed(&self) -> bool {
        self.state == CircuitBreakerState::Closed
    }

    /// Check if the circuit is open (blocking requests)
    pub fn is_open(&self) -> bool {
        self.state == CircuitBreakerState::Open
    }

    /// Check if the circuit is half-open (allowing test requests)
    pub fn is_half_open(&self) -> bool {
        self.state == CircuitBreakerState::HalfOpen
    }

    /// Get the current state
    pub fn state(&self) -> CircuitBreakerState {
        self.state
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                // Nothing to do in closed state
            }
            CircuitBreakerState::HalfOpen => {
                // In half-open state, count successes
                self.half_open_successes += 1;
                
                // If we've reached the success threshold, close the circuit
                if self.half_open_successes >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.half_open_successes = 0;
                    info!(
                        name = %self.name,
                        "Circuit breaker closed after successful test requests"
                    );
                }
            }
            CircuitBreakerState::Open => {
                // Check if it's time to transition to half-open
                if let Some(opened_at) = self.opened_at {
                    if opened_at.elapsed() >= self.config.reset_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        self.half_open_successes = 1; // Count this success
                        info!(
                            name = %self.name,
                            "Circuit breaker transitioned from open to half-open"
                        );
                    }
                }
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        // Add the failure to the history
        self.failures.push(Instant::now());
        
        // Remove failures outside the window
        self.failures.retain(|t| t.elapsed() < self.config.failure_window);
        
        match self.state {
            CircuitBreakerState::Closed => {
                // If we've reached the failure threshold, open the circuit
                if self.failures.len() >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    self.opened_at = Some(Instant::now());
                    warn!(
                        name = %self.name,
                        failures = %self.failures.len(),
                        threshold = %self.config.failure_threshold,
                        "Circuit breaker opened due to too many failures"
                    );
                }
            }
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                self.state = CircuitBreakerState::Open;
                self.opened_at = Some(Instant::now());
                self.half_open_successes = 0;
                warn!(
                    name = %self.name,
                    "Circuit breaker reopened after failure in half-open state"
                );
            }
            CircuitBreakerState::Open => {
                // Nothing to do in open state
            }
        }
    }

    /// Execute an operation with circuit breaker protection
    pub async fn execute<F, T, E>(&mut self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T, E>,
        E: std::fmt::Display,
    {
        match self.state {
            CircuitBreakerState::Open => {
                // Check if it's time to transition to half-open
                if let Some(opened_at) = self.opened_at {
                    if opened_at.elapsed() >= self.config.reset_timeout {
                        self.state = CircuitBreakerState::HalfOpen;
                        info!(
                            name = %self.name,
                            "Circuit breaker transitioned from open to half-open"
                        );
                    } else {
                        // Circuit is still open, fail fast
                        return Err(AppError::CircuitBreakerOpen(format!(
                            "Circuit breaker '{}' is open",
                            self.name
                        )));
                    }
                }
            }
            _ => {}
        }
        
        // Execute the operation
        match operation() {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(err) => {
                self.record_failure();
                Err(AppError::ExternalServiceError(format!(
                    "Operation failed: {}",
                    err
                )))
            }
        }
    }
}

/// Actor that manages circuit breakers
#[derive(Actor)]
pub struct CircuitBreakerActor {
    /// Circuit breakers by name
    circuit_breakers: HashMap<String, CircuitBreaker>,
}

impl CircuitBreakerActor {
    /// Create a new circuit breaker actor
    pub fn new() -> Self {
        Self {
            circuit_breakers: HashMap::new(),
        }
    }

    /// Create a circuit breaker
    pub fn create_circuit_breaker(
        &mut self,
        name: impl Into<String>,
        config: CircuitBreakerConfig,
    ) -> String {
        let name = name.into();
        self.circuit_breakers.insert(
            name.clone(),
            CircuitBreaker::new(name.clone(), config),
        );
        name
    }

    /// Get a circuit breaker by name
    pub fn get_circuit_breaker(&self, name: &str) -> Option<&CircuitBreaker> {
        self.circuit_breakers.get(name)
    }

    /// Get a mutable reference to a circuit breaker by name
    pub fn get_circuit_breaker_mut(&mut self, name: &str) -> Option<&mut CircuitBreaker> {
        self.circuit_breakers.get_mut(name)
    }
}

/// Message to create a circuit breaker
#[derive(Debug, Clone)]
pub struct CreateCircuitBreaker {
    pub name: String,
    pub config: CircuitBreakerConfig,
}

impl Message<CreateCircuitBreaker> for CircuitBreakerActor {
    type Reply = String;

    async fn handle(
        &mut self,
        msg: CreateCircuitBreaker,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.create_circuit_breaker(msg.name, msg.config)
    }
}

/// Message to execute an operation with circuit breaker protection
pub struct ExecuteWithCircuitBreaker<T: Send + 'static> {
    pub circuit_breaker_name: String,
    pub operation: Box<dyn FnOnce() -> Result<T> + Send + 'static>,
}

impl<T: Send + 'static> Message<ExecuteWithCircuitBreaker<T>> for CircuitBreakerActor {
    type Reply = Result<T>;

    async fn handle(
        &mut self,
        msg: ExecuteWithCircuitBreaker<T>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        // Get the circuit breaker
        let circuit_breaker = self.get_circuit_breaker_mut(&msg.circuit_breaker_name)
            .ok_or_else(|| AppError::NotFoundError(format!(
                "Circuit breaker '{}' not found",
                msg.circuit_breaker_name
            )))?;
        
        // Execute the operation with circuit breaker protection
        circuit_breaker.execute(|| (msg.operation)()).await
    }
}

/// Heartbeat monitor for detecting actor failures
#[derive(Actor)]
pub struct HeartbeatMonitorActor {
    /// Actors being monitored
    monitored_actors: HashMap<ActorID, MonitoredActor>,
    /// Heartbeat interval
    heartbeat_interval: Duration,
    /// Heartbeat timeout
    heartbeat_timeout: Duration,
    /// Subscribers to heartbeat events
    subscribers: Vec<mpsc::Sender<HeartbeatEvent>>,
}

/// Information about a monitored actor
struct MonitoredActor {
    /// Actor reference
    actor_ref: ActorRef<dyn Any>,
    /// Last heartbeat time
    last_heartbeat: Instant,
    /// Whether the actor is currently considered alive
    is_alive: bool,
}

/// Heartbeat events
#[derive(Debug, Clone)]
pub enum HeartbeatEvent {
    /// Actor heartbeat received
    HeartbeatReceived { actor_id: ActorID },
    /// Actor heartbeat missed
    HeartbeatMissed { actor_id: ActorID },
    /// Actor considered dead (multiple heartbeats missed)
    ActorDead { actor_id: ActorID },
}

impl HeartbeatMonitorActor {
    /// Create a new heartbeat monitor actor
    pub fn new(heartbeat_interval: Duration, heartbeat_timeout: Duration) -> Self {
        Self {
            monitored_actors: HashMap::new(),
            heartbeat_interval,
            heartbeat_timeout,
            subscribers: Vec::new(),
        }
    }

    /// Start the heartbeat check loop
    async fn start_heartbeat_check_loop(&self, ctx: &mut Context<Self, ()>) {
        let actor_ref = ctx.actor_ref();
        
        // Spawn a task to periodically check heartbeats
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(1));
            
            loop {
                check_interval.tick().await;
                
                // Send a message to check heartbeats
                if let Err(e) = actor_ref.tell(&CheckHeartbeats).await {
                    error!("Failed to send heartbeat check message: {}", e);
                    break;
                }
            }
        });
    }

    /// Subscribe to heartbeat events
    pub async fn subscribe(&mut self) -> mpsc::Receiver<HeartbeatEvent> {
        let (tx, rx) = mpsc::channel(100);
        self.subscribers.push(tx);
        rx
    }

    /// Publish a heartbeat event to all subscribers
    async fn publish_event(&mut self, event: HeartbeatEvent) {
        // Remove closed channels
        self.subscribers.retain(|tx| !tx.is_closed());
        
        // Send the event to all subscribers
        for tx in &self.subscribers {
            if let Err(e) = tx.send(event.clone()).await {
                warn!("Failed to send heartbeat event: {}", e);
            }
        }
    }

    /// Monitor an actor for heartbeats
    pub async fn monitor_actor<A: Actor + 'static>(
        &mut self,
        actor_ref: ActorRef<A>,
    ) -> Result<()> {
        let actor_id = actor_ref.id();
        
        // Add to monitored actors
        self.monitored_actors.insert(
            actor_id,
            MonitoredActor {
                actor_ref: actor_ref.into_any(),
                last_heartbeat: Instant::now(),
                is_alive: true,
            },
        );
        
        info!("Now monitoring actor {} for heartbeats", actor_id);
        
        Ok(())
    }

    /// Record a heartbeat from an actor
    pub fn record_heartbeat(&mut self, actor_id: ActorID) -> Result<()> {
        if let Some(monitored) = self.monitored_actors.get_mut(&actor_id) {
            monitored.last_heartbeat = Instant::now();
            
            // If the actor was previously considered dead, mark it as alive
            if !monitored.is_alive {
                monitored.is_alive = true;
                info!("Actor {} is now alive", actor_id);
            }
            
            // Publish heartbeat received event
            tokio::spawn({
                let mut this = self.clone();
                let actor_id = actor_id;
                async move {
                    this.publish_event(HeartbeatEvent::HeartbeatReceived { actor_id }).await;
                }
            });
            
            Ok(())
        } else {
            Err(AppError::NotFoundError(format!(
                "Actor with ID {} not being monitored for heartbeats",
                actor_id
            )))
        }
    }

    /// Check heartbeats for all monitored actors
    async fn check_heartbeats(&mut self) {
        let now = Instant::now();
        
        for (actor_id, monitored) in &mut self.monitored_actors {
            let elapsed = now.duration_since(monitored.last_heartbeat);
            
            if elapsed > self.heartbeat_timeout {
                // Actor has missed too many heartbeats, consider it dead
                if monitored.is_alive {
                    monitored.is_alive = false;
                    warn!("Actor {} considered dead (no heartbeat for {:?})", actor_id, elapsed);
                    
                    // Publish actor dead event
                    self.publish_event(HeartbeatEvent::ActorDead { actor_id: *actor_id }).await;
                }
            } else if elapsed > self.heartbeat_interval {
                // Actor has missed a heartbeat
                warn!("Actor {} missed a heartbeat", actor_id);
                
                // Publish heartbeat missed event
                self.publish_event(HeartbeatEvent::HeartbeatMissed { actor_id: *actor_id }).await;
            }
        }
    }
}

/// Message to check heartbeats
#[derive(Debug, Clone)]
pub struct CheckHeartbeats;

impl Message<CheckHeartbeats> for HeartbeatMonitorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: CheckHeartbeats,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.check_heartbeats().await;
    }
}

/// Message to record a heartbeat
#[derive(Debug, Clone)]
pub struct RecordHeartbeat {
    pub actor_id: ActorID,
}

impl Message<RecordHeartbeat> for HeartbeatMonitorActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: RecordHeartbeat,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.record_heartbeat(msg.actor_id)
    }
}

/// Message to monitor an actor for heartbeats
#[derive(Debug, Clone)]
pub struct MonitorActorHeartbeat<A: Actor + 'static> {
    pub actor_ref: ActorRef<A>,
}

impl<A: Actor + 'static> Message<MonitorActorHeartbeat<A>> for HeartbeatMonitorActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: MonitorActorHeartbeat<A>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.monitor_actor(msg.actor_ref).await
    }
}

impl Actor for HeartbeatMonitorActor {
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Start the heartbeat check loop
        self.start_heartbeat_check_loop(ctx);
    }
}

/// Message to subscribe to heartbeat events
#[derive(Debug, Clone)]
pub struct SubscribeToHeartbeatEvents;

impl Message<SubscribeToHeartbeatEvents> for HeartbeatMonitorActor {
    type Reply = mpsc::Receiver<HeartbeatEvent>;

    async fn handle(
        &mut self,
        _msg: SubscribeToHeartbeatEvents,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.subscribe().await
    }
}

/// Extension trait for ActorRef to add heartbeat capabilities
pub trait HeartbeatExt<A: Actor + 'static> {
    /// Send a heartbeat to the monitor
    async fn send_heartbeat(
        &self,
        monitor: &ActorRef<HeartbeatMonitorActor>,
    ) -> Result<()>;
    
    /// Start sending periodic heartbeats
    async fn start_heartbeats(
        &self,
        monitor: &ActorRef<HeartbeatMonitorActor>,
        interval: Duration,
    ) -> Result<()>;
}

impl<A: Actor + 'static> HeartbeatExt<A> for ActorRef<A> {
    async fn send_heartbeat(
        &self,
        monitor: &ActorRef<HeartbeatMonitorActor>,
    ) -> Result<()> {
        monitor
            .ask(&RecordHeartbeat {
                actor_id: self.id(),
            })
            .await
    }
    
    async fn start_heartbeats(
        &self,
        monitor: &ActorRef<HeartbeatMonitorActor>,
        interval: Duration,
    ) -> Result<()> {
        let actor_id = self.id();
        let monitor = monitor.clone();
        
        // Start monitoring the actor
        monitor
            .ask(&MonitorActorHeartbeat {
                actor_ref: self.clone(),
            })
            .await?;
        
        // Spawn a task to send periodic heartbeats
        tokio::spawn(async move {
            let mut heartbeat_interval = interval(interval);
            
            loop {
                heartbeat_interval.tick().await;
                
                // Send a heartbeat
                if let Err(e) = monitor
                    .ask(&RecordHeartbeat { actor_id })
                    .await
                {
                    error!("Failed to send heartbeat: {}", e);
                    break;
                }
            }
        });
        
        Ok(())
    }
}

/// Actor that detects timeouts for long-running operations
#[derive(Actor)]
pub struct TimeoutDetectorActor {
    /// Operations being monitored
    operations: HashMap<Uuid, MonitoredOperation>,
}

/// Information about a monitored operation
struct MonitoredOperation {
    /// Operation name
    name: String,
    /// Start time
    start_time: Instant,
    /// Timeout duration
    timeout: Duration,
    /// Completion channel
    completion_tx: Option<oneshot::Sender<()>>,
}

impl TimeoutDetectorActor {
    /// Create a new timeout detector actor
    pub fn new() -> Self {
        Self {
            operations: HashMap::new(),
        }
    }

    /// Start monitoring an operation
    pub fn start_operation(
        &mut self,
        operation_id: Uuid,
        name: impl Into<String>,
        timeout: Duration,
    ) -> oneshot::Receiver<()> {
        let (tx, rx) = oneshot::channel();
        
        self.operations.insert(
            operation_id,
            MonitoredOperation {
                name: name.into(),
                start_time: Instant::now(),
                timeout,
                completion_tx: Some(tx),
            },
        );
        
        rx
    }

    /// Complete an operation
    pub fn complete_operation(&mut self, operation_id: Uuid) -> Result<()> {
        if let Some(mut operation) = self.operations.remove(&operation_id) {
            if let Some(tx) = operation.completion_tx.take() {
                let _ = tx.send(());
            }
            Ok(())
        } else {
            Err(AppError::NotFoundError(format!(
                "Operation with ID {} not found",
                operation_id
            )))
        }
    }

    /// Check for timed out operations
    pub fn check_timeouts(&mut self) {
        let now = Instant::now();
        let mut timed_out = Vec::new();
        
        // Identify timed out operations
        for (operation_id, operation) in &self.operations {
            if now.duration_since(operation.start_time) > operation.timeout {
                timed_out.push(*operation_id);
            }
        }
        
        // Handle timed out operations
        for operation_id in timed_out {
            if let Some(operation) = self.operations.remove(&operation_id) {
                warn!(
                    operation_id = %operation_id,
                    name = %operation.name,
                    timeout = ?operation.timeout,
                    "Operation timed out"
                );
                
                // The completion channel will be dropped, which will signal a timeout
            }
        }
    }

    /// Start the timeout check loop
    async fn start_timeout_check_loop(&self, ctx: &mut Context<Self, ()>) {
        let actor_ref = ctx.actor_ref();
        
        // Spawn a task to periodically check timeouts
        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(1));
            
            loop {
                check_interval.tick().await;
                
                // Send a message to check timeouts
                if let Err(e) = actor_ref.tell(&CheckTimeouts).await {
                    error!("Failed to send timeout check message: {}", e);
                    break;
                }
            }
        });
    }
}

/// Message to check timeouts
#[derive(Debug, Clone)]
pub struct CheckTimeouts;

impl Message<CheckTimeouts> for TimeoutDetectorActor {
    type Reply = ();

    async fn handle(
        &mut self,
        _msg: CheckTimeouts,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.check_timeouts();
    }
}

/// Message to start monitoring an operation
#[derive(Debug, Clone)]
pub struct StartOperation {
    pub operation_id: Uuid,
    pub name: String,
    pub timeout: Duration,
}

impl Message<StartOperation> for TimeoutDetectorActor {
    type Reply = oneshot::Receiver<()>;

    async fn handle(
        &mut self,
        msg: StartOperation,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.start_operation(msg.operation_id, msg.name, msg.timeout)
    }
}

/// Message to complete an operation
#[derive(Debug, Clone)]
pub struct CompleteOperation {
    pub operation_id: Uuid,
}

impl Message<CompleteOperation> for TimeoutDetectorActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: CompleteOperation,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.complete_operation(msg.operation_id)
    }
}

impl Actor for TimeoutDetectorActor {
    fn on_start(&mut self, ctx: &mut Context<Self, ()>) {
        // Start the timeout check loop
        self.start_timeout_check_loop(ctx);
    }
}

/// Execute an operation with timeout detection
pub async fn execute_with_timeout<F, T>(
    timeout_detector: &ActorRef<TimeoutDetectorActor>,
    name: impl Into<String>,
    timeout_duration: Duration,
    operation: F,
) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let operation_id = Uuid::new_v4();
    let name = name.into();
    
    // Start monitoring the operation
    let completion_rx = timeout_detector
        .ask(&StartOperation {
            operation_id,
            name: name.clone(),
            timeout: timeout_duration,
        })
        .await;
    
    // Execute the operation with timeout
    let result = timeout(
        timeout_duration,
        async {
            // Execute the operation
            let result = operation();
            
            // Complete the operation
            if let Err(e) = timeout_detector
                .ask(&CompleteOperation { operation_id })
                .await
            {
                warn!("Failed to complete operation: {}", e);
            }
            
            result
        },
    )
    .await;
    
    // Handle the result
    match result {
        Ok(result) => result,
        Err(_) => {
            // Timeout occurred
            Err(AppError::OperationTimeout(format!(
                "Operation '{}' timed out after {:?}",
                name, timeout_duration
            )))
        }
    }
}

/// Create a circuit breaker actor
pub fn create_circuit_breaker_actor() -> ActorRef<CircuitBreakerActor> {
    CircuitBreakerActor::spawn(CircuitBreakerActor::new())
}

/// Create a heartbeat monitor actor
pub fn create_heartbeat_monitor(
    heartbeat_interval: Duration,
    heartbeat_timeout: Duration,
) -> ActorRef<HeartbeatMonitorActor> {
    HeartbeatMonitorActor::spawn(
        HeartbeatMonitorActor::new(heartbeat_interval, heartbeat_timeout)
    )
}

/// Create a timeout detector actor
pub fn create_timeout_detector() -> ActorRef<TimeoutDetectorActor> {
    TimeoutDetectorActor::spawn(TimeoutDetectorActor::new())
}