use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tokio::task;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::System::SystemInformation::GetTickCount;

use crate::telemetry::window_tracker::get_active_window;
use crate::telemetry::clipboard::check_clipboard;
use crate::telemetry::accessibility::get_focused_element;
use crate::telemetry::state_machine::StateMachine;

pub fn spawn_telemetry_loop(state: Arc<Mutex<StateMachine>>) {
    task::spawn_blocking(move || {
        let mut last_window = String::new();
        let mut last_clipboard = String::new();

        loop {
            // Enterprise Audit Phase 1.4: Tối ưu hiệu năng bằng Idle Detection
            unsafe {
                let mut last_input = LASTINPUTINFO {
                    cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                    dwTime: 0,
                };
                if GetLastInputInfo(&mut last_input) != 0 {
                    let current_tick = GetTickCount();
                    let idle_time_ms = current_tick.saturating_sub(last_input.dwTime);
                    
                    // Nếu user không làm gì trong 60 giây (idle), tạm ngưng quét để tiết kiệm Pin và CPU
                    if idle_time_ms > 60_000 {
                        sleep(Duration::from_secs(10));
                        continue; // Bỏ qua lượt quét này
                    }
                }
            }

            let mut window_or_clipboard_changed = false;

            // 1. Theo dõi Window
            if let Some(win) = get_active_window() {
                if win.title != last_window {
                    last_window = win.title.clone();
                    window_or_clipboard_changed = true;
                    if let Ok(state_lock) = state.lock() {
                        let _ = state_lock.log_window_change(&last_window, win.process_id);
                    }
                }
            }

            // 2. Theo dõi Clipboard
            if let Some(clip) = check_clipboard() {
                if clip.lineage_id != last_clipboard {
                    last_clipboard = clip.lineage_id.clone();
                    window_or_clipboard_changed = true; // Trạng thái màn hình có biến động
                    if let Ok(state_lock) = state.lock() {
                        let _ = state_lock.log_clipboard_event(&last_clipboard, &clip.content);
                    }
                }
            }

            // 3. Dynamic Polling: Chỉ gọi UIAutomation (Cực tốn tài nguyên COM) khi màn hình/clipboard vừa thay đổi
            if window_or_clipboard_changed {
                if let Some(focused) = get_focused_element() {
                    if let Ok(state_lock) = state.lock() {
                        let _ = state_lock.log_focused_text(&focused.name, &focused.text_value);
                    }
                }
            }

            // Chu kỳ 2 giây/lần
            sleep(Duration::from_secs(2));
        }
    });
}
