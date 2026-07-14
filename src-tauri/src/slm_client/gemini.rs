use reqwest::{Client, Error as ReqwestError};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AiError {
    Network(ReqwestError),
    ApiError(String),
}

impl From<ReqwestError> for AiError {
    fn from(err: ReqwestError) -> Self {
        AiError::Network(err)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RadarScore {
    pub d1_context: u8,
    pub d2_interaction: u8,
    pub d3_customization: u8,
    pub d4_efficiency: u8,
    pub d5_security: u8,
    pub d6_collaboration: u8,
}

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        GeminiClient {
            client: Client::new(),
            api_key,
        }
    }

    /// Nạp dữ liệu thô và yêu cầu Gemini phân tích trả về JSON.
    /// Tuân thủ Fail-fast, không dùng mock data.
    pub async fn analyze_timeline(&self, raw_logs: &str) -> Result<RadarScore, AiError> {
        let endpoint = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-pro:generateContent?key={}",
            self.api_key
        );

        let prompt = format!(
            "Analyze the following OS timeline and output ONLY a JSON object representing 6-dimension scores (d1_context, d2_interaction, d3_customization, d4_efficiency, d5_security, d6_collaboration) from 0 to 100.\n\nLogs:\n{}",
            raw_logs
        );

        let payload = serde_json::json!({
            "contents": [{
                "parts": [{"text": prompt}]
            }]
        });

        let res = self.client.post(&endpoint)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!("Gemini API failed: {}", error_text)));
        }

        // Dummy struct để parse Gemini response (Đơn giản hóa cho MVP)
        #[derive(Deserialize)]
        struct GeminiResponse {
            candidates: Vec<Candidate>,
        }
        #[derive(Deserialize)]
        struct Candidate {
            content: Content,
        }
        #[derive(Deserialize)]
        struct Content {
            parts: Vec<Part>,
        }
        #[derive(Deserialize)]
        struct Part {
            text: String,
        }

        let response_data: GeminiResponse = res.json().await?;
        if let Some(candidate) = response_data.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                // Lọc bỏ markdown block nếu có
                let json_str = part.text.trim().trim_start_matches("```json").trim_end_matches("```").trim();
                let score: RadarScore = serde_json::from_str(json_str)
                    .map_err(|e| AiError::ApiError(format!("Failed to parse JSON: {}", e)))?;
                return Ok(score);
            }
        }

        Err(AiError::ApiError("Empty response from Gemini".into()))
    }
}
