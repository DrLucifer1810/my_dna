use serde_json::Value;
use chrono::Utc;
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;

pub struct NormalizedEvent {
    pub event_type: String,
    pub title: String,
    pub content: String,
    pub timestamp: String,
}

impl NormalizedEvent {
    pub fn save_to_db(&self, db: Arc<Mutex<StateMachine>>) -> Result<(), String> {
        let db_lock = db.lock().map_err(|_| "DB Lock Error".to_string())?;
        db_lock.conn.execute(
            "INSERT INTO events (event_type, window_title, raw_content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            (&self.event_type, &self.title, &self.content, &self.timestamp)
        ).map_err(|e| format!("Insert error: {}", e))?;
        Ok(())
    }
}

pub struct IngestionEngine;

impl IngestionEngine {
    
    /// Chuẩn hóa dữ liệu từ các Extension (VS Code / Chrome)
    pub fn parse_extension_payload(payload: Value) -> Option<NormalizedEvent> {
        let tool = payload.get("tool").and_then(|v| v.as_str()).unwrap_or("UNKNOWN_TOOL");
        let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("interaction");
        let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
        
        if content.trim().is_empty() {
            return None; // Bỏ qua payload rỗng
        }

        Some(NormalizedEvent {
            event_type: "EXTENSION_LLM".to_string(),
            title: format!("[{}] {}", tool.to_uppercase(), action),
            content: content.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    /// Chuẩn hóa dữ liệu từ MCP (GitHub)
    pub fn parse_github_mcp(payload: Value) -> Option<NormalizedEvent> {
        // MCP payload thường chứa "messages" hoặc "content"
        // Giả lập logic bóc tách thông tin Commit Message / PR Review
        let commit_msg = payload.get("commit_message").and_then(|v| v.as_str()).unwrap_or("");
        let repo_name = payload.get("repository").and_then(|v| v.as_str()).unwrap_or("Unknown Repo");
        
        if commit_msg.is_empty() {
            return None;
        }

        Some(NormalizedEvent {
            event_type: "MCP_GITHUB".to_string(),
            title: format!("Repository: {}", repo_name),
            content: format!("Commit: {}", commit_msg),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    /// Chuẩn hóa dữ liệu từ MCP (Jira)
    pub fn parse_jira_mcp(payload: Value) -> Option<NormalizedEvent> {
        let task_name = payload.get("task_title").and_then(|v| v.as_str()).unwrap_or("Unknown Task");
        let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("Updated");

        Some(NormalizedEvent {
            event_type: "MCP_JIRA".to_string(),
            title: format!("Jira Task: {}", task_name),
            content: format!("Status changed to: {}", status),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    /// Chuẩn hóa dữ liệu từ MCP (Slack)
    pub fn parse_slack_mcp(payload: Value) -> Option<NormalizedEvent> {
        let channel = payload.get("channel").and_then(|v| v.as_str()).unwrap_or("Direct Message");
        let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
        
        // Loại bỏ emoji phức tạp, chỉ lấy chữ
        let clean_text = text.replace(|c: char| !c.is_ascii() && !c.is_alphanumeric(), " ");

        Some(NormalizedEvent {
            event_type: "MCP_SLACK".to_string(),
            title: format!("Slack Channel: {}", channel),
            content: clean_text.trim().to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    /// Chuẩn hóa dữ liệu từ Antigravity (Google DeepMind)
    pub fn parse_antigravity_log(payload: Value) -> Option<NormalizedEvent> {
        let step_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("");
        
        if step_type == "USER_INPUT" || step_type == "PLANNER_RESPONSE" {
            return Some(NormalizedEvent {
                event_type: "AGENTIC_LLM".to_string(),
                title: format!("[ANTIGRAVITY] {}", step_type),
                content: content.to_string(),
                timestamp: Utc::now().to_rfc3339(),
            });
        }
        None
    }

    /// Chuẩn hóa dữ liệu từ Claude Code (Anthropic CLI)
    pub fn parse_claude_code_log(payload: Value) -> Option<NormalizedEvent> {
        let command = payload.get("command").and_then(|v| v.as_str()).unwrap_or("");
        let response = payload.get("response").and_then(|v| v.as_str()).unwrap_or("");
        
        Some(NormalizedEvent {
            event_type: "AGENTIC_LLM".to_string(),
            title: format!("[CLAUDE_CODE] Command: {}", command),
            content: response.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }

    /// Chuẩn hóa dữ liệu từ OpenClaw (Autonomous CLI)
    pub fn parse_openclaw_log(payload: Value) -> Option<NormalizedEvent> {
        let action = payload.get("action").and_then(|v| v.as_str()).unwrap_or("Action");
        let details = payload.get("details").and_then(|v| v.as_str()).unwrap_or("");
        
        Some(NormalizedEvent {
            event_type: "AGENTIC_LLM".to_string(),
            title: format!("[OPENCLAW] {}", action),
            content: details.to_string(),
            timestamp: Utc::now().to_rfc3339(),
        })
    }
}
