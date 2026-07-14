use libp2p::{
    gossipsub, identify, identity, kad, noise, ping, request_response, tcp, yamux, Multiaddr, PeerId, SwarmBuilder, StreamProtocol
};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TOPIC_RECRUITMENT: &str = "/mydna/recruitment/1.0.0";
const TOPIC_FREELANCE: &str = "/mydna/freelance/1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchIntent {
    pub peer_id: String,
    pub is_recruiting: bool,
    pub is_looking_for_job: bool,
    pub is_hiring_freelancer: bool,
    pub is_freelancing: bool,
    pub contact_email: String,
    pub skills: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRequest {
    pub from_peer_id: String,
    pub topic: String,
    pub matched_skills: Vec<String>,
    pub contact_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResponse {
    pub accepted: bool,
    pub my_contact_email: String,
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyDnaBehaviourEvent")]
struct MyDnaBehaviour {
    identify: identify::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    ping: ping::Behaviour,
    gossipsub: gossipsub::Behaviour,
    reqres: request_response::cbor::Behaviour<MatchRequest, MatchResponse>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
enum MyDnaBehaviourEvent {
    Identify(identify::Event),
    Kademlia(kad::Event),
    Ping(ping::Event),
    Gossipsub(gossipsub::Event),
    RequestResponse(request_response::Event<MatchRequest, MatchResponse>),
}

impl From<identify::Event> for MyDnaBehaviourEvent {
    fn from(e: identify::Event) -> Self { Self::Identify(e) }
}
impl From<kad::Event> for MyDnaBehaviourEvent {
    fn from(e: kad::Event) -> Self { Self::Kademlia(e) }
}
impl From<ping::Event> for MyDnaBehaviourEvent {
    fn from(e: ping::Event) -> Self { Self::Ping(e) }
}
impl From<gossipsub::Event> for MyDnaBehaviourEvent {
    fn from(e: gossipsub::Event) -> Self { Self::Gossipsub(e) }
}
impl From<request_response::Event<MatchRequest, MatchResponse>> for MyDnaBehaviourEvent {
    fn from(e: request_response::Event<MatchRequest, MatchResponse>) -> Self { Self::RequestResponse(e) }
}

pub struct P2pNetworkManager;

impl P2pNetworkManager {
    pub async fn start_node(port: u16, bootstrap_nodes: Vec<String>, private_key_bytes: &mut [u8; 32]) -> Result<(), String> {
        let local_key = identity::Keypair::ed25519_from_bytes(private_key_bytes)
            .map_err(|e| format!("Failed to parse private key: {}", e))?;
        let local_peer_id = PeerId::from(local_key.public());
        println!("[P2P] Local Node ID: {}", local_peer_id);

        let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
            .with_tokio()
            .with_tcp(
                tcp::Config::default().nodelay(true),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| format!("Transport error: {}", e))?
            .with_behaviour(|key| {
                let peer_id = PeerId::from(key.public());
                let store = kad::store::MemoryStore::new(peer_id);
                let mut kademlia = kad::Behaviour::new(peer_id, store);
                kademlia.set_mode(Some(kad::Mode::Server));

                let gossipsub_config = gossipsub::ConfigBuilder::default()
                    .validation_mode(gossipsub::ValidationMode::Permissive)
                    .build()
                    .unwrap();
                let mut gossipsub = gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub_config,
                ).unwrap();

                gossipsub.subscribe(&gossipsub::IdentTopic::new(TOPIC_RECRUITMENT)).unwrap();
                gossipsub.subscribe(&gossipsub::IdentTopic::new(TOPIC_FREELANCE)).unwrap();

                let reqres_protocols = [(
                    StreamProtocol::new("/mydna/match/1.0.0"),
                    request_response::ProtocolSupport::Full,
                )];
                let reqres = request_response::cbor::Behaviour::<MatchRequest, MatchResponse>::new(
                    reqres_protocols,
                    request_response::Config::default(),
                );

                let identify = identify::Behaviour::new(identify::Config::new(
                    "/mydna/1.0.0".to_string(),
                    key.public(),
                ));

                Ok(MyDnaBehaviour {
                    identify,
                    kademlia,
                    ping: ping::Behaviour::new(ping::Config::new()),
                    gossipsub,
                    reqres,
                })
            })
            .map_err(|e| format!("Behaviour error: {}", e))?
            .build();

        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", port).parse().unwrap();
        swarm.listen_on(listen_addr).unwrap();

        for raw_addr in bootstrap_nodes {
            if let Ok(addr) = raw_addr.parse::<Multiaddr>() {
                println!("[P2P] Dialing Bootstrap Node: {}", addr);
                let _ = swarm.dial(addr);
            }
        }

        tokio::spawn(async move {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("[P2P] Listening on {:?}", address);
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("[P2P] Connected to peer: {}", peer_id);
                    }
                    SwarmEvent::Behaviour(MyDnaBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source,
                        message,
                        ..
                    })) => {
                        let topic = message.topic.to_string();
                        println!("[P2P] Received Gossip on Topic: {}", topic);
                        
                        if let Ok(intent) = serde_json::from_slice::<MatchIntent>(&message.data) {
                            println!("[P2P] Extracted Match Intent from {}: {:?}", intent.peer_id, intent.contact_email);
                            
                            // Tự động phân tích nhu cầu chéo (Cross-match logic)
                            // Ví dụ: B đang cần thuê freelancer, còn A (mình) đang làm freelancer
                            let is_match = (topic == TOPIC_FREELANCE && intent.is_hiring_freelancer)
                                        || (topic == TOPIC_RECRUITMENT && intent.is_recruiting);
                            
                            if is_match {
                                println!("[P2P] MATCH FOUND with {}! Bắt tay 1-1...", intent.contact_email);
                                
                                // Gửi Request-Response trực tiếp đến node đó
                                if let Ok(target_peer) = intent.peer_id.parse::<PeerId>() {
                                    let req = MatchRequest {
                                        from_peer_id: local_peer_id.to_string(),
                                        topic: topic.clone(),
                                        matched_skills: intent.skills.clone(),
                                        contact_email: "my.email@gmail.com".to_string(), // TODO: Fetch real email from OAuth / Local DB
                                    };
                                    swarm.behaviour_mut().reqres.send_request(&target_peer, req);
                                }
                            }
                        }
                    }
                    SwarmEvent::Behaviour(MyDnaBehaviourEvent::RequestResponse(request_response::Event::Message { peer, message })) => {
                        match message {
                            request_response::Message::Request { request_id, request, channel } => {
                                println!("[P2P] Received Direct Match Proposal from {}: {:?}", peer, request);
                                // TODO: Present to UI or Auto Accept if skills align
                            }
                            request_response::Message::Response { request_id, response } => {
                                println!("[P2P] Received Match Response: {:?}", response);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
}
