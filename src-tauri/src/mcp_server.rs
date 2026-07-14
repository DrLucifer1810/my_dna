use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;
use tokio::net::TcpListener;

pub struct McpServer;

impl McpServer {
    pub async fn start(state: Arc<Mutex<StateMachine>>) {
        let app = Router::new()
            .route("/mcp/resources/user_dna", get({
                let state_clone = state.clone();
                move || get_user_dna(state_clone)
            }));

        println!("MCP Server starting on http://localhost:5050");
        let listener = match TcpListener::bind("0.0.0.0:5050").await {
            Ok(l) => l,
            Err(e) => {
                eprintln!("MCP Server failed to bind port 5050: {}", e);
                return;
            }
        };
        if let Err(e) = axum::serve(listener, app).await {
            eprintln!("MCP Server error: {}", e);
        }
    }
}

async fn get_user_dna(state: Arc<Mutex<StateMachine>>) -> Json<Value> {
    let db_lock = match state.lock() {
        Ok(lock) => lock,
        Err(_) => return Json(json!({"error": "Failed to lock database"}))
    };
    
    // Fetch latest DNA from each agent
    let fetch_latest = |agent_type: &str| -> Value {
        if let Ok(mut stmt) = db_lock.conn.prepare(
            "SELECT extracted_traits, signature FROM user_dna WHERE agent_type = ?1 ORDER BY timestamp DESC LIMIT 1"
        ) {
            if let Ok(mut rows) = stmt.query([agent_type]) {
                if let Ok(Some(row)) = rows.next() {
                    let traits_str: String = row.get(0).unwrap_or_default();
                    let signature: String = row.get(1).unwrap_or_default();
                    if crate::telemetry::crypto::verify_signature(&traits_str, &signature) {
                        return serde_json::from_str(&traits_str).unwrap_or(json!({}));
                    } else {
                        return json!({"error": "DATA_TAMPERED"});
                    }
                }
            }
        }
        json!({})
    };

    let code_dna = fetch_latest("CodeAnalyzer");
    let comm_dna = fetch_latest("CommunicationAnalyzer");
    let career_dna = fetch_latest("CareerDiagnostic");

    if code_dna.get("error").is_some() || comm_dna.get("error").is_some() || career_dna.get("error").is_some() {
        return Json(json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32603,
                "message": "DATA_TAMPERED: The user DNA profile has been illegally modified."
            }
        }));
    }

    let mcp_response = json!({
        "jsonrpc": "2.0",
        "result": {
            "name": "mydna_user_context",
            "description": "Comprehensive User DNA for AI Personalization",
            "content": {
                "seniority": career_dna["profession"].as_str().unwrap_or("Unknown"),
                "daily_focus": career_dna["daily_focus"].as_str().unwrap_or("Unknown"),
                "coding_habits": {
                    "good": code_dna["good_habits"].clone(),
                    "bad_to_avoid": code_dna["bad_habits"].clone(),
                    "principles": code_dna["principles"].clone()
                },
                "communication": {
                    "tone": comm_dna["tone"].clone(),
                    "voice": comm_dna["voice"].clone(),
                    "quirks": comm_dna["quirks"].clone()
                }
            }
        }
    });

    Json(mcp_response)
}
