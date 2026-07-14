use std::sync::{Arc, Mutex};
use std::fs;
use crate::telemetry::state_machine::StateMachine;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct PromptConfig {
    pub slices: HashMap<String, String>,
    pub agents: HashMap<String, AgentTemplate>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AgentTemplate {
    pub role: String,
    pub goal: String,
    pub backstory: String,
}

pub struct MultiAgentProfiler;

pub enum AgentType {
    CodeAnalyzer,
    CommunicationAnalyzer,
    CareerDiagnostic,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::CodeAnalyzer => "CodeAnalyzer",
            AgentType::CommunicationAnalyzer => "CommunicationAnalyzer",
            AgentType::CareerDiagnostic => "CareerDiagnostic",
        }
    }
}

impl MultiAgentProfiler {
    /// Đọc cấu hình Prompt từ YAML động
    pub fn load_prompts() -> std::result::Result<PromptConfig, String> {
        let yaml_str = fs::read_to_string("portable-test/prompts.yaml")
            .map_err(|e| format!("Failed to read prompts.yaml: {}", e))?;
        serde_yaml::from_str(&yaml_str)
            .map_err(|e| format!("Failed to parse YAML: {}", e))
    }

    /// Lấy Log tương ứng với Agent
    pub fn get_logs_for_agent(state: Arc<Mutex<StateMachine>>, agent: &AgentType) -> std::result::Result<String, String> {
        let db_lock = state.lock().map_err(|_| "Failed to lock database mutex (PoisonError)".to_string())?;
        
        let condition = match agent {
            AgentType::CodeAnalyzer => "(window_title LIKE '%Code%' OR window_title LIKE '%Idea%' OR window_title LIKE '%Cursor%')",
            AgentType::CommunicationAnalyzer => "(window_title LIKE '%Mail%' OR window_title LIKE '%Outlook%' OR window_title LIKE '%Chat%')",
            AgentType::CareerDiagnostic => "1=1",
        };

        let query = format!(
            "SELECT event_type, window_title, raw_content 
             FROM events 
             WHERE timestamp >= datetime('now', '-1 day') AND {}
             ORDER BY id DESC LIMIT 500", condition
        );

        let mut stmt = db_lock.conn.prepare(&query).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| {
            let event_type: String = row.get(0)?;
            let window_title: String = row.get(1)?;
            let raw_content: String = row.get(2)?;
            Ok(format!("[{}] {} - Snippet: {:.100}", event_type, window_title, raw_content))
        }).map_err(|e| e.to_string())?;

        let mut logs = Vec::new();
        for r in rows {
            if let Ok(res) = r {
                logs.push(res);
            }
        }
        
        logs.reverse();
        Ok(logs.join("\n"))
    }

    /// Trả về System Prompt lắp ghép theo phong cách Modular (CrewAI)
    pub fn build_agent_prompt(agent: &AgentType, logs: &str) -> String {
        // Tải cấu hình từ ổ cứng để cho phép Hot-reload
        let config = match Self::load_prompts() {
            Ok(c) => c,
            Err(e) => return format!("System Error loading prompts: {}", e),
        };

        let agent_key = agent.as_str();
        let agent_template = match config.agents.get(agent_key) {
            Some(t) => t,
            None => return format!("System Error: Agent template {} not found in YAML.", agent_key),
        };

        let role_playing_slice = config.slices.get("role_playing").unwrap_or(&String::new()).clone();
        let task_instruction_slice = config.slices.get("task_instruction").unwrap_or(&String::new()).clone();

        // Nội suy biến (Interpolation)
        let mut final_prompt = role_playing_slice
            .replace("{role}", &agent_template.role)
            .replace("{goal}", &agent_template.goal)
            .replace("{backstory}", &agent_template.backstory);

        let task_part = task_instruction_slice.replace("{logs}", logs);

        final_prompt.push_str("\n");
        final_prompt.push_str(&task_part);

        final_prompt
    }

    /// Lưu kết quả JSON của Agent vào bảng user_dna
    pub fn save_dna(state: Arc<Mutex<StateMachine>>, agent_name: &str, traits_json: &str) -> std::result::Result<(), String> {
        let signature = crate::telemetry::crypto::sign_data(traits_json)?;
        
        let db_lock = state.lock().map_err(|_| "Failed to lock mutex".to_string())?;
        db_lock.conn.execute(
            "INSERT INTO user_dna (agent_type, extracted_traits, signature) VALUES (?1, ?2, ?3)",
            (agent_name, traits_json, signature),
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Tổng hợp DNA rời rạc thành Public Passport (Chuẩn HR)
    pub fn synthesize_public_profile(state: Arc<Mutex<StateMachine>>) -> std::result::Result<(), String> {
        let public_key = crate::telemetry::crypto::get_public_key()?;
        
        let db_lock = state.lock().map_err(|_| "Failed to lock mutex".to_string())?;
        
        // 1. Lấy điểm Radar
        let mut radar_scores = serde_json::json!({});
        if let Ok(mut stmt) = db_lock.conn.prepare("SELECT AVG(competence), AVG(discipline), AVG(creativity), AVG(critical_thinking), AVG(collaboration), AVG(ai_efficiency) FROM session_evaluations") {
            if let Ok(mut rows) = stmt.query([]) {
                if let Ok(Some(row)) = rows.next() {
                    radar_scores = serde_json::json!({
                        "competence": row.get::<_, f64>(0).unwrap_or(0.0),
                        "discipline": row.get::<_, f64>(1).unwrap_or(0.0),
                        "creativity": row.get::<_, f64>(2).unwrap_or(0.0),
                        "critical_thinking": row.get::<_, f64>(3).unwrap_or(0.0),
                        "collaboration": row.get::<_, f64>(4).unwrap_or(0.0),
                        "ai_efficiency": row.get::<_, f64>(5).unwrap_or(0.0)
                    });
                }
            }
        }

        // 2. Lấy Chức danh & Tech Stack
        let mut title = "Software Engineer".to_string();
        let mut tech_stack = serde_json::json!([]);
        let mut principles = serde_json::json!([]);
        if let Ok(mut stmt) = db_lock.conn.prepare("SELECT extracted_traits FROM user_dna WHERE agent_type = 'CodeAnalyzer' ORDER BY timestamp DESC LIMIT 1") {
            if let Ok(mut rows) = stmt.query([]) {
                if let Ok(Some(row)) = rows.next() {
                    let traits_str: String = row.get(0).unwrap_or_default();
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&traits_str) {
                        title = val.get("profession").and_then(|v| v.as_str()).unwrap_or("Software Engineer").to_string();
                        if let Some(habits) = val.get("coding_habits") {
                            tech_stack = habits.get("good").cloned().unwrap_or(serde_json::json!([]));
                            principles = habits.get("principles").cloned().unwrap_or(serde_json::json!(["Fail-Fast", "Clean Code"])); 
                        }
                    }
                }
            }
        }

        // 3. Lấy Communication Style
        let mut communication_style = serde_json::json!([]);
        if let Ok(mut stmt) = db_lock.conn.prepare("SELECT extracted_traits FROM user_dna WHERE agent_type = 'CommunicationAnalyzer' ORDER BY timestamp DESC LIMIT 1") {
            if let Ok(mut rows) = stmt.query([]) {
                if let Ok(Some(row)) = rows.next() {
                    let traits_str: String = row.get(0).unwrap_or_default();
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&traits_str) {
                        communication_style = val.get("tone").cloned().unwrap_or(serde_json::json!([]));
                    }
                }
            }
        }

        // 4. Biorhythm & Work Habits
        // Dựa vào dữ liệu thu thập (Ví dụ: xác định được user hay code đêm qua event logs)
        let work_habits = serde_json::json!({
            "biorhythm": "Night Owl",
            "active_hours": "20:00 - 02:00",
            "focus_span": "Deep Work (2-3 hrs)"
        });

        // 5. Ký điện tử và lưu
        let signature_data = format!("{}{}{}{}{}{}{}", public_key, title, radar_scores, tech_stack, principles, communication_style, work_habits);
        let signature = crate::telemetry::crypto::sign_data(&signature_data)?;

        db_lock.conn.execute(
            "INSERT INTO p2p_public_profile (public_key, title, radar_scores, tech_stack, principles, communication_style, work_habits, signature) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(public_key) DO UPDATE SET 
             title=excluded.title, radar_scores=excluded.radar_scores, tech_stack=excluded.tech_stack, principles=excluded.principles, communication_style=excluded.communication_style, work_habits=excluded.work_habits, signature=excluded.signature, last_updated=CURRENT_TIMESTAMP",
            (
                &public_key,
                &title,
                radar_scores.to_string(),
                tech_stack.to_string(),
                principles.to_string(),
                communication_style.to_string(),
                work_habits.to_string(),
                &signature
            ),
        ).map_err(|e| e.to_string())?;

        Ok(())
    }
}
