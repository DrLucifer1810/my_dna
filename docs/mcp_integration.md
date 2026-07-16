# Hướng dẫn Tích hợp MCP (Model Context Protocol)

MyDNA sở hữu **Kiến trúc MCP Hai Chiều (Dual-Mode)** cực kỳ linh hoạt. Hệ thống vừa đóng vai trò là **Máy chủ Ngữ cảnh (Server)** cung cấp DNA cho các AI khác đọc, vừa đóng vai trò là **Máy khách (Client)** để đi thu thập dữ liệu hành vi từ các nền tảng doanh nghiệp.

---

## PHẦN A: MyDNA LÀ MÁY CHỦ (MCP SERVER / PROVIDER)
Vai trò này cho phép các công cụ AI bên thứ ba (Cursor IDE, Claude Desktop, v.v.) truy cập vào Hồ sơ Năng lực (DNA Profile) của bạn để cá nhân hóa văn phong và phong cách lập trình.

### 1. Cấu hình Máy chủ (Server Details)
- **Giao thức:** SSE (Server-Sent Events) qua HTTP.
- **URL Endpoint:** `http://localhost:5050/mcp/resources/user_dna`
- **Port mặc định:** 5050 (Chạy nền trên Desktop của người dùng).

## 2. Kết nối với Cursor IDE
Để Cursor tự động tuân thủ các nguyên tắc code (Coding Habits & Principles) được cấu trúc trong MyDNA, bạn cần trỏ Context vào máy chủ MCP:

1. Mở phần **Cursor Settings** -> **Features** -> **MCP**.
2. Nhấn **+ Add New MCP Server**.
3. Chọn loại kết nối (Type) là `SSE`.
4. Nhập URL: `http://localhost:5050/mcp/resources/user_dna`.
5. Đặt tên (Name): `MyDNA_Enterprise`.

Khi bạn trò chuyện với Cursor AI, nó sẽ tự động nạp ngữ cảnh từ URL này.

## 3. Cấu trúc Dữ liệu Trả về (JSON Payload)
Máy chủ sẽ trả về một gói JSON tuân thủ chuẩn Resource của MCP. Nếu phát hiện tệp tin bị can thiệp trái phép, máy chủ sẽ trả về lỗi.

### Payload Hợp lệ (Ví dụ)
```json
{
  "jsonrpc": "2.0",
  "result": {
    "name": "mydna_user_context",
    "description": "Comprehensive User DNA for AI Personalization",
    "content": {
      "seniority": "Senior Rust Engineer",
      "daily_focus": "Backend Architecture & Memory Safety",
      "coding_habits": {
        "good": ["Uses Result for error handling", "Writes docstrings"],
        "bad_to_avoid": ["using .unwrap()"],
        "principles": ["Fail-Fast", "No-Mock Policy"]
      },
      "communication": {
        "tone": ["Professional", "Direct"],
        "voice": ["Technical", "Objective"],
        "quirks": []
      }
    }
  }
}
```

### Xử lý Ngoại lệ (Anti-Tampering)
Nếu người dùng cố tình thay đổi file Database `local_events.db` bằng tay để khai khống điểm số, mã băm HMAC-SHA256 (Chữ ký điện tử) sẽ bị sai lệch. 
Lúc này, máy chủ MyDNA sẽ từ chối truy cập và trả về chuẩn lỗi của MCP:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "DATA_TAMPERED: The user DNA profile has been illegally modified."
  }
}
```
Các ứng dụng bên thứ 3 nên kiểm tra mã lỗi `-32603` để cảnh báo hệ thống Admin về hành vi gian lận.

---

## PHẦN B: MyDNA LÀ MÁY KHÁCH (MCP CLIENT / CONSUMER)
Ngược lại với Phần A, ở vai trò này MyDNA sẽ chủ động kết nối đến các Máy chủ MCP của các ứng dụng bên thứ 3 để "bòn rút" dữ liệu hành vi của bạn (nhằm phục vụ cho việc chấm điểm).

### 1. Kiến trúc Ingestion
- Từ giao diện (Tab Integration Hub), bạn dán Token của các dịch vụ như **GitHub, Jira, Slack, Notion**.
- MyDNA (Rust Backend) sẽ dùng Token này để kết nối tới các MCP Server (thông thường chạy qua npx, ví dụ: `npx @modelcontextprotocol/server-github`).
- Toàn bộ dữ liệu thô (Raw JSON) đẩy về sẽ đi qua **Data Ingestion Engine** (`ingestion.rs`).

### 2. Quá trình Chuẩn hóa Dữ liệu (Normalization)
Mỗi dịch vụ sẽ có một Adapter riêng để gọt giũa dữ liệu, tránh tình trạng LLM bị bội thực Token:
- **GitHub Adapter:** Bóc tách Commit Message, Code Diff, PR Review. Gán nhãn `MCP_GITHUB`.
- **Jira Adapter:** Bóc tách Task Name, Status Transitions (để đo Velocity). Gán nhãn `MCP_JIRA`.
- **Slack Adapter:** Lọc Text thuần túy, bỏ Emoji/Tags, đo lường kỹ năng giao tiếp (Tone/Voice). Gán nhãn `MCP_SLACK`.

Dữ liệu cuối cùng được lưu gọn gàng vào Database và chờ Gemini phân tích tổng hợp cuối ngày. Mọi luồng xử lý diễn ra hoàn toàn tự động!
