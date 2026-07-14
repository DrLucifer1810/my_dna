use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use tokio::task;

use crate::telemetry::state_machine::StateMachine;

pub fn spawn_file_watcher(state: Arc<Mutex<StateMachine>>, watch_dir: &str) {
    let watch_path = watch_dir.to_string();
    
    // Tạo thư mục nếu chưa có
    if !Path::new(&watch_path).exists() {
        let _ = fs::create_dir_all(&watch_path);
    }

    task::spawn_blocking(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Sử dụng cấu hình mặc định, có thể bỏ qua lỗi unwrap vì chạy nền
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        
        // Bắt đầu theo dõi
        watcher.watch(Path::new(&watch_path), RecursiveMode::Recursive).unwrap();

        for res in rx {
            match res {
                Ok(Event { kind: EventKind::Modify(_), paths, .. }) => {
                    for path in paths {
                        // Bỏ qua các file ẩn/tạm
                        if let Some(ext) = path.extension() {
                            if ext == "tmp" || ext == "crdownload" { continue; }
                        }
                        
                        // Đọc nội dung file
                        if let Ok(content) = fs::read_to_string(&path) {
                            if !content.is_empty() {
                                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                                if let Ok(state_lock) = state.lock() {
                                    let _ = state_lock.log_file_saved(&filename, &content);
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
        }
    });
}
