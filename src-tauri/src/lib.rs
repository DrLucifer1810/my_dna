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
async fn force_analyze(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<slm_client::gemini::RadarScore, String> {
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

    match state.gemini.analyze_timeline(app, &safe_logs).await {
        Ok(score) => Ok(score),
        Err(e) => Err(format!("AI Analysis Error: {:?}", e)),
    }
}

#[tauri::command]
async fn login_google(app: tauri::AppHandle) -> Result<String, String> {
    // Gọi trực tiếp hàm login của Webview Companion
    crate::slm_client::gemini_companion::ensure_gemini_login(app).await?;
    Ok("OAuth Webview Opened and Session Authenticated".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Đảm bảo thư mục tồn tại trước khi mở DB hoặc khởi tạo file watcher (Sửa lỗi Crash do thiếu Folder)
    std::fs::create_dir_all("portable-test/workspace").unwrap_or_default();

    // Khởi tạo SQLite tại portable-test/local_events.db
    let db_path = "portable-test/local_events.db";
    let state_machine = StateMachine::new(db_path).expect("Failed to initialize SQLite DB");
    let shared_db = Arc::new(Mutex::new(state_machine));

    spawn_telemetry_loop(shared_db.clone());
    telemetry::file_watcher::spawn_file_watcher(shared_db.clone(), "portable-test/workspace");

    let gemini_client = GeminiClient::new();

    tauri::Builder::default()
        .manage(AppState {
            db: shared_db,
            gemini: gemini_client,
        })
        .manage(crate::slm_client::gemini_companion::PendingPrompts::default())
        .manage(crate::slm_client::gemini_companion::PendingContexts::default())
        .manage(crate::slm_client::gemini_companion::GeminiLock::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            force_analyze, 
            login_google,
            crate::slm_client::gemini_companion::ensure_gemini_login,
            crate::slm_client::gemini_companion::run_gemini_background_prompt,
            crate::slm_client::gemini_companion::warm_up_gemini_bg,
            crate::slm_client::gemini_companion::get_gemini_debug_log,
            crate::slm_client::gemini_companion::gemini_has_session,
            crate::slm_client::gemini_companion::gemini_switch_account,
            crate::slm_client::gemini_companion::receive_gemini_done,
            crate::slm_client::gemini_companion::receive_gemini_chunk,
            crate::slm_client::gemini_companion::receive_gemini_log,
            crate::slm_client::gemini_companion::receive_gemini_context,
            crate::slm_client::webview_selectors::update_gemini_selectors
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[tokio::test]
    async fn test_telemetry_capture() {
        std::fs::create_dir_all("portable-test/workspace").unwrap_or_default();
        let sm = telemetry::state_machine::StateMachine::new("portable-test/test_telemetry.db").unwrap();
        let shared_db = Arc::new(Mutex::new(sm));
        
        // Kích hoạt engine telemetry thực thụ
        telemetry::worker::spawn_telemetry_loop(shared_db.clone());
        
        std::thread::sleep(Duration::from_secs(1));
        
        // Mô phỏng người dùng copy văn bản thật trên Windows
        if let Ok(_) = clipboard_win::set_clipboard_string("REAL_PRODUCTION_DATA_123") {
            // Đợi 3 giây để worker quét và lưu vào DB (loop chạy mỗi 2s)
            std::thread::sleep(Duration::from_secs(3));
            
            let db_lock = shared_db.lock().unwrap();
            let logs = db_lock.get_recent_logs().unwrap();
            println!("--- REAL CAPTURED TELEMETRY LOGS ---");
            println!("{}", logs);
            println!("------------------------------------");
            assert!(logs.contains("REAL_PRODUCTION_DATA_123"));
        }
    }
}
