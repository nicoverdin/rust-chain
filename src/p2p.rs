use libp2p::{
    gossipsub, mdns, noise, 
    swarm::{NetworkBehaviour, SwarmEvent, Config}, 
    tcp, yamux, PeerId, SwarmBuilder,
    Transport
};
use libp2p::futures::StreamExt;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::{Mutex, mpsc, oneshot};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::chain::Blockchain;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkMessage {
    NewBlock { data: String },
    NewTransaction { data: String },
    FullChain { data: String },
}

#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub async fn start_network(
    chain_shared: Arc<Mutex<Blockchain>>,
    mut p2p_receiver: mpsc::Receiver<NetworkMessage>,
    init_signal: oneshot::Sender<()>
) -> Result<(), Box<dyn Error>> {

    let id_keys = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    println!("Node Peer ID: {}", peer_id);

    let _ = init_signal.send(());

    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(&id_keys).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    let message_id_fn = |message: &gossipsub::Message| {
        let mut s = DefaultHasher::new();
        message.data.hash(&mut s);
        gossipsub::MessageId::from(s.finish().to_string())
    };

    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .message_id_fn(message_id_fn)
        .build()
        .expect("Valid config");

    let mut gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    ).expect("Correct config");

    let topic_blocks = gossipsub::IdentTopic::new("blocks");
    let topic_txs = gossipsub::IdentTopic::new("transactions");
    let topic_sync = gossipsub::IdentTopic::new("sync");
    
    gossipsub.subscribe(&topic_blocks)?;
    gossipsub.subscribe(&topic_txs)?;
    gossipsub.subscribe(&topic_sync)?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_other_transport(|_key| transport)?
        .with_dns()?
        .with_behaviour(|_key| {
            AppBehaviour { gossipsub, mdns }
        })?
        .with_swarm_config(|cfg: Config| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    loop {
        tokio::select! {
            Some(msg) = p2p_receiver.recv() => {
                let msg_json = serde_json::to_string(&msg).expect("Json Error");
                
                let topic = match msg {
                    NetworkMessage::NewBlock { .. } => topic_blocks.clone(),
                    NetworkMessage::NewTransaction { .. } => topic_txs.clone(),
                    NetworkMessage::FullChain { .. } => topic_sync.clone(),
                };

                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, msg_json.as_bytes()) {
                    println!("Error publishing message: {:?}", e);
                }
            }

            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                
                SwarmEvent::Behaviour(AppBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                },

                SwarmEvent::Behaviour(AppBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: _,
                    message,
                })) => {
                    let msg_json = String::from_utf8_lossy(&message.data);
                    
                    if let Ok(net_msg) = serde_json::from_str::<NetworkMessage>(&msg_json) {
                        match net_msg {
                            NetworkMessage::NewBlock { data } => {
                                if let Ok(block) = serde_json::from_str::<crate::block::Block>(&data) {
                                    println!("\n📦 Received Block #{} from {}.", block.height, peer_id);
                                    let mut chain = chain_shared.lock().await;
                                    if chain.receive_block(block) {
                                        println!("   Block accepted.");
                                    } else {
                                        println!("   Block rejected.");
                                    }
                                }
                            },
                            NetworkMessage::NewTransaction { data } => {
                                if let Ok(tx) = serde_json::from_str::<Transaction>(&data) {
                                    println!("\nReceived Tx from {}.", peer_id);
                                    let mut chain = chain_shared.lock().await;
                                    chain.add_transaction(tx);
                                }
                            },
                            NetworkMessage::FullChain { data } => {
                                if let Ok(remote_blocks) = serde_json::from_str::<Vec<crate::block::Block>>(&data) {
                                    println!("\nReceived Full Chain (Height: {}) from {}.", remote_blocks.len(), peer_id);
                                    let mut chain = chain_shared.lock().await;
                                    if chain.replace_chain(remote_blocks) {
                                        println!("   CHAIN SYNC COMPLETE: Local chain replaced.");
                                    } else {
                                        println!("   Chain rejected.");
                                    }
                                }
                            }
                        }
                    }
                },

                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                },

                SwarmEvent::NewListenAddr { .. } => {
                    // SILENCED
                },
                _ => {}
            }
        }
    }
}