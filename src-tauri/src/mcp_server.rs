use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;
use tokio::net::TcpListener;

pub struct McpServer {
    state: Arc<Mutex<StateMachine>>
}

impl McpServer {
    pub async fn start(state: Arc<Mutex<StateMachine>>) {
        let app = Router::new()
            .route("/mcp/resources/user_dna", get({
                let state_clone = state.clone();
                move || get_user_dna(state_clone)
            }));

        println!("MCP Server starting on http://localhost:5050");
        let listener = TcpListener::bind("0.0.0.0:5050").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}

async fn get_user_dna(state: Arc<Mutex<StateMachine>>) -> Json<Value> {
    let db_lock = state.lock().unwrap();
    
    // Fetch latest DNA from each agent
    let fetch_latest = |agent_type: &str| -> Value {
        if let Ok(mut stmt) = db_lock.conn.prepare(
            "SELECT extracted_traits FROM user_dna WHERE agent_type = ?1 ORDER BY timestamp DESC LIMIT 1"
        ) {
            if let Ok(mut rows) = stmt.query([agent_type]) {
                if let Ok(Some(row)) = rows.next() {
                    let traits_str: String = row.get(0).unwrap_or_default();
                    return serde_json::from_str(&traits_str).unwrap_or(json!({}));
                }
            }
        }
        json!({})
    };

    let code_dna = fetch_latest("CodeAnalyzer");
    let comm_dna = fetch_latest("CommunicationAnalyzer");
    let career_dna = fetch_latest("CareerDiagnostic");

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
