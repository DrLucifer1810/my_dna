use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};
use windows::Win32::Foundation::HWND;

pub struct ActiveWindow {
    pub title: String,
    pub process_id: u32,
}

pub fn get_active_window() -> Option<ActiveWindow> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        let mut buffer: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        
        let title = if len > 0 {
            String::from_utf16_lossy(&buffer[..len as usize])
        } else {
            String::new()
        };

        Some(ActiveWindow {
            title,
            process_id,
        })
    }
}
