use clipboard_win::{formats, get_clipboard};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct ClipboardEvent {
    pub lineage_id: String,
    pub content: String,
}

pub fn get_clipboard_text() -> Option<String> {
    if let Ok(text) = get_clipboard(formats::Unicode) {
        Some(text)
    } else {
        None
    }
}

pub fn generate_lineage_id(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn check_clipboard() -> Option<ClipboardEvent> {
    if let Ok(text) = clipboard_win::get_clipboard_string() {
        if !text.is_empty() {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            let lineage_id = format!("{:x}", hasher.finish());
            
            return Some(ClipboardEvent {
                lineage_id,
                content: text,
            });
        }
    }
    None
}
