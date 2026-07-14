# AI Matching & P2P Scoring Architecture

## 1. Mục đích
Tài liệu này mô tả kiến trúc phân tách và so khớp (Matching) đa chiều giữa các node (Ứng viên và Nhà tuyển dụng) thông qua AI (Gemini) và mạng lưới P2P GossipSub trong dự án MyDNA.

## 2. Cấu trúc Matching Profile Đa Chiều
Dữ liệu JD thô (từ Nhà tuyển dụng) hoặc Hồ sơ/CV (từ Ứng viên) sẽ được AI tự động phân tách và chuyển hóa thành cấu trúc JSON chuẩn `MatchingProfile` như sau:
```json
{
    "tech_stack": [{"name": "Rust", "weight": 1.0}, {"name": "React", "weight": 0.8}],
    "domain_knowledge": [{"name": "Blockchain", "weight": 0.5}],
    "seniority_level": "Senior",
    "work_model": "Remote",
    "min_salary": 2000,
    "max_salary": 5000
}
```

## 3. Quy trình Trích xuất (Parsing) qua AI
- Thông qua SLM Client (Gemini WebView Companion ẩn `gemini-companion-bg`), frontend gửi đoạn văn bản tự do của người dùng.
- Tauri commands `parse_jd_to_profile` và `parse_cv_to_profile` sẽ tự động tiêm (inject) System Prompt yêu cầu AI chỉ trả về cấu trúc JSON duy nhất mà không có giải thích.
- Kết quả được lấy từ DOM và trả về UI để người dùng xác nhận trước khi phát sóng lên mạng lưới.

## 4. Thuật toán Đối soát Chéo (Cross-match Scoring)
Quá trình so khớp diễn ra hoàn toàn P2P ở background, khi một node nhận được `MatchIntent` qua topic `/mydna/recruitment/1.0.0` hoặc `/mydna/freelance/1.0.0`:

1. **Lọc Cứng (Hard Filters):**
   - Lọc bỏ ngay nếu `min_salary` của ứng viên cao hơn `max_salary` của nhà tuyển dụng và ngược lại.
   - Lọc bỏ nếu hình thức làm việc (`work_model`) yêu cầu không khớp.

2. **Chấm điểm Trọng số (Dot Product for Tech Stack):**
   - Thuật toán tính tổng điểm (total_score) dựa trên phép nhân trọng số kỹ năng tương đương của cả 2 phía.
   - Tính tỷ lệ khớp `(total_score / total_weight) * 100%`.
   - Ngưỡng tối thiểu là **60%**, dưới mức này sẽ tự động loại bỏ (reject).

## 5. Cập nhật Giao diện
- Giao diện cung cấp một `textarea` để user dán nội dung JD hoặc CV.
- Nút "Dùng AI Sinh Trọng Số Matching" sẽ kích hoạt pipeline xử lý và tính toán.
- 100% dữ liệu xử lý cục bộ trên browser WebView Companion của người dùng. Không có API Gateway tập trung.
