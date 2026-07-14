use hmac::{Hmac, Mac};
use hmac::KeyInit;
use sha2::Sha256;
use keyring::Entry;

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
