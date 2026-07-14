pub mod telemetry;
pub mod storage;
pub mod slm_client;
pub mod mcp_server;

use std::sync::{Arc, Mutex};
use telemetry::state_machine::StateMachine;
use slm_client::gemini::GeminiClient;

pub struct AppState {
    pub db: Arc<Mutex<StateMachine>>,
    pub gemini: GeminiClient,
}

#[tauri::command]
async fn get_evaluation_metrics(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db_lock = state.db.lock().map_err(|_| "Failed to lock database".to_string())?;
    
    // Đọc metrics mới nhất từ session_evaluations
    let mut stmt = db_lock.conn.prepare(
        "SELECT competence, discipline, creativity, critical_thinking, collaboration, ai_efficiency 
         FROM session_evaluations 
         ORDER BY session_id DESC LIMIT 1"
    ).map_err(|e| e.to_string())?;

    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
    
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let metrics: Vec<i32> = vec![
            row.get(0).unwrap_or(0),
            row.get(1).unwrap_or(0),
            row.get(2).unwrap_or(0),
            row.get(3).unwrap_or(0),
            row.get(4).unwrap_or(0),
            row.get(5).unwrap_or(0),
        ];
        return Ok(serde_json::json!({ "metrics": metrics }));
    }
    
    // Fail-Fast: Nếu không có dữ liệu, báo lỗi không có dữ liệu thay vì trả mock
    Err("No evaluation data available. Please wait for the background agent to analyze your session.".to_string())
}

#[tauri::command]
async fn force_profile_diagnostic(state: tauri::State<'_, AppState>) -> std::result::Result<String, String> {
    // Cập nhật lại Public Profile Passport (Phase 1.13)
    let _ = crate::telemetry::user_profiler::MultiAgentProfiler::synthesize_public_profile(state.inner().db.clone());
    
    get_dna_profile(state).await.map(|v| v.to_string())
}

#[tauri::command]
async fn get_dna_profile(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let db_lock = state.db.lock().map_err(|_| "Failed to lock database".to_string())?;
    
    let fetch_latest = |agent_type: &str| -> serde_json::Value {
        if let Ok(mut stmt) = db_lock.conn.prepare(
            "SELECT extracted_traits, signature FROM user_dna WHERE agent_type = ?1 ORDER BY timestamp DESC LIMIT 1"
        ) {
            if let Ok(mut rows) = stmt.query([agent_type]) {
                if let Ok(Some(row)) = rows.next() {
                    let traits_str: String = row.get(0).unwrap_or_default();
                    let signature: String = row.get(1).unwrap_or_default();
                    
                    if crate::telemetry::crypto::verify_signature(&traits_str, &signature) {
                        return serde_json::from_str(&traits_str).unwrap_or(serde_json::json!({}));
                    } else {
                        return serde_json::json!({"error": "DATA_TAMPERED"});
                    }
                }
            }
        }
        serde_json::json!({})
    };

    let code_dna = fetch_latest("CodeAnalyzer");
    let comm_dna = fetch_latest("CommunicationAnalyzer");
    let career_dna = fetch_latest("CareerDiagnostic");

    if code_dna.get("error").and_then(|v| v.as_str()) == Some("DATA_TAMPERED") ||
       comm_dna.get("error").and_then(|v| v.as_str()) == Some("DATA_TAMPERED") ||
       career_dna.get("error").and_then(|v| v.as_str()) == Some("DATA_TAMPERED") {
        return Err("DATA_TAMPERED: Tệp hồ sơ DNA của bạn đã bị can thiệp trái phép. Hệ thống từ chối truy cập.".to_string());
    }

    // Nếu không có dữ liệu nào, báo lỗi
    if code_dna.as_object().unwrap_or(&serde_json::Map::new()).is_empty() && 
       career_dna.as_object().unwrap_or(&serde_json::Map::new()).is_empty() {
        return Err("DNA Profile is still being built. Please use the system longer to gather data.".to_string());
    }

    Ok(serde_json::json!({
        "profession": career_dna["profession"].as_str().unwrap_or("Analyzing..."),
        "daily_focus": career_dna["daily_focus"].as_str().unwrap_or("Analyzing..."),
        "coding_habits": {
            "good": code_dna["good_habits"].clone(),
            "bad": code_dna["bad_habits"].clone(),
            "principles": code_dna["principles"].clone()
        },
        "tone": comm_dna["tone"].clone()
    }))
}

#[tauri::command]
async fn force_analyze_logs(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<slm_client::gemini::RadarScore, String> {
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

#[tauri::command]
async fn login_and_sync_google_drive(app: tauri::AppHandle) -> Result<String, String> {
    crate::telemetry::google_sync::GoogleSyncManager::login_and_sync(app).await
}

#[tauri::command]
async fn start_p2p_network(
    intent_recruiting: bool,
    intent_looking_job: bool,
    intent_hiring_freelancer: bool,
    intent_freelancing: bool,
    contact_email: String,
    matching_profile_json: Option<String>,
    state: tauri::State<'_, AppState>
) -> Result<String, String> {
    // Port and Bootstrap nodes could be injected via .env for local testing multiple nodes
    let port: u16 = std::env::var("MYDNA_P2P_PORT")
        .unwrap_or_else(|_| "8000".to_string())
        .parse()
        .unwrap_or(8000);
    
    let bootstrap_str = std::env::var("MYDNA_BOOTSTRAP_NODES").unwrap_or_default();
    let bootstrap_nodes: Vec<String> = if bootstrap_str.is_empty() {
        vec![]
    } else {
        bootstrap_str.split(',').map(|s| s.to_string()).collect()
    };

    // Hỗ trợ Clustering giả lập: Nếu chạy nhiều Node trên 1 máy
    let node_suffix = std::env::var("MYDNA_TEST_NODE").unwrap_or_default();
    let service_name = if node_suffix.is_empty() {
        "MyDNA_Enterprise_P2P".to_string()
    } else {
        format!("MyDNA_Enterprise_P2P_{}", node_suffix)
    };

    let entry = keyring::Entry::new(&service_name, "SystemCore_Ed25519")
        .map_err(|e| format!("Keychain error: {}", e))?;
        
    let key_hex = match entry.get_password() {
        Ok(k) => k,
        Err(_) => {
            // Tự động sinh Key cho Test Node giả lập
            if !node_suffix.is_empty() {
                let kp = libp2p::identity::ed25519::Keypair::generate();
                let hex = hex::encode(kp.to_bytes());
                let _ = entry.set_password(&hex);
                hex
            } else {
                return Err("Private key not found. Please sync identity first.".to_string());
            }
        }
    };
    
    let mut key_bytes = [0u8; 32];
    hex::decode_to_slice(&key_hex, &mut key_bytes).map_err(|_| "Invalid key hex".to_string())?;

    // Fetch real skills from DNA Profile
    let mut skills = vec![];
    if let Ok(val) = get_dna_profile(state.clone()).await {
        if let Some(good) = val["coding_habits"]["good"].as_array() {
             for s in good { skills.push(s.as_str().unwrap_or("").to_string()); }
        }
        if let Some(prof) = val["profession"].as_str() {
             skills.push(prof.to_string());
        }
    }

    // Kết hợp Email chính từ Google và Email bổ sung
    let primary_email = keyring::Entry::new("MyDNA_Enterprise_Sync", "GoogleEmail")
        .and_then(|e| e.get_password())
        .unwrap_or_else(|_| "".to_string());
        
    let final_email = if primary_email.is_empty() && contact_email.is_empty() {
        // Fallback for test cluster
        "test.node@localhost".to_string()
    } else if primary_email.is_empty() {
        contact_email
    } else if contact_email.is_empty() || contact_email == primary_email {
        primary_email
    } else {
        format!("{}, {}", primary_email, contact_email)
    };

    let matching_profile = if let Some(json_str) = matching_profile_json {
        serde_json::from_str::<crate::telemetry::p2p_network::MatchingProfile>(&json_str).ok()
    } else {
        None
    };

    let user_intent = crate::telemetry::p2p_network::MatchIntent {
        peer_id: "".to_string(), // Set later
        is_recruiting: intent_recruiting,
        is_looking_for_job: intent_looking_job,
        is_hiring_freelancer: intent_hiring_freelancer,
        is_freelancing: intent_freelancing,
        contact_email: final_email,
        skills,
        matching_profile,
        integrity_snapshot: None, // Set later
    };

    crate::telemetry::p2p_network::P2pNetworkManager::start_node(port, bootstrap_nodes, &mut key_bytes, user_intent)
        .await?;

    Ok(format!("P2P Network started on port {}", port))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let node_suffix = std::env::var("MYDNA_TEST_NODE").unwrap_or_default();
    let db_dir = if node_suffix.is_empty() {
        "portable-test".to_string()
    } else {
        format!("portable-test/{}", node_suffix)
    };

    // Đảm bảo thư mục tồn tại trước khi mở DB hoặc khởi tạo file watcher
    std::fs::create_dir_all(format!("{}/system-logs", db_dir)).unwrap_or_default();
    std::fs::create_dir_all(format!("{}/workspace", db_dir)).unwrap_or_default();

    // Khởi tạo SQLite
    let db_path = format!("{}/local_events.db", db_dir);
    let state_machine = StateMachine::new(&db_path).expect("Failed to initialize SQLite DB");
    let shared_db = Arc::new(Mutex::new(state_machine));

    telemetry::worker::spawn_telemetry_loop(shared_db.clone());
    let watch_path = format!("{}/workspace", db_dir);
    telemetry::file_watcher::spawn_file_watcher(shared_db.clone(), &watch_path);

    // Phase 1.9: Khởi chạy MCP Server ở port 5050 (Hoặc port động nếu test)
    let mcp_port = std::env::var("MYDNA_MCP_PORT").unwrap_or_else(|_| "5050".to_string());
    // (Bỏ qua cấu hình MCP Server động trong ví dụ này để tránh phình to code)
    let mcp_db = shared_db.clone();
    tokio::spawn(async move {
        crate::mcp_server::McpServer::start(mcp_db).await;
    });

    let gemini_client = GeminiClient::new();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(24 * 3600));
                loop {
                    interval.tick().await;
                    // Chạy ngầm: Chỉ đồng bộ nếu đã có refresh token (Không mở popup login)
                    let _ = crate::telemetry::google_sync::GoogleSyncManager::login_and_sync(app_handle.clone()).await;
                }
            });
            Ok(())
        })
        .manage(AppState {
            db: shared_db,
            gemini: gemini_client,
        })
        .manage(crate::slm_client::gemini_companion::PendingPrompts::default())
        .manage(crate::slm_client::gemini_companion::PendingContexts::default())
        .manage(crate::slm_client::gemini_companion::GeminiLock::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_evaluation_metrics,
            get_dna_profile,
            force_profile_diagnostic,
            force_analyze_logs,
            login_google,
            login_and_sync_google_drive,
            start_p2p_network,
            crate::slm_client::gemini_companion::ensure_gemini_login,
            crate::slm_client::gemini_companion::run_gemini_background_prompt,
            crate::slm_client::gemini_companion::parse_jd_to_profile,
            crate::slm_client::gemini_companion::parse_cv_to_profile,
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
