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
    pub standard_hash: String,
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

    let data_to_sign = format!("{}_{}_{}_{}", public_key, intent, profile.seniority_level.clone(), profile.work_model.clone());
    let real_signature = match crate::telemetry::crypto::sign_data(&data_to_sign) {
        Ok(sig) => sig,
        Err(e) => {
            println!("Lỗi bảo mật: Không thể ký số dữ liệu P2P. Hệ thống từ chối đồng bộ Global Stats (Lỗi: {})", e);
            return;
        }
    };
    
    let standard_hash = crate::telemetry::standard_manager::StandardManager::get_current_standard_hash()
        .unwrap_or_else(|_| "".to_string());

    let payload = GlobalStatsPayload {
        public_key,
        intent,
        seniority: profile.seniority_level.clone(),
        work_model: profile.work_model.clone(),
        top_skills,
        salary_expectation: profile.min_salary.unwrap_or(0),
        signature: real_signature,
        standard_hash,
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
