use teloxide::prelude::*;
use std::sync::Arc;
use crate::telemetry::state_machine::StateMachine;
use crate::slm_client::gemini_companion::run_gemini_background_prompt;

pub async fn start_telegram_listener(app_handle: tauri::AppHandle, state_machine: Arc<std::sync::Mutex<StateMachine>>) {
    // Lấy token từ DB
    let token = match state_machine.lock().unwrap().get_telegram_token() {
        Ok(Some(t)) => t,
        _ => return, // Chưa cài token thì thoát
    };

    if token.is_empty() {
        return;
    }

    // Khởi tạo bot bằng teloxide
    let bot = Bot::new(token);
    
    println!("Khởi động Telegram Event Listener (Teloxide) cục bộ...");
    
    // Tạo dispatcher để lắng nghe sự kiện
    let sm_clone = state_machine.clone();
    let handler = Update::filter_message().endpoint(
        move |bot: Bot, msg: Message| {
            let sm = sm_clone.clone();
            let app_handle = app_handle.clone();
            async move {
                if let Some(text) = msg.text() {
                    let chat_id = msg.chat.id;
                    
                    // Xử lý lệnh /start
                    if text.starts_with("/start") {
                        let _ = sm.lock().unwrap().set_telegram_chat_id(&chat_id.to_string());
                        let _ = bot.send_message(chat_id, "Liên kết thành công! MyDNA Mentor đã sẵn sàng phân tích kỹ năng của bạn.").await;
                        return respond(());
                    }

                    // Nếu là tin nhắn bình thường hỏi đáp 2 chiều
                    let _ = bot.send_message(chat_id, "Đang suy nghĩ dựa trên Code Log cục bộ...").await;
                    
                    // Lấy log gần đây để làm Context cho Gemini
                    let context = sm.lock().unwrap().get_recent_logs().unwrap_or_default();
                    let prompt = format!("Dưới đây là hoạt động gần đây của tôi:\n{}\n\nCâu hỏi: {}", context, text);
                    
                    // Giao tiếp với Gemini (WebView hoặc API)
                    // Vì closure move vào tokio::spawn, ta clone app_handle để truyền vào
                    match run_gemini_background_prompt(app_handle.clone(), prompt, None, None).await {
                        Ok(answer) => {
                            let _ = bot.send_message(chat_id, answer).await;
                        }
                        Err(e) => {
                            let _ = bot.send_message(chat_id, format!("Lỗi AI: {}", e)).await;
                        }
                    }
                }
                respond(())
            }
        }
    );

    // Chạy event loop bất đồng bộ ngầm
    tokio::spawn(async move {
        Dispatcher::builder(bot, handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    });
}
