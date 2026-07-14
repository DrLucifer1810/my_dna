use mydna_lib::telemetry::p2p_network::{P2pNetworkManager, MatchIntent};
use libp2p::identity;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("MYDNA_P2P_PORT").unwrap_or_else(|_| "8000".to_string()).parse().unwrap();
    let bootstrap = std::env::var("MYDNA_BOOTSTRAP_NODES").unwrap_or_default();
    let bootstrap_nodes = if bootstrap.is_empty() { vec![] } else { vec![bootstrap] };
    
    let mut key_bytes = [0u8; 32];
    let kp = identity::ed25519::Keypair::generate();
    key_bytes.copy_from_slice(&kp.to_bytes());

    let is_freelancing = std::env::var("IS_FREELANCING").is_ok();
    let is_hiring_freelancer = std::env::var("IS_HIRING_FREELANCER").is_ok();
    
    let email = if is_freelancing { "freelancer@test.com" } else { "hr@test.com" };

    let intent = MatchIntent {
        peer_id: "".to_string(),
        is_recruiting: true,
        is_looking_for_job: false,
        is_hiring_freelancer: false,
        is_freelancing: false,
        contact_email: "test1@domain.com".to_string(),
        skills: vec!["Rust".to_string()],
        matching_profile: None,
        integrity_snapshot: None,
    };

    println!("[TEST-NODE] Starting Node on port {}...", port);
    println!("[TEST-NODE] Intent: Freelancer={}, Hiring={}", is_freelancing, is_hiring_freelancer);
    
    // Tạo folder test
    let node_suffix = std::env::var("MYDNA_TEST_NODE").unwrap_or_default();
    if !node_suffix.is_empty() {
        std::fs::create_dir_all(format!("portable-test/{}", node_suffix)).unwrap();
    }

    P2pNetworkManager::start_node(port, bootstrap_nodes, &mut key_bytes, intent).await.unwrap();
    
    // Keep alive
    for i in 0..15 {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        println!("[TEST-NODE] Running... ({}s)", i * 2);
    }
}
