# Hướng dẫn Môi trường Kiểm thử Đa trạm (Multi-Node Testing) cho P2P

Để đảm bảo hệ thống MyDNA P2P Network hoạt động chính xác theo chuẩn Enterprise phi tập trung, chúng ta đã thiết lập một hệ thống biến môi trường (Environment Variables) đặc biệt để có thể chạy giả lập (Clustering) nhiều Node song song ngay trên cùng một máy tính Window duy nhất.

Cấu trúc này cho phép anh kiểm chứng chức năng Cross-Verification, Matching, và Gossip mà không cần tốn tiền thuê nhiều VPS.

## 1. Cơ chế Multi-Node 
Nếu hệ thống phát hiện có biến môi trường `MYDNA_TEST_NODE`, phần mềm sẽ:
1. **Tách biệt Dữ liệu (Isolated Database):** Không lưu vào `portable-test/local_events.db` nữa, mà sẽ rẽ nhánh sang `portable-test/{MYDNA_TEST_NODE}/local_events.db`.
2. **Tách biệt Định danh (Isolated Identity):** Bỏ qua Key OS Keychain chung của User. Hệ thống sẽ tự động khởi tạo ngẫu nhiên một Private Key (Ed25519) mới và lưu vào Keychain riêng lẻ biệt lập `MyDNA_Enterprise_P2P_{MYDNA_TEST_NODE}`.
3. **Mạng độc lập:** Cho phép lắng nghe ở các cổng mạng (Port) khác nhau (thông qua `MYDNA_P2P_PORT`) nhưng vẫn có thể nối ghép với nhau qua Bootstrap.

## 2. Kịch bản Khởi động (Test Script)

Anh có thể mở **2 cửa sổ PowerShell** riêng biệt, cd vào `src-tauri` và thực hiện các lệnh sau:

### Cửa sổ 1: Đóng vai trò (Node 1 - Freelancer)
Cửa sổ này đóng vai trò là "Node Gốc" (Bootstrap Node) chạy ở cổng `8000`.
```powershell
$env:MYDNA_P2P_PORT="8000"
$env:MYDNA_BOOTSTRAP_NODES=""
$env:MYDNA_TEST_NODE="node1"

cargo run
```
> Khi mở giao diện Node 1: Đánh dấu vào ô "Tôi làm Freelancer" và nhập Email giả (vd: node1@gmail.com). Sau đó bấm `Khởi động Mạng P2P`.

### Cửa sổ 2: Đóng vai trò (Node 2 - Nhà tuyển dụng)
Cửa sổ này chạy ở cổng `8001`, và kết nối với Node 1 để tham gia mạng lưới.
```powershell
$env:MYDNA_P2P_PORT="8001"
$env:MYDNA_BOOTSTRAP_NODES="/ip4/127.0.0.1/tcp/8000"
$env:MYDNA_TEST_NODE="node2"

cargo run
```
> Khi mở giao diện Node 2: Đánh dấu vào ô "Tôi thuê Freelancer" và nhập Email giả (vd: hr@company.com). Sau đó bấm `Khởi động Mạng P2P`.

## 3. Điều gì sẽ xảy ra?
1. Ngay khi Node 2 bấm Khởi động, nó sẽ tự động gửi Broadcast (Gossip) nhu cầu.
2. Node 1 nhận được Gossip, nhận diện (Cross-match) thấy khớp nhu cầu.
3. Node 1 sẽ đối chiếu Bằng chứng tính toàn vẹn (Integrity Snapshot) của Node 2 với file `releases.json`.
4. Nếu Node 2 chưa hack mã nguồn, Node 1 sẽ gửi gói Bắt tay (Handshake) qua giao thức Request-Response, đính kèm `node1@gmail.com`.
5. Thông báo Match thành công sẽ hiển thị ở Console/Log của Node 2!
