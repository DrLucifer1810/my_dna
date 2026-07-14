# Kiến trúc Kỹ thuật MyDNA (Decentralized AI-Coworker Passport)

Tài liệu này mô tả chi tiết toàn bộ các kỹ thuật thực tế (Production-Ready) đã được triển khai, đảm bảo nguyên tắc **không sử dụng dữ liệu giả lập (No-Mocking)** trong lõi hệ thống.

## 1. Cross-App Telemetry Engine (Rust / Win32 API)
Kiến trúc truy vết được thiết kế ở cấp độ hệ điều hành Windows, loại bỏ sự phụ thuộc vào các Web Scraper truyền thống.

*   **Window Tracker (`window_tracker.rs`)**: 
    Sử dụng hàm `GetForegroundWindow` và `GetWindowThreadProcessId` từ thư viện `windows-rs` (Microsoft Win32 API) để ghi nhận ứng dụng nào đang được người dùng thao tác theo thời gian thực (Active Window).
*   **Clipboard Lineage (`clipboard.rs`)**: 
    Sử dụng thư viện `clipboard-win` để bắt luồng sự kiện Copy/Paste. Áp dụng thuật toán băm (Hashing) qua `DefaultHasher` để khởi tạo `Lineage ID`, giúp liên kết dòng chảy dữ liệu giữa các ứng dụng mà không cần lưu trữ toàn bộ nội dung text (đảm bảo quyền riêng tư).
*   **Accessibility Hook (`accessibility.rs`)**: 
    Tương tác trực tiếp với COM API qua thư viện `uiautomation`. Hệ thống trích xuất `Name` và `ClassName` của phần tử đang được focus (Focused Element) để nắm bắt ngữ cảnh thao tác của người dùng.
*   **State Machine (`state_machine.rs`)**: 
    Quản lý trạng thái và ghi log vào cơ sở dữ liệu nội bộ SQLite (thông qua `rusqlite`). Cơ sở dữ liệu được cấu hình tại thư mục `portable-test/local_events.db` (git ignored) nhằm bảo vệ không gian source code.

## 2. Data Synthesizer (Gemini AI API)
Xử lý dữ liệu không thông qua mock data mà kết nối trực tiếp đến **Google Gemini API** (`gemini.rs`):
*   Sử dụng thư viện `reqwest` gửi HTTP Request.
*   Nạp đầu vào là chuỗi sự kiện thô (Raw Timeline Logs).
*   Gemini phân tích và trả về cấu trúc dữ liệu JSON biểu diễn **Biểu đồ Radar 6 Chiều** (D1-D6).
*   **Cơ chế Fail-Fast**: Nếu lỗi mạng, API Key không hợp lệ hoặc model trả về sai cấu trúc, hệ thống lập tức Raise Error để báo cáo, không bao giờ dùng `[0,0,0,0,0,0]` cứng trong backend để lấp liếm lỗi kết nối.

## 3. Storage & Cloud Sync (Google Drive)
Mô-đun `gdrive.rs` phụ trách tương tác với Google Drive API (OAuth2) sử dụng `reqwest`:
*   Khởi tạo và quản lý thư mục `DACP_Workspace` qua API `drive/v3/files`.
*   Cung cấp hàm `upload_log` để tải luồng sự kiện (Multipart Upload) lên Drive.
*   Luồng gọi API đảm bảo kiểm tra HTTP Status (`is_success()`) để Fail-Fast nếu có lỗi phân quyền.

## 4. Giao diện Frontend (Tauri + Chart.js)
Giao diện người dùng Desktop tĩnh (Vanilla JS, HTML, CSS):
*   Tương tác mượt mà nhờ chạy trong WebView của Tauri, không tiêu tốn RAM như Electron.
*   Render biểu đồ Radar trực quan (Radar Chart) thông qua Chart.js.
*   Có kết nối chờ IPC (`invoke`) tới Rust để thiết lập luồng đăng nhập Google OAuth2 trong tương lai.
