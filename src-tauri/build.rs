fn main() {
    // Thử load .env file nếu tồn tại ở local development
    let _ = dotenvy::dotenv();

    // Xuất biến môi trường cho trình biên dịch (cargo) đọc qua macro env!()
    if let Ok(client_id) = std::env::var("MYDNA_GOOGLE_CLIENT_ID") {
        println!("cargo:rustc-env=MYDNA_GOOGLE_CLIENT_ID={}", client_id);
    }
    if let Ok(client_secret) = std::env::var("MYDNA_GOOGLE_CLIENT_SECRET") {
        println!("cargo:rustc-env=MYDNA_GOOGLE_CLIENT_SECRET={}", client_secret);
    }

    // Đảm bảo tauri build cũng được chạy
    tauri_build::build();
}
