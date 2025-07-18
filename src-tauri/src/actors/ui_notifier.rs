use kameo::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::{
    actors::{agents::AgentResponseEvent, conversation::SendMessage},
    keys::Signed,
};

#[derive(Actor)]
pub struct UINotifierActor {
    pub handle: AppHandle,
}

impl Message<AgentResponseEvent> for UINotifierActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: AgentResponseEvent,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle.emit("agent-response", msg).ok();
    }
}

impl Message<Signed<AgentResponseEvent>> for UINotifierActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: Signed<AgentResponseEvent>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle.emit("agent-response", msg.into_inner()).ok();
    }
}

impl Message<SendMessage> for UINotifierActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: SendMessage,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle.emit("send-message", msg).ok();
    }
}

impl Message<Signed<SendMessage>> for UINotifierActor {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: Signed<SendMessage>,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.handle.emit("send-message", msg.into_inner()).ok();
    }
}
