use std::collections::HashMap;

use color_eyre::eyre::eyre;
use futures_util::{StreamExt, stream};
use kameo::prelude::{ActorRef as LocalActorRef, *};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    actors::{
        ActorRef,
        agents::{AgentManagerActor, AgentResponseEvent, StreamedPart},
        database::{DatabaseActor, GetConversationParticipantIds},
        gateway::GatewayActor,
        tools::ToolExecutorActor,
    },
    entities::{
        Conversation, ConversationStatus, ConversationType, CreateConversation,
        CreateConversationParticipant, Message as ChatMessage, ParticipantType,
    },
    error::Result,
    keys::Signed,
    utils::get_gateway_id,
};

#[derive(Actor)]
pub struct ConversationManagerActor {
    pub agent_manager: LocalActorRef<AgentManagerActor>,
    pub db: LocalActorRef<DatabaseActor>,
    pub conversation_peers: HashMap<Uuid, Vec<RemoteActorRef<GatewayActor>>>,
}

impl Message<SendMessage> for ConversationManagerActor {
    type Reply = DelegatedReply<Result<()>>;

    async fn handle(
        &mut self,
        msg: SendMessage,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let (delegated, sender) = ctx.reply_sender();
        tokio::spawn({
            let db = self.db.clone();
            let agent_manager = self.agent_manager.clone();
            async move {
                let res = async {
                    let conversation_id = msg.conversation_id;
                    let participants = db
                        .ask(GetConversationParticipantIds(conversation_id))
                        .await?;
                    for p in participants {
                        match p {
                            ParticipantType::User(id) => {
                                todo!()
                            }
                            ParticipantType::Agent(id) => {
                                todo!()
                            }
                            ParticipantType::Contact(id) => {
                                todo!()
                            }
                            _ => {} // Ignore system participants
                        }
                    }
                    Ok(())
                }
                .await;
                if let Some(tx) = sender {
                    tx.send(res);
                }
            }
        });
        delegated
    }
}

impl Message<Signed<ChatMessage>> for ConversationManagerActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: Signed<ChatMessage>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let msg = msg.into_inner();
        // self.db.ask(msg).await?;

        todo!()
    }
}

impl Message<InviteParticipants> for ConversationManagerActor {
    type Reply = Result<()>;

    async fn handle(
        &mut self,
        msg: InviteParticipants,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        todo!()
    }
}

impl Message<AgentResponseEvent> for ConversationManagerActor {
    type Reply = DelegatedReply<()>;

    async fn handle(
        &mut self,
        msg: AgentResponseEvent,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let (delegated, sender) = ctx.reply_sender();
        let participants = match self.conversation_peers.get(&msg.conversation_id).cloned() {
            Some(participants) => participants,
            None => {
                let Ok(participants) = self
                    .db
                    .ask(GetConversationParticipantIds(msg.conversation_id))
                    .await
                else {
                    if let Some(tx) = sender {
                        tx.send(());
                    }
                    return delegated;
                };
                todo!()
            }
        };
        let signed = Signed::new(msg);
        tokio::spawn(async move {
            stream::iter(participants).for_each_concurrent(5, |p| {
                let signed = signed.clone();
                async move {
                    p.tell(&signed).await.ok();
                }
            });
        });
        delegated
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SendMessage {
    /// The id of the conversation to send the message to
    /// If None, a new conversation will be created
    pub conversation_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub content: String,
    pub type_: ParticipantType,
}

#[derive(Clone)]
pub struct InviteParticipants {
    pub conversation_id: Uuid,
    pub participants: Vec<Uuid>,
}
