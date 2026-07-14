use serde::{Deserialize, Serialize};
use reqwest::Client;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use keyring::Entry;
use std::fs;
use std::path::Path;
use tauri_plugin_opener::OpenerExt;
use std::env;

const REDIRECT_URI: &str = "http://127.0.0.1:13337";

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct DriveFileList {
    files: Option<Vec<DriveFile>>,
}

#[derive(Deserialize, Debug)]
struct DriveFile {
    id: String,
    #[allow(dead_code)]
    name: String,
}

pub struct GoogleSyncManager;

impl GoogleSyncManager {
    /// Bắt đầu quy trình Đăng nhập & Đồng bộ
    pub async fn login_and_sync(app: tauri::AppHandle) -> Result<String, String> {
        // Nhúng (Bake) khóa trực tiếp vào file nhị phân lúc biên dịch (Compile-time)
        let client_id = env!("MYDNA_GOOGLE_CLIENT_ID").to_string();
        let client_secret = env!("MYDNA_GOOGLE_CLIENT_SECRET").to_string();

        let access_token = match Self::get_access_token_from_refresh(&client_id, &client_secret).await {
            Ok(token) => token,
            Err(_) => {
                // Nếu chưa có refresh_token, mở luồng OAuth2
                Self::perform_oauth2(app, &client_id, &client_secret).await?
            }
        };

        // 1. Đồng bộ Private Key
        Self::sync_identity_key(&access_token).await?;

        // Hỗ trợ multi-node testing
        let node_suffix = std::env::var("MYDNA_TEST_NODE").unwrap_or_default();
        let db_dir = if node_suffix.is_empty() {
            "portable-test".to_string()
        } else {
            format!("portable-test/{}", node_suffix)
        };

        // 2. Đồng bộ Database
        Self::sync_file_to_drive(&access_token, "local_events.db", &format!("{}/local_events.db", db_dir)).await?;

        // 3. Đồng bộ Snapshot tính toàn vẹn (Cross-Verification)
        Self::sync_file_to_drive(&access_token, "my_snapshot.json", &format!("{}/my_snapshot.json", db_dir)).await?;

        Ok("Đồng bộ dữ liệu an toàn thành công!".to_string())
    }

    /// Lấy Access Token từ Refresh Token đã lưu
    async fn get_access_token_from_refresh(client_id: &str, client_secret: &str) -> Result<String, String> {
        let entry = Entry::new("MyDNA_Enterprise_Sync", "GoogleRefreshToken")
            .map_err(|e| e.to_string())?;
        
        let refresh_token = entry.get_password().map_err(|e| e.to_string())?;
        
        let client = Client::new();
        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("refresh_token", &refresh_token),
            ("grant_type", "refresh_token"),
        ];

        let res = client.post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send().await.map_err(|e| e.to_string())?;

        let token: TokenResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(token.access_token)
    }

    /// Mở trình duyệt và lắng nghe Code trả về qua Loopback IP
    async fn perform_oauth2(app: tauri::AppHandle, client_id: &str, client_secret: &str) -> Result<String, String> {
        let auth_url = format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/drive.appdata&access_type=offline&prompt=consent",
            client_id, REDIRECT_URI
        );

        let _ = app.opener().open_url(auth_url.clone(), None::<String>);

        let listener = TcpListener::bind("127.0.0.1:13337").await.map_err(|e| e.to_string())?;
        let (mut stream, _) = listener.accept().await.map_err(|e| e.to_string())?;

        let mut buf = [0; 2048];
        let n = stream.read(&mut buf).await.map_err(|e| e.to_string())?;
        let request = String::from_utf8_lossy(&buf[..n]);

        let code = Self::extract_code(&request).ok_or("Không tìm thấy mã xác thực")?;

        let response = "HTTP/1.1 200 OK\r\n\r\n<html><body>Xác thực thành công. Bạn có thể đóng tab này.</body><script>window.close()</script></html>";
        let _ = stream.write_all(response.as_bytes()).await;

        // Exchange token
        let client = Client::new();
        let params = [
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", &code),
            ("redirect_uri", REDIRECT_URI),
            ("grant_type", "authorization_code"),
        ];

        let res = client.post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send().await.map_err(|e| e.to_string())?;

        let token_resp: TokenResponse = res.json().await.map_err(|e| e.to_string())?;

        if let Some(rt) = token_resp.refresh_token {
            let entry = Entry::new("MyDNA_Enterprise_Sync", "GoogleRefreshToken").unwrap();
            let _ = entry.set_password(&rt);
        }

        Ok(token_resp.access_token)
    }

    fn extract_code(request: &str) -> Option<String> {
        let first_line = request.lines().next()?;
        if let Some(pos) = first_line.find("code=") {
            let rest = &first_line[pos + 5..];
            let end_pos = rest.find(' ').unwrap_or(rest.len());
            let code = &rest[..end_pos];
            let code = code.split('&').next().unwrap_or(code);
            return Some(code.to_string());
        }
        None
    }

    /// Đồng bộ định danh (Private Key) để đảm bảo 1 User = 1 Identity
    async fn sync_identity_key(access_token: &str) -> Result<(), String> {
        let client = Client::new();
        // Tìm file identity.key trong appDataFolder
        let search_url = "https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='identity.key'";
        let res = client.get(search_url)
            .bearer_auth(access_token)
            .send().await.map_err(|e| e.to_string())?;
        
        let file_list: DriveFileList = res.json().await.unwrap_or(DriveFileList { files: None });

        let entry = Entry::new("MyDNA_Enterprise_P2P", "SystemCore_Ed25519")
            .map_err(|e| e.to_string())?;

        if let Some(files) = file_list.files {
            if !files.is_empty() {
                // Tải file từ Drive (Đã tồn tại)
                let file_id = &files[0].id;
                let download_url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file_id);
                let content = client.get(&download_url)
                    .bearer_auth(access_token)
                    .send().await.map_err(|e| e.to_string())?
                    .text().await.map_err(|e| e.to_string())?;
                
                // Đè vào OS Keyring để sử dụng cùng Identity trên máy mới
                let _ = entry.set_password(&content);
                return Ok(());
            }
        }

        // Nếu chưa có trên Drive, lấy key hiện tại và Upload lên
        if let Ok(local_key) = entry.get_password() {
            let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";
            let metadata = serde_json::json!({
                "name": "identity.key",
                "parents": ["appDataFolder"]
            });

            let form = reqwest::multipart::Form::new()
                .part("metadata", reqwest::multipart::Part::text(metadata.to_string()).mime_str("application/json").unwrap())
                .part("file", reqwest::multipart::Part::text(local_key));

            let _ = client.post(url)
                .bearer_auth(access_token)
                .multipart(form)
                .send().await.map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Backup local_events.db lên appDataFolder
    /// Backup một file lên appDataFolder
    async fn sync_file_to_drive(access_token: &str, file_name: &str, file_path: &str) -> Result<(), String> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read(path).map_err(|e| e.to_string())?;
        let client = Client::new();

        let search_url = format!("https://www.googleapis.com/drive/v3/files?spaces=appDataFolder&q=name='{}'", file_name);
        let res = client.get(&search_url)
            .bearer_auth(access_token)
            .send().await.map_err(|e| e.to_string())?;
        
        let file_list: DriveFileList = res.json().await.unwrap_or(DriveFileList { files: None });

        let metadata = serde_json::json!({
            "name": file_name,
            "parents": ["appDataFolder"]
        });

        if let Some(files) = file_list.files {
            if !files.is_empty() {
                // Update file hiện có (Patch)
                let file_id = &files[0].id;
                let patch_url = format!("https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=media", file_id);
                let _ = client.patch(&patch_url)
                    .bearer_auth(access_token)
                    .body(content)
                    .send().await.map_err(|e| e.to_string())?;
                return Ok(());
            }
        }

        // Upload mới
        let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart";
        let form = reqwest::multipart::Form::new()
            .part("metadata", reqwest::multipart::Part::text(metadata.to_string()).mime_str("application/json").unwrap())
            .part("file", reqwest::multipart::Part::bytes(content).mime_str("application/octet-stream").unwrap());

        let _ = client.post(url)
            .bearer_auth(access_token)
            .multipart(form)
            .send().await.map_err(|e| e.to_string())?;

        Ok(())
    }
}
