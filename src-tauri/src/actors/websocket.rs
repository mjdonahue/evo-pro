use futures_util::stream::{SplitSink, SplitStream};
use kameo::prelude::{ActorRef as LocalActorRef, *};
use serde::{Deserialize, Serialize};
use tokio::net::TcpSocket;
use tokio_tungstenite::WebSocketStream;

use crate::actors::{SystemEventBus, conversation::SendMessage};

#[derive(Actor)]
pub struct WesocketMagnagerActor {
    bus: LocalActorRef<SystemEventBus>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WebsocketRequest {
    SendMessage(SendMessage),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WebsocketResponse {
    MessageSent,
}

#[derive(Actor)]
pub struct WebsocketReceiver {
    send: LocalActorRef<WebsocketSender>,
    recv: SplitStream<WebsocketRequest>,
    bus: LocalActorRef<SystemEventBus>,
}

#[derive(Actor)]
pub struct WebsocketSender {
    send: SplitSink<WebSocketStream<TcpSocket>, WebsocketResponse>,
    bus: LocalActorRef<SystemEventBus>,
}
