# 🤖 QUY CHUẨN LÀM VIỆC & PHÁT TRIỂN DỰ ÁN

---

## 1. Khảo vấn & Làm rõ (Grill & Align):
- **Quy tắc:** Không vội vàng viết code khi yêu cầu chưa rõ ràng (Misalignment là nguyên nhân thất bại số 1).
- **Hành động:** Khi nhận yêu cầu phức tạp hoặc thiếu bối cảnh, AI phải **tự động đóng vai người phỏng vấn** (tương tự kỹ năng `/grill-me` hoặc `/grill-with-docs`). Liên tục đặt câu hỏi chi tiết về requirements, edge cases, và thống nhất từ vựng chuyên ngành (ubiquitous language) với user.
- **Kết quả:** Xây dựng được bối cảnh chung (Shared Context) vững chắc trước khi tạo Implementation Plan.

## 2. Test-Driven Development Loop (TDD):
- **Quy tắc:** Tuân thủ vòng lặp Red-Green-Refactor.
- **Hành động:** Viết test hoặc định nghĩa rõ contract/kịch bản test trước khi implement logic. Đảm bảo code được xác minh tính đúng đắn ở mỗi bước thay vì viết một loạt code rồi mới test (tương tự kỹ năng `/tdd`).

## 3. Phân tách Yêu cầu & PRD (Product Requirements Document):
- **Quy tắc:** Task lớn phải được bẻ nhỏ.
- **Hành động:** Chuyển đổi các cuộc hội thoại dài thành tài liệu PRD có cấu trúc, sau đó chia nhỏ thành các issue theo chiều dọc (vertical-slice issues) để xử lý từng phần một cách triệt để (tương tự kỹ năng `/to-prd` và `/to-issues`).

## 4. Bàn giao & Chuyển giao Ngữ cảnh (Handoff):
- **Quy tắc:** Không để mất context giữa các phiên làm việc.
- **Hành động:** Khi kết thúc một task lớn hoặc chuẩn bị dừng phiên làm việc, AI cần tóm tắt lại gọn gàng trạng thái hiện tại (đã làm gì, đang vướng gì, bước tiếp theo là gì) vào file markdown tạm (ví dụ `handoff.md`) hoặc báo cáo trực tiếp để dev/agent sau có thể tiếp tục công việc ngay lập tức (tương tự kỹ năng `/handoff`).

## 5. Kiến trúc & Triage:
- Khi sửa code, luôn chủ động phân tích cơ hội refactor để tăng tính testable và giảm độ rườm rà (tương tự `/improve-codebase-architecture`).
- Tổ chức các task chưa rõ ràng thành các action item cụ thể (tương tự `/triage`).
- Cập nhật đầy đủ chi tiết nội dung điều chỉnh, nâng cấp vào các files thích hợp theo đúng yêu cầu quy cách tiêu chuẩn.

---

## 6. Môi Trường & Dữ Liệu (Production-Ready & Pathing)
- **Tuyệt đối không Mockup/Stub trong Source:** Không được phép chèn các lệnh "mockup", "pass" hay "stub" vào mã nguồn thật với lý do "để bảo vệ file gốc khi chạy test". Mã nguồn phải luôn được viết theo chuẩn Production-ready 100%. Việc bảo vệ dữ liệu là nhiệm vụ của việc chọn đúng môi trường chạy.
- **Tôn trọng Relative Path (Đường dẫn tương đối):** Mọi thao tác đọc/ghi file, lưu DB phải sử dụng cấu hình đường dẫn tương đối (ví dụ: `DATA_DIR`) để nương theo môi trường nơi App thực sự khởi chạy. 
- **Không xả rác vào Codebase (Source vs. Runtime):** Codebase gốc chỉ chứa Source Code và các file cấu hình chuẩn (Knowledge/Skill gốc) dùng để đóng gói (Build). Mọi dữ liệu phát sinh trong quá trình chạy thử hoặc kiểm chứng **phải được sinh ra và cô lập hoàn toàn trong môi trường chạy (ví dụ: thư mục `portable-test`)**. Tuyệt đối không ghi đè dữ liệu runtime hoặc tạo file test rác ngược trở lại Codebase gốc.

## 7. Nguyên tắc chống ảo giác & cấm mock data trong luồng kiểm chứng (Anti-hallucination & No-mock policy)
**Tuyệt đối cấm viết code "giả lập kết quả AI" (Hardcoded AI Fallback/Mocks) trong mọi tình huống:**
- **Không Hardcode kết quả AI:** Khi một hàm có chức năng gọi AI (LLM, Vision, RAG), nếu hệ thống không kết nối được với AI, code **BẮT BUỘC PHẢI BÁO LỖI (RAISE EXCEPTION)** hoặc trả về chuỗi lỗi rõ ràng. Nghiêm cấm việc nhét các chuỗi văn bản cố định (như "Grok là...", "Google Flow...") để "chữa cháy" hoặc vờ như AI đang hoạt động.
- **Fail-Fast thay vì Fail-Silent:** Việc trả về kết quả giả (Mock data) khi test sẽ gây ra hiện tượng **Ảo giác Hệ thống (System Hallucination)**, khiến Dev lầm tưởng tính năng đang hoạt động trơn tru trong khi thực tế module AI đang hỏng. Hệ thống phải Fail-Fast (crash hoặc văng lỗi ngay lập tức) để bộc lộ lỗi kết nối/cấu hình.

---

## ⚡ 11. TOP RULES - KHÔNG BAO GIỜ VI PHẠM

**❌ KHÔNG BAO GIỜ:**
- Tự ý thay đổi logic/architecture hiện có → **Hỏi trước khi thay đổi**
- Compile TypeScript Cloudflare Functions thành JavaScript → **Cloudflare tự compile**
- Sửa backend để phù hợp test → **Sửa test để phù hợp backend contract**
- Giữ lại file plan/report sau khi hoàn thành → **Xóa ngay khi xong**
- **Viết code tạm thời/"tạm bợ" → Phải xử lý kỹ lưỡng, hoàn chỉnh ngay từ đầu**
- **Gọi trực tiếp backend domain từ frontend → Bắt buộc qua Cloudflare proxy**
- **`git add -A` khi chỉ sửa một service cụ thể → Luôn chỉ định đúng path, tránh stage nhầm submodule pointer**

**✅ LUÔN LUÔN:**
- Đọc docs của service trước khi code (nếu có)
- Test đầy đủ trước khi commit
- Commit push cập nhật repo sau mỗi cập nhật.
- Update guides nếu có thay đổi architecture
- Xóa file temp/completed plans ngay khi hoàn thành
- Follow existing patterns và naming conventions
