use rusqlite::Result;
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;

pub struct EvaluationCore;

impl EvaluationCore {
    /// Prompt cho Agent chấm điểm Lập Trình (Code Analyzer)
    pub fn build_code_analyzer_prompt(category: &str, timeline: &str, duration: i64, context_switches: i64) -> String {
        format!(
            "You are an Enterprise AI Evaluator specializing in Software Engineering and Prompt Engineering.
Analyze the user's AI-assisted development session based on their continuous event timeline.
Category: {}
Duration: {} seconds
Context Switches: {}

Raw Timeline (Keystrokes, Window Switches, Focused Screen Text):
---
{}
---

Evaluate the user's interaction based on these Enterprise metrics (score 0-100):
1. 'prompting_skill': Did they use clear, structured prompts to direct the AI? Did they effectively orchestrate the AI to fix hallucinations or bugs, rather than just manually fixing the AI's broken code?
2. 'verification_skill': Did they verify the AI's output by alt-tabbing to documentation, writing tests, or carefully reviewing the generated code?
3. 'competence': Their overall ability to solve the problem using AI.
4. 'discipline': Following standard practices (e.g., adding comments, error handling, formatting).
5. 'creativity': Adapting the AI output beyond simple copy-pasting.

Respond ONLY with a JSON object:
{{
    \"prompting_skill\": 85,
    \"verification_skill\": 80,
    \"competence\": 85,
    \"discipline\": 90,
    \"creativity\": 70,
    \"tips\": \"Great orchestration, but verify edge cases more carefully.\",
    \"quality_reason\": \"Detailed reason based on timeline behavior.\"
}}",
            category, duration, context_switches, timeline
        )
    }

    /// Prompt cho Agent chấm điểm Giao tiếp (Communication Agent)
    pub fn build_communication_prompt(category: &str, timeline: &str) -> String {
        format!(
            "You are an Enterprise AI Evaluator specializing in Professional Communication.
Analyze the user's communication draft session.
Category: {}

Raw Timeline (Keystrokes, Screen Text):
---
{}
---

Evaluate (score 0-100):
1. 'competence': Clarity, tone, and professional phrasing of the message.
2. 'collaboration': Empathy and readiness for team sharing.
3. 'discipline': Proper grammar and structured emails/messages.

Respond ONLY with JSON:
{{
    \"competence\": 90,
    \"collaboration\": 85,
    \"discipline\": 80,
    \"tips\": \"Clear and professional, consider being more concise.\",
    \"quality_reason\": \"...\"
}}",
            category, timeline
        )
    }

    /// Đọc các session chưa được đánh giá và chuẩn bị Prompt (để UI gọi Webview Companion).
    pub fn get_pending_evaluations(state: Arc<Mutex<StateMachine>>) -> Result<Vec<(i64, String)>> {
        let db_lock = state.lock().unwrap();
        let mut stmt = db_lock.conn.prepare(
            "SELECT s.id, s.category, s.raw_context, s.context_switches, s.duration_seconds 
             FROM sessions s 
             JOIN session_evaluations e ON s.id = e.session_id 
             WHERE e.tips IS NULL LIMIT 5"
        )?;

        let rows = stmt.query_map([], |row: &rusqlite::Row| {
            let session_id: i64 = row.get(0)?;
            let category: String = row.get(1)?;
            let raw_context: String = row.get(2)?;
            let context_switches: i64 = row.get(3).unwrap_or(0);
            let duration: i64 = row.get(4).unwrap_or(0);
            
            let prompt = if category == "Communication" {
                Self::build_communication_prompt(&category, &raw_context)
            } else {
                // Default to Code Analyzer for Development, Research, and Other
                Self::build_code_analyzer_prompt(&category, &raw_context, duration, context_switches)
            };
            
            Ok((session_id, prompt))
        })?;

        let mut results = Vec::new();
        for r in rows {
            if let Ok(res) = r {
                results.push(res);
            }
        }
        Ok(results)
    }
}
