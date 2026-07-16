use libp2p::{
    gossipsub, identify, identity, kad, noise, ping, request_response, tcp, yamux, Multiaddr, PeerId, SwarmBuilder, StreamProtocol
};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use libp2p::futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const TOPIC_RECRUITMENT: &str = "/mydna/recruitment/1.0.0";
const TOPIC_FREELANCE: &str = "/mydna/freelance/1.0.0";

use crate::telemetry::integrity::{IntegrityManager, IntegritySnapshot};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillWeight {
    pub name: String,
    pub weight: f32, // 0.0 -> 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchingProfile {
    pub tech_stack: Vec<SkillWeight>,
    pub domain_knowledge: Vec<SkillWeight>,
    pub seniority_level: String,
    pub work_model: String,
    pub min_salary: Option<u32>,
    pub max_salary: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchIntent {
    pub peer_id: String,
    pub is_recruiting: bool,
    pub is_looking_for_job: bool,
    pub is_hiring_freelancer: bool,
    pub is_freelancing: bool,
    pub contact_email: String,
    pub skills: Vec<String>, // Giữ lại cho backward compatibility
    pub matching_profile: Option<MatchingProfile>, // Cấu trúc trọng số đa chiều mới
    pub integrity_snapshot: Option<IntegritySnapshot>,
    pub standard_hash: Option<String>,
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
    pub async fn start_node(port: u16, bootstrap_nodes: Vec<String>, private_key_bytes: &mut [u8; 32], mut user_intent: MatchIntent) -> Result<(), String> {
        let local_key = identity::Keypair::ed25519_from_bytes(&mut *private_key_bytes)
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

        // Generate Local Integrity Snapshot
        let my_snapshot = IntegrityManager::generate_snapshot(&local_peer_id.to_string(), private_key_bytes)
            .map_err(|e| format!("Snapshot generation error: {}", e))?;
        
        user_intent.peer_id = local_peer_id.to_string();
        user_intent.integrity_snapshot = Some(my_snapshot.clone());
        user_intent.standard_hash = crate::telemetry::standard_manager::StandardManager::get_current_standard_hash().ok();
        
        let record = kad::Record {
            key: kad::RecordKey::new(&local_peer_id.to_bytes()),
            value: serde_json::to_vec(&my_snapshot).unwrap(),
            publisher: Some(local_peer_id),
            expires: Some(std::time::Instant::now() + Duration::from_secs(7 * 24 * 3600)), // 7 days TTL
        };
        swarm.behaviour_mut().kademlia.put_record(record, kad::Quorum::One).ok();
        
        // Ghi ra đĩa để backup lên Google Drive
        let node_suffix = std::env::var("MYDNA_TEST_NODE").unwrap_or_default();
        let snapshot_path = if node_suffix.is_empty() {
            "portable-test/my_snapshot.json".to_string()
        } else {
            format!("portable-test/{}/my_snapshot.json", node_suffix)
        };
        std::fs::write(&snapshot_path, serde_json::to_string_pretty(&my_snapshot).unwrap()).ok();

        for raw_addr in bootstrap_nodes {
            if let Ok(addr) = raw_addr.parse::<Multiaddr>() {
                println!("[P2P] Dialing Bootstrap Node: {}", addr);
                let _ = swarm.dial(addr);
            }
        }

        let mut tick = tokio::time::interval(Duration::from_secs(10)); // Broadcast every 10 secs

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        let data = serde_json::to_vec(&user_intent).unwrap();
                        if user_intent.is_recruiting || user_intent.is_looking_for_job {
                            let _ = swarm.behaviour_mut().gossipsub.publish(gossipsub::IdentTopic::new(TOPIC_RECRUITMENT), data.clone());
                        }
                        if user_intent.is_hiring_freelancer || user_intent.is_freelancing {
                            let _ = swarm.behaviour_mut().gossipsub.publish(gossipsub::IdentTopic::new(TOPIC_FREELANCE), data);
                        }
                    }
                    event = swarm.select_next_some() => {
                        match event {
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
                            
                            // Verify Standard Hash
                            if let Some(hash) = &intent.standard_hash {
                                if !crate::telemetry::standard_manager::StandardManager::is_hash_allowed(hash) {
                                    println!("[P2P] SECURITY ALERT: Blocked intent from {} due to unauthorized Standard Hash!", intent.peer_id);
                                    continue;
                                }
                            } else {
                                println!("[P2P] SECURITY ALERT: Blocked intent from {} due to missing Standard Hash!", intent.peer_id);
                                continue;
                            }
                            
                            // Tự động phân tích nhu cầu chéo (Cross-match logic)
                            let mut is_match = (topic == TOPIC_FREELANCE && intent.is_hiring_freelancer)
                                        || (topic == TOPIC_RECRUITMENT && intent.is_recruiting);
                            
                            // TÍNH ĐIỂM TRỌNG SỐ ĐA CHIỀU (Advanced Weighted Scoring)
                            if is_match {
                                if let (Some(my_prof), Some(peer_prof)) = (&user_intent.matching_profile, &intent.matching_profile) {
                                    // 1. Lọc cứng (Hard Filters)
                                    // Nếu mình là Ứng viên (min_salary > 0), đối tác là HR (max_salary > 0)
                                    if my_prof.min_salary > Some(0) && peer_prof.max_salary > Some(0) && my_prof.min_salary > peer_prof.max_salary {
                                        is_match = false;
                                    } else if peer_prof.min_salary > Some(0) && my_prof.max_salary > Some(0) && peer_prof.min_salary > my_prof.max_salary {
                                        is_match = false;
                                    }

                                    if is_match && !my_prof.work_model.is_empty() && !peer_prof.work_model.is_empty() && my_prof.work_model != peer_prof.work_model {
                                        is_match = false;
                                    }

                                    // 2. Tính điểm Tech Stack (Dot Product)
                                    if is_match {
                                        let mut total_score = 0.0;
                                        let mut total_weight = 0.0;
                                        
                                        for my_skill in &my_prof.tech_stack {
                                            total_weight += my_skill.weight;
                                            if let Some(peer_skill) = peer_prof.tech_stack.iter().find(|s| s.name.to_lowercase() == my_skill.name.to_lowercase()) {
                                                total_score += my_skill.weight * peer_skill.weight;
                                            }
                                        }
                                        
                                        if total_weight > 0.0 {
                                            let match_percentage = (total_score / total_weight) * 100.0;
                                            println!("[P2P] {} vs {}: Tỉ lệ khớp = {:.2}%", user_intent.contact_email, intent.contact_email, match_percentage);
                                            // Ngưỡng chốt (Threshold): >= 60% mới bắt tay
                                            if match_percentage < 60.0 {
                                                is_match = false;
                                            }
                                        }
                                    }
                                }
                            }
                            
                            if is_match {
                                // 1. BẮT BẮT ĐẦU ĐỐI SOÁT CHÉO (CROSS-VERIFICATION)
                                let mut is_valid = true;
                                if let Some(snapshot) = &intent.integrity_snapshot {
                                    // Ở đây chúng ta tạm dùng public key từ peer_id. Thực tế cần cơ chế trao đổi pubkey an toàn hơn.
                                    // Hoặc query Kademlia DHT: swarm.behaviour_mut().kademlia.get_record(kad::RecordKey::new(&target_peer.to_bytes()));
                                    
                                    // Hardcode danh sách mã băm chuẩn trực tiếp vào file chạy (.exe) lúc biên dịch
                                    // Điều này ngăn chặn hacker sửa file releases.json cục bộ để qua mặt hệ thống.
                                    let releases_data = include_str!("../../releases.json");
                                    if let Ok(releases) = serde_json::from_str::<serde_json::Value>(releases_data) {
                                        if releases.get(&snapshot.app_version).is_none() {
                                            println!("[P2P-SECURITY] Báo động: Node {} dùng phiên bản App không hợp lệ (Mod/Hack). Đã khóa!", intent.peer_id);
                                            is_valid = false;
                                        }
                                    }
                                } else {
                                    println!("[P2P-SECURITY] Node {} không gửi kèm Bằng chứng toàn vẹn dữ liệu.", intent.peer_id);
                                }

                                if is_valid {
                                    println!("[P2P] MATCH FOUND & VERIFIED with {}! Bắt tay 1-1...", intent.contact_email);
                                    
                                    if let Ok(target_peer) = intent.peer_id.parse::<PeerId>() {
                                        let req = MatchRequest {
                                            from_peer_id: local_peer_id.to_string(),
                                            topic: topic.clone(),
                                            matched_skills: intent.skills.clone(),
                                            contact_email: user_intent.contact_email.clone(),
                                        };
                                        swarm.behaviour_mut().reqres.send_request(&target_peer, req);
                                    }
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
                }
            }
        });
        
        Ok(())
    }
}
