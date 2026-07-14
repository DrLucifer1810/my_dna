use rusqlite::Result;
use std::sync::{Arc, Mutex};
use crate::telemetry::state_machine::StateMachine;

pub struct EvaluationCore;

impl EvaluationCore {
    /// Sinh ra System Prompt để yêu cầu LLM đánh giá chất lượng phiên làm việc của User.
    /// Dựa trên ma trận đánh giá Enterprise: Competence, Discipline, Creativity, Critical Thinking, v.v.
    pub fn build_evaluation_prompt(category: &str, ai_output: &str, final_user_output: &str, edit_ratio: f64) -> String {
        format!(
            "You are an Enterprise AI Evaluator. Analyze the user's AI-assisted session.
Category: {}
AI Original Output:
---
{}
---
User's Final Saved Output:
---
{}
---
Edit Ratio (0.0 means user kept AI output exactly, 1.0 means user rewrote completely): {:.2}

Evaluate the user's interaction based on these Enterprise metrics (score 0-100):
1. 'competence': Did the user successfully integrate the AI output to solve a problem?
2. 'discipline': Did the user follow standard practices (e.g., adding comments, error handling) in the final output?
3. 'creativity': Did the user creatively adapt the AI output beyond a simple copy-paste?
4. 'critical_thinking': Did the user fix AI hallucinations or logic flaws? (If edit_ratio is high, and final output is better, this is high).
5. 'collaboration': Does the final output indicate readiness for team sharing (clarity, readability)?
6. 'ai_efficiency': How efficient was the AI usage? (High if edit_ratio is low but output is high quality).
7. 'prompt_quality': Guess how good the user's prompt was based on how well the AI output matched the user's final need.

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
            category, ai_output, final_user_output, edit_ratio
        )
    }

    /// Đọc các session chưa được đánh giá và chuẩn bị Prompt (để UI gọi Webview Companion).
    pub fn get_pending_evaluations(state: Arc<Mutex<StateMachine>>) -> Result<Vec<(i64, String)>> {
        let db_lock = state.lock().unwrap();
        let mut stmt = db_lock.conn.prepare(
            "SELECT s.id, s.category, s.raw_context, e.edit_ratio 
             FROM sessions s 
             JOIN session_evaluations e ON s.id = e.session_id 
             WHERE e.prompt_quality IS NULL LIMIT 5"
        )?;

        let rows = stmt.query_map([], |row: &rusqlite::Row| {
            let session_id: i64 = row.get(0)?;
            let category: String = row.get(1)?;
            let ai_output: String = row.get(2)?;
            let edit_ratio: f64 = row.get(3)?;
            
            // Giả định file_saved raw_content không lưu trực tiếp trong session để tối ưu, ta truyền raw_context tạm thời
            // (Trong thực tế ta sẽ query lại event cuối cùng của session_id này)
            let prompt = Self::build_evaluation_prompt(&category, &ai_output, "User Final Version Here", edit_ratio);
            
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
