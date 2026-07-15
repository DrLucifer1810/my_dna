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

        // Đi xuống để gom text
        dump_element_tree(&walker, &window_element, 5, 0, &mut full_text);
        
        let text_value = format!("[{}]: {}\n--- Context ---\n{}", class_name, name, full_text);
        
        Some(FocusedElement {
            name,
            text_value,
        })
    } else {
        None
    }
}

fn dump_element_tree(walker: &UITreeWalker, element: &UIElement, max_depth: i32, current_depth: i32, out: &mut String) {
    if current_depth > max_depth { return; }
    
    if let Ok(name) = element.get_name() {
        if !name.is_empty() && name.len() > 3 { // Bỏ qua text quá ngắn
            out.push_str(&name);
            out.push_str("\n");
        }
    }
    
    if let Ok(child) = walker.get_first_child(element) {
        dump_element_tree(walker, &child, max_depth, current_depth + 1, out);
        
        let mut current_sibling = child;
        while let Ok(sibling) = walker.get_next_sibling(&current_sibling) {
            dump_element_tree(walker, &sibling, max_depth, current_depth + 1, out);
            current_sibling = sibling;
        }
    }
}
