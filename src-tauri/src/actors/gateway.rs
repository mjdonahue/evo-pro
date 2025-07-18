use std::{any::Any, collections::HashMap, sync::OnceLock};

use color_eyre::eyre::eyre;
use kameo::prelude::{ActorRef as LocalActorRef, *};
use kameo_actors::message_bus::Publish;
use rig::completion::ToolDefinition;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::{
    actors::{
        ActorRef, AgentManagerActor, DatabaseActor, SystemEventBus,
        agents::{AgentRequest, AgentResponseEvent},
        conversation::SendMessage,
        tools::{GetTools, ToolExecutorActor, UseTool},
    },
    entities::{Conversation, CreateConversation, Message as ChatMessage},
    error::{AppError, Result},
    keys::Signed,
    utils::SaveTask,
};

pub static GATEWAY_ACTOR: OnceLock<LocalActorRef<GatewayActor>> = OnceLock::new();

macro_rules! task_impl {
    ($name:ty, $id:literal) => {
        #[remote_message($id)]
        impl Message<Signed<$name>> for GatewayActor {
            type Reply = ();

            async fn handle(
                &mut self,
                msg: Signed<$name>,
                _ctx: &mut Context<Self, Self::Reply>,
            ) -> Self::Reply {
                if !msg.verify_signature() {
                    return;
                }
                if let Some(task_id) = msg.task_id()
                    && let Some(sender) = self.active_tasks.remove(task_id)
                {
                    let _ = sender.send(Box::new(msg.into_inner()));
                }
            }
        }
    };
}

// State inside the client's GatewayActor
#[derive(Actor, RemoteActor)]
pub struct GatewayActor {
    pub db: LocalActorRef<DatabaseActor>,
    pub bus: LocalActorRef<SystemEventBus>,
    pub agent_manager: LocalActorRef<AgentManagerActor>,
    pub tool_executor: LocalActorRef<ToolExecutorActor>,
    pub active_tasks: HashMap<Uuid, oneshot::Sender<Box<dyn Any + Send + Sync + 'static>>>,
}

// Handles incoming network messages to create a conversation on this peer.
#[remote_message("313da359-6c9a-4d16-9ee0-d567b00f67c9")]
impl Message<Signed<CreateConversation>> for GatewayActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: Signed<CreateConversation>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if !msg.verify_signature() {
            return Err(eyre!("Invalid signature").into());
        }
        self.bus.tell(Publish(msg)).await.ok();
        Ok(())
    }
}

// Handles incoming network messages to send a chat message on this peer.
#[remote_message("313da359-6c9a-4d16-9ee0-d567b00f67d0")]
impl Message<Signed<SendMessage>> for GatewayActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: Signed<SendMessage>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if !msg.verify_signature() {
            return Err(eyre!("Invalid signature").into());
        }
        self.bus.tell(Publish(msg)).await.ok();
        Ok(())
    }
}

// Existing handlers...

#[remote_message("e8f5abf6-a4af-4410-b0da-38e9c1ffe06e")]
impl Message<Signed<GetTools>> for GatewayActor {
    type Reply = Result<Vec<rig::completion::ToolDefinition>>;

    async fn handle(
        &mut self,
        msg: Signed<GetTools>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if !msg.verify_signature() {
            return Err(eyre!("Invalid signature").into());
        }
        Ok(self.tool_executor.ask(msg.into_inner()).await?)
    }
}

#[remote_message("e8f5abf6-a4af-4410-b0da-38e9c1ffe07e")]
impl Message<Signed<UseTool>> for GatewayActor {
    type Reply = Result<String>;

    async fn handle(
        &mut self,
        msg: Signed<UseTool>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if !msg.verify_signature() {
            return Err(eyre!("Invalid signature").into());
        }
        Ok(self.tool_executor.ask(msg.into_inner()).await?)
    }
}

#[remote_message("e8f5abf6-a4af-4410-b0da-38e9c1ffe08e")]
impl Message<Signed<AgentResponseEvent>> for GatewayActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: Signed<AgentResponseEvent>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if !msg.verify_signature() {
            return Err(eyre!("Invalid signature").into());
        }
        self.bus.tell(Publish(msg)).await.ok();
        Ok(())
    }
}

#[remote_message("e8f5abf6-a4af-4410-b0da-38e9c1ffe09e")]
impl Message<Signed<AgentRequest>> for GatewayActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: Signed<AgentRequest>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if !msg.verify_signature() {
            return Err(eyre!("Invalid signature").into());
        }
        self.bus.tell(Publish(msg)).await;
        Ok(())
    }
}

task_impl!(Result<String>, "9593a23b-71ea-4d50-91e8-a905784628e4");
task_impl!(
    Result<Vec<ToolDefinition>>,
    "9593a23b-71ea-4d50-91e8-a905784628e4"
);

impl Message<SaveTask> for GatewayActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SaveTask,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.active_tasks.insert(msg.task_id, msg.sender);
    }
}
