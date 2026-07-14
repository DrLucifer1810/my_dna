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
            r#"You are an Expert AI/Human Interaction Behavior Analyst. 
Your task is to analyze an OS Event Timeline log of a user and score their "AI Competency DNA".
You MUST output ONLY a valid JSON object matching the exact structure below, with integer scores from 0 to 100. Do not include any text outside the JSON.

{{
    "d1_context": <Score 0-100>,
    "d2_interaction": <Score 0-100>,
    "d3_customization": <Score 0-100>,
    "d4_efficiency": <Score 0-100>,
    "d5_security": <Score 0-100>,
    "d6_collaboration": <Score 0-100>
}}

Evaluation Rubric:
- d1_context (Context Definition): High score if the user gathers context across multiple windows before pasting/typing into an AI tool.
- d2_interaction (Interaction Flexibility): High score if the user switches smoothly between different applications (IDE, Browser, AI).
- d3_customization (Customization): High score if the user edits or focuses on the text (FOCUSED_TEXT events) after copying from an AI tool, rather than blindly pasting.
- d4_efficiency (Workflow Efficiency): High score if the problem-solving loop (Copy -> AI -> Paste) is logically structured and fast.
- d5_security (Security & Privacy): Score low if the user copies sensitive-looking strings (like API keys) into public AI windows. Score high if the workflow is safe.
- d6_collaboration (Collaboration): High score if the final output from AI is pasted into communication/team apps.

Analyze the following OS timeline logs and generate the exact JSON scores:
{}"#,
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
