# MyDNA - Decentralized AI-Coworker Passport

## Tổng quan
MyDNA là một hệ thống Agentic cục bộ (Local Agent) hoạt động ngầm trên thiết bị của bạn. Nó đóng vai trò như một chiếc "Hộ chiếu Kỹ năng", liên tục quan sát cách bạn làm việc với AI và các công cụ hàng ngày để đánh giá năng lực thực sự của bạn một cách khách quan.

## Tính năng cốt lõi (Core Features)

### 1. Enterprise Evaluation Framework (Matrix 6 Chiều)
Hệ thống không chỉ đếm số lượng copy/paste, mà sử dụng AI để đánh giá bạn dựa trên 6 khía cạnh chuẩn Enterprise:
- **Năng lực làm việc (Competence):** Khả năng giải quyết vấn đề bằng AI.
- **Tính kỷ luật (Discipline):** Code/Văn bản có tuân thủ quy chuẩn không?
- **Khả năng sáng tạo (Creativity):** Bạn có tùy biến kết quả AI hay chỉ copy mù quáng?
- **Phản biện (Critical Thinking):** Bạn có nhận ra và sửa lỗi của AI (Hallucination) không?
- **Làm việc nhóm (Collaboration):** Kết quả cuối cùng có dễ đọc, dễ chia sẻ cho team không?
- **Hiệu suất AI (AI Token Efficiency):** Bạn dùng prompt có ngắn gọn, tiết kiệm chi phí Token nhưng mang lại hiệu quả cao không?

### 2. Diff Engine & Token Efficiency
Hệ thống tự động theo dõi khi bạn nhận kết quả từ AI (Clipboard Copy) và lưu file (File Saved).
- Tính toán **Edit Ratio**: Đo lường lượng thời gian bạn phải bỏ ra để sửa lại kết quả của AI.
- Tính toán **Token Efficiency**: Đánh giá chi phí (dung lượng prompt) so với chất lượng cuối cùng.

### 3. User Profiling Diagnostic
Dựa trên tất cả các tài liệu bạn đã đọc, code bạn đã viết, email bạn đã gửi trong 24 giờ qua:
- AI sẽ tự động tổng hợp và **chẩn đoán chức danh nghề nghiệp** của bạn (ví dụ: Senior Rust Developer, Marketing Lead).
- Phát hiện **Daily Focus**: Bạn đang dành thời gian nhiều nhất cho việc gì?
- Xác định **Tech Stack**: Các công cụ, ngôn ngữ bạn đang thao tác.

### 4. UI Dashboard & Settings
Giao diện HTML/JS trực quan, mượt mà:
- **Radar Chart:** Biểu đồ mạng nhện hiển thị trực tiếp chỉ số Năng lực 6 chiều của bạn.
- **Settings:** Tuỳ chỉnh tắt/bật các bộ theo dõi (Window, Clipboard, File) hoặc bắt buộc chạy luồng phân tích lập tức (Force Diagnostic).

## Cài đặt & Chạy ứng dụng

Yêu cầu:
- Trình biên dịch Rust & Cargo (`rustup`)
- Node.js & npm

```bash
cd mydna
npm install
npm run tauri dev
```

> **Bảo mật & Quyền riêng tư (PIR):** Mọi thông tin nhạy cảm (Email, API Keys, Passwords, Thẻ tín dụng) đều bị bộ lọc PIR (Privacy Information Retrieval) che lại thành `[REDACTED]` ngay trên máy tính của bạn trước khi đi qua luồng đánh giá AI.
