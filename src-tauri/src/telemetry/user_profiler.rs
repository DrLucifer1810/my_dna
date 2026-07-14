use rusqlite::Result;
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;

pub struct UserProfiler;

impl UserProfiler {
    /// Lấy toàn bộ Log (Window Titles & Focused Text) trong 24h qua.
    pub fn get_daily_activity_summary(state: Arc<Mutex<StateMachine>>) -> Result<String> {
        let db_lock = state.lock().unwrap();
        // Lấy 1000 events gần nhất để phân tích (Đúng theo tinh thần "lấy hết" của User, giới hạn 1000 để tránh sập Memory)
        let mut stmt = db_lock.conn.prepare(
            "SELECT event_type, window_title, raw_content 
             FROM events 
             WHERE timestamp >= datetime('now', '-1 day')
             ORDER BY id DESC LIMIT 1000"
        )?;

        let rows = stmt.query_map([], |row| {
            let event_type: String = row.get(0)?;
            let window_title: String = row.get(1)?;
            let raw_content: String = row.get(2)?;
            Ok(format!("[{}] {} - Snippet: {:.100}", event_type, window_title, raw_content))
        })?;

        let mut logs = Vec::new();
        for r in rows {
            if let Ok(res) = r {
                logs.push(res);
            }
        }
        
        // Đảo ngược để theo đúng trình tự thời gian
        logs.reverse();
        Ok(logs.join("\n"))
    }

    /// Tạo Prompt chẩn đoán chuyên môn gửi cho LLM.
    pub fn build_diagnostic_prompt(daily_logs: &str) -> String {
        format!(
            "You are an HR Tech AI. Analyze the following 24-hour activity log of a user.
Your task is to profile this user based on their actual daily work.

### Activity Logs:
{}

### Required Output:
Analyze their activities and deduce:
1. 'profession': Their likely job title (e.g., 'Senior Rust Developer', 'Marketing Manager').
2. 'seniority': Their experience level (Junior, Mid, Senior, Lead).
3. 'daily_focus': What they spent the most time doing today.
4. 'tech_stack': Any tools, frameworks, or languages they used.

Respond ONLY with a short, professional paragraph summarizing their profile. Do not use JSON. Example:
'This user is a Senior Rust Developer. Their daily focus is on building concurrent backend systems using Tokio and SQLite. They show high proficiency in system architecture.'",
            daily_logs
        )
    }

    /// Lưu kết quả chẩn đoán vào bảng user_profiles
    pub fn save_profile(state: Arc<Mutex<StateMachine>>, profile_text: &str) -> Result<()> {
        let db_lock = state.lock().unwrap();
        db_lock.conn.execute(
            "INSERT INTO user_profiles (profile_text) VALUES (?1)",
            (profile_text,),
        )?;
        Ok(())
    }
}
