use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum DriveError {
    Network(ReqwestError),
    AuthError(String),
    FolderCreationError,
}

impl From<ReqwestError> for DriveError {
    fn from(err: ReqwestError) -> Self {
        DriveError::Network(err)
    }
}

pub struct GoogleDriveClient {
    client: Client,
    access_token: String,
}

impl GoogleDriveClient {
    pub fn new(access_token: String) -> Self {
        GoogleDriveClient {
            client: Client::new(),
            access_token,
        }
    }

    /// Khởi tạo thư mục DACP_Workspace.
    /// Nếu lỗi mạng hoặc auth, bắn lỗi thẳng ra ngoài (Fail-Fast).
    pub async fn init_workspace_folder(&self) -> Result<String, DriveError> {
        let payload = serde_json::json!({
            "name": "DACP_Workspace",
            "mimeType": "application/vnd.google-apps.folder"
        });

        let res = self.client.post("https://www.googleapis.com/drive/v3/files")
            .bearer_auth(&self.access_token)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            return Err(DriveError::AuthError(format!("Failed to create folder. Status: {}", status)));
        }

        #[derive(Deserialize)]
        struct FolderResponse {
            id: String,
        }

        let folder: FolderResponse = res.json().await?;
        Ok(folder.id)
    }

    /// Đồng bộ log sự kiện lên thư mục
    pub async fn upload_log(&self, folder_id: &str, file_name: &str, content: &str) -> Result<(), DriveError> {
        // ... Logic upload multipart tới Google Drive API
        // Áp dụng tính chất Fail-fast:
        let res = self.client.post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart")
            .bearer_auth(&self.access_token)
            // Trong thực tế, cần multipart form. Ở đây minh họa request.
            .body(content.to_string())
            .send()
            .await?;
        
        if !res.status().is_success() {
            return Err(DriveError::AuthError("Upload failed".into()));
        }

        Ok(())
    }
}
