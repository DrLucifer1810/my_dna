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
    pub task_category: Option<String>,
    pub final_quality_score: Option<u8>,
    pub quality_reason: Option<String>,
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
            r#"You are an Expert AI/Human Interaction Behavior Analyst & Quality Rater. 
Your task is to analyze an OS Event Timeline log (which includes actual RAW CONTENT of Prompts, AI Outputs, and Final Saved Files).
You must dynamically categorize the domain of the task (e.g., Coding, Email, Planning) and score the final output quality.

You MUST output ONLY a valid JSON object matching the exact structure below, with integer scores from 0 to 100. Do not include any text outside the JSON.

{{
    "d1_context": <Score 0-100>,
    "d2_interaction": <Score 0-100>,
    "d3_customization": <Score 0-100>,
    "d4_efficiency": <Score 0-100>,
    "d5_security": <Score 0-100>,
    "d6_collaboration": <Score 0-100>,
    "task_category": "<String: Code|Email|Plan|Other>",
    "final_quality_score": <Score 0-100>,
    "quality_reason": "<Short string explaining the quality score>"
}}

Evaluation Rubric:
- d1_context (Prompt Quality): Assess the actual RAW_CONTENT of the prompt. Is it clear and contextual?
- d2_interaction: How well the user switches tools.
- d3_customization (Semantic Diff): Compare RAW_CONTENT of CLIPBOARD_COPY (AI output) against FILE_SAVED (Final output). Score HIGH if the user actively edited the AI output. Score LOW if it is blindly copy-pasted.
- d4_efficiency: Time taken from Prompt to Final File Saved.
- d5_security: Safe data usage.
- d6_collaboration: Sharing output to others.
- final_quality_score: Evaluate the actual RAW_CONTENT of the FILE_SAVED event based on its category (Is the code bug-free? Is the email professional? Is the plan actionable?).

Analyze the following OS timeline logs (including Content) and generate the exact JSON scores:
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
