use kameo_actors::message_bus::Publish;
use std::{collections::HashSet, io, num::NonZero, time::Duration};
use tracing::debug;

use futures_util::StreamExt;
use kameo::{
    Actor,
    prelude::*,
    remote::{
        ActorSwarmBehaviour, ActorSwarmBehaviourEvent, ActorSwarmEvent, ActorSwarmHandler,
        SwarmBehaviour,
    },
};
use libp2p::{
    Multiaddr, PeerId, Swarm, TransportError,
    core::ConnectedPoint,
    dcutr, identify, ping, relay,
    swarm::{ConnectionId, NetworkBehaviour, SwarmEvent},
};

use crate::state::ActorManager;

#[derive(NetworkBehaviour)]
pub struct Behaviour {
    pub kameo: ActorSwarmBehaviour,
    pub dcutr: dcutr::Behaviour,
    pub identify: identify::Behaviour,
    pub relay_client: relay::client::Behaviour,
    pub ping: ping::Behaviour,
}

impl SwarmBehaviour for Behaviour {
    fn ask(
        &mut self,
        peer: &libp2p::PeerId,
        actor_id: kameo::prelude::ActorID,
        actor_remote_id: std::borrow::Cow<'static, str>,
        message_remote_id: std::borrow::Cow<'static, str>,
        payload: Vec<u8>,
        mailbox_timeout: Option<std::time::Duration>,
        reply_timeout: Option<std::time::Duration>,
        immediate: bool,
    ) -> libp2p::request_response::OutboundRequestId {
        self.kameo.ask(
            peer,
            actor_id,
            actor_remote_id,
            message_remote_id,
            payload,
            mailbox_timeout,
            reply_timeout,
            immediate,
        )
    }

    fn tell(
        &mut self,
        peer: &libp2p::PeerId,
        actor_id: kameo::prelude::ActorID,
        actor_remote_id: std::borrow::Cow<'static, str>,
        message_remote_id: std::borrow::Cow<'static, str>,
        payload: Vec<u8>,
        mailbox_timeout: Option<std::time::Duration>,
        immediate: bool,
    ) -> libp2p::request_response::OutboundRequestId {
        self.kameo.tell(
            peer,
            actor_id,
            actor_remote_id,
            message_remote_id,
            payload,
            mailbox_timeout,
            immediate,
        )
    }

    fn link(
        &mut self,
        actor_id: kameo::prelude::ActorID,
        actor_remote_id: std::borrow::Cow<'static, str>,
        sibbling_id: kameo::prelude::ActorID,
        sibbling_remote_id: std::borrow::Cow<'static, str>,
    ) -> libp2p::request_response::OutboundRequestId {
        self.kameo
            .link(actor_id, actor_remote_id, sibbling_id, sibbling_remote_id)
    }

    fn unlink(
        &mut self,
        actor_id: kameo::prelude::ActorID,
        actor_remote_id: std::borrow::Cow<'static, str>,
        sibbling_id: kameo::prelude::ActorID,
    ) -> libp2p::request_response::OutboundRequestId {
        self.kameo.unlink(actor_id, actor_remote_id, sibbling_id)
    }

    fn signal_link_died(
        &mut self,
        dead_actor_id: kameo::prelude::ActorID,
        notified_actor_id: kameo::prelude::ActorID,
        notified_actor_remote_id: std::borrow::Cow<'static, str>,
        stop_reason: kameo::prelude::ActorStopReason,
    ) -> libp2p::request_response::OutboundRequestId {
        self.kameo.signal_link_died(
            dead_actor_id,
            notified_actor_id,
            notified_actor_remote_id,
            stop_reason,
        )
    }

    fn send_ask_response(
        &mut self,
        channel: libp2p::request_response::ResponseChannel<kameo::remote::SwarmResponse>,
        result: Result<Vec<u8>, kameo::prelude::RemoteSendError<Vec<u8>>>,
    ) -> Result<(), kameo::remote::SwarmResponse> {
        self.kameo.send_ask_response(channel, result)
    }

    fn send_tell_response(
        &mut self,
        channel: libp2p::request_response::ResponseChannel<kameo::remote::SwarmResponse>,
        result: Result<(), kameo::prelude::RemoteSendError>,
    ) -> Result<(), kameo::remote::SwarmResponse> {
        self.kameo.send_tell_response(channel, result)
    }

    fn send_link_response(
        &mut self,
        channel: libp2p::request_response::ResponseChannel<kameo::remote::SwarmResponse>,
        result: Result<(), kameo::prelude::RemoteSendError<kameo::error::Infallible>>,
    ) -> Result<(), kameo::remote::SwarmResponse> {
        self.kameo.send_link_response(channel, result)
    }

    fn send_unlink_response(
        &mut self,
        channel: libp2p::request_response::ResponseChannel<kameo::remote::SwarmResponse>,
        result: Result<(), kameo::prelude::RemoteSendError<kameo::error::Infallible>>,
    ) -> Result<(), kameo::remote::SwarmResponse> {
        self.kameo.send_unlink_response(channel, result)
    }

    fn send_signal_link_died_response(
        &mut self,
        channel: libp2p::request_response::ResponseChannel<kameo::remote::SwarmResponse>,
        result: Result<(), kameo::prelude::RemoteSendError<kameo::error::Infallible>>,
    ) -> Result<(), kameo::remote::SwarmResponse> {
        self.kameo.send_signal_link_died_response(channel, result)
    }

    fn kademlia_add_address(
        &mut self,
        peer: &libp2p::PeerId,
        address: libp2p::Multiaddr,
    ) -> libp2p::kad::RoutingUpdate {
        self.kameo.kademlia_add_address(peer, address)
    }

    fn kademlia_set_mode(&mut self, mode: Option<libp2p::kad::Mode>) {
        self.kameo.kademlia_set_mode(mode);
    }

    fn kademlia_get_record(&mut self, key: libp2p::kad::RecordKey) -> libp2p::kad::QueryId {
        self.kameo.kademlia_get_record(key)
    }

    fn kademlia_get_record_local(
        &mut self,
        key: &libp2p::kad::RecordKey,
    ) -> Option<std::borrow::Cow<'_, libp2p::kad::Record>> {
        self.kameo.kademlia_get_record_local(key)
    }

    fn kademlia_put_record(
        &mut self,
        record: libp2p::kad::Record,
        quorum: libp2p::kad::Quorum,
    ) -> Result<libp2p::kad::QueryId, libp2p::kad::store::Error> {
        self.kameo.kademlia_put_record(record, quorum)
    }

    fn kademlia_put_record_local(
        &mut self,
        record: libp2p::kad::Record,
    ) -> Result<(), libp2p::kad::store::Error> {
        self.kameo.kademlia_put_record_local(record)
    }

    fn kademlia_remove_record(&mut self, key: &libp2p::kad::RecordKey) {
        self.kameo.kademlia_remove_record(key);
    }

    fn kademlia_remove_record_local(&mut self, key: &libp2p::kad::RecordKey) {
        self.kameo.kademlia_remove_record_local(key);
    }
}

pub async fn swarm_handler(
    swarm: &mut Swarm<Behaviour>,
    handler: &mut ActorSwarmHandler,
    actors: ActorManager,
) {
    loop {
        tokio::select! {
            event = swarm.next() => {
                match event.unwrap() {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        tracing::info!(%address, "Listening on address");
                    }
                    event => panic!("{event:?}"),
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // Likely listening on all interfaces now, thus continuing by breaking the loop.
                break;
            }
        }
    }
    loop {
        tokio::select! {
            Some(cmd) = handler.next_command() => handler.handle_command(swarm, cmd),
            Some(event) = swarm.next() => {
                match event {
                    SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {
                        actors.bus.tell(Publish(ConnectionClosed {
                            peer_id: peer_id.clone(),
                            connection_id: connection_id.clone(),
                            endpoint: endpoint.clone(),
                            num_established: num_established.clone(),
                        })).await.ok();
                        debug!("--> Disconnected from peer: {}. Caused by: {:?}", peer_id, cause);
                        handler.handle_event(swarm, ActorSwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause });
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::RelayClient(
                        relay::client::Event::ReservationReqAccepted { .. }
                    )) => {
                        debug!("✅ Relay accepted our reservation request. We are now publicly reachable.");
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Dcutr(dcutr::Event{ result, .. })) => {
                        debug!("Hole punching event: {:?}", result);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => {
                        actors.bus.tell(Publish(ConnectionEstablished {
                            peer_id,
                            connection_id,
                            endpoint,
                            num_established,
                            established_in,
                        })).await.ok();
                        debug!("--> Connected to peer: {}. Concurrent dial errors: {:?}", peer_id, concurrent_dial_errors);
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Kameo(ActorSwarmBehaviourEvent::Kademlia(event))) => {
                        handler.handle_event(swarm, ActorSwarmEvent::Behaviour(Box::new(ActorSwarmBehaviourEvent::Kademlia(event))));
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Kameo(ActorSwarmBehaviourEvent::RequestResponse(event))) => {
                        handler.handle_event(swarm, ActorSwarmEvent::Behaviour(Box::new(ActorSwarmBehaviourEvent::RequestResponse(event))));
                    }
                    SwarmEvent::Behaviour(BehaviourEvent::Kameo(ActorSwarmBehaviourEvent::Mdns(event))) => {
                        handler.handle_event(swarm, ActorSwarmEvent::Behaviour(Box::new(ActorSwarmBehaviourEvent::Mdns(event))));
                    }
                    _ => {},
                }
            }
            else => unreachable!("actor swarm should never stop since its stored globally and will never return None when progressing"),
        }
    }
}

#[derive(Clone)]
pub struct ConnectionEstablished {
    peer_id: PeerId,
    connection_id: ConnectionId,
    endpoint: ConnectedPoint,
    num_established: NonZero<u32>,
    established_in: Duration,
}

#[derive(Clone)]
pub struct ConnectionClosed {
    peer_id: PeerId,
    connection_id: ConnectionId,
    endpoint: ConnectedPoint,
    num_established: u32,
}

#[derive(Actor)]
pub struct ConnectionManager {
    pub active_connections: HashSet<PeerId>,
}

pub struct GetInactiveConnections {
    pub query: HashSet<PeerId>,
}

impl Message<GetInactiveConnections> for ConnectionManager {
    type Reply = HashSet<PeerId>;

    async fn handle(
        &mut self,
        msg: GetInactiveConnections,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        msg.query
            .difference(&self.active_connections)
            .cloned()
            .collect()
    }
}

impl Message<ConnectionEstablished> for ConnectionManager {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ConnectionEstablished,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.active_connections.insert(msg.peer_id);
    }
}

impl Message<ConnectionClosed> for ConnectionManager {
    type Reply = ();

    async fn handle(
        &mut self,
        msg: ConnectionClosed,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.active_connections.remove(&msg.peer_id);
    }
}
