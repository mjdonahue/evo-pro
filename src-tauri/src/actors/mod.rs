use std::collections::{HashMap, HashSet};

use color_eyre::eyre::eyre;
use futures_util::{StreamExt, stream};
use kameo::{
    prelude::{ActorRef as LocalActorRef, *},
    remote::{ActorSwarmBehaviour, ActorSwarmBehaviourEvent, ActorSwarmEvent, ActorSwarmHandler},
};
use kameo_actors::{
    DeliveryStrategy,
    message_bus::{MessageBus, Register},
    pool::ActorPool,
};
use libp2p::{
    Multiaddr, Swarm, SwarmBuilder, dcutr, identify,
    multiaddr::Protocol,
    noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    yamux,
};
use serde::{Serialize, de::DeserializeOwned};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info};

// NEW: Define your bootstrap nodes. In a real app, this would come from a config file.
const BOOTSTRAP_NODES: &[&str] = &["/ip4/150.136.100.92/udp/4001/quic-v1"];

pub mod agents;
pub mod conversation;
pub mod database;
pub mod fault_detection;
pub mod gateway;
pub mod ipc;
pub mod lifecycle;
pub mod lifecycle_utils;
pub mod metrics;
pub mod supervision;
pub mod supervision_tree;
pub mod swarm;
pub mod tools;
pub mod ui_notifier;
pub mod websocket;

#[macro_export]
macro_rules! signed_impl {
    ($name:ty, $actor:ty) => {
        impl ::kameo::prelude::Message<$crate::keys::Signed<$name>> for $actor {
            type Reply = ();

            async fn handle(
                &mut self,
                msg: $crate::keys::Signed<$name>,
                ctx: &mut Context<Self, Self::Reply>,
            ) -> Self::Reply {
                use $crate::{
                    actors::{GatewayActor, RemoteActorRef},
                    error::AppError,
                    keys::Signed,
                    utils::get_gateway_id,
                };
                let peer_id = msg.client_peer_id().clone();
                let task_id = msg.task_id().cloned();
                let msg = msg.into_inner();
                let Ok(Some(remote)) =
                    RemoteActorRef::<GatewayActor>::lookup(&get_gateway_id(&peer_id)).await
                else {
                    // We couldn't make the connnection back to the gateway
                    // and this will always be sent by a `tell` call so just retur
                    return;
                };
                let actor_ref = ctx.actor_ref();
                tokio::task::spawn(async move {
                    let out = actor_ref
                        .ask(msg)
                        .await
                        .map_err(|e| AppError::SendError(format!("Could not send message!: {e}")));
                    let signed = Signed::with_task(out, task_id);
                    remote.tell(&signed).await.ok();
                });
            }
        }
    };
}

#[macro_export]
macro_rules! register_actor {
    ($bus:expr, $actor:expr, [$($msg_type:ty),+]) => {
        {
            $(
                $bus.tell(kameo_actors::message_bus::Register(
                    $actor.clone().recipient::<$msg_type>(),
                ))
                .await?;
            )+
        }
    };
}

use crate::{
    actors::{
        agents::{AgentActor, AgentManagerActor, AgentResponseEvent},
        conversation::{ConversationManagerActor, SendMessage},
        database::DatabaseActor,
        gateway::{GATEWAY_ACTOR, GatewayActor},
        swarm::{
            Behaviour, ConnectionClosed, ConnectionEstablished, ConnectionManager, swarm_handler,
        },
        tools::ToolExecutorActor,
        ui_notifier::UINotifierActor,
    },
    error::Result,
    keys::{PEER_ID, Signed, fetch_peer_keypair},
    repositories::RepositoryFactory,
    state::ActorManager,
    storage::db::DatabaseManager,
};

/// A trait to ensure that a reply type is a subtype of the actual reply type.
/// And that the actual reply is able to be sent over the peer-to-peer network.
pub trait Askable<T: Send + 'static>: Message<T> {
    type ActualReply: Serialize + DeserializeOwned + Send + Sync + 'static;
}

pub type SystemEventBus = MessageBus;

pub async fn setup_actors(handle: AppHandle, db: DatabaseManager) -> Result<ActorManager> {
    let key_pair = fetch_peer_keypair();
    let mut swarm = SwarmBuilder::with_existing_identity(key_pair)
        .with_tokio()
        .with_quic()
        .with_dns()?
        .with_relay_client(noise::Config::new, yamux::Config::default)
        .expect("Failed to initialize relay client")
        .with_behaviour(|keypair, relay_behaviour| {
            Ok(Behaviour {
                kameo: ActorSwarmBehaviour::new(keypair)?,
                relay_client: relay_behaviour,
                ping: ping::Behaviour::new(ping::Config::new()),
                identify: identify::Behaviour::new(identify::Config::new(
                    format!("/evo-design/{}", env!("CARGO_PKG_VERSION")),
                    keypair.public(),
                )),
                dcutr: dcutr::Behaviour::new(keypair.public().to_peer_id()),
            })
        })
        .expect("Failed to initialize behaviour")
        .build();
    let (actor_swarm, mut handler) = ActorSwarm::bootstrap_manual(*swarm.local_peer_id())
        .ok_or(eyre!("Failed to bootstrap p2p swarm!"))?;
    let system_event_bus_ref =
        SystemEventBus::spawn(SystemEventBus::new(DeliveryStrategy::BestEffort));

    // Initialize repository factory with database pool
    let repo_factory = RepositoryFactory::new(db.pool.clone());

    // Initialize database actor with db and repository factory
    let db_actor = DatabaseActor::spawn(DatabaseActor { 
        db,
        repo_factory,
    });
    let agent_manager = AgentManagerActor::spawn(AgentManagerActor {
        bus: system_event_bus_ref.clone(),
        pool: ActorPool::spawn(ActorPool::new(8, {
            let bus = system_event_bus_ref.clone();
            move || AgentActor::spawn(AgentActor { bus: bus.clone() })
        })),
    });
    let tool_executor = ToolExecutorActor::spawn(ToolExecutorActor {
        tools: HashMap::new(),
    });
    let conversation_manager = ConversationManagerActor::spawn(ConversationManagerActor {
        agent_manager: agent_manager.clone(),
        db: db_actor.clone(),
        conversation_peers: HashMap::new(),
    });
    let ui_notifier = UINotifierActor::spawn(UINotifierActor {
        handle: handle.clone(),
    });
    PEER_ID.set(*actor_swarm.local_peer_id()).ok();
    let gateway = GatewayActor::spawn(GatewayActor {
        db: db_actor.clone(), // Use db_actor here
        bus: system_event_bus_ref.clone(),
        agent_manager: agent_manager.clone(),
        tool_executor: tool_executor.clone(),
        active_tasks: HashMap::new(),
    });
    let connection_manager = ConnectionManager::spawn(ConnectionManager {
        active_connections: HashSet::new(),
    });
    register_actor!(
        system_event_bus_ref,
        conversation_manager,
        [AgentResponseEvent]
    );
    register_actor!(system_event_bus_ref, ui_notifier, [AgentResponseEvent, Signed<AgentResponseEvent>, SendMessage, Signed<SendMessage>]);
    register_actor!(
        system_event_bus_ref,
        connection_manager,
        [ConnectionEstablished, ConnectionClosed]
    );
    GATEWAY_ACTOR.set(gateway.clone()).ok();
    gateway
        .register(&format!("gateway-{}", &PEER_ID.get().unwrap()))
        .await?;
    // MODIFIED: Listen on dynamic ports instead of a fixed one.
    actor_swarm
        .listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse().unwrap())
        .await?;
    actor_swarm
        .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
        .await?;

    let manager = ActorManager {
        bus: system_event_bus_ref,
        db: db_actor,
        agent_manager: agent_manager,
        tool_ref: tool_executor,
        conversation_manager,
    };

    // NEW: Dial the bootstrap nodes in a background task to join the network.
    tokio::spawn(async move {
        stream::iter(
            BOOTSTRAP_NODES
                .iter()
                .filter_map(|addr| addr.parse::<Multiaddr>().ok()),
        )
        .for_each_concurrent(10, |multiaddr| async {
            let listen_addr = multiaddr.with(Protocol::P2pCircuit);
            if let Err(e) = actor_swarm.listen_on(listen_addr.clone()).await {
                error!("Failed to listen on bootstrap node: {}", e);
            }
            info!("Listening on bootstrap node: {}", listen_addr);
        })
        .await;
    });
    // Start the swarm handler in a separate thread
    tokio::spawn({
        let manager = manager.clone();
        async move { swarm_handler(&mut swarm, &mut handler, manager).await }
    });

    Ok(manager)
}

pub enum ActorRef<A: Actor> {
    Local(LocalActorRef<A>),
    Remote(RemoteActorRef<GatewayActor>),
}

impl<A: Actor> Clone for ActorRef<A> {
    fn clone(&self) -> Self {
        match self {
            ActorRef::Local(local) => ActorRef::Local(local.clone()),
            ActorRef::Remote(remote) => ActorRef::Remote(remote.clone()),
        }
    }
}

impl<A: Actor> ActorRef<A> {
    pub fn id(&self) -> ActorID {
        match self {
            ActorRef::Local(local) => local.id(),
            ActorRef::Remote(remote) => remote.id(),
        }
    }
}
