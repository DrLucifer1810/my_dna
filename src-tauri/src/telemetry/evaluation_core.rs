use rusqlite::Result;
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;

pub struct EvaluationCore;

impl EvaluationCore {
    /// Sinh ra System Prompt để yêu cầu LLM đánh giá chất lượng phiên làm việc của User.
    /// Dựa trên ma trận đánh giá Enterprise: Competence, Discipline, Creativity, Critical Thinking, v.v.
    pub fn build_evaluation_prompt(category: &str, ai_output: &str, final_user_output: &str, edit_ratio: f64) -> String {
        let ai_token_estimate = ai_output.len() / 4;
        let final_token_estimate = final_user_output.len() / 4;

        format!(
            "You are an Enterprise AI Evaluator. Analyze the user's AI-assisted session.
Category: {}
AI Original Output (Est. {} tokens):
---
{}
---
User's Final Saved Output (Est. {} tokens):
---
{}
---
Edit Ratio (0.0 means user kept AI output exactly, 1.0 means user rewrote completely): {:.2}

Evaluate the user's interaction based on these Enterprise metrics (score 0-100), utilizing the Dreyfus Model of Skill Acquisition and Bloom's Taxonomy:
1. 'competence': (Bloom's Taxonomy) Did the user successfully integrate the AI output to solve a problem? High score if they Applied/Analyzed, low if they merely Remembered/Copied.
2. 'discipline': (Codecademy Matrix) Did the user follow standard practices (e.g., adding comments, error handling, edge cases) in the final output?
3. 'creativity': (Dreyfus Model) Did the user act as an Expert/Proficient by creatively adapting the AI output beyond a simple copy-paste?
4. 'critical_thinking': (Dreyfus Model) Did the user act as a 'Competent' or 'Proficient' engineer by fixing AI hallucinations or logic flaws? (If edit_ratio is high, and final output is better, this is high).
5. 'collaboration': Does the final output indicate readiness for team sharing (clarity, readability, maintainability)?
6. 'ai_efficiency': Calculate AI Token Efficiency. If the AI generated {} tokens but the user deleted most of it (high edit ratio), efficiency is LOW (waste of tokens). If the user kept it (low edit ratio) and it solved the problem, efficiency is HIGH.
7. 'prompt_quality': (Bloom's Taxonomy) Guess how good the user's prompt was. A prompt at the 'Evaluate/Create' level gets a higher score than a vague 'Understand' level prompt.

Respond ONLY with a JSON object:
{{
    \"competence\": 85,
    \"discipline\": 90,
    \"creativity\": 70,
    \"critical_thinking\": 80,
    \"collaboration\": 85,
    \"ai_efficiency\": 95,
    \"prompt_quality\": 80,
    \"tips\": \"Your AI usage was efficient, but you can improve discipline by adding more docstrings.\"
}}",
            category, ai_token_estimate, ai_output, final_token_estimate, final_user_output, edit_ratio, ai_token_estimate
        )
    }

    /// Đọc các session chưa được đánh giá và chuẩn bị Prompt (để UI gọi Webview Companion).
    pub fn get_pending_evaluations(state: Arc<Mutex<StateMachine>>) -> Result<Vec<(i64, String)>> {
        let db_lock = state.lock().unwrap();
        let mut stmt = db_lock.conn.prepare(
            "SELECT s.id, s.category, s.raw_context, s.final_content, e.edit_ratio 
             FROM sessions s 
             JOIN session_evaluations e ON s.id = e.session_id 
             WHERE e.prompt_quality IS NULL LIMIT 5"
        )?;

        let rows = stmt.query_map([], |row: &rusqlite::Row| {
            let session_id: i64 = row.get(0)?;
            let category: String = row.get(1)?;
            let ai_output: String = row.get(2)?;
            let final_content: Option<String> = row.get(3)?;
            let edit_ratio: f64 = row.get(4)?;
            
            // Xoá bỏ đoạn mã Mock giả tạo - Sử dụng đoạn text thật của User
            let final_text = final_content.unwrap_or_else(|| "".to_string());
            let prompt = Self::build_evaluation_prompt(&category, &ai_output, &final_text, edit_ratio);
            
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
