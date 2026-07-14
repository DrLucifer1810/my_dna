use uiautomation::UIAutomation;
use uiautomation::UIElement;

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
        
        let text_value = format!("[{}]: {}", class_name, name);
        
        Some(FocusedElement {
            name,
            text_value,
        })
    } else {
        None
    }
}
