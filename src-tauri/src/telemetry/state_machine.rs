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
}
