use hmac::{Hmac, Mac};
use hmac::KeyInit;
use sha2::Sha256;
use keyring::Entry;
use ed25519_dalek::SigningKey;

type HmacSha256 = Hmac<Sha256>;

const KEYRING_SERVICE: &str = "MyDNA_Enterprise";
const KEYRING_USER: &str = "SystemCore";

fn get_or_create_secret_key() -> Result<String, String> {
    let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|e| format!("Failed to access OS Keyring: {}", e))?;

    match entry.get_password() {
        Ok(secret) => Ok(secret),
        Err(_) => {
            // Khởi tạo một khóa bí mật 32 bytes ngẫu nhiên
            let key: [u8; 32] = rand::random();
            let secret = hex::encode(key);

            entry.set_password(&secret)
                .map_err(|e| format!("Failed to save secret to OS Keyring: {}", e))?;

            Ok(secret)
        }
    }
}

pub fn sign_data(data: &str) -> Result<String, String> {
    let secret = get_or_create_secret_key()?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| format!("HMAC initialization failed: {}", e))?;
    
    mac.update(data.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

pub fn verify_signature(data: &str, signature: &str) -> bool {
    if let Ok(expected_signature) = sign_data(data) {
        expected_signature == signature
    } else {
        false
    }
}

pub fn get_public_key() -> Result<String, String> {
    let entry = Entry::new("MyDNA_Enterprise_P2P", "SystemCore_Ed25519")
        .map_err(|e| format!("Failed to access OS Keyring for P2P: {}", e))?;

    let signing_key = match entry.get_password() {
        Ok(secret_hex) => {
            let bytes = hex::decode(&secret_hex).unwrap_or(vec![0; 32]);
            let mut arr = [0u8; 32];
            let len = std::cmp::min(bytes.len(), 32);
            arr[..len].copy_from_slice(&bytes[..len]);
            SigningKey::from_bytes(&arr)
        },
        Err(_) => {
            let key_bytes: [u8; 32] = rand::random();
            let secret_hex = hex::encode(key_bytes);
            let _ = entry.set_password(&secret_hex);
            SigningKey::from_bytes(&key_bytes)
        }
    };
    
    let verifying_key = signing_key.verifying_key();
    Ok(hex::encode(verifying_key.to_bytes()))
}

pub fn verify_global_signature(data: &str, signature_hex: &str, public_key_hex: &str) -> bool {
    use ed25519_dalek::{Verifier, VerifyingKey, Signature};
    
    let pk_bytes = match hex::decode(public_key_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    
    let mut pk_arr = [0u8; 32];
    if pk_bytes.len() != 32 { return false; }
    pk_arr.copy_from_slice(&pk_bytes);
    
    let verifying_key = match VerifyingKey::from_bytes(&pk_arr) {
        Ok(k) => k,
        Err(_) => return false,
    };
    
    let sig_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    
    let signature = match Signature::from_slice(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };
    
    verifying_key.verify(data.as_bytes(), &signature).is_ok()
}
