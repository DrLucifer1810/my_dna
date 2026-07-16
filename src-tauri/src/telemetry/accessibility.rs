use uiautomation::UIAutomation;
use uiautomation::UIElement;
use uiautomation::UITreeWalker;

pub struct FocusedElement {
    pub name: String,
    pub text_value: String,
}

pub fn get_focused_element() -> Option<FocusedElement> {
    let automation = match UIAutomation::new() {
        Ok(a) => a,
        Err(_) => return None,
    };

    if let Ok(element) = automation.get_focused_element() {
        let name = element.get_name().unwrap_or_default();
        let class_name = element.get_classname().unwrap_or_default();
        
        let mut full_text = String::new();
        let mut browser_url = String::new();

        // Cố gắng đi ngược lên cửa sổ chính (Window)
        let walker = automation.get_control_view_walker().unwrap_or(automation.get_raw_view_walker().unwrap());
        
        let mut current = element.clone();
        let mut window_element = element.clone();
        for _ in 0..10 {
            if let Ok(parent) = walker.get_parent(&current) {
                if parent.get_classname().unwrap_or_default().contains("Window") {
                    window_element = parent;
                    break;
                }
                current = parent;
            } else {
                break;
            }
        }

        // Đi xuống để gom text và tìm URL
        dump_element_tree(&walker, &window_element, 5, 0, &mut full_text, &mut browser_url);
        
        let mut text_value = String::new();
        if !browser_url.is_empty() {
            text_value.push_str(&format!("[URL Detected]: {}\n", browser_url));
        }
        text_value.push_str(&format!("[{}]: {}\n--- UIA Snapshot ---\n{}", class_name, name, full_text));
        
        Some(FocusedElement {
            name,
            text_value,
        })
    } else {
        None
    }
}

fn dump_element_tree(walker: &UITreeWalker, element: &UIElement, max_depth: i32, current_depth: i32, out: &mut String, url_out: &mut String) {
    if current_depth > max_depth { return; }
    
    if let Ok(name) = element.get_name() {
        // Heuristic bắt URL từ Address Bar của trình duyệt
        let lower_name = name.to_lowercase();
        if lower_name.starts_with("http://") || lower_name.starts_with("https://") || lower_name.starts_with("www.") {
            if url_out.is_empty() {
                *url_out = name.clone();
            }
        }

        if !name.is_empty() && name.len() > 3 { // Bỏ qua text quá ngắn
            out.push_str(&name);
            out.push_str("\n");
        }
    }
    
    if let Ok(child) = walker.get_first_child(element) {
        dump_element_tree(walker, &child, max_depth, current_depth + 1, out, url_out);
        
        let mut current_sibling = child;
        while let Ok(sibling) = walker.get_next_sibling(&current_sibling) {
            dump_element_tree(walker, &sibling, max_depth, current_depth + 1, out, url_out);
            current_sibling = sibling;
        }
    }
}
