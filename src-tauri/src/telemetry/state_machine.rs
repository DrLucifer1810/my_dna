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

        Ok(StateMachine { conn })
    }

    pub fn log_window_change(&self, title: &str, process_id: u32) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, window_title, process_id) VALUES (?1, ?2, ?3)",
            ("WINDOW_CHANGE", title, process_id),
        )?;
        Ok(())
    }

    pub fn log_clipboard_event(&self, lineage_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, clipboard_lineage) VALUES (?1, ?2)",
            ("CLIPBOARD_COPY", lineage_id),
        )?;
        Ok(())
    }

    pub fn log_focused_text(&self, name: &str, text: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO events (event_type, window_title, focused_text) VALUES (?1, ?2, ?3)",
            ("FOCUSED_TEXT", name, text),
        )?;
        Ok(())
    }

    pub fn get_recent_logs(&self) -> Result<String> {
        let mut stmt = self.conn.prepare("SELECT event_type, window_title, clipboard_lineage, focused_text, timestamp FROM events ORDER BY id DESC LIMIT 50")?;
        let rows = stmt.query_map([], |row| {
            let event_type: String = row.get(0).unwrap_or_default();
            let title: String = row.get(1).unwrap_or_default();
            let clipboard: String = row.get(2).unwrap_or_default();
            let focused: String = row.get(3).unwrap_or_default();
            let timestamp: String = row.get(4).unwrap_or_default();
            Ok(format!("[{}] {} | Win: {} | Clip: {} | Focus: {}", timestamp, event_type, title, clipboard, focused))
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
