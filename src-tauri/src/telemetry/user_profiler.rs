use rusqlite::Result;
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
    pub fn get_logs_for_agent(state: Arc<Mutex<StateMachine>>, agent: &AgentType) -> Result<String> {
        let db_lock = state.lock().unwrap();
        
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
    pub fn save_dna(state: Arc<Mutex<StateMachine>>, agent_name: &str, traits_json: &str) -> Result<()> {
        let db_lock = state.lock().unwrap();
        db_lock.conn.execute(
            "INSERT INTO user_dna (agent_type, extracted_traits) VALUES (?1, ?2)",
            (agent_name, traits_json),
        )?;
        Ok(())
    }
}
