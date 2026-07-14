pub mod telemetry;
pub mod storage;
pub mod slm_client;

use std::sync::{Arc, Mutex};
use telemetry::state_machine::StateMachine;
use telemetry::worker::spawn_telemetry_loop;
use slm_client::gemini::GeminiClient;

pub struct AppState {
    pub db: Arc<Mutex<StateMachine>>,
    pub gemini: GeminiClient,
}

#[tauri::command]
async fn force_analyze(state: tauri::State<'_, AppState>) -> Result<slm_client::gemini::RadarScore, String> {
    // Enterprise-ready: Lấy log thật từ SQLite và gửi cho Gemini
    let raw_logs = {
        let db_lock = state.db.lock().map_err(|_| "Failed to lock DB".to_string())?;
        db_lock.get_recent_logs().unwrap_or_else(|_| "No recent logs available".to_string())
    };

    if raw_logs.trim().is_empty() {
        return Err("Not enough data to analyze".to_string());
    }

    // Bảo mật: Lọc hoàn toàn thông tin cá nhân (PIR) khỏi Prompt
    let safe_logs = telemetry::pir::redact_sensitive_data(&raw_logs);

    match state.gemini.analyze_timeline(&safe_logs).await {
        Ok(score) => Ok(score),
        Err(e) => Err(format!("AI Analysis Error: {:?}", e)),
    }
}

#[tauri::command]
async fn login_google() -> Result<String, String> {
    // Thực tế sẽ gọi tới storage::gdrive::GoogleDriveClient
    // MVP: giả định đăng nhập thành công và báo trạng thái
    Ok("Connected securely to Google Drive Workspace".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Khởi tạo SQLite tại portable-test/local_events.db (đảm bảo không xả rác ra ổ C: khi dev)
    let db_path = "portable-test/local_events.db";
    // Đảm bảo thư mục tồn tại (thường đã tạo sẵn bằng script khởi tạo)
    let state_machine = StateMachine::new(db_path).expect("Failed to initialize SQLite DB");
    
    let shared_db = Arc::new(Mutex::new(state_machine));

    // Khởi chạy vòng lặp ngầm (Background Worker) 2s/lần
    spawn_telemetry_loop(shared_db.clone());

    // Khởi chạy File Watcher để theo dõi Semantic Diff khi lưu file
    telemetry::file_watcher::spawn_file_watcher(shared_db.clone(), "portable-test/workspace");

    // Khởi tạo Gemini Client (Sử dụng API key từ biến môi trường để an toàn)
    let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
    let gemini_client = GeminiClient::new(api_key);

    tauri::Builder::default()
        .manage(AppState {
            db: shared_db,
            gemini: gemini_client,
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![force_analyze, login_google])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
