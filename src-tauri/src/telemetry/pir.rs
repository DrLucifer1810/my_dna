use regex::Regex;
use std::sync::OnceLock;

// Khởi tạo Regex một lần (Singleton) để tối ưu hiệu năng
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static API_KEY_REGEX: OnceLock<Regex> = OnceLock::new();
static BEARER_TOKEN_REGEX: OnceLock<Regex> = OnceLock::new();
static CREDIT_CARD_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn redact_sensitive_data(input: &str) -> String {
    let email_re = EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()
    });
    
    let api_key_re = API_KEY_REGEX.get_or_init(|| {
        // Bắt các chuỗi dạng sk-..., AIzaSy... (Google), ghv... (GitHub)
        Regex::new(r"(?i)(sk-[a-zA-Z0-9]{32,}|AIza[0-9A-Za-z-_]{35}|gh[pousr]_[A-Za-z0-9_]{36,})").unwrap()
    });

    let bearer_re = BEARER_TOKEN_REGEX.get_or_init(|| {
        Regex::new(r"(?i)Bearer\s+[A-Za-z0-9\-\._~\+/]+=*").unwrap()
    });

    let cc_re = CREDIT_CARD_REGEX.get_or_init(|| {
        Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap()
    });

    let mut redacted = input.to_string();
    
    redacted = email_re.replace_all(&redacted, "[REDACTED_EMAIL]").to_string();
    redacted = api_key_re.replace_all(&redacted, "[REDACTED_API_KEY]").to_string();
    redacted = bearer_re.replace_all(&redacted, "[REDACTED_BEARER_TOKEN]").to_string();
    redacted = cc_re.replace_all(&redacted, "[REDACTED_CREDIT_CARD]").to_string();

    redacted
}
