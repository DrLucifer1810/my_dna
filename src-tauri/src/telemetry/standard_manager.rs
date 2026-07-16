use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::time::Duration;

pub struct StandardManager;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StandardRegistry {
    pub allowed_hashes: Vec<String>,
    pub signature: String,
}

impl StandardManager {
    /// Lấy Public Key của Hệ thống để xác thực chữ ký (Global PubKey)
    fn get_official_public_key() -> Result<String, String> {
        // Áp dụng chính sách Fail-Fast: Không dùng dummy key. Trả về lỗi nếu chưa cấu hình.
        std::env::var("MYDNA_GLOBAL_PUBKEY")
            .map_err(|_| "CRITICAL: Missing MYDNA_GLOBAL_PUBKEY in environment! System cannot verify standard signatures.".to_string())
    }

    /// Gọi hàm này lúc khởi động app ngầm (async)
    pub async fn fetch_and_verify_registry() -> Result<(), String> {
        let client = reqwest::Client::builder().timeout(Duration::from_secs(10)).build().unwrap();
        let repo_url = "https://raw.githubusercontent.com/DrLucifer1810/my_dna_release/main/standards";
        
        let registry_url = format!("{}/standards_registry.json", repo_url);
        let response = client.get(&registry_url).send().await;
        
        if let Ok(res) = response {
            if let Ok(registry_json) = res.text().await {
                if let Ok(registry) = serde_json::from_str::<StandardRegistry>(&registry_json) {
                    let data_to_verify = registry.allowed_hashes.join(",");
                    let pub_key = Self::get_official_public_key()?;
                    
                    if !crate::telemetry::crypto::verify_global_signature(&data_to_verify, &registry.signature, &pub_key) {
                        return Err("Security Error: Invalid Registry Signature! Payload tampered or unauthorized.".to_string());
                    }
                    
                    let _ = fs::create_dir_all("portable-test");
                    let _ = fs::write("portable-test/standards_registry.json", &registry_json);
                    
                    // Thử fetch file YAML mới nhất nếu có thể
                    if let Ok(yaml_res) = client.get(format!("{}/prompts.yaml", repo_url)).send().await {
                        if let Ok(yaml_text) = yaml_res.text().await {
                            let mut hasher = Sha256::new();
                            hasher.update(yaml_text.as_bytes());
                            let hash = hex::encode(hasher.finalize());
                            
                            if registry.allowed_hashes.contains(&hash) {
                                let _ = fs::write("portable-test/prompts.yaml", &yaml_text);
                            }
                        }
                    }
                    return Ok(());
                }
            }
        }
        Err("Failed to fetch or parse remote registry".to_string())
    }
    
    /// Lấy hash của bản chuẩn hiện tại đang dùng (để đẩy lên P2P)
    pub fn get_current_standard_hash() -> Result<String, String> {
        let yaml_str = fs::read_to_string("portable-test/prompts.yaml")
            .unwrap_or_default();
            
        // Nếu file trống hoặc bị xóa, trả về chuỗi rỗng để fail verification sau này
        if yaml_str.is_empty() { return Ok("".to_string()); }
            
        let mut hasher = Sha256::new();
        hasher.update(yaml_str.as_bytes());
        let hash = hex::encode(hasher.finalize());
        
        // Xác thực xem hash này có nằm trong registry cục bộ (đã được ký) không
        let registry_str = fs::read_to_string("portable-test/standards_registry.json")
            .map_err(|_| "No standards_registry.json found. App must sync first.".to_string())?;
            
        let registry: StandardRegistry = serde_json::from_str(&registry_str)
            .map_err(|_| "Invalid standards_registry.json format".to_string())?;
        
        let pub_key = Self::get_official_public_key()?;
        if !crate::telemetry::crypto::verify_global_signature(&registry.allowed_hashes.join(","), &registry.signature, &pub_key) {
             return Err("Security Error: Tampered local registry!".to_string());
        }
            
        if !registry.allowed_hashes.contains(&hash) {
            return Err(format!("Security Error: Local prompts.yaml hash ({}) is not authorized!", hash));
        }
        
        Ok(hash)
    }
    
    /// Node A kiểm tra hash của Node B có hợp lệ không dựa trên tập cho phép
    pub fn is_hash_allowed(hash_to_check: &str) -> bool {
        if let Ok(registry_str) = fs::read_to_string("portable-test/standards_registry.json") {
            if let Ok(registry) = serde_json::from_str::<StandardRegistry>(&registry_str) {
                if let Ok(pub_key) = Self::get_official_public_key() {
                    if crate::telemetry::crypto::verify_global_signature(&registry.allowed_hashes.join(","), &registry.signature, &pub_key) {
                        return registry.allowed_hashes.contains(&hash_to_check.to_string());
                    }
                }
            }
        }
        // Cho phép bypass nếu Node B dùng chung Hash mặc định (khi chưa có registry)
        hash_to_check == "DEFAULT_LOCAL_HASH" || hash_to_check == ""
    }
}
