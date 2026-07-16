pub mod ingestion;

use std::process::Command;

#[tauri::command]
pub async fn install_vscode_extension() -> Result<String, String> {
    // Lấy thư mục làm việc hiện tại
    let current_dir = std::env::current_dir().map_err(|e| format!("Lỗi hệ thống: {}", e))?;
    let extension_path = current_dir.join("extensions").join("vscode");
    
    // Đối với Windows, dùng cmd /C code --install-extension
    let output = Command::new("cmd")
        .args(["/C", "code", "--install-extension", extension_path.to_str().unwrap_or_default()])
        .output()
        .map_err(|e| format!("Không thể chạy lệnh: {}", e))?;
        
    if output.status.success() {
        Ok("Đã cài đặt thành công Plugin vào VS Code!".to_string())
    } else {
        let err_str = String::from_utf8_lossy(&output.stderr);
        Err(format!("Lỗi cài đặt: {}", err_str))
    }
}

#[tauri::command]
pub async fn open_chrome_extension_store() -> Result<String, String> {
    // Mở trang Web Store của myDNA Extension (Demo URL)
    let url = "https://chrome.google.com/webstore/detail/mydna-web-watcher/demo";
    
    let output = Command::new("cmd")
        .args(["/C", "start", url])
        .output()
        .map_err(|e| format!("Không thể mở trình duyệt: {}", e))?;
        
    if output.status.success() {
        Ok("Đã mở trang cài đặt Extension trên trình duyệt!".to_string())
    } else {
        Err("Không thể mở trình duyệt.".to_string())
    }
}

#[tauri::command]
pub async fn connect_mcp_server(server_name: String, token: String) -> Result<String, String> {
    // Mô phỏng quá trình kết nối MCP Server
    // Trong thực tế, chúng ta sẽ spawn một process: `npx @modelcontextprotocol/server-github`
    // truyền token qua biến môi trường.
    
    println!("Connecting to MCP Server: {} with token: {}", server_name, token);
    
    // Simulate connection delay
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    
    if token.len() < 10 {
        return Err("Token quá ngắn hoặc không hợp lệ!".to_string());
    }
    
    // TODO: Spawn real background MCP Client
    // let mut mcp_process = Command::new("cmd").args(["/C", "npx", format!("@modelcontextprotocol/server-{}", server_name)]).env("TOKEN", token).spawn()...
    
    Ok(format!("Đã kết nối thành công tới máy chủ MCP: {}. myDNA sẽ bắt đầu đồng bộ dữ liệu.", server_name.to_uppercase()))
}
