use rusqlite::{Connection, Result};
use std::path::Path;

pub struct StateMachine {
    conn: Connection,
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
}
