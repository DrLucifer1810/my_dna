use rusqlite::{Connection, Result};
use std::path::Path;

pub struct StateMachine {
    pub conn: Connection,
}

impl StateMachine {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        
        // Khởi tạo bảng sự kiện
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                window_title TEXT,
                process_id INTEGER,
                clipboard_lineage TEXT,
                focused_text TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Phase 1.3: Thêm cột raw_content để lưu trữ nội dung thực tế (Semantic Diff)
        let _ = conn.execute("ALTER TABLE events ADD COLUMN raw_content TEXT", []);

        // Phase 1.7: Bảng Sessions để gom nhóm sự kiện theo luồng công việc
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                start_time DATETIME DEFAULT CURRENT_TIMESTAMP,
                end_time DATETIME,
                category TEXT,
                raw_context TEXT
            )",
            [],
        )?;

        // Phase 1.7: Bảng Session Evaluations để lưu điểm đánh giá từ LLM (Chuẩn Enterprise Matrix)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_evaluations (
                session_id INTEGER PRIMARY KEY,
                prompt_quality INTEGER,
                edit_ratio REAL,
                competence INTEGER,
                discipline INTEGER,
                creativity INTEGER,
                critical_thinking INTEGER,
                collaboration INTEGER,
                ai_efficiency INTEGER,
                tips TEXT,
                signature TEXT,
                FOREIGN KEY(session_id) REFERENCES sessions(id)
            )",
            [],
        )?;

        // Phase 1.9: Bảng User DNA để lưu kết quả phân tích từ nhiều Agent khác nhau
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_dna (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_type TEXT NOT NULL,
                extracted_traits TEXT NOT NULL,
                signature TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Phase 1.13: Bảng P2P Public Profile để công khai trên mạng
        conn.execute(
            "CREATE TABLE IF NOT EXISTS p2p_public_profile (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                public_key TEXT UNIQUE,
                title TEXT,
                radar_scores TEXT,
                tech_stack TEXT,
                principles TEXT,
                communication_style TEXT,
                work_habits TEXT,
                signature TEXT,
                last_updated DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // Phase 4: Bảng Settings cho cấu hình Telegram MentorAI
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1), -- Đảm bảo chỉ có 1 dòng
                telegram_bot_token TEXT,
                telegram_chat_id TEXT,
                mentor_ai_enabled BOOLEAN DEFAULT 1
            )",
            [],
        )?;
        
        // Chèn dòng cấu hình mặc định nếu chưa có
        conn.execute(
            "INSERT OR IGNORE INTO settings (id, mentor_ai_enabled) VALUES (1, 1)",
            [],
        )?;

        Ok(StateMachine { conn })
    }

    pub fn log_window_change(&self, title: &str, process_id: u32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, window_title, process_id) VALUES (?1, ?2, ?3)",
            ("WINDOW_CHANGE", title, process_id),
        )?;
        Ok(())
    }

    pub fn log_clipboard_event(&self, lineage_id: &str, raw_content: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, clipboard_lineage, raw_content) VALUES (?1, ?2, ?3)",
            ("CLIPBOARD_COPY", lineage_id, raw_content),
        )?;
        Ok(())
    }

    pub fn log_focused_text(&self, name: &str, text: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, window_title, focused_text, raw_content) VALUES (?1, ?2, ?3, ?4)",
            ("FOCUSED_TEXT", name, text, text), // Lưu text vào raw_content luôn để dễ đọc
        )?;
        Ok(())
    }

    pub fn log_file_saved(&self, file_path: &str, content: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, window_title, raw_content) VALUES (?1, ?2, ?3)",
            ("FILE_SAVED", file_path, content),
        )?;
        Ok(())
    }

    pub fn get_recent_logs(&self) -> Result<String> {
        let mut stmt = self.conn.prepare("SELECT event_type, window_title, clipboard_lineage, focused_text, raw_content, timestamp FROM events ORDER BY id DESC LIMIT 50")?;
        let rows = stmt.query_map([], |row| {
            let event_type: String = row.get(0).unwrap_or_default();
            let title: String = row.get(1).unwrap_or_default();
            let clipboard: String = row.get(2).unwrap_or_default();
            let focused: String = row.get(3).unwrap_or_default();
            let raw: String = row.get(4).unwrap_or_default();
            let timestamp: String = row.get(5).unwrap_or_default();
            
            // Format output để LLM đọc được Content
            let mut log = format!("[{}] {} | Win: {} | Clip: {} | Focus: {}", timestamp, event_type, title, clipboard, focused);
            if !raw.is_empty() {
                log.push_str(&format!(" | RAW_CONTENT: {}", raw.replace("\n", " ")));
            }
            Ok(log)
        })?;

        let mut logs = Vec::new();
        for row in rows {
            if let Ok(log) = row {
                logs.push(log);
            }
        }
        
        // Đảo ngược lại để theo đúng trình tự thời gian tăng dần
        logs.reverse();
        Ok(logs.join("\n"))
    }

    // --- CÁC HÀM CHO PHASE 4: SETTINGS ---
    pub fn get_telegram_token(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT telegram_bot_token FROM settings WHERE id = 1")?;
        let token: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap_or(None);
        Ok(token)
    }

    pub fn set_telegram_token(&self, token: &str) -> Result<()> {
        self.conn.execute("UPDATE settings SET telegram_bot_token = ?1 WHERE id = 1", (token,))?;
        Ok(())
    }

    pub fn get_telegram_chat_id(&self) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT telegram_chat_id FROM settings WHERE id = 1")?;
        let chat_id: Option<String> = stmt.query_row([], |row| row.get(0)).unwrap_or(None);
        Ok(chat_id)
    }

    pub fn set_telegram_chat_id(&self, chat_id: &str) -> Result<()> {
        self.conn.execute("UPDATE settings SET telegram_chat_id = ?1 WHERE id = 1", (chat_id,))?;
        Ok(())
    }
    
    pub fn is_mentor_enabled(&self) -> Result<bool> {
        let mut stmt = self.conn.prepare("SELECT mentor_ai_enabled FROM settings WHERE id = 1")?;
        let enabled: bool = stmt.query_row([], |row| row.get(0)).unwrap_or(true);
        Ok(enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_machine_in_memory() {
        // Dùng SQLite in-memory để test không chạm đĩa
        let sm = StateMachine::new(":memory:").expect("Failed to init in-memory DB");
        
        sm.log_window_change("MySecretApp", 1234).unwrap();
        sm.log_clipboard_event("clip_123", "Pasted Code 100%").unwrap();
        sm.log_file_saved("C:\\path\\code.rs", "fn main() {}").unwrap();
        
        let logs = sm.get_recent_logs().unwrap();
        assert!(logs.contains("MySecretApp"));
        assert!(logs.contains("Pasted Code 100%"));
        assert!(logs.contains("fn main() {}"));
    }

    #[test]
    fn test_real_db_capture() {
        if let Ok(sm) = StateMachine::new("portable-test/local_events.db") {
            println!("--- REAL PRODUCTION LOGS ---");
            println!("{}", sm.get_recent_logs().unwrap_or_default());
            println!("----------------------------");
        }
    }
}
