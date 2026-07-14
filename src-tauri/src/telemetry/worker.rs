use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

use crate::telemetry::window_tracker::get_active_window;
use crate::telemetry::clipboard::check_clipboard;
use crate::telemetry::accessibility::get_focused_element;
use crate::telemetry::state_machine::StateMachine;

pub fn spawn_telemetry_loop(state: Arc<Mutex<StateMachine>>) {
    tokio::spawn(async move {
        let mut last_window = String::new();
        let mut last_clipboard = String::new();

        loop {
            // Theo dõi Active Window
            if let Some(window) = get_active_window() {
                if window.title != last_window {
                    last_window = window.title.clone();
                    if let Ok(state_lock) = state.lock() {
                        let _ = state_lock.log_window_change(&last_window, window.process_id);
                    }
                }
            }

            // Theo dõi Clipboard
            if let Some(clip) = check_clipboard() {
                if clip.lineage_id != last_clipboard {
                    last_clipboard = clip.lineage_id.clone();
                    if let Ok(state_lock) = state.lock() {
                        let _ = state_lock.log_clipboard_event(&last_clipboard, &clip.content);
                    }
                }
            }

            // Theo dõi Accessibility Text (Focused Text)
            // Lưu ý: Chỉ lấy text nếu có thay đổi để tránh ngập lụt Log (Spam)
            if let Some(focused) = get_focused_element() {
                if !focused.text_value.is_empty() {
                    // Để tối ưu MVP, chúng ta có thể mở rộng bảng events để lưu Focused Text
                    // Tạm thời, gọi thẳng SQL qua state_lock nếu cần mở rộng.
                    // (Sẽ triển khai hàm `log_focused_text` trong state_machine.rs)
                    if let Ok(state_lock) = state.lock() {
                        let _ = state_lock.log_focused_text(&focused.name, &focused.text_value);
                    }
                }
            }

            // Chu kỳ 2 giây/lần
            sleep(Duration::from_secs(2)).await;
        }
    });
}
