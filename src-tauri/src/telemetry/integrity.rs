use ed25519_dalek::{Signer, Verifier, Signature, VerifyingKey, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegritySnapshot {
    pub peer_id: String,
    pub app_version: String,
    pub db_hash: String,
    pub timestamp: u64,
    pub signature_hex: String,
}

pub struct IntegrityManager;

impl IntegrityManager {
    /// Tính mã băm (Hash) của file local_events.db (Giả lập)
    fn hash_local_database() -> String {
        let db_path = "portable-test/local_events.db";
        let content = std::fs::read(db_path).unwrap_or_else(|_| b"empty_db".to_vec());
        let mut hasher = Sha256::new();
        hasher.update(&content);
        hex::encode(hasher.finalize())
    }

    /// Tạo Snapshot có chữ ký
    pub fn generate_snapshot(peer_id: &str, private_key_bytes: &[u8; 32]) -> Result<IntegritySnapshot, String> {
        let app_version = env!("CARGO_PKG_VERSION").to_string();
        let db_hash = Self::hash_local_database();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        // Chuỗi dữ liệu cần ký
        let payload = format!("{}:{}:{}:{}", peer_id, app_version, db_hash, timestamp);
        
        let signing_key = SigningKey::from_bytes(private_key_bytes);
        let signature = signing_key.sign(payload.as_bytes());

        Ok(IntegritySnapshot {
            peer_id: peer_id.to_string(),
            app_version,
            db_hash,
            timestamp,
            signature_hex: hex::encode(signature.to_bytes()),
        })
    }

    /// Xác minh Snapshot từ Peer khác
    pub fn verify_snapshot(snapshot: &IntegritySnapshot, peer_public_key_bytes: &[u8; 32]) -> bool {
        let payload = format!("{}:{}:{}:{}", snapshot.peer_id, snapshot.app_version, snapshot.db_hash, snapshot.timestamp);
        
        let Ok(signature_bytes) = hex::decode(&snapshot.signature_hex) else { return false };
        let Ok(signature) = Signature::from_slice(&signature_bytes) else { return false };
        let Ok(verifying_key) = VerifyingKey::from_bytes(peer_public_key_bytes) else { return false };

        verifying_key.verify(payload.as_bytes(), &signature).is_ok()
    }
}
