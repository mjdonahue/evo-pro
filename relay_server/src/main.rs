use futures_util::StreamExt;
use libp2p::{
    SwarmBuilder, identity,
    multiaddr::{Multiaddr, Protocol},
    ping,
    relay::Behaviour as RelayServerBehaviour,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use std::error::Error;
use tracing::info;

// The NetworkBehaviour for our relay node.
// It only needs the relay server, identify to tell others its address,
// and ping for keep-alives. It does NOT need dcutr or a relay client.
#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    relay: RelayServerBehaviour,
    ping: ping::Behaviour,
    identify: libp2p::identify::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    // Create a stable identity for the relay.
    // In a real deployment, you would load this from a file to have a persistent PeerId.
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = local_key.public().to_peer_id();
    info!("Relay PeerId: {}", local_peer_id);

    let mut swarm = SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(
            Default::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let identify = libp2p::identify::Behaviour::new(libp2p::identify::Config::new(
                "/evo-relay/1.0.0".to_string(),
                key.public(),
            ));
            RelayBehaviour {
                relay: RelayServerBehaviour::new(key.public().to_peer_id(), Default::default()),
                ping: ping::Behaviour::new(ping::Config::new()),
                identify,
            }
        })?
        .build();

    // Listen on all interfaces on a fixed port for both TCP and QUIC.
    // '0.0.0.0' makes it accessible from outside the local machine.
    let listen_addr_tcp = "/ip4/0.0.0.0/tcp/4001".parse::<Multiaddr>()?;
    let listen_addr_quic = "/ip4/0.0.0.0/udp/4001/quic-v1".parse::<Multiaddr>()?;
    swarm.listen_on(listen_addr_tcp)?;
    swarm.listen_on(listen_addr_quic)?;

    // Main event loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                // This is the crucial part. Print the full public address for clients to use.
                info!(
                    "Relay listening on: {}",
                    address.with(Protocol::P2p(local_peer_id))
                );
            }
            SwarmEvent::Behaviour(RelayBehaviourEvent::Identify(event)) => {
                info!("Identify event: {:?}", event);
            }
            SwarmEvent::Behaviour(RelayBehaviourEvent::Relay(event)) => {
                info!("Relay server event: {:?}", event);
            }
            _ => {}
        }
    }
}
