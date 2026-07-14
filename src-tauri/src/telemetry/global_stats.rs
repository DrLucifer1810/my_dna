use serde::Serialize;
use std::time::Duration;
use reqwest::Client;
use crate::telemetry::p2p_network::MatchingProfile;

#[derive(Debug, Serialize)]
pub struct GlobalStatsPayload {
    pub public_key: String,
    pub intent: String, // "Ứng viên", "Nhà tuyển dụng", "Freelancer"
    pub seniority: String,
    pub work_model: String,
    pub top_skills: Vec<String>,
    pub salary_expectation: u32,
    pub signature: String, // Có thể bỏ qua trong bản prototype này, nhưng để schema sẵn
}

pub async fn start_global_stats_sync(
    webhook_url: String,
    public_key: String,
    intent: String,
    profile: MatchingProfile
) {
    let client = Client::new();
    
    // Tạo payload từ thông tin local
    let top_skills = profile.tech_stack
        .into_iter()
        .map(|s| s.name)
        .collect::<Vec<String>>();

    let payload = GlobalStatsPayload {
        public_key,
        intent,
        seniority: profile.seniority_level.clone(),
        work_model: profile.work_model.clone(),
        top_skills,
        salary_expectation: profile.min_salary,
        signature: "ed25519_dummy_sig".to_string(), // Tương lai tích hợp lấy chữ ký
    };

    // Vòng lặp đồng bộ mỗi 24 giờ
    tokio::spawn(async move {
        loop {
            println!("Đang đồng bộ Global Stats lên Webhook...");
            let res = client.post(&webhook_url)
                .json(&payload)
                .send()
                .await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        println!("Đồng bộ Global Stats thành công!");
                    } else {
                        println!("Lỗi đồng bộ Global Stats: {}", response.status());
                    }
                }
                Err(e) => {
                    println!("Không thể kết nối đến Webhook: {}", e);
                }
            }

            // Sleep 24 giờ
            tokio::time::sleep(Duration::from_secs(24 * 3600)).await;
        }
    });
}
