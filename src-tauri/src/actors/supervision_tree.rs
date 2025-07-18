use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use kameo::prelude::*;
use tracing::{debug, error, info, warn};

use crate::error::{AppError, Result};
use crate::logging;
use crate::actors::supervision::{SupervisionStrategy, SupervisorActor, SupervisionExt};
use crate::actors::fault_detection::{HeartbeatMonitorActor, HeartbeatExt, HeartbeatEvent};
use crate::actors::lifecycle::{LifecycleManagerActor, LifecycleExt, ActorHealthStatus};

/// Supervision tree node types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionTreeNodeType {
    /// One-for-one supervision: each child is supervised independently
    OneForOne,
    /// One-for-all supervision: if one child fails, all are restarted
    OneForAll,
    /// Rest-for-one supervision: if one child fails, all children started after it are restarted
    RestForOne,
}

impl Default for SupervisionTreeNodeType {
    fn default() -> Self {
        SupervisionTreeNodeType::OneForOne
    }
}

/// Node in a supervision tree
#[derive(Actor)]
pub struct SupervisionTreeNode<A: Actor + Clone + 'static> {
    /// Name of the node
    pub name: String,
    /// Node type
    pub node_type: SupervisionTreeNodeType,
    /// Supervisor actor
    pub supervisor: ActorRef<SupervisorActor<A>>,
    /// Child nodes
    pub children: HashMap<String, ActorRef<SupervisionTreeNode<A>>>,
    /// Parent node
    pub parent: Option<ActorRef<SupervisionTreeNode<A>>>,
    /// Default supervision strategy
    pub default_strategy: SupervisionStrategy,
    /// Heartbeat monitor
    pub heartbeat_monitor: Option<ActorRef<HeartbeatMonitorActor>>,
    /// Lifecycle manager
    pub lifecycle_manager: Option<ActorRef<LifecycleManagerActor>>,
}

impl<A: Actor + Clone + 'static> SupervisionTreeNode<A> {
    /// Create a new supervision tree node
    pub fn new(
        name: impl Into<String>,
        node_type: SupervisionTreeNodeType,
        default_strategy: SupervisionStrategy,
    ) -> Self {
        let name = name.into();
        let supervisor = SupervisorActor::spawn(
            SupervisorActor::new(name.clone(), default_strategy)
        );
        
        Self {
            name,
            node_type,
            supervisor,
            children: HashMap::new(),
            parent: None,
            default_strategy,
            heartbeat_monitor: None,
            lifecycle_manager: None,
        }
    }
    
    /// Set the heartbeat monitor
    pub fn with_heartbeat_monitor(
        mut self,
        heartbeat_monitor: ActorRef<HeartbeatMonitorActor>,
    ) -> Self {
        self.heartbeat_monitor = Some(heartbeat_monitor);
        self
    }
    
    /// Set the lifecycle manager
    pub fn with_lifecycle_manager(
        mut self,
        lifecycle_manager: ActorRef<LifecycleManagerActor>,
    ) -> Self {
        self.lifecycle_manager = Some(lifecycle_manager);
        self
    }
    
    /// Add a child node
    pub async fn add_child(
        &mut self,
        ctx: &mut Context<Self, ()>,
        child_name: impl Into<String>,
        child_node_type: SupervisionTreeNodeType,
        child_strategy: SupervisionStrategy,
    ) -> Result<ActorRef<SupervisionTreeNode<A>>> {
        let child_name = child_name.into();
        
        // Create the child node
        let child = SupervisionTreeNode::new(
            format!("{}/{}", self.name, child_name),
            child_node_type,
            child_strategy,
        );
        
        // Set optional components if we have them
        let child = if let Some(ref hm) = self.heartbeat_monitor {
            child.with_heartbeat_monitor(hm.clone())
        } else {
            child
        };
        
        let child = if let Some(ref lm) = self.lifecycle_manager {
            child.with_lifecycle_manager(lm.clone())
        } else {
            child
        };
        
        // Spawn the child node
        let child_ref = SupervisionTreeNode::spawn(child);
        
        // Set the parent reference
        child_ref.tell(&SetParent { parent: ctx.actor_ref().clone() }).await?;
        
        // Store the child
        self.children.insert(child_name, child_ref.clone());
        
        Ok(child_ref)
    }
    
    /// Supervise an actor
    pub async fn supervise_actor(
        &mut self,
        actor: A,
        strategy: Option<SupervisionStrategy>,
    ) -> Result<ActorRef<A>> {
        // Supervise the actor with our supervisor
        let actor_ref = self.supervisor
            .ask(&crate::actors::supervision::SuperviseActor {
                actor,
                strategy,
            })
            .await?;
        
        // If we have a heartbeat monitor, start sending heartbeats
        if let Some(ref monitor) = self.heartbeat_monitor {
            actor_ref.start_heartbeats(monitor, Duration::from_secs(5)).await?;
        }
        
        Ok(actor_ref)
    }
    
    /// Handle escalated failures from children
    async fn handle_escalated_failure(
        &mut self,
        ctx: &mut Context<Self, ()>,
        child_name: String,
        reason: ActorStopReason,
    ) -> Result<()> {
        info!(
            node = %self.name,
            child = %child_name,
            reason = ?reason,
            "Handling escalated failure from child node"
        );
        
        match self.node_type {
            SupervisionTreeNodeType::OneForOne => {
                // Just restart the failed child
                if let Some(child) = self.children.get(&child_name) {
                    // Remove and recreate the child
                    self.children.remove(&child_name);
                    
                    // Create a new child with the same configuration
                    // In a real implementation, we would store the child configuration
                    // For now, we'll just create a new child with default settings
                    let new_child = self.add_child(
                        ctx,
                        child_name,
                        SupervisionTreeNodeType::OneForOne,
                        self.default_strategy,
                    ).await?;
                    
                    info!(
                        node = %self.name,
                        child = %child_name,
                        "Child node restarted after escalated failure"
                    );
                }
            },
            SupervisionTreeNodeType::OneForAll => {
                // Restart all children
                let child_names: Vec<String> = self.children.keys().cloned().collect();
                self.children.clear();
                
                for name in child_names {
                    // Create a new child with the same configuration
                    let new_child = self.add_child(
                        ctx,
                        name.clone(),
                        SupervisionTreeNodeType::OneForOne,
                        self.default_strategy,
                    ).await?;
                    
                    info!(
                        node = %self.name,
                        child = %name,
                        "Child node restarted as part of one-for-all strategy"
                    );
                }
            },
            SupervisionTreeNodeType::RestForOne => {
                // Find the index of the failed child
                let child_names: Vec<String> = self.children.keys().cloned().collect();
                if let Some(index) = child_names.iter().position(|name| name == &child_name) {
                    // Restart the failed child and all children after it
                    let to_restart = &child_names[index..];
                    
                    for name in to_restart {
                        self.children.remove(name);
                        
                        // Create a new child with the same configuration
                        let new_child = self.add_child(
                            ctx,
                            name.clone(),
                            SupervisionTreeNodeType::OneForOne,
                            self.default_strategy,
                        ).await?;
                        
                        info!(
                            node = %self.name,
                            child = %name,
                            "Child node restarted as part of rest-for-one strategy"
                        );
                    }
                }
            },
        }
        
        Ok(())
    }
    
    /// Escalate a failure to the parent node
    async fn escalate_failure(
        &self,
        reason: ActorStopReason,
    ) -> Result<()> {
        if let Some(ref parent) = self.parent {
            parent.tell(&EscalateFailure {
                child_name: self.name.clone(),
                reason,
            }).await?;
            
            info!(
                node = %self.name,
                parent = %parent.id(),
                reason = ?reason,
                "Escalated failure to parent node"
            );
        } else {
            // We're the root node, log the failure
            error!(
                node = %self.name,
                reason = ?reason,
                "Root node received escalated failure, no parent to escalate to"
            );
        }
        
        Ok(())
    }
}

/// Message to set the parent node
#[derive(Clone)]
pub struct SetParent<A: Actor + Clone + 'static> {
    pub parent: ActorRef<SupervisionTreeNode<A>>,
}

impl<A: Actor + Clone + 'static> Message<SetParent<A>> for SupervisionTreeNode<A> {
    type Reply = Result<()>;
    
    async fn handle(
        &mut self,
        msg: SetParent<A>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.parent = Some(msg.parent);
        Ok(())
    }
}

/// Message to escalate a failure to a parent node
#[derive(Clone)]
pub struct EscalateFailure {
    pub child_name: String,
    pub reason: ActorStopReason,
}

impl<A: Actor + Clone + 'static> Message<EscalateFailure> for SupervisionTreeNode<A> {
    type Reply = Result<()>;
    
    async fn handle(
        &mut self,
        msg: EscalateFailure,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle_escalated_failure(ctx, msg.child_name, msg.reason).await
    }
}

/// Create a supervision tree root node
pub fn create_supervision_tree_root<A: Actor + Clone + 'static>(
    name: impl Into<String>,
    node_type: SupervisionTreeNodeType,
    default_strategy: SupervisionStrategy,
) -> ActorRef<SupervisionTreeNode<A>> {
    SupervisionTreeNode::spawn(
        SupervisionTreeNode::new(name, node_type, default_strategy)
    )
}

/// Extension trait for ActorRef to add supervision tree capabilities
pub trait SupervisionTreeExt<A: Actor + Clone + 'static> {
    /// Supervise this actor with a supervision tree node
    async fn with_supervision_tree(
        self,
        tree_node: &ActorRef<SupervisionTreeNode<A>>,
        strategy: Option<SupervisionStrategy>,
    ) -> Result<Self>
    where
        Self: Sized;
}

impl<A: Actor + Clone + 'static> SupervisionTreeExt<A> for ActorRef<A> {
    async fn with_supervision_tree(
        self,
        tree_node: &ActorRef<SupervisionTreeNode<A>>,
        strategy: Option<SupervisionStrategy>,
    ) -> Result<Self> {
        // Get the actor state
        let actor_state = self.get_state().await?;
        
        // Supervise the actor with the tree node
        let supervised_ref = tree_node
            .ask(&SuperviseTreeActor {
                actor: actor_state,
                strategy,
            })
            .await?;
        
        Ok(supervised_ref)
    }
}

/// Message to supervise an actor with a tree node
#[derive(Clone)]
pub struct SuperviseTreeActor<A: Actor + Clone + 'static> {
    pub actor: A,
    pub strategy: Option<SupervisionStrategy>,
}

impl<A: Actor + Clone + 'static> Message<SuperviseTreeActor<A>> for SupervisionTreeNode<A> {
    type Reply = Result<ActorRef<A>>;
    
    async fn handle(
        &mut self,
        msg: SuperviseTreeActor<A>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.supervise_actor(msg.actor, msg.strategy).await
    }
}