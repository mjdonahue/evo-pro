//! Inter-process communication patterns for actors
//!
//! This module provides patterns for actor communication across process boundaries.
//! It builds on the existing actor system and provides a clean API for cross-process
//! communication.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use serde::{Serialize, Deserialize};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::actors::gateway::GatewayActor;
use crate::actors::metrics::{MetricType, MetricValue, MetricsExt};
use crate::actors::supervision::{SupervisionStrategy, SupervisorActor, SupervisionExt};

/// Communication channel types for inter-process communication
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IpcChannelType {
    /// Named pipe (Windows) or Unix domain socket (Unix)
    NamedPipe,
    /// TCP socket
    TcpSocket,
    /// Shared memory
    SharedMemory,
    /// Message queue
    MessageQueue,
}

/// Configuration for an IPC channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcChannelConfig {
    /// Channel type
    pub channel_type: IpcChannelType,
    /// Channel name or address
    pub name: String,
    /// Buffer size for the channel
    pub buffer_size: usize,
    /// Timeout for operations
    pub timeout: Duration,
}

impl Default for IpcChannelConfig {
    fn default() -> Self {
        Self {
            channel_type: IpcChannelType::NamedPipe,
            name: "default-ipc-channel".to_string(),
            buffer_size: 1024,
            timeout: Duration::from_secs(5),
        }
    }
}

/// Actor that manages IPC channels
#[derive(Actor)]
pub struct IpcManagerActor {
    /// Channels by name
    channels: HashMap<String, IpcChannel>,
    /// Process ID
    process_id: String,
}

/// IPC channel for communication between processes
pub struct IpcChannel {
    /// Channel configuration
    pub config: IpcChannelConfig,
    /// Sender for outgoing messages
    pub sender: mpsc::Sender<IpcMessage>,
    /// Receiver for incoming messages
    pub receiver: Option<mpsc::Receiver<IpcMessage>>,
}

/// Message sent over an IPC channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Message ID
    pub id: Uuid,
    /// Source process ID
    pub source: String,
    /// Destination process ID
    pub destination: String,
    /// Actor ID of the sender
    pub sender_id: ActorID,
    /// Actor ID of the recipient
    pub recipient_id: ActorID,
    /// Message type
    pub message_type: String,
    /// Serialized message payload
    pub payload: Vec<u8>,
    /// Whether this is a request (expecting a response)
    pub is_request: bool,
    /// Whether this is a response to a request
    pub is_response: bool,
    /// ID of the request this is a response to
    pub request_id: Option<Uuid>,
    /// Timestamp when the message was sent
    pub timestamp: std::time::SystemTime,
}

impl IpcManagerActor {
    /// Create a new IPC manager actor
    pub fn new(process_id: impl Into<String>) -> Self {
        Self {
            channels: HashMap::new(),
            process_id: process_id.into(),
        }
    }

    /// Create an IPC channel
    pub async fn create_channel(&mut self, config: IpcChannelConfig) -> Result<()> {
        let (tx, rx) = mpsc::channel(config.buffer_size);
        
        let channel = IpcChannel {
            config: config.clone(),
            sender: tx,
            receiver: Some(rx),
        };
        
        self.channels.insert(config.name.clone(), channel);
        
        // Start listening on the channel based on its type
        self.start_channel_listener(&config.name).await?;
        
        info!(
            process_id = %self.process_id,
            channel = %config.name,
            channel_type = ?config.channel_type,
            "Created IPC channel"
        );
        
        Ok(())
    }

    /// Start listening on an IPC channel
    async fn start_channel_listener(&mut self, channel_name: &str) -> Result<()> {
        let channel = self.channels.get_mut(channel_name).ok_or_else(|| {
            AppError::NotFoundError(format!("IPC channel '{}' not found", channel_name))
        })?;
        
        let mut receiver = channel.receiver.take().ok_or_else(|| {
            AppError::InvalidStateError("Channel receiver already taken".to_string())
        })?;
        
        let process_id = self.process_id.clone();
        let config = channel.config.clone();
        
        // Spawn a task to listen for incoming messages
        tokio::spawn(async move {
            info!(
                process_id = %process_id,
                channel = %config.name,
                "Started listening on IPC channel"
            );
            
            while let Some(message) = receiver.recv().await {
                // Handle the incoming message
                if message.destination != process_id {
                    // Message is not for this process, ignore it
                    continue;
                }
                
                debug!(
                    process_id = %process_id,
                    message_id = %message.id,
                    sender = %message.sender_id,
                    recipient = %message.recipient_id,
                    "Received IPC message"
                );
                
                // TODO: Route the message to the appropriate actor
                // This would involve looking up the actor by ID and sending the message
            }
            
            info!(
                process_id = %process_id,
                channel = %config.name,
                "Stopped listening on IPC channel"
            );
        });
        
        Ok(())
    }

    /// Send a message over an IPC channel
    pub async fn send_message(
        &self,
        channel_name: &str,
        message: IpcMessage,
    ) -> Result<()> {
        let channel = self.channels.get(channel_name).ok_or_else(|| {
            AppError::NotFoundError(format!("IPC channel '{}' not found", channel_name))
        })?;
        
        // Send the message
        channel.sender.send(message.clone()).await.map_err(|_| {
            AppError::SendError(format!("Failed to send message on channel '{}'", channel_name))
        })?;
        
        debug!(
            process_id = %self.process_id,
            channel = %channel_name,
            message_id = %message.id,
            recipient = %message.recipient_id,
            "Sent IPC message"
        );
        
        Ok(())
    }

    /// Send a request over an IPC channel and wait for a response
    pub async fn send_request<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        channel_name: &str,
        destination: &str,
        sender_id: ActorID,
        recipient_id: ActorID,
        request: T,
    ) -> Result<R> {
        let channel = self.channels.get(channel_name).ok_or_else(|| {
            AppError::NotFoundError(format!("IPC channel '{}' not found", channel_name))
        })?;
        
        // Create a response channel
        let (response_tx, response_rx) = oneshot::channel();
        
        // Generate a request ID
        let request_id = Uuid::new_v4();
        
        // Serialize the request
        let payload = bincode::serialize(&request).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize request: {}", e))
        })?;
        
        // Create the message
        let message = IpcMessage {
            id: request_id,
            source: self.process_id.clone(),
            destination: destination.to_string(),
            sender_id,
            recipient_id,
            message_type: std::any::type_name::<T>().to_string(),
            payload,
            is_request: true,
            is_response: false,
            request_id: None,
            timestamp: std::time::SystemTime::now(),
        };
        
        // TODO: Register the response channel with a response handler
        
        // Send the message
        channel.sender.send(message.clone()).await.map_err(|_| {
            AppError::SendError(format!("Failed to send request on channel '{}'", channel_name))
        })?;
        
        debug!(
            process_id = %self.process_id,
            channel = %channel_name,
            request_id = %request_id,
            recipient = %recipient_id,
            "Sent IPC request"
        );
        
        // Wait for the response with timeout
        let response = tokio::time::timeout(
            channel.config.timeout,
            response_rx
        ).await.map_err(|_| {
            AppError::TimeoutError(format!(
                "Timed out waiting for response to request {} after {:?}",
                request_id, channel.config.timeout
            ))
        })??;
        
        // Deserialize the response
        let response: R = bincode::deserialize(&response).map_err(|e| {
            AppError::DeserializationError(format!("Failed to deserialize response: {}", e))
        })?;
        
        Ok(response)
    }
}

/// Message to create an IPC channel
#[derive(Debug, Clone)]
pub struct CreateIpcChannel {
    pub config: IpcChannelConfig,
}

impl Message<CreateIpcChannel> for IpcManagerActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: CreateIpcChannel,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.create_channel(msg.config).await
    }
}

/// Message to send a message over an IPC channel
#[derive(Debug, Clone)]
pub struct SendIpcMessage {
    pub channel_name: String,
    pub message: IpcMessage,
}

impl Message<SendIpcMessage> for IpcManagerActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: SendIpcMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.send_message(&msg.channel_name, msg.message).await
    }
}

/// Message to send a request over an IPC channel
#[derive(Debug, Clone)]
pub struct SendIpcRequest<T: Serialize + Send + Sync + 'static, R: for<'de> Deserialize<'de> + Send + 'static> {
    pub channel_name: String,
    pub destination: String,
    pub sender_id: ActorID,
    pub recipient_id: ActorID,
    pub request: T,
    _phantom: std::marker::PhantomData<R>,
}

impl<T: Serialize + Send + Sync + 'static, R: for<'de> Deserialize<'de> + Send + 'static> SendIpcRequest<T, R> {
    pub fn new(
        channel_name: impl Into<String>,
        destination: impl Into<String>,
        sender_id: ActorID,
        recipient_id: ActorID,
        request: T,
    ) -> Self {
        Self {
            channel_name: channel_name.into(),
            destination: destination.into(),
            sender_id,
            recipient_id,
            request,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Serialize + Send + Sync + 'static, R: for<'de> Deserialize<'de> + Send + 'static> Message<SendIpcRequest<T, R>> for IpcManagerActor {
    type Reply = Result<R>;

    async fn handle(
        &mut self,
        msg: SendIpcRequest<T, R>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.send_request(
            &msg.channel_name,
            &msg.destination,
            msg.sender_id,
            msg.recipient_id,
            msg.request,
        ).await
    }
}

/// Actor proxy for communicating with actors in other processes
pub struct IpcActorProxy<A: Actor + 'static> {
    /// IPC manager actor reference
    ipc_manager: ActorRef<IpcManagerActor>,
    /// Channel name to use for communication
    channel_name: String,
    /// Destination process ID
    destination: String,
    /// Actor ID of the remote actor
    remote_actor_id: ActorID,
    /// Actor ID of the local proxy
    local_actor_id: ActorID,
    /// Phantom data for the actor type
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Actor + 'static> IpcActorProxy<A> {
    /// Create a new IPC actor proxy
    pub fn new(
        ipc_manager: ActorRef<IpcManagerActor>,
        channel_name: impl Into<String>,
        destination: impl Into<String>,
        remote_actor_id: ActorID,
        local_actor_id: ActorID,
    ) -> Self {
        Self {
            ipc_manager,
            channel_name: channel_name.into(),
            destination: destination.into(),
            remote_actor_id,
            local_actor_id,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Send a message to the remote actor
    pub async fn tell<M: Serialize + Send + Sync + 'static>(&self, message: &M) -> Result<()> {
        // Serialize the message
        let payload = bincode::serialize(message).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize message: {}", e))
        })?;
        
        // Create the IPC message
        let ipc_message = IpcMessage {
            id: Uuid::new_v4(),
            source: "".to_string(), // This will be filled in by the IPC manager
            destination: self.destination.clone(),
            sender_id: self.local_actor_id,
            recipient_id: self.remote_actor_id,
            message_type: std::any::type_name::<M>().to_string(),
            payload,
            is_request: false,
            is_response: false,
            request_id: None,
            timestamp: std::time::SystemTime::now(),
        };
        
        // Send the message
        self.ipc_manager.ask(&SendIpcMessage {
            channel_name: self.channel_name.clone(),
            message: ipc_message,
        }).await
    }

    /// Send a request to the remote actor and wait for a response
    pub async fn ask<M: Serialize + Send + Sync + 'static, R: for<'de> Deserialize<'de> + Send + 'static>(&self, message: &M) -> Result<R> {
        self.ipc_manager.ask(&SendIpcRequest::<M, R>::new(
            self.channel_name.clone(),
            self.destination.clone(),
            self.local_actor_id,
            self.remote_actor_id,
            message.clone(),
        )).await
    }
}

/// Extension trait for ActorRef to add IPC capabilities
pub trait IpcExt<A: Actor + 'static> {
    /// Create an IPC proxy for this actor
    fn create_ipc_proxy(
        &self,
        ipc_manager: &ActorRef<IpcManagerActor>,
        channel_name: impl Into<String>,
        destination: impl Into<String>,
        remote_actor_id: ActorID,
    ) -> IpcActorProxy<A>;
}

impl<A: Actor + 'static> IpcExt<A> for ActorRef<A> {
    fn create_ipc_proxy(
        &self,
        ipc_manager: &ActorRef<IpcManagerActor>,
        channel_name: impl Into<String>,
        destination: impl Into<String>,
        remote_actor_id: ActorID,
    ) -> IpcActorProxy<A> {
        IpcActorProxy::new(
            ipc_manager.clone(),
            channel_name,
            destination,
            remote_actor_id,
            self.id(),
        )
    }
}

/// Create an IPC manager actor
pub fn create_ipc_manager(process_id: impl Into<String>) -> ActorRef<IpcManagerActor> {
    IpcManagerActor::spawn(IpcManagerActor::new(process_id))
}

/// Register an actor for IPC communication
pub async fn register_actor_for_ipc<A: Actor + 'static>(
    actor_ref: &ActorRef<A>,
    ipc_manager: &ActorRef<IpcManagerActor>,
    channel_name: &str,
) -> Result<()> {
    // TODO: Implement actor registration for IPC
    // This would involve setting up message handlers for the actor
    // to receive messages from other processes
    
    Ok(())
}

/// Message handler for IPC messages
#[derive(Actor)]
pub struct IpcMessageHandlerActor {
    /// IPC manager actor reference
    ipc_manager: ActorRef<IpcManagerActor>,
    /// Actor registry
    actor_registry: HashMap<ActorID, ActorRef<dyn Any>>,
    /// Response handlers
    response_handlers: HashMap<Uuid, oneshot::Sender<Vec<u8>>>,
}

impl IpcMessageHandlerActor {
    /// Create a new IPC message handler actor
    pub fn new(ipc_manager: ActorRef<IpcManagerActor>) -> Self {
        Self {
            ipc_manager,
            actor_registry: HashMap::new(),
            response_handlers: HashMap::new(),
        }
    }

    /// Register an actor with the handler
    pub fn register_actor<A: Actor + 'static>(&mut self, actor_ref: ActorRef<A>) {
        self.actor_registry.insert(actor_ref.id(), actor_ref.into_any());
    }

    /// Handle an incoming IPC message
    pub async fn handle_message(&mut self, message: IpcMessage) -> Result<()> {
        // Check if this is a response to a request
        if message.is_response {
            if let Some(request_id) = message.request_id {
                if let Some(handler) = self.response_handlers.remove(&request_id) {
                    // Send the response to the handler
                    if let Err(_) = handler.send(message.payload) {
                        warn!(
                            message_id = %message.id,
                            request_id = %request_id,
                            "Failed to send response to handler"
                        );
                    }
                    return Ok(());
                }
            }
        }
        
        // Look up the recipient actor
        let recipient_id = message.recipient_id;
        let actor_ref = self.actor_registry.get(&recipient_id).ok_or_else(|| {
            AppError::NotFoundError(format!("Actor with ID {} not found", recipient_id))
        })?;
        
        // TODO: Deserialize the message and send it to the actor
        // This would involve knowing the message type and using reflection
        // or a registry of message types
        
        Ok(())
    }

    /// Register a response handler
    pub fn register_response_handler(
        &mut self,
        request_id: Uuid,
        handler: oneshot::Sender<Vec<u8>>,
    ) {
        self.response_handlers.insert(request_id, handler);
    }
}

/// Message to handle an IPC message
#[derive(Debug, Clone)]
pub struct HandleIpcMessage {
    pub message: IpcMessage,
}

impl Message<HandleIpcMessage> for IpcMessageHandlerActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: HandleIpcMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle_message(msg.message).await
    }
}

/// Message to register an actor with the IPC message handler
#[derive(Debug, Clone)]
pub struct RegisterActorWithIpc<A: Actor + 'static> {
    pub actor_ref: ActorRef<A>,
}

impl<A: Actor + 'static> Message<RegisterActorWithIpc<A>> for IpcMessageHandlerActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: RegisterActorWithIpc<A>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.register_actor(msg.actor_ref);
    }
}

/// Create an IPC message handler actor
pub fn create_ipc_message_handler(
    ipc_manager: ActorRef<IpcManagerActor>,
) -> ActorRef<IpcMessageHandlerActor> {
    IpcMessageHandlerActor::spawn(IpcMessageHandlerActor::new(ipc_manager))
}