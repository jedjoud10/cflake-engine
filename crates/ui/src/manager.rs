use crate::{element::ButtonState, element::ElementType, Root};

// The UI manager
#[derive(Default)]
pub struct UIManager {
    pub root: Root,
}

// Actually UI functions
impl UIManager {
    // Get the state of a specific button element
    pub fn get_button_state(&self, element_id: &u16) -> &ButtonState {
        // Get the element
        let elem = self.root.smart_element_list.get_element(element_id).unwrap();
        let state = match elem.element_type {
            ElementType::Button(ref state) => state,
            _ => &ButtonState::Released,
        };
        return state;
    }
    // Set the text of a text element
    pub fn set_text_state(&mut self, element_id: &u16, text: &str) {
        // Get the element mutably
        let elem = self.root.smart_element_list.get_element_mut(element_id).unwrap();
        match elem.element_type {
            ElementType::Text(ref mut last_text) => {
                // Set the text
                *last_text = text.to_string();
            }
            _ => {}
        }
    }
}
