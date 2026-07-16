# Kiến trúc Kỹ thuật MyDNA (Decentralized AI-Coworker Passport)

Tài liệu này mô tả chi tiết toàn bộ các kỹ thuật thực tế (Production-Ready) đã được triển khai, đảm bảo nguyên tắc **không sử dụng dữ liệu giả lập (No-Mocking)** trong lõi hệ thống. Hệ thống đã được nâng cấp với Kiến trúc Multi-Agent và Model Context Protocol (MCP).

## 1. Omniscient Telemetry Engine (Rust / Win32 API)
Kiến trúc truy vết được thiết kế ở cấp độ hệ điều hành Windows để thu thập 100% dữ liệu tương tác mà không bỏ sót bất kỳ chi tiết nào, hoạt động cho mọi ngành nghề (Coder, Marketer, HR, etc.).

*   **Deep UI Tree Walker (`accessibility.rs`)**: Không chỉ lấy thẻ đang focus, hệ thống sử dụng `UITreeWalker` (UIAutomation) để trích xuất (dump) toàn bộ văn bản hiển thị trên màn hình hiện tại (từ Log lỗi Terminal, Email đến nội dung Chat) thành Raw Text.
*   **Global Keylogger (`worker.rs` & `rdev`)**: Bắt toàn bộ sự kiện gõ phím ở mức OS Level để biết chính xác quá trình tư duy và sửa lỗi của người dùng.
*   **Window Tracker & Clipboard (`window_tracker.rs`, `clipboard.rs`)**: Lắng nghe sự thay đổi cửa sổ để đo `context_switches` và lưu vết Clipboard để liên kết luồng Copy/Paste.
*   **Chronological Sessionizer (`sessionizer.rs`)**: Kết nối toàn bộ Screen Dumps, Keystrokes, AI Outputs thành một Chuỗi Thời Gian (Chronological Stream) liền mạch gửi thẳng cho LLM phân tích.
*   **State Machine (`state_machine.rs`)**: Quản lý State cục bộ và ghi log thô vào `portable-test/local_events.db` (SQLite).

## 2. Omni-Channel Integration Hub & Data Ingestion Engine
MyDNA sở hữu kiến trúc thu thập dữ liệu đa nguồn thế hệ mới, thay thế hoàn toàn phương pháp thăm dò (Polling) kém hiệu quả:
*   **Local Native Sensors (WebSockets):** Các Extension độc lập (VS Code, Chrome) kết nối trực tiếp với backend qua WebSocket.
*   **Agentic Log Watcher (Zero-Setup):** Hệ thống sử dụng Rust `notify` để âm thầm quét và lắng nghe sự thay đổi từ các thư mục nhật ký của Autonomous AI Agents (Antigravity, Claude Code, OpenClaw, Cline) mà không cần cấu hình proxy.
*   **Data Ingestion Pipeline (`ingestion.rs`):** Bộ chuẩn hóa dữ liệu trung tâm. Mọi gói JSON hỗn tạp từ MCP (GitHub, Jira, Slack) hay Extensions đều đi qua các Adapters để gọt giũa, chỉ giữ lại ngữ cảnh cốt lõi (VD: Tên Commit, Lệnh AI, Nội dung Chat) và dán nhãn chuẩn (`MCP_GITHUB`, `AGENTIC_LLM`) trước khi lưu vào SQLite. Điều này chống "bội thực Token" (Token bloat) cho LLM.

## 3. Dynamic Data Synthesizer (Gemini AI API)
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

## 5. MCP (Model Context Protocol) - Kiến trúc Hai chiều (Dual-Mode)
MyDNA hoạt động cả ở vai trò **Server** lẫn **Client** trong hệ sinh thái MCP:
*   **MCP Context Server (Provider):** 
    - Tích hợp Framework `axum` chạy nền trong Tauri (Port `5050`). Cung cấp SSE stream.
    - Endpoint: `http://localhost:5050/mcp/resources/user_dna`.
    - Các AI IDE (Cursor, Claude Desktop) kết nối vào để đọc Hồ sơ DNA, tự động tuỳ chỉnh văn phong, nguyên tắc lập trình sao cho khớp với chính User.
*   **MCP Client (Consumer):**
    - MyDNA làm Client kết nối đến các MCP Server của doanh nghiệp (GitHub, Jira, Slack, Notion) để bòn rút dữ liệu hành vi (VD: Đo lường chất lượng PR Review, tiến độ Jira Task, kỹ năng giao tiếp Slack) làm đầu vào cho Ingestion Engine.

## 6. Storage & Cloud Sync (Google Drive)
*   OAuth2 qua Google API để kết nối trực tiếp với Google Drive người dùng.
*   Thư mục DACP_Workspace.
*   Đồng bộ luồng sự kiện (Logs/DNA) lên Cloud.

## 6. Giao diện Frontend (Tauri + Vanilla JS)
*   Tương tác mượt mà qua Tauri WebView.
*   Cung cấp Biểu đồ màng nhện (Radar Chart) hiển thị năng lực làm việc với AI qua `Chart.js`.
*   Hiển thị trực quan dữ liệu **Hồ sơ năng lực (DNA Profile)** gồm Seniority, Coding Habits, Tone & Voice lấy trực tiếp từ bảng `user_dna`.

## 7. Auto-Updater Mechanism (Cập nhật Không chạm)
*   **Tauri Updater Plugin:** Hệ thống tích hợp `tauri-plugin-updater` để kết nối trực tiếp đến một Repository mã nguồn mở (`DrLucifer1810/my_dna_release`), cho phép kiểm tra phiên bản mới theo định kỳ.
*   **Seamless Reboot:** Tải file `.msi` / `.exe` bản quyền từ GitHub Releases và khởi động lại ứng dụng thông qua `tauri-plugin-process` mà không cần người dùng thao tác.

## 8. Anti-Tampering & Network Security (Bảo mật Toàn vẹn)
*   **Row-level Cryptographic Signature:** Mọi điểm số và DNA được ghi vào Database đều phải ký chữ ký điện tử HMAC-SHA256.
*   **Fail-Fast Integrity Check:** API cung cấp dữ liệu liên tục verify chữ ký lúc Runtime.
*   **Dynamic Standard Engine (Ed25519 Global Verification):** Tự động tải `standards_registry.json` từ trung tâm và kiểm tra chữ ký số Ed25519 (Fail-fast). Nếu user tự sửa nội dung `prompts.yaml`, hàm băm bị sai khác, app sẽ văng lỗi ngay lập tức mà không tiếp tục.
*   **P2P Cross-Validation (Đã Triển Khai):** Khởi chạy mạng Kademlia/GossipSub. Gói tin `MatchIntent` nhúng cứng `standard_hash`. Mọi Node nhận được tin sẽ kiểm tra chéo (Cross-Validate) cái hash đó với danh sách chuẩn của riêng nó. Nếu phát hiện hash rác, hệ thống ngầm tẩy chay (Block) toàn bộ gói tin, đảm bảo hacker bị cô lập hoàn toàn khỏi mạng P2P. Mọi cơ chế đều thực thi chuẩn (Zero-Mocking).
*   **Decentralized Event Listener**: Ứng dụng tích hợp trực tiếp SDK Telegram Bot (`teloxide`) chạy ngầm trong máy tính của người dùng (Zero Conflict, Zero Polling Overhead). 
*   **100% Privacy**: Mỗi user dùng Token bot của riêng mình. Không yêu cầu máy chủ trung gian (Serverless / Webhook / Cloudflare), đảm bảo tin nhắn không bao giờ bị lộ ra ngoài.
*   **Proactive Push**: Tích hợp luồng phân tích 24h tự động để đúc kết "Thói quen xấu" (Bad Habits) và đẩy trực tiếp một lời khuyên ngắn gọn, mang tính hành động qua Telegram mỗi ngày. User có thể chat 2 chiều để hỏi sâu hơn dựa vào Context cục bộ.
