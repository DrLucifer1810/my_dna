# Kiến trúc Kỹ thuật MyDNA (Decentralized AI-Coworker Passport)

Tài liệu này mô tả chi tiết toàn bộ các kỹ thuật thực tế (Production-Ready) đã được triển khai, đảm bảo nguyên tắc **không sử dụng dữ liệu giả lập (No-Mocking)** trong lõi hệ thống. Hệ thống đã được nâng cấp với Kiến trúc Multi-Agent và Model Context Protocol (MCP).

## 1. Cross-App Telemetry Engine (Rust / Win32 API)
Kiến trúc truy vết được thiết kế ở cấp độ hệ điều hành Windows, loại bỏ sự phụ thuộc vào các Web Scraper truyền thống.

*   **Window Tracker (`window_tracker.rs`)**: Sử dụng API `GetForegroundWindow` và `GetWindowThreadProcessId` (Microsoft Win32 API) để ghi nhận ứng dụng đang Active.
*   **Clipboard Lineage (`clipboard.rs`)**: Lắng nghe sự kiện Copy/Paste qua `clipboard-win`, áp dụng băm `Lineage ID` để liên kết luồng thông tin mà không lưu toàn bộ Text nhằm bảo mật quyền riêng tư.
*   **Accessibility Hook (`accessibility.rs`)**: Tương tác với COM API qua `uiautomation` để lấy Element được Focus.
*   **State Machine (`state_machine.rs`)**: Quản lý State cục bộ và ghi log thô vào `portable-test/local_events.db` (SQLite).

## 2. Dynamic Data Synthesizer (Gemini AI API)
Hệ thống kết nối trực tiếp đến **Google Gemini API** (`gemini.rs`) để chấm điểm hiệu suất sử dụng AI:
*   Đánh giá theo Dreyfus Model (Mức độ thành thạo) và Bloom's Taxonomy (Mức độ nhận thức).
*   Chấm điểm chất lượng (Quality Assessment) của code hoặc email cuối cùng.
*   Tính toán Radar Score (Biểu đồ năng lực doanh nghiệp) qua D1-D6.

## 3. Modular Multi-Agent Profiler
Phân rã tác vụ nội bộ thành mô hình đa đặc vụ (Multi-Agent) chuyên biệt:
*   **Kiến trúc:** Thay vì hardcode Prompts, hệ thống sử dụng cấu hình YAML (`portable-test/prompts.yaml`) nạp động qua thư viện `serde_yaml`. Cho phép sửa Prompt mà không cần build lại App (Hot-Reload).
*   **Agent Slices:** Prompts được lắp ghép từ 3 Lát Cắt (Role, Goal, Backstory) và nội suy tự động (Interpolation).
*   **Các Agents hiện tại:**
    - `CodeAnalyzerAgent`: Phân tích Habit, Principle, Tech Stack.
    - `CommunicationAnalyzerAgent`: Phân tích Tone, Voice, Quirks.
    - `CareerDiagnosticAgent`: Đoán chức danh và trọng tâm công việc hàng ngày.
*   Dữ liệu sau khi phân tích sẽ được lưu dưới dạng JSON phân mảnh vào bảng `user_dna`.

## 4. MCP Context Server (Model Context Protocol)
MyDNA hoạt động như một máy chủ ngữ cảnh nội bộ (Context Provider):
*   Tích hợp Framework `axum` chạy nền trong Tauri (Port `5050`).
*   Endpoint: `http://localhost:5050/mcp/resources/user_dna`. Cung cấp SSE stream theo chuẩn MCP.
*   Trình thông dịch AI bên thứ 3 (Cursor, Claude Desktop, DevTools) có thể kết nối vào đây để tự động tuỳ chỉnh văn phong, nguyên tắc lập trình sao cho khớp với chính User.

## 5. Storage & Cloud Sync (Google Drive)
*   OAuth2 qua Google API để kết nối trực tiếp với Google Drive người dùng.
*   Thư mục DACP_Workspace.
*   Đồng bộ luồng sự kiện (Logs/DNA) lên Cloud.

## 6. Giao diện Frontend (Tauri + Vanilla JS)
*   Tương tác mượt mà qua Tauri WebView.
*   Cung cấp Biểu đồ màng nhện (Radar Chart) hiển thị năng lực làm việc với AI qua `Chart.js`.
*   Hiển thị trực quan dữ liệu **Hồ sơ năng lực (DNA Profile)** gồm Seniority, Coding Habits, Tone & Voice lấy trực tiếp từ bảng `user_dna`.

## 7. Anti-Tampering & Network Security (Bảo mật Toàn vẹn)
*   **Row-level Cryptographic Signature:** Mọi điểm số và DNA được ghi vào Database đều phải ký chữ ký điện tử HMAC-SHA256 bằng Khóa bí mật (Secret Key) được lưu trữ sâu trong **Windows Credential Manager (OS Keyring)**. Tuyệt đối không hardcode khóa trong source code.
*   **Fail-Fast Integrity Check:** API cung cấp dữ liệu liên tục verify chữ ký lúc Runtime. Nếu phát hiện user dùng tool (như DB Browser) sửa trộm điểm thành "100", hệ thống lập tức ném mã lỗi `-32603 (DATA_TAMPERED)`.
*   **P2P Cross-Validation (Phase 2 Roadmap):** Để ngăn chặn rủi ro user can thiệp tận cấp độ mã băm hệ thống, ứng dụng sẽ được mở rộng bằng cơ chế Mạng ngang hàng (P2P Network). Hồ sơ DNA của một nút mạng (node) sẽ được gửi đi để đối soát chéo (cross-validated) bởi các máy tính khác trong mạng lưới Enterprise. Nếu băm sai lệch so với phân phối chung, hồ sơ sẽ bị hệ thống đánh dấu "Unverified".
