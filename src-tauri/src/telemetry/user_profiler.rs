use rusqlite::Result;
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;

pub struct MultiAgentProfiler;

pub enum AgentType {
    CodeAnalyzer,
    CommunicationAnalyzer,
    CareerDiagnostic,
}

impl MultiAgentProfiler {
    /// Lấy Log tương ứng với Agent
    pub fn get_logs_for_agent(state: Arc<Mutex<StateMachine>>, agent: &AgentType) -> Result<String> {
        let db_lock = state.lock().unwrap();
        
        let condition = match agent {
            AgentType::CodeAnalyzer => "(window_title LIKE '%Code%' OR window_title LIKE '%Idea%' OR window_title LIKE '%Cursor%')",
            AgentType::CommunicationAnalyzer => "(window_title LIKE '%Mail%' OR window_title LIKE '%Outlook%' OR window_title LIKE '%Chat%')",
            AgentType::CareerDiagnostic => "1=1", // Lấy tất cả
        };

        let query = format!(
            "SELECT event_type, window_title, raw_content 
             FROM events 
             WHERE timestamp >= datetime('now', '-1 day') AND {}
             ORDER BY id DESC LIMIT 500", condition
        );

        let mut stmt = db_lock.conn.prepare(&query)?;

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
        
        logs.reverse();
        Ok(logs.join("\n"))
    }

    /// Trả về System Prompt tương ứng với Từng Agent
    pub fn build_agent_prompt(agent: &AgentType, logs: &str) -> String {
        match agent {
            AgentType::CodeAnalyzer => {
                format!(
                    "You are a Code Analyzer Agent. Read the developer's logs:
{}
Extract their coding DNA:
1. 'good_habits' (e.g. adding docs, early returns).
2. 'bad_habits' (e.g. leaving console.log, raw unwraps).
3. 'principles' (e.g. uses snake_case, prefers functional style).
Return ONLY a JSON object: {{\"good_habits\": [], \"bad_habits\": [], \"principles\": []}}",
                    logs
                )
            },
            AgentType::CommunicationAnalyzer => {
                format!(
                    "You are a Communication Analyzer Agent. Read the user's chat/email logs:
{}
Extract their communication DNA:
1. 'tone' (e.g. formal, casual, direct, polite).
2. 'voice' (e.g. uses active voice, short sentences).
3. 'quirks' (e.g. uses emojis, signs off with 'Cheers').
Return ONLY a JSON object: {{\"tone\": [], \"voice\": [], \"quirks\": []}}",
                    logs
                )
            },
            AgentType::CareerDiagnostic => {
                format!(
                    "You are a Career Diagnostic Agent. Read the user's daily logs:
{}
Extract their career DNA:
1. 'profession' (e.g. Senior Rust Engineer).
2. 'daily_focus' (e.g. Debugging Tokio async issues).
Return ONLY a JSON object: {{\"profession\": \"\", \"daily_focus\": \"\"}}",
                    logs
                )
            }
        }
    }

    /// Lưu kết quả JSON của Agent vào bảng user_dna
    pub fn save_dna(state: Arc<Mutex<StateMachine>>, agent_name: &str, traits_json: &str) -> Result<()> {
        let db_lock = state.lock().unwrap();
        db_lock.conn.execute(
            "INSERT INTO user_dna (agent_type, extracted_traits) VALUES (?1, ?2)",
            (agent_name, traits_json),
        )?;
        Ok(())
    }
}
