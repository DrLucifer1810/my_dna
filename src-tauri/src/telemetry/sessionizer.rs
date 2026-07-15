use similar::{ChangeTag, TextDiff};
use std::sync::{Arc, Mutex};
use rusqlite::Result;
use crate::telemetry::state_machine::StateMachine;

pub struct Sessionizer;

impl Sessionizer {
    /// Tính toán Edit Ratio: Tỉ lệ User phải sửa lại code/văn bản do AI tạo ra.
    /// Giá trị trả về từ 0.0 (Giữ nguyên 100%) đến 1.0 (Xóa/Sửa lại toàn bộ).
    pub fn calculate_edit_ratio(ai_output: &str, final_user_output: &str) -> f64 {
        if ai_output.is_empty() {
            return 1.0;
        }

        let diff = TextDiff::from_lines(ai_output, final_user_output);
        let mut total_lines = 0;
        let mut changed_lines = 0;

        for change in diff.iter_all_changes() {
            total_lines += 1;
            match change.tag() {
                ChangeTag::Delete | ChangeTag::Insert => changed_lines += 1,
                ChangeTag::Equal => {}
            }
        }

        if total_lines == 0 {
            return 0.0;
        }

        (changed_lines as f64) / (total_lines as f64)
    }

    /// Background Batch Job: Gom nhóm các Events thành Sessions và lưu vào DB.
    pub fn process_raw_events(state: Arc<Mutex<StateMachine>>) -> Result<()> {
        let db_lock = state.lock().unwrap();
        // Lấy tất cả events
        let mut stmt = db_lock.conn.prepare(
            "SELECT id, event_type, window_title, raw_content, timestamp FROM events ORDER BY id ASC"
        )?;
        
        let rows = stmt.query_map([], |row: &rusqlite::Row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?
            ))
        })?;

        let mut current_ai_output = String::new();
        let mut session_start_time = String::new();
        
        for row in rows {
            if let Ok((_id, event_type, title, raw_content, timestamp)) = row {
                let lower_title = title.to_lowercase();
                
                // User copy từ trình duyệt AI -> Đây là AI Output (Bắt đầu Session sử dụng AI)
                if event_type == "CLIPBOARD_COPY" && (lower_title.contains("gemini") || lower_title.contains("chat") || lower_title.is_empty()) {
                    current_ai_output = raw_content;
                    session_start_time = timestamp;
                    continue;
                }

                // Nếu có AI Output trước đó, và giờ user thực hiện Save File hoặc Sửa Text -> Kết thúc Session
                if !current_ai_output.is_empty() && (event_type == "FILE_SAVED" || event_type == "FOCUSED_TEXT") {
                    // Tránh các event rác quá ngắn
                    if raw_content.len() < 10 { continue; }

                    let category = if lower_title.contains("code") || lower_title.contains("cursor") || lower_title.contains("idea") {
                        "Coding"
                    } else if lower_title.contains("mail") || lower_title.contains("outlook") {
                        "Email"
                    } else {
                        "Planning/Other"
                    };

                    let edit_ratio = Self::calculate_edit_ratio(&current_ai_output, &raw_content);

                    // Insert Session (Lưu cả đầu vào của AI và code cuối cùng của User)
                    db_lock.conn.execute(
                        "INSERT INTO sessions (start_time, end_time, category, raw_context, final_content) VALUES (?1, ?2, ?3, ?4, ?5)",
                        (&session_start_time, &timestamp, category, &current_ai_output, &raw_content),
                    )?;

                    let session_id = db_lock.conn.last_insert_rowid();

                    // Đánh dấu DB tạm với edit_ratio, phần LLM Evaluation sẽ được chạy riêng sau
                    db_lock.conn.execute(
                        "INSERT INTO session_evaluations (session_id, edit_ratio) VALUES (?1, ?2)",
                        (session_id, edit_ratio),
                    )?;

                    // Reset để đón Session tiếp theo
                    current_ai_output.clear();
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_edit_ratio() {
        let ai_code = "fn main() {\n    println!(\"Hello AI\");\n}";
        
        // Kịch bản 1: User xài luôn không sửa chữ nào (Hài lòng 100%)
        let exact_match = "fn main() {\n    println!(\"Hello AI\");\n}";
        assert_eq!(Sessionizer::calculate_edit_ratio(ai_code, exact_match), 0.0);

        // Kịch bản 2: User sửa một nửa (Hài lòng trung bình)
        let edited_code = "fn main() {\n    println!(\"Hello Human\");\n}";
        let ratio = Sessionizer::calculate_edit_ratio(ai_code, edited_code);
        assert!(ratio > 0.0 && ratio < 1.0);

        // Kịch bản 3: User vứt code AI đi viết lại hoàn toàn (Thất vọng 100%)
        let rewrite_code = "fn run() {\n    // Code by me\n}";
        assert_eq!(Sessionizer::calculate_edit_ratio(ai_code, rewrite_code), 1.0);
    }
}
