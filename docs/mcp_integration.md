# Hướng dẫn Tích hợp MCP (Model Context Protocol)

MyDNA đóng vai trò là một **Local Context Server** (Máy chủ Ngữ cảnh Cục bộ) tuân thủ hoàn toàn tiêu chuẩn MCP do Anthropic đề xuất.
Điều này cho phép các công cụ AI bên thứ ba (Cursor IDE, Claude Desktop, v.v.) truy cập vào Hồ sơ Năng lực (DNA Profile) của bạn để cá nhân hóa văn phong và phong cách lập trình.

## 1. Cấu hình Máy chủ (Server Details)
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
