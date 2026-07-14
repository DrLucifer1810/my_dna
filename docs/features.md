# Mô tả Tính năng MyDNA (Features)

MyDNA là một **"Hộ chiếu Năng lực Kháng giả lập" (Decentralized AI-Coworker Passport)**. Phần mềm chạy ngầm trên Desktop để thu thập, phân tích và tự động cung cấp bộ hồ sơ hành vi của bạn cho các AI khác.

## 1. Radar Chấm điểm Hiệu suất AI (AI Efficiency Radar)
Thay vì chỉ thu thập thông số gõ phím vô tri, MyDNA theo dõi **luồng tư duy** của bạn khi làm việc với các hệ thống AI (ChatGPT, Cursor, Claude):
- **Tính toán Tỷ lệ Tùy biến (Semantic Diff):** Xem xét việc bạn bê nguyên xi code từ AI hay có tư duy chỉnh sửa trước khi dán vào IDE.
- **Đánh giá Đa góc độ (Radar Chart):** Vẽ biểu đồ mạng nhện 6 cánh (Competence, Discipline, Creativity, Critical Thinking, Collaboration, AI Efficiency) để biểu diễn sức mạnh làm việc thực tế của bạn so với quy chuẩn Enterprise.

## 2. Trích xuất DNA Hành vi (Multi-Agent Profiling)
MyDNA sở hữu nhiều "Đặc vụ ngầm" (Agents) chuyên biệt:
- **Kỹ sư Review Code:** Quét mã nguồn và lịch sử commit/sửa code để tìm ra các thói quen lập trình tốt (VD: luôn viết docstring) hoặc xấu (VD: quên xử lý lỗi) của bạn.
- **Chuyên gia Nhân sự & Giao tiếp:** Theo dõi cách bạn phản hồi email, chat trong Outlook/Slack để bóc tách văn phong (Tone & Voice), thói quen dùng từ (Formal/Casual/Direct).
- **Chuyên gia Đánh giá Cấp bậc:** Dựa trên những gì hiển thị trên màn hình để kết luận bạn là Senior, Junior hay Manager.

*Các chỉ số này sẽ được hiển thị minh bạch ngay trong tab "Hồ sơ Năng lực" (DNA Profile) trên ứng dụng.*

## 3. Máy chủ Cung cấp Ngữ cảnh AI (MCP Server)
Đây là "vũ khí bí mật" của MyDNA:
- Khởi chạy một trạm phát sóng cục bộ tại `localhost:5050` tuân thủ chuẩn **Model Context Protocol (MCP)**.
- Khi bạn cài mới bất kỳ AI nào (Cursor IDE, Claude Desktop), chỉ cần cấu hình để nó trỏ tới cổng `5050` của MyDNA.
- Lập tức, các con AI này sẽ "đọc vị" được bạn (biết bạn viết code style nào, hay dùng biến gì, ăn nói ra sao) và tự điều chỉnh cách phản hồi cho giống với chính bạn **mà không cần bạn phải ngồi cấu hình Prompt thủ công**.

## 4. Tùy chỉnh Nâng cao qua YAML (Advanced Prompting)
- Các công thức AI mà MyDNA dùng để đánh giá bạn không bị giấu kín trong mã nguồn. 
- Bạn hoàn toàn có thể vào thư mục `portable-test/` mở file `prompts.yaml` bằng text editor (Notepad/VSCode) và thay đổi "nhân cách" hoặc mục tiêu phân tích của các Agents theo ý đồ doanh nghiệp. Phần mềm tự động áp dụng bản cập nhật ngay tắp lự.

## 5. Quyền riêng tư & Bảo mật
- **Không gửi Raw Data lên mây mờ ám:** Mọi sự kiện Copy/Paste đều chạy qua hàm băm (Lineage ID), không lưu trọn bộ văn bản.
- **Chống Gian lận Điện tử (Enterprise Anti-Tamper):** Áp dụng thuật toán mã hóa chữ ký số HMAC-SHA256 gắn trực tiếp vào Database thông qua OS Keyring (Windows Credential Manager). Nếu bạn cố tình dùng tool sửa database nhằm "buff" điểm kỹ năng hay chỉnh sửa nhận xét của AI, hệ thống sẽ phát hiện sai lệch băm (Hash Mismatch) và khóa tính năng xuất dữ liệu.
- **Đối soát chéo Mạng ngang hàng (P2P Cross-Validation):** Trong tương lai (Phase 2), MyDNA sẽ chặn đứng mọi thủ thuật can thiệp băm cấp hệ điều hành bằng cách đối chiếu mã băm chéo giữa các máy tính (Agents) trong cùng một công ty, tạo ra một mạng lưới bảo mật không thể bị thao túng bởi một cá nhân độc lập.
- Database lưu cục bộ (`sqlite`) trên máy tính người dùng. Việc đẩy dữ liệu lên Google Drive là do người dùng chủ động cho phép qua tài khoản Google cá nhân.
