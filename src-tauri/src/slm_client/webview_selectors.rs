use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiSelectors {
    pub logged_in: Vec<String>,
    pub sign_in: Vec<String>,
    pub inputs: Vec<String>,
    pub upload_buttons: Vec<String>,
    pub send_buttons: Vec<String>,
    pub response_blocks: Vec<String>,
    pub stop_buttons: Vec<String>,
}

impl Default for GeminiSelectors {
    fn default() -> Self {
        Self {
            logged_in: vec![
                "[data-testid=\"user-menu\"]".into(),
                "[aria-label=\"Google Account\"]".into(),
                "[aria-label*=\"Google Account\"]".into(),
                "img[alt*=\"Profile\"]".into(),
                "img[src*=\"googleusercontent.com\"]".into(),
                "a[aria-label*=\"Google Account\"]".into(),
                "button[aria-label*=\"Google Account\"]".into(),
                "a[gem-open-account-menu]".into(),
                "[gem-open-account-menu]".into(),
                ".mavatar-image".into(),
                ".mavatar-container".into(),
                "a[href*=\"SignOutOptions\"]".into(),
                "img.gbii".into(),
                "[data-testid=\"textbox-container\"] textarea".into(),
                "textarea".into(),
                "g-textarea".into(),
                "[role=\"textbox\"]".into(),
                "div[contenteditable=\"true\"]".into(),
                ".ql-editor[contenteditable=\"true\"]".into(),
                "[data-testid=\"conversation-turn\"]".into(),
            ],
            sign_in: vec![
                "[data-testid=\"sign-in-button\"]".into(),
                "button[jsname=\"LgbsSe\"]".into(),
                "a[href*=\"accounts.google.com\"]".into(),
                "div[role=\"button\"][data-identifier]".into(),
                "button:has-text(\"Sign in\")".into(),
                "a:has-text(\"Sign in\")".into(),
                "button[aria-label*=\"Sign in\"]".into(),
                "a[aria-label*=\"Sign in\"]".into(),
            ],
            inputs: vec![
                "[data-testid=\"textbox-container\"] textarea".into(),
                "[data-testid=\"textbox-container\"] [contenteditable=\"true\"]".into(),
                "rich-textarea textarea".into(),
                "rich-textarea [contenteditable=\"true\"]".into(),
                "textarea[placeholder*=\"Enter a prompt\"]".into(),
                "textarea[placeholder*=\"Message Gemini\"]".into(),
                ".ql-editor[contenteditable=\"true\"]".into(),
                "[role=\"textbox\"]".into(),
                "textarea".into(),
                "div[contenteditable=\"true\"]".into(),
                "[aria-label*=\"prompt\"]".into(),
                "[placeholder*=\"Ask\"]".into(),
                "[placeholder*=\"Gemini\"]".into(),
            ],
            upload_buttons: vec![
                "div.uploader-button-container button[aria-label*=\"tải\"]".into(),
                "div.uploader-button-container button[aria-label*=\"upload\"]".into(),
                "div.uploader-button-container button.upload-card-button".into(),
                "button[data-test-id=\"image-upload-open-button\"]".into(),
            ],
            send_buttons: vec![
                "button[aria-label*=\"Send message\"]".into(),
                "button:has(mat-icon[fonticon*=\"send\"])".into(),
                "button:has(svg[data-icon=\"send\"])".into(),
                "button[data-testid=\"send-button\"]".into(),
                "button[data-testid=\"send-prompt-button\"]".into(),
                ".send-button".into(),
            ],
            response_blocks: vec![
                "model-response .message-content".into(),
                ".message-content".into(),
                "model-response .response-content".into(),
                "model-response message-content".into(),
                ".model-response-text".into(),
                "response-element .response-content".into(),
                ".response-container .model-response-text".into(),
                "[data-testid=\"conversation-turn\"] .model-response-text".into(),
                "[data-testid=\"conversation-turn\"]".into(),
            ],
            stop_buttons: vec![
                "mat-icon[fonticon=\"stop\"]".into(),
                "button:has(mat-icon[fonticon*=\"stop\"])".into(),
                "button:has(mat-icon[fonticon*=\"square\"])".into(),
                "button:has(svg circle)".into(),
                "[data-testid=\"stop-generating-button\"]".into(),
                ".stop-response-button".into(),
                "button.stop-button".into(),
                "bard-xsrf-token-refresher + div button[aria-label*=\"Stop\"]".into(),
                "button[jsaction*=\"stop\"]".into(),
                "button[aria-label*=\"Stop response\"]".into(),
                "button[aria-label*=\"Stop generating\"]".into(),
                "[data-test-id=\"stop-button\"]".into(),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatGPTSelectors {
    pub logged_in: Vec<String>,
    pub sign_in: Vec<String>,
    pub inputs: Vec<String>,
    pub upload_buttons: Vec<String>,
    pub send_buttons: Vec<String>,
    pub response_blocks: Vec<String>,
}

impl Default for ChatGPTSelectors {
    fn default() -> Self {
        Self {
            logged_in: vec![
                "button[aria-label*=\"Profile\"]".into(),
                "[data-testid=\"profile-button\"]".into(),
                "img[alt*=\"User\"]".into(),
            ],
            sign_in: vec![
                "button[data-provider=\"google\"]".into(),
                "[data-testid=\"login-button\"]".into(),
            ],
            inputs: vec![
                "div#prompt-textarea[contenteditable=\"true\"]".into(),
                "textarea#prompt-textarea".into(),
                "div[id=\"prompt-textarea\"]".into(),
                "[data-placeholder=\"Ask anything\"]".into(),
            ],
            upload_buttons: vec![
                "button[data-testid=\"composer-plus-btn\"]".into(),
                "button[aria-label*=\"Add files\"]".into(),
            ],
            send_buttons: vec![
                "button[data-testid=\"send-button\"]".into(),
                "button[aria-label*=\"Send\"]".into(),
            ],
            response_blocks: vec![
                "div[data-message-author-role=\"assistant\"]".into(),
                ".markdown".into(),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeSelectors {
    pub inputs: Vec<String>,
    pub send_buttons: Vec<String>,
    pub response_blocks: Vec<String>,
}

impl Default for ClaudeSelectors {
    fn default() -> Self {
        Self {
            inputs: vec![
                "div[data-testid=\"chat-input\"]".into(),
                "div.ProseMirror".into(),
            ],
            send_buttons: vec![
                "button[aria-label*=\"Send\"]".into(),
            ],
            response_blocks: vec![
                ".font-claude-message".into(),
            ],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrokSelectors {
    pub inputs: Vec<String>,
    pub send_buttons: Vec<String>,
    pub response_blocks: Vec<String>,
}

impl Default for GrokSelectors {
    fn default() -> Self {
        Self {
            inputs: vec![
                "div[aria-label*=\"Ask Grok\"]".into(),
                "textarea".into(),
            ],
            send_buttons: vec![
                "span svg".into(),
                "button[aria-label*=\"Send\"]".into(),
            ],
            response_blocks: vec![
                ".message-content".into(),
            ],
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WebviewSelectors {
    pub gemini: GeminiSelectors,
    pub chatgpt: ChatGPTSelectors,
    pub claude: ClaudeSelectors,
    pub grok: GrokSelectors,
}

impl Default for WebviewSelectors {
    fn default() -> Self {
        Self {
            gemini: GeminiSelectors::default(),
            chatgpt: ChatGPTSelectors::default(),
            claude: ClaudeSelectors::default(),
            grok: GrokSelectors::default(),
        }
    }
}

pub fn get_selectors_path(app: &AppHandle) -> PathBuf {
    // Tách riêng file cấu hình ra thư mục working directory (vd: portable-test/knowledge/selectors/)
    // để user tiện chỉnh sửa trực tiếp. Nếu không có quyền ghi thì fallback về app_data_dir.
    let base_dir = std::env::current_dir().unwrap_or_else(|_| {
        app.path().app_data_dir().unwrap_or_else(|_| std::env::temp_dir())
    });
    let mut path = base_dir;
    path.push("knowledge");
    path.push("selectors");
    path.push("webview_selectors.json");
    path
}

pub fn load_selectors(app: &AppHandle) -> WebviewSelectors {
    let path = get_selectors_path(app);
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(selectors) = serde_json::from_str(&content) {
                return selectors;
            }
        }
    }
    // Default fallback
    let default_selectors = WebviewSelectors::default();
    let _ = save_selectors(app, &default_selectors);
    default_selectors
}

pub fn save_selectors(app: &AppHandle, selectors: &WebviewSelectors) -> Result<(), String> {
    let path = get_selectors_path(app);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(selectors).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_gemini_selectors(
    app: AppHandle,
    new_selectors: GeminiSelectors,
) -> Result<(), String> {
    let mut config = load_selectors(&app);
    config.gemini = new_selectors;
    save_selectors(&app, &config)
}
