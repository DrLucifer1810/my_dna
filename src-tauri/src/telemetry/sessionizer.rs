use std::sync::{Arc, Mutex};
use rusqlite::Result;
use crate::telemetry::state_machine::StateMachine;
use std::collections::HashMap;
use regex::Regex;
use chrono::{DateTime, Utc};

pub struct Sessionizer;

struct RegexRule {
    category: &'static str,
    pattern: Regex,
}

impl Sessionizer {
    /// Phân loại cửa sổ bằng Regex Rule Engine (Mượn từ ActivityWatch)
    fn categorize_window(rules: &[RegexRule], title: &str) -> String {
        for rule in rules {
            if rule.pattern.is_match(title) {
                return rule.category.to_string();
            }
        }
        "Other".to_string()
    }



    /// Background Batch Job: Gom nhóm các Events thành Sessions ngữ nghĩa
    pub fn process_raw_events(state: Arc<Mutex<StateMachine>>) -> Result<()> {
        let db_lock = state.lock().unwrap();
        
        // Khởi tạo Regex Rules (Lấy cảm hứng từ classify.rs của ActivityWatch)
        let rules = vec![
            RegexRule { category: "Development", pattern: Regex::new(r"(?i)(code|cursor|idea|pycharm|windsurf|terminal|nvim|vim)").unwrap() },
            RegexRule { category: "Communication", pattern: Regex::new(r"(?i)(slack|teams|discord|outlook|mail|messenger|zalo)").unwrap() },
            RegexRule { category: "Research", pattern: Regex::new(r"(?i)(stackoverflow|github|docs|localhost|chatgpt|gemini|claude)").unwrap() },
            RegexRule { category: "Distraction", pattern: Regex::new(r"(?i)(youtube|facebook|tiktok|netflix|twitter|reddit)").unwrap() },
        ];

        let mut stmt = db_lock.conn.prepare(
            "SELECT id, event_type, window_title, raw_content, timestamp FROM events ORDER BY id ASC"
        )?;
        
        let rows = stmt.query_map([], |row: &rusqlite::Row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?
            ))
        })?;

        let mut current_context = String::new();
        let mut session_start_time = String::new();
        let mut last_timestamp = String::new();
        let mut context_switches = 0;
        let mut dominant_categories: HashMap<String, i32> = HashMap::new();
        
        // Phục vụ thuật toán Flood (Khử nhiễu AFK / Alt-tab ngắn)
        let mut active_window = String::new();
        let mut active_window_start: Option<DateTime<Utc>> = None;
        let mut pending_window_switch: Option<(String, DateTime<Utc>, String)> = None; // (title, timestamp, cat)

        let mut _last_event_id = 0;

        for row in rows {
            if let Ok((id, event_type, title, raw_content, timestamp_str)) = row {
                _last_event_id = id;
                if session_start_time.is_empty() {
                    session_start_time = timestamp_str.clone();
                }
                
                let current_time = DateTime::parse_from_rfc3339(&timestamp_str).map(|dt| dt.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());

                // Chia Session khi dữ liệu đủ lớn (Activity Burst)
                if current_context.len() > 15000 {
                    let category = dominant_categories.iter()
                        .max_by_key(|entry| entry.1)
                        .map(|(k, _)| k.to_string())
                        .unwrap_or_else(|| "Other".to_string());

                    db_lock.conn.execute(
                        "INSERT INTO sessions (start_time, end_time, category, raw_context, final_content, context_switches, duration_seconds) 
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, CAST(strftime('%s', ?2) - strftime('%s', ?1) AS INTEGER))",
                        (&session_start_time, &last_timestamp, &category, &current_context, "", context_switches),
                    )?;

                    let session_id = db_lock.conn.last_insert_rowid();
                    
                    db_lock.conn.execute(
                        "INSERT INTO session_evaluations (session_id) VALUES (?1)",
                        (session_id,),
                    )?;

                    current_context.clear();
                    session_start_time = timestamp_str.clone();
                    context_switches = 0;
                    dominant_categories.clear();
                }

                let cat = Self::categorize_window(&rules, &title);
                *dominant_categories.entry(cat.clone()).or_insert(0) += 1;
                
                if event_type == "WINDOW_CHANGE" {
                    // THUẬT TOÁN FLOOD: Khử nhiễu Alt-Tab ngắn (< 5 giây)
                    if let Some(start_time) = active_window_start {
                        let duration = current_time.signed_duration_since(start_time).num_seconds();
                        if title == active_window {
                            // User quay lại window cũ. Kiểm tra xem window xen giữa (pending) có phải là nhiễu không
                            if let Some((_, pending_time, _)) = pending_window_switch.take() {
                                let gap = current_time.signed_duration_since(pending_time).num_seconds();
                                if gap < 5 {
                                    // Gap < 5s -> ĐÂY LÀ NHIỄU (Noise/Flood). Hủy context switch xen giữa.
                                    // Bỏ qua không tăng context_switches.
                                } else {
                                    // Gap >= 5s -> Chuyển cửa sổ thực sự.
                                    context_switches += 1;
                                    current_context.push_str(&format!("\n[{}] Switched to Window: {} (Category: {})\n", timestamp_str, title, cat));
                                }
                            }
                        } else {
                            // Chuyển sang một cửa sổ hoàn toàn mới.
                            // Lưu cửa sổ hiện tại thành pending để chờ xem có quay lại không (Flood)
                            pending_window_switch = Some((title.clone(), current_time, cat.clone()));
                        }
                    } else {
                        // Lần đầu tiên
                        active_window = title.clone();
                        active_window_start = Some(current_time);
                        current_context.push_str(&format!("\n[{}] Switched to Window: {} (Category: {})\n", timestamp_str, title, cat));
                    }
                    
                    if pending_window_switch.is_none() {
                       active_window = title.clone();
                       active_window_start = Some(current_time);
                    }
                    
                } else if event_type == "FOCUSED_TEXT" {
                    // Xử lý UIA Snapshot (Không dùng Keylogger nữa)
                    if raw_content.len() > 20 { // Text đủ dài
                        current_context.push_str(&format!("\n[{}] {}\n", timestamp_str, raw_content));
                    }
                }
                last_timestamp = timestamp_str;
            }
        }
        
        Ok(())
    }
}

